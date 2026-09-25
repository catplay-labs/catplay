use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};

use catplay_util::ModSeq;
use log::{debug, trace, warn};

use crate::audio::{RingBuffer, RingConsumer, RingProducer, codec::AudioDecoder};
use crate::rtp::AsRtpPacket;

/// Consumes RTP packets, performs de-jitter and codec decode, and manages producer side of audio ring buffer.
pub struct RtpPlayer<T: AudioDecoder> {
    ring: RingProducer<T::Sample>,
    decoder: T,

    frame_size_in_samples: NonZeroUsize,
    last_decoded_ts: Option<ModSeq<u32>>,
    last_decoded_seq: Option<ModSeq<u16>>,
    lost_counter: u64,

    buffer_ms: Duration,
    _play: Option<Instant>,
    start: Option<Instant>,

    is_legacy_buffering_stream: bool,
}

impl<T: AudioDecoder> RtpPlayer<T> {
    pub fn new(
        decoder: T,
        _sample_rate: NonZeroUsize,
        frame_size_in_samples: NonZeroUsize,
        latency_ms: Duration,
        _start: Instant,
    ) -> (Self, RingConsumer<T::Sample>) {
        let output_type = decoder.output_type(); // PCM i16 with a customized sample_rate

        // Apple was originally using latency_ms * 20 which is too aggressive and wastes as much as 4MB of ram
        // let ring_size_ms = latency_ms * 20;
        let ring_size_ms = latency_ms + Duration::from_millis(1000);

        let ring_size = output_type.ms_to_samples(ring_size_ms) as usize * frame_size_in_samples.get();
        let ring = RingBuffer::spsc(ring_size);

        let me = Self {
            ring: ring.0,
            decoder,
            frame_size_in_samples,
            last_decoded_ts: None,
            last_decoded_seq: None,
            lost_counter: 0,
            buffer_ms: latency_ms,
            _play: None,
            start: None,
            is_legacy_buffering_stream: true, // TODO
        };

        // me.write_latency_frames();
        (me, ring.1)
    }

    fn _write_latency_frames(&mut self) {
        let latency_samples = self.decoder.output_type().ms_to_samples(self.buffer_ms) * self.frame_size_in_samples.get() as u32;
        self.ring.write_default(latency_samples as _);
        self._play.replace(Instant::now());
    }

    pub fn should_play(&mut self) -> bool {
        match self.start {
            None => false,
            Some(v) => Instant::now() - v >= self.buffer_ms,
        }
    }

    // pub fn feedback(&mut self) -> (usize, Instant) {
    //     let consumed = self.ring.total_consumed(); // TODO: is this supposed to be in samples or frames?
    //     let now = Instant::now();
    //     (consumed, now)
    // }

    pub fn push(&mut self, packet: &dyn AsRtpPacket) {
        let (header, _payload) = packet.split();
        let ts = ModSeq(header.timestamp);

        if let Some(expected) = self.last_decoded_ts {
            let delta = ts.distance(expected);
            trace!("Received packet with delta {delta} / ts {ts}");
        } else {
            trace!("Received first packet / ts {ts}");
        }

        self.push_decode(packet);
    }

