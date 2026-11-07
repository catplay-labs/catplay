use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{Deadline, EventSleeper};
use log::debug;

use crate::{
    audio::{AudioPlayerBox, AudioPlayerEvent, codec::AudioDecoder},
    cipher::AirPlayStreamEncryption,
    rtp::{AsRtpPacket, RtpAesCbcDecoder, RtpChaChaDecoder, RtpCipher, play::RtpPlayer},
    rtsp_frame::{RtspError, RtspResult},
};

#[derive(EventSleeper)]
pub struct RtpSession<C: AudioDecoder> {
    codec: RtpCipher,

    player: Option<RtpPlayer<C>>,
    sink: Option<AudioPlayerBox<C::Sample>>,
    playing: bool,
    anchored: bool,
    stopped: bool,
    #[sleep]
    start_deadline: Deadline,
}

impl<C: AudioDecoder> RtpSession<C> {
    pub fn new(
        encryption: AirPlayStreamEncryption,
        stream_connection_id: u64,
        latency_ms: Duration,
        codec: C,

        sink: AudioPlayerBox<C::Sample>,
        server: bool,
    ) -> RtspResult<Self> {
        let cipher = match encryption {
            AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => RtpCipher::None,
            AirPlayStreamEncryption::Aes { key, iv } => RtpCipher::AesCbc(RtpAesCbcDecoder::new(&key, &iv)),
            AirPlayStreamEncryption::ChaCha { shared_secret } => {
                RtpCipher::ChaCha(RtpChaChaDecoder::new(shared_secret, stream_connection_id, server))
            }
        };
        Self::new_with_cipher(cipher, latency_ms, codec, sink)
    }

    fn new_with_cipher(cipher: RtpCipher, latency_ms: Duration, codec: C, mut sink: AudioPlayerBox<C::Sample>) -> RtspResult<Self> {
        let format = codec.output_type();
        let sample_rate = format.sample_rate as usize;
        let frame_size_in_samples: usize = codec.output_type().channels_per_frame as _;

        let start = Instant::now();
        // RtpPlayer -> feeds audio ring as producer
        let (player, ring) = RtpPlayer::new(
            codec,
            sample_rate.try_into().unwrap(),
            frame_size_in_samples.try_into().unwrap(),
            latency_ms,
            start,
        );
        sink.init(ring, format)?;

        let mut me = Self {
            codec: cipher,
            player: Some(player),
            sink: Some(sink),
            playing: false,
            anchored: false,
            stopped: false,
            start_deadline: Deadline::Never,
        };

        me.start_playback_if_ready();
        Ok(me)
    }

    fn start_playback_if_ready(&mut self) {
        if self.stopped || self.playing {
            self.start_deadline = Deadline::Never;
            return;
        }
        let deadline = self
            .player
            .as_ref()
            .and_then(|player| player.start_deadline());
        match deadline {
            Some(at) if Instant::now() >= at => {
                self.playing = true;
                self.start_deadline = Deadline::Never;
                if let Some(sink) = self.sink.as_mut() {
                    sink.on_event(AudioPlayerEvent::Start);
                }
            }
            Some(at) => self.start_deadline = Deadline::new(at),
            None => self.start_deadline = Deadline::Never,
        }
    }

    fn stop(&mut self) {
        if !self.stopped {
            self.stopped = true;
            self.start_deadline = Deadline::Never;
            if let Some(sink) = self.sink.as_mut() {
                sink.on_event(AudioPlayerEvent::Stop);
            }
        }
    }

    fn on_recv(&mut self, data: &mut [u8]) -> RtspResult<()> {
        if self.stopped {
            return Ok(());
        }
        if !self.anchored {
            self.anchored = true;
            if let Some(sink) = self.sink.as_mut() {
                sink.on_event(AudioPlayerEvent::Anchor);
            }
        }
        // TODO: detect player `read_head` stall, rewind and restart
        // TODO: timeout when there are no incoming packets for too long

        let packet = self.codec.decode(data)?;
        debug!("New RTP packet: {:?}", packet.header());

        if let Some(player) = self.player.as_mut() {
            let written = player.total_written();
            player.push(&packet);
            if player.total_written() != written {
                if let Some(sink) = self.sink.as_mut() {
                    sink.on_event(AudioPlayerEvent::AddedDataToRing);
                }
            }
        }
        self.start_playback_if_ready();

        Ok(())
    }
}

