use std::{
    collections::VecDeque,
    error::Error,
    num::{NonZeroU32, NonZeroUsize},
    time::{Duration, Instant},
};

use super::{RtpPacketMeta, RtpTxPacer};
use catplay_util::ModSeq;
use log::{trace, warn};

use crate::{
    audio::{RingBuffer, RingConsumer, RingProducer, codec::AudioEncoder},
    msg::StreamType,
    rtp::{RTP_HEADER_SIZE, RTP_PACKET_OVERHEAD_CHACHA, RtpChaChaBuffer, RtpCipher, RtpHeader},
    rtsp_frame::RtspError,
};

pub struct RtpRecorder<T: AudioEncoder> {
    ring: RingConsumer<T::Sample>,

    cipher: RtpCipher,

    stream_type: StreamType,
    /// Stream latency negotiated during `SETUP`, in milliseconds.
    latency_ms: u64,
    // Extended RTP timestamp; truncate only when writing the wire header.
    ts: u64,
    seq: ModSeq<u16>,
    encoder: T,

    samples_per_packet: NonZeroUsize,
    frame_size_in_samples: NonZeroUsize,
    pacer: RtpTxPacer,

    /// Empty UDP packet buffers; all buffers are initially added here, then moved to `pending_transmit` after encode
    /// and transmission returns them here after `sendmmsg`.
    ///
    /// The goal is to keep the latency-sensitive encode process going while the transmit task may wait for permits
    /// to avoid flooding remote's UDP receive buffer.
    empty_buffers: VecDeque<Box<RtpChaChaBuffer>>,
    pending_transmit: VecDeque<EncodedPacket>,
}

struct EncodedPacket {
    buffer: Box<RtpChaChaBuffer>,
    meta: RtpPacketMeta,
}

/// Critical error in RTP recorder session that should terminate it; non-critical errors
/// result in a timeline jump instead.
#[derive(thiserror::Error, Debug)]
pub enum RtpRecorderError<T: Error> {
    #[error("{0}")]
    Codec(T),
    #[error("{0}")]
    Rtsp(#[from] RtspError),
    #[error("Overflow")]
    Overflow,
}

impl<T: AudioEncoder> RtpRecorder<T> {
    const UDP_QUEUE_SIZE: usize = 16;
    const PACKETS_RING_MULTIPLIER: usize = 50;

    pub fn new(
        encoder: T,
        stream_type: StreamType,
        latency_ms: u64,
        samples_per_packet: NonZeroUsize,
        frame_size_in_samples: NonZeroUsize,
        cipher: RtpCipher,
    ) -> (Self, RingProducer<T::Sample>) {
        let sample_rate = NonZeroU32::new(encoder.output_type().sample_rate).expect("RTP sample rate must be nonzero");
        // Bound both decoded audio queued ahead and bursts into the remote UDP queue.
        let pacer = RtpTxPacer::new(
            sample_rate,
            // Enforce real-time pacing except for MainHighAudio.
            if stream_type == StreamType::MainHighAudio {
                    Duration::from_millis(150)
                } else {
                    Duration::ZERO
                },
            Self::udp_interval(sample_rate, samples_per_packet.get() / frame_size_in_samples.get()),
            NonZeroU32::new(4).unwrap(),
        );
        let (producer, ring) = RingBuffer::spsc(samples_per_packet.get() * Self::PACKETS_RING_MULTIPLIER);

        let mut empty_buffers = VecDeque::new();
        for _ in 0..Self::UDP_QUEUE_SIZE {
            empty_buffers.push_back(RtpChaChaBuffer::new().into());
        }
        (
            RtpRecorder {
                ring,
                cipher,
                stream_type,
                latency_ms,
                ts: 0,
                seq: ModSeq(0),
                encoder,
                samples_per_packet,
                frame_size_in_samples,
                pacer,
                empty_buffers,
                pending_transmit: VecDeque::new(),
            },
            producer,
        )
    }