    fn push_decode(&mut self, packet: &dyn AsRtpPacket) {
        let decode_start = Instant::now(); // assumed to be a timestamp where packet was received

        let ts = ModSeq(packet.header().timestamp);
        let seq = ModSeq(packet.header().sequence_number);

        let frame_size_in_samples = self.frame_size_in_samples;
        // For supports_high_accuracy_timestamps expect ts to start with zero, for legacy streams expect random value...
        if self.last_decoded_ts.is_none() {
            trace!("Initializing RTP timestamp base at {ts}");
            self.last_decoded_ts = Some(ts);
            self.last_decoded_seq = Some(seq);
        }

        let Some(last_decoded_ts) = self.last_decoded_ts else {
            return;
        };

        let ts_delta = ts.distance(last_decoded_ts);

        if ts < last_decoded_ts {
            // This is informative for modern streams (supports_high_accuracy_timestamps) but super spammy for legacy streams
            // where iOS spams us with retransmissions constantly as a redundancy
            if self.is_legacy_buffering_stream {
                debug!("Ignored late packet: {ts} vs {last_decoded_ts}");
            } else {
                warn!("Ignored late packet: {ts} vs {last_decoded_ts}");
            }
            return;
        }

        {
            // When packet was chosen to be decoded, and there is a gap in the timeline
            // we need to write a silence frame, repeated by the size of the gap.
            if ts_delta > 0 {
                debug!("Before decoding, fill gap of {ts_delta}");
                self.write_silence_gap(ts_delta as _);
            } else {
                trace!("No gap fill required");
            }

            // At this point we don't know if packet decode will be successful
            // - if it is, it will set last_decoded_ts to be equal to `ts` + amount of decoded frames
            // Otherwise, if decode fails, we consider the gap to be properly accounted for until `ts`
            // - which represents ts of first frame in the packet; we don't precisely know how many frames
            // it holds without a successful decode
            trace!("Jump -> {last_decoded_ts} -> {ts} at delta {ts_delta}");

            if let Some(last_decoded_seq) = self.last_decoded_seq {
                let seq_delta = seq.distance(last_decoded_seq);
                if seq_delta > 1 {
                    let lost = seq_delta - 1;
                    self.lost_counter += lost as u64;
                    warn!("Lost {lost} packets lost_counter={}!", self.lost_counter);
                }
            }

            self.last_decoded_ts.replace(ts);
            self.last_decoded_seq.replace(seq);
        }

        let decoded = self
            .decoder
            .decode(packet.payload(), self.ring.writable_slice());

        match decoded {
            Err(err) => {
                warn!("Ignoring packet ts={ts}, failed decode: {err}");
            }
            Ok((decoded_samples, consumed)) => {
                if decoded_samples == 0 {
                    debug!("??? Received decoded_samples = 0 at ts={ts}, can't adjust timeline");
                    return;
                }

                if consumed != packet.payload().len() {
                    debug!(
                        "??? Expected full data consumption by decoder at ts={ts}, got {consumed}/{}",
                        packet.payload().len()
                    );
                    return;
                }

                let decoded_frames = decoded_samples / frame_size_in_samples;
                if decoded_samples % frame_size_in_samples != 0 {
                    debug!(
                        "??? Received decoded_samples % frame_size_in_samples != 0 at ts={ts}: {decoded_samples} % {frame_size_in_samples} != 0",
                    );
                }

                self.ring.write_commit(decoded_samples);
                if self.start.is_none() {
                    self.start.replace(decode_start);
                }

                let stat = self.ring.stat().as_audio_stats(
                    self.decoder.output_type().sample_rate as _,
                    self.frame_size_in_samples.get() as _,
                    self.start.unwrap(),
                );

                debug!(
                    "Decoded {decoded_samples} samples / frame_size_in_samples {frame_size_in_samples} at ts={ts}; buffer={:?}/{:?}",
                    stat.readable, stat.capacity
                );

                if !stat.overflow.is_zero() {
                    warn!("RTP player queue overflown after decode, is remote flooding us? Is audio player keeping up? {stat:?}");
                }

                self.last_decoded_ts
                    .replace(ts + ModSeq(decoded_frames as u32));
            }
        }
    }

    fn write_silence_gap(&mut self, mut gap_frames: usize) {
        trace!("Filling gap of {gap_frames}");
        let frame_size_in_samples = self.frame_size_in_samples;

        while gap_frames > 0 {
            // Try concealment (optional)
            match self.decoder.conceal_lost_packet(self.ring.writable_slice()) {
                Err(err) => {
                    debug!("Conceal returned error: {err:?}");
                }
                Ok(decoded_samples) if decoded_samples > 0 => {
                    debug!(
                        "Concealed {decoded_samples} samples x frame_size_in_samples {}",
                        frame_size_in_samples
                    );
                    self.ring.write_commit(decoded_samples);

                    let decoded_frames = decoded_samples / frame_size_in_samples;
                    let used = decoded_frames.min(gap_frames);
                    gap_frames -= used;
                    continue;
                }
                _ => {}
            }

            // Fallback to silence
            let gap_samples = gap_frames * frame_size_in_samples.get();
            debug!("Writing silence {gap_frames} x {frame_size_in_samples} = {gap_samples}");
            self.ring.write_default(gap_samples);
            break;
        }
    }
}