impl<C: AudioDecoder> UdpSession for RtpSession<C> {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        self.on_recv(data)
    }

    fn on_eof(&mut self, error: Option<Self::Error>) {
        debug!("Detected Eof on RTP socket: {error:?}");
        self.stop();
    }

    fn reconcile(&mut self, _peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        self.start_playback_if_ready();
        Ok(())
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        debug!("Connected to remote {peer}");
        Ok(())
    }
}

impl<C: AudioDecoder> Drop for RtpSession<C> {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        audio::{AudioPlayer, AudioPlayerEvent, AudioStreamBasicDescription, RingConsumer},
        rtp::RtpHeader,
    };
    use std::{
        convert::Infallible,
        sync::{Arc, Mutex},
    };

    struct Decoder;

    impl AudioDecoder for Decoder {
        type Error = Infallible;
        type Sample = i16;

        fn output_type(&self) -> AudioStreamBasicDescription {
            AudioStreamBasicDescription::fill_pcm(48_000, 16, 16, 1, false)
        }

        fn decode(&mut self, data: &[u8], output: &mut [i16]) -> Result<(usize, usize), Infallible> {
            for (dst, src) in output.iter_mut().zip(data) {
                *dst = *src as i16;
            }
            Ok((data.len(), data.len()))
        }
    }

    #[derive(Default)]
    struct Playback {
        ring: Option<RingConsumer<i16>>,
        events: Vec<AudioPlayerEvent>,
    }

    struct Player(Arc<Mutex<Playback>>);

    impl AudioPlayer for Player {
        type Sample = i16;

        fn init(&mut self, ring: RingConsumer<i16>, _format: AudioStreamBasicDescription) -> RtspResult<()> {
            self.0.lock().unwrap().ring = Some(ring);
            Ok(())
        }

        fn on_event(&mut self, event: AudioPlayerEvent) {
            self.0.lock().unwrap().events.push(event);
        }
    }

    #[tokio::test]
    async fn latency_wakes_player_without_another_datagram() {
        let playback = Arc::new(Mutex::new(Playback::default()));
        let mut session = RtpSession::new_with_cipher(
            RtpCipher::None,
            Duration::from_millis(20),
            Decoder,
            Box::new(Player(playback.clone())),
        )
        .unwrap();
        assert!(matches!(session.start_deadline, Deadline::Never));
        assert!(playback.lock().unwrap().events.is_empty());
        // Even an invalid first datagram anchors the session. It neither
        // starts playback nor schedules Start without decoded audio.
        assert!(session.on_recv(&mut []).is_err());
        assert_eq!(playback.lock().unwrap().events, [AudioPlayerEvent::Anchor]);
        assert!(matches!(session.start_deadline, Deadline::Never));
        assert!(session.on_recv(&mut []).is_err());
        assert_eq!(playback.lock().unwrap().events, [AudioPlayerEvent::Anchor]);
        let header = RtpHeader::from_buf(&[0x80, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        let mut packet = header.to_buf().to_vec();
        packet.extend_from_slice(&[1, 2, 3, 4]);
        session.on_recv(&mut packet).unwrap();
        assert_eq!(
            playback.lock().unwrap().events,
            [AudioPlayerEvent::Anchor, AudioPlayerEvent::AddedDataToRing]
        );
        assert_eq!(
            playback
                .lock()
                .unwrap()
                .ring
                .as_mut()
                .unwrap()
                .stat()
                .consumed,
            0
        );

        assert!(
            tokio::time::timeout(Duration::from_secs(1), session.sleep())
                .await
                .unwrap()
                .is_some()
        );
        session.reconcile(None).unwrap();
        assert_eq!(
            playback.lock().unwrap().events,
            [AudioPlayerEvent::Anchor, AudioPlayerEvent::AddedDataToRing, AudioPlayerEvent::Start]
        );
        assert!(matches!(session.start_deadline, Deadline::Never));
        session.reconcile(None).unwrap();
        session.on_eof(None);
        drop(session);
        let mut playback = playback.lock().unwrap();
        assert_eq!(
            playback.events,
            [
                AudioPlayerEvent::Anchor,
                AudioPlayerEvent::AddedDataToRing,
                AudioPlayerEvent::Start,
                AudioPlayerEvent::Stop
            ]
        );
        assert_eq!(
            playback
                .ring
                .as_mut()
                .unwrap()
                .iter_segments(4, false)
                .collect::<Vec<_>>(),
            [crate::audio::RingSegment::Data(&[1, 2, 3, 4])]
        );
    }
}