    fn udp_interval(sample_rate: NonZeroU32, packet_frames: usize) -> Duration {
        // Allow at least twice the nominal packet rate so UDP pacing can also
        // recover from scheduling delays. Count frames, not interleaved samples.
        let ns = (packet_frames as u128 * 1_000_000_000) / (sample_rate.get() as u128 * 2);
        Duration::from_nanos(ns.min(1_000_000) as u64)
    }

    pub fn latency_ms(&self) -> u64 {
        self.latency_ms
    }

    pub fn encode_into(encoder: &mut T, frames: &[T::Sample], payload: &mut [u8]) -> Result<(usize, usize), RtpRecorderError<T::Error>> {
        let packet_samples = frames.len();

        match encoder.encode(frames, payload) {
            Err(err) => {
                warn!("Failed to encode audio, skipping by {packet_samples} samples: {err}");
                Err(RtpRecorderError::Codec(err))
            }
            Ok((written, consumed)) => {
                if consumed != packet_samples {
                    warn!("Encoder did not consume all samples! written={written} consumed={consumed}/{packet_samples}");
                    return Err(RtpRecorderError::Overflow);
                }

                Ok((written, consumed))
            }
        }
    }

    pub fn prepare_header(stream_type: u8, ts: ModSeq<u32>, seq: ModSeq<u16>) -> RtpHeader {
        RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: stream_type,
            sequence_number: seq.0,
            timestamp: ts.0,
            ssrc: 0,
        }
    }

    pub fn prepare_packet(
        cipher: &mut RtpCipher,
        encoder: &mut T,
        frames: &[T::Sample],
        buf: &mut RtpChaChaBuffer,
        stream_type: u8,
        ts: ModSeq<u32>,
        seq: ModSeq<u16>,
    ) -> Result<(), RtpRecorderError<T::Error>> {
        // Take an aligned buffer and start writing payload past the 12 bytes RTP header mark.
        let buf_unsafe = unsafe { buf.uninit_slice_mut() };
        let buf_len = buf_unsafe.len();
        let payload = &mut buf_unsafe[RTP_HEADER_SIZE..buf_len - RTP_PACKET_OVERHEAD_CHACHA];

        let (written, _consumed) = Self::encode_into(encoder, frames, payload)?;

        let header = Self::prepare_header(stream_type, ts, seq);
        trace!("Header: {header:?}");
        let len = cipher.encode(buf_unsafe, written, &header)?.len();
        unsafe { buf.assume_written(len) }

        Ok(())
    }

    pub fn _drop_packets_older_than_ts(&mut self, ts: u64) {
        while self.pending_transmit.front().is_some_and(|packet| packet.meta.first_ts < ts) {
            let packet = self.pending_transmit.pop_front().unwrap();
            self.empty_buffers.push_back(packet.buffer);
        }
    }

    pub fn rewind_rounding(&mut self, samples: usize) {
        // If we are forced to rewind, for safety round the number of samples to expected number of samples in a packet.
        // This isn't a _strict_ protocol requirement, but for some receivers it may be an undefined behavior.

        let packet_samples = self.samples_per_packet.get();
        let packets = samples.div_ceil(packet_samples);
        let packet_ts = (packet_samples / self.frame_size_in_samples.get()) as u64;
        self.ts += packets as u64 * packet_ts;
        let samples_to_commit = packets * packet_samples;
        
        self.ring.read_commit(samples_to_commit);
    }

