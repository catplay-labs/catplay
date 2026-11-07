use std::{net::SocketAddr, num::NonZero, time::Duration};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{Deadline, EventSleeper, notify::Notify};
use log::{debug, warn};

use crate::{
    audio::{AudioCodec, AudioRecorderBox, AudioRecorderControl, AudioRecorderEvent, codec::AudioEncoder},
    cipher::AirPlayStreamEncryption,
    msg::StreamType,
    rtp::{RtpAesCbcDecoder, RtpChaChaDecoder, RtpCipher, record::RtpRecorder},
    rtsp_frame::{RtspError, RtspResult},
};

#[derive(EventSleeper)]
pub struct RtpSessionTx<E: AudioEncoder> {
    recorder: Option<RtpRecorder<E>>,
    #[sleep]
    recorder_notify: Notify,
    #[sleep]
    transmit_deadline: Deadline,
    recorder_sink: Option<AudioRecorderBox<E::Sample>>,
    control: AudioRecorderControl,
    ready: bool,
    stopped: bool,
    applied_pause: Duration,
}

impl<E: AudioEncoder> RtpSessionTx<E> {
    pub fn new(
        encryption: AirPlayStreamEncryption,
        stream_connection_id: u64,
        stream_type: StreamType,
        latency_ms: u64,

        codec: E,
        recoder: AudioRecorderBox<E::Sample>,
        server: bool,
    ) -> RtspResult<Self> {
        let cipher = match encryption {
            AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => RtpCipher::None,
            AirPlayStreamEncryption::Aes { key, iv } => RtpCipher::AesCbc(RtpAesCbcDecoder::new(&key, &iv)),
            AirPlayStreamEncryption::ChaCha { shared_secret } => {
                RtpCipher::ChaCha(RtpChaChaDecoder::new(shared_secret, stream_connection_id, server))
            }
        };
        Self::new_with_cipher(cipher, stream_type, latency_ms, codec, recoder)
    }

    fn new_with_cipher(
        cipher: RtpCipher,
        stream_type: StreamType,
        latency_ms: u64,
        codec: E,
        mut recoder: AudioRecorderBox<E::Sample>,
    ) -> RtspResult<Self> {
        const MAX_FRAMES_PER_PACKET_PCM: usize = 352;

        let mut samples_per_packet = (codec.output_type().frames_per_packet as usize * codec.output_type().channels() as usize)
            .try_into()
            .unwrap();
        let frame_size_in_samples: NonZero<usize> = (codec.output_type().channels_per_frame as usize)
            .try_into()
            .unwrap();

        if codec.output_type().format == AudioCodec::LinearPcm {
            samples_per_packet = (MAX_FRAMES_PER_PACKET_PCM * frame_size_in_samples.get())
                .try_into()
                .unwrap();
        }

        let control = AudioRecorderControl::default();
        let format = codec.input_type();
        // RtpRecorder -> owns audio ring consumer side
        let (recorder, ring) = RtpRecorder::new(codec, stream_type, latency_ms, samples_per_packet, frame_size_in_samples, cipher);

        recoder.init(ring, format, control.clone())?;

        Ok(Self {
            recorder: Some(recorder),
            recorder_notify: control.notify.clone(),
            transmit_deadline: Deadline::Never,
            recorder_sink: Some(recoder),
            control,
            ready: false,
            stopped: false,
            applied_pause: Duration::ZERO,
        })
    }

    fn stop(&mut self) {
        if !self.stopped {
            self.stopped = true;
            self.control.pause();
            self.transmit_deadline = Deadline::Never;
            if let Some(sink) = self.recorder_sink.as_mut() {
                sink.on_event(AudioRecorderEvent::Stop);
            }
        }
    }