    pub fn encode_all(&mut self) -> Result<(), RtpRecorderError<T::Error>> {
        let packet_samples = self.samples_per_packet.get();
        let ts_jump = (packet_samples / self.frame_size_in_samples.get()) as u64;

        // If we know for certain that remote is playing a TS that's `> read_head` we could also `rewind_rounding`
        // and `drop_packets_older_than_ts` here, but it's a an unlikely scenario.

        while !self.empty_buffers.is_empty() {
            let (overflow, readable) = self.ring.readable_slice();
            if overflow > 0 {
                warn!("Overflow during recording; skipping {overflow} samples");
                self.rewind_rounding(overflow);
                continue;
            }

            let mut samples_to_commit = 0;

            let packets = (readable.len() / packet_samples).min(self.empty_buffers.len());
            for frames in readable[..packets * packet_samples].chunks_exact(packet_samples) {
                let buffer = self.empty_buffers.front_mut().unwrap();
                if let Err(err) = Self::prepare_packet(
                    &mut self.cipher,
                    &mut self.encoder,
                    frames,
                    buffer,
                    self.stream_type as _,
                    ModSeq(self.ts as u32),
                    self.seq,
                ) {
                    self.ring.read_commit(samples_to_commit);
                    return Err(err);
                }

                let meta = RtpPacketMeta {
                    first_ts: self.ts,
                    end_ts: self.ts + ts_jump,
                };
                self.ts += ts_jump;
                self.seq = self.seq + 1;
                samples_to_commit += packet_samples;
                self.pending_transmit.push_back(EncodedPacket {
                    buffer: self.empty_buffers.pop_front().unwrap(),
                    meta,
                });
            }

            self.ring.read_commit(samples_to_commit);
            if packets == 0 {
                break;
            }
        }

        Ok(())
    }

    /// Encode and transmit until the input is exhausted or pacing requires a wakeup.
    pub fn flush(
        &mut self,
        callback: &mut dyn FnMut(&[&[u8]]) -> Result<usize, RtspError>,
    ) -> Result<Option<Instant>, RtpRecorderError<T::Error>> {
        loop {
            self.encode_all()?;
            if self.pending_transmit.is_empty() {
                return Ok(None);
            }
            if let Some(wake_at) = self.drain_encoded(callback)? {
                return Ok(Some(wake_at));
            }
        }
    }