    fn flush_recorder(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        self.transmit_deadline = Deadline::Never;
        let (running, paused_total) = self.control.playback_state();
        if self.stopped || !self.ready || !running {
            return Ok(());
        }
        if let (Some(recorder), Some(peer)) = (self.recorder.as_mut(), peer) {
            recorder.delay_pacing(paused_total - self.applied_pause);
            self.applied_pause = paused_total;
            let consumed = recorder.total_consumed();
            let result = recorder.flush(&mut |batch| peer.send_multiple(batch));
            if recorder.total_consumed() != consumed {
                if let Some(sink) = self.recorder_sink.as_mut() {
                    sink.on_event(AudioRecorderEvent::DrainedDataFromRing);
                }
            }
            match result {
                Ok(wake_at) => {
                    self.transmit_deadline = wake_at.map(Deadline::new).unwrap_or(Deadline::Never);
                }
                Err(err) => {
                    warn!("RTP recorder crash: {err:?}");
                    return Err(RtspError::DisconnectNow);
                }
            }
        }
        Ok(())
    }
}

impl<E: AudioEncoder> UdpSession for RtpSessionTx<E> {
    type Error = RtspError;

    fn on_datagram(&mut self, _data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reconcile(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> Result<(), Self::Error> {
        self.flush_recorder(peer)
    }

    fn on_eof(&mut self, error: Option<Self::Error>) {
        debug!("Detected Eof on RTP Tx socket: {error:?}");
        self.stop();
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        debug!("Connected to remote {peer}");
        if !self.ready && !self.stopped {
            self.ready = true;
            if let Some(sink) = self.recorder_sink.as_mut() {
                sink.on_event(AudioRecorderEvent::Start);
            }
        }
        Ok(())
    }
}

impl<E: AudioEncoder> Drop for RtpSessionTx<E> {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::super::rtp_recorder::tests::{TestEncoder, recorder};
    use super::*;
    use crate::audio::{AudioRecorder, AudioStreamBasicDescription, RingProducer};
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
        time::Duration,
    };

    struct Peer {
        blocked: Cell<bool>,
        sent: Cell<usize>,
    }

    impl<E: AudioEncoder> UdpSocketPeer<RtpSessionTx<E>> for Peer {
        fn send(&self, _: &[u8]) -> RtspResult<()> {
            unreachable!()
        }

        fn send_multiple(&self, batch: &[&[u8]]) -> RtspResult<usize> {
            if self.blocked.replace(false) {
                return Ok(0);
            }
            self.sent.set(self.sent.get() + batch.len());
            Ok(batch.len())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pending_transmit_wakes_session_without_new_audio() {
        let (recorder, mut producer) = recorder();
        producer.write(&vec![0; 960]);
        let control = AudioRecorderControl::default();
        control.start();
        let mut session = RtpSessionTx {
            recorder: Some(recorder),
            recorder_notify: control.notify.clone(),
            transmit_deadline: Deadline::Never,
            recorder_sink: None,
            control,
            ready: true,
            stopped: false,
            applied_pause: Duration::ZERO,
        };
        let peer = Peer {
            blocked: Cell::new(true),
            sent: Cell::new(0),
        };
        // Consume start() notification so only the transmit deadline can wake us.
        session.sleep().await;
        session.reconcile(Some(&peer)).unwrap();
        assert!(matches!(session.transmit_deadline, Deadline::SleepUntil(_)));
        assert_eq!(peer.sent.get(), 0);

        let event = tokio::time::timeout(Duration::from_secs(1), session.sleep())
            .await
            .unwrap();
        assert!(event.is_some());
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(peer.sent.get(), 1);
        assert!(matches!(session.transmit_deadline, Deadline::Never));
    }

    #[derive(Default)]
    struct Capture {
        ring: Option<RingProducer<i16>>,
        control: Option<AudioRecorderControl>,
        events: Vec<AudioRecorderEvent>,
    }

    struct Sink(Arc<Mutex<Capture>>);

    impl AudioRecorder for Sink {
        type Sample = i16;

        fn init(&mut self, ring: RingProducer<i16>, _format: AudioStreamBasicDescription, control: AudioRecorderControl) -> RtspResult<()> {
            let mut capture = self.0.lock().unwrap();
            capture.ring = Some(ring);
            capture.control = Some(control);
            Ok(())
        }

        fn on_event(&mut self, event: AudioRecorderEvent) {
            self.0.lock().unwrap().events.push(event);
        }
    }

    fn session<E: AudioEncoder<Sample = i16>>(encoder: E) -> (RtpSessionTx<E>, Arc<Mutex<Capture>>, Peer) {
        let capture = Arc::new(Mutex::new(Capture::default()));
        let session =
            RtpSessionTx::new_with_cipher(RtpCipher::None, StreamType::MainAudio, 32, encoder, Box::new(Sink(capture.clone()))).unwrap();
        (
            session,
            capture,
            Peer {
                blocked: Cell::new(false),
                sent: Cell::new(0),
            },
        )
    }

    #[tokio::test]
    async fn buffered_audio_waits_for_user_start_and_transport() {
        let (mut session, capture, peer) = session(TestEncoder);
        {
            let mut capture = capture.lock().unwrap();
            capture.ring.as_mut().unwrap().write(&[1; 704]);
            capture.control.as_ref().unwrap().added_data_to_ring();
        }
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 0);
        assert_eq!(peer.sent.get(), 0);
        let control = capture.lock().unwrap().control.as_ref().unwrap().clone();
        control.start();
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 0);
        session
            .on_connect("127.0.0.1:1234".parse().unwrap(), &peer)
            .unwrap();
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 704);
        assert_eq!(peer.sent.get(), 1);
        assert_eq!(
            capture.lock().unwrap().events,
            [AudioRecorderEvent::Start, AudioRecorderEvent::DrainedDataFromRing]
        );

        control.pause();
        capture
            .lock()
            .unwrap()
            .ring
            .as_mut()
            .unwrap()
            .write(&[2; 704]);
        control.added_data_to_ring();
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 704);
        assert!(matches!(session.transmit_deadline, Deadline::Never));
        // Consume all pending notifications; start() alone must wake the session.
        session.sleep().await;
        control.start();
        assert!(
            tokio::time::timeout(Duration::from_secs(1), session.sleep())
                .await
                .unwrap()
                .is_some()
        );
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 1408);
        session.on_eof(None);
        session.reconcile(Some(&peer)).unwrap();
        drop(session);
        assert_eq!(
            capture
                .lock()
                .unwrap()
                .events
                .iter()
                .filter(|event| **event == AudioRecorderEvent::Stop)
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn explicit_write_notification_wakes_encoder_and_gaps_notify_drain() {
        let (mut session, capture, peer) = session(TestEncoder);
        session
            .on_connect("127.0.0.1:1234".parse().unwrap(), &peer)
            .unwrap();
        let control = capture.lock().unwrap().control.as_ref().unwrap().clone();
        control.start();
        session.sleep().await;
        session.reconcile(Some(&peer)).unwrap();
        capture
            .lock()
            .unwrap()
            .ring
            .as_mut()
            .unwrap()
            .advance_timeline(100);
        control.added_data_to_ring();
        assert!(
            tokio::time::timeout(Duration::from_secs(1), session.sleep())
                .await
                .unwrap()
                .is_some()
        );
        session.reconcile(Some(&peer)).unwrap();
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 704);
        assert_eq!(peer.sent.get(), 0);
        assert_eq!(
            capture.lock().unwrap().events,
            [AudioRecorderEvent::Start, AudioRecorderEvent::DrainedDataFromRing]
        );
    }

    struct FailedEncoder;

    impl AudioEncoder for FailedEncoder {
        type Sample = i16;
        type Error = std::io::Error;

        fn input_type(&self) -> AudioStreamBasicDescription {
            TestEncoder.input_type()
        }
        fn output_type(&self) -> AudioStreamBasicDescription {
            TestEncoder.output_type()
        }
        fn encode(&mut self, _samples: &[i16], _output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
            Err(std::io::Error::other("test codec failure"))
        }
    }

    #[test]
    fn codec_error_still_notifies_freed_capacity() {
        let (mut session, capture, peer) = session(FailedEncoder);
        session
            .on_connect("127.0.0.1:1234".parse().unwrap(), &peer)
            .unwrap();
        {
            let mut capture = capture.lock().unwrap();
            capture.ring.as_mut().unwrap().write(&[1; 704]);
            capture.control.as_ref().unwrap().start();
        }
        assert!(session.reconcile(Some(&peer)).is_err());
        assert_eq!(session.recorder.as_mut().unwrap().total_consumed(), 704);
        assert_eq!(
            capture.lock().unwrap().events,
            [AudioRecorderEvent::Start, AudioRecorderEvent::DrainedDataFromRing]
        );
    }
}