    fn drain_encoded(&mut self, callback: &mut dyn FnMut(&[&[u8]]) -> Result<usize, RtspError>) -> Result<Option<Instant>, RtspError> {
        const UDP_BATCH_SIZE: usize = 16;

        while !self.pending_transmit.is_empty() {
            let now = Instant::now();
            let decision = self
                .pacer
                .inspect(now, self.pending_transmit.iter().map(|packet| packet.meta), UDP_BATCH_SIZE);
            if decision.permits == 0 {
                return Ok(decision.wake_at);
            }

            let batch = RtpChaChaBuffer::build_send_slices::<UDP_BATCH_SIZE>(
                self.pending_transmit
                    .iter()
                    .take(decision.permits)
                    .map(|packet| packet.buffer.as_ref()),
            );
            let sent = match callback(&batch[..decision.permits]) {
                Ok(sent) => sent,
                Err(RtspError::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => 0,
                Err(err) => return Err(err),
            };
            assert!(sent <= decision.permits);
            self.pacer.commit(
                now,
                self.pending_transmit
                    .iter()
                    .take(sent)
                    .map(|packet| packet.meta),
            );
            self.empty_buffers.extend(
                self.pending_transmit
                    .drain(..sent)
                    .map(|packet| packet.buffer),
            );

            // The socket peer has no writable notification. Retry without losing
            // the unsent suffix or spinning when the socket is backpressured.
            if sent < decision.permits {
                return Ok(Some(Instant::now() + self.pacer.udp_interval()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::audio::AudioStreamBasicDescription;
    use std::convert::Infallible;

    pub(crate) struct TestEncoder;

    impl AudioEncoder for TestEncoder {
        type Error = Infallible;
        type Sample = i16;

        fn input_type(&self) -> AudioStreamBasicDescription {
            AudioStreamBasicDescription::fill_pcm(48_000, 16, 16, 2, false)
        }

        fn output_type(&self) -> AudioStreamBasicDescription {
            self.input_type()
        }

        fn encode(&mut self, samples: &[i16], output: &mut [u8]) -> Result<(usize, usize), Infallible> {
            output[0] = 42;
            Ok((1, samples.len()))
        }
    }

    pub(crate) fn recorder() -> (RtpRecorder<TestEncoder>, RingProducer<i16>) {
        RtpRecorder::new(
            TestEncoder,
            StreamType::MainAudio,
            250,
            NonZeroUsize::new(960).unwrap(),
            NonZeroUsize::new(2).unwrap(),
            RtpCipher::None,
        )
    }

    #[test]
    fn udp_budget_sustains_realtime_across_packet_sizes_and_sample_rates() {
        // AAC-LC, AAC-ELD, Opus, PCM and packets requiring more than 1000 pps.
        for frames in [1024, 512, 480, 240, 120, 352, 16, 1] {
            for rate in [8_000, 16_000, 24_000, 44_100, 48_000, 96_000, 192_000] {
                let sample_rate = NonZeroU32::new(rate).unwrap();
                let interval = RtpRecorder::<TestEncoder>::udp_interval(sample_rate, frames);
                let mut pacer = RtpTxPacer::new(sample_rate, Duration::ZERO, interval, NonZeroU32::new(1).unwrap());
                let start = Instant::now();
                for index in 0..1000u64 {
                    let first_ts = index * frames as u64;
                    let packet = RtpPacketMeta {
                        first_ts,
                        end_ts: first_ts + frames as u64,
                    };
                    let elapsed_ns = (first_ts as u128 * 1_000_000_000).div_ceil(rate as u128);
                    let now = start + Duration::from_nanos(elapsed_ns as u64);
                    let decision = pacer.inspect(now, [packet], 1);
                    assert_eq!(decision.permits, 1, "frames={frames}, rate={rate}, index={index}");
                    assert!(decision.wake_at.is_none());
                    pacer.commit(now, [packet]);
                }
            }
        }
    }

    #[test]
    fn partial_send_commits_only_sent_prefix() {
        let (mut recorder, mut producer) = recorder();
        producer.write(&vec![0; 960 * 4]);
        let wake_at = recorder
            .flush(&mut |batch| {
                assert_eq!(batch.len(), 4);
                Ok(1)
            })
            .unwrap();
        assert!(wake_at.is_some());
        assert_eq!(recorder.pending_transmit.len(), 3);
        assert_eq!(recorder.pacer.last_end_ts(), Some(480));
        assert_eq!(recorder.pending_transmit.front().unwrap().meta.first_ts, 480);
        assert_eq!(recorder.empty_buffers.len(), 13);
    }

    #[test]
    fn blocked_or_failed_send_keeps_packets_and_anchor() {
        let (mut recorder, mut producer) = recorder();
        producer.write(&vec![0; 960]);
        let wake_at = recorder
            .flush(&mut |_| Err(std::io::Error::from(std::io::ErrorKind::WouldBlock).into()))
            .unwrap();
        assert!(wake_at.is_some());
        assert!(!recorder.pacer.has_anchor());
        assert_eq!(recorder.pending_transmit.len(), 1);
        assert!(
            recorder
                .flush(&mut |_| Err(RtspError::DisconnectNow))
                .is_err()
        );
        assert!(!recorder.pacer.has_anchor());
        assert_eq!(recorder.pending_transmit.len(), 1);
        assert!(recorder.flush(&mut |_| Ok(1)).unwrap().is_none());
        assert!(recorder.pending_transmit.is_empty());
        assert_eq!(recorder.empty_buffers.len(), 16);
    }

    #[test]
    fn rtp_wrap_and_rewind_preserve_monotonic_pacer_timestamps() {
        let (mut recorder, mut producer) = recorder();
        recorder.ts = u32::MAX as u64 - 239;
        producer.write(&vec![0; 960 * 2]);
        recorder
            .flush(&mut |batch| {
                let first = RtpHeader::from_buf(batch[0]).unwrap();
                let second = RtpHeader::from_buf(batch[1]).unwrap();
                assert_eq!(first.timestamp, u32::MAX - 239);
                assert_eq!(second.timestamp, 240);
                Ok(batch.len())
            })
            .unwrap();
        let previous = recorder.pacer.last_end_ts().unwrap();
        recorder.rewind_rounding(1);
        producer.write(&vec![0; 960 * 2]);
        recorder.flush(&mut |batch| Ok(batch.len())).unwrap();
        assert_eq!(recorder.pacer.last_end_ts(), Some(previous + 960));
    }
}
