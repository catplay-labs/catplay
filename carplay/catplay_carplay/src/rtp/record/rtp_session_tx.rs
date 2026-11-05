use std::{net::SocketAddr, num::NonZero};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{Deadline, EventSleeper, notify::Notify};
use log::{debug, warn};

use crate::{
    audio::{AudioCodec, AudioRecorderBox, codec::AudioEncoder},
    cipher::AirPlayStreamEncryption,
    msg::StreamType,
    rtp::{
        RtpAesCbcDecoder, RtpChaChaDecoder, RtpCipher,
        record::{RtpRecorder, RtpSink},
    },
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

        let encoder_wakeup = Notify::new();
        // RtpRecorder -> owns audio ring consumer side
        let (recorder, ring) = RtpRecorder::new(codec, stream_type, latency_ms, samples_per_packet, frame_size_in_samples, cipher);

        // RtpSink -> owns audio ring producer side and provides data to AudioRecorder (sink)
        let sink = {
            let encoder_wakeup = encoder_wakeup.clone();
            RtpSink::new(ring, samples_per_packet, move || encoder_wakeup.notify())
        };

        recoder.init(Box::new(sink))?;

        Ok(Self {
            recorder: Some(recorder),
            recorder_notify: encoder_wakeup,
            transmit_deadline: Deadline::Never,
            recorder_sink: Some(recoder),
        })
    }

    pub fn start_recording(&mut self) {
        if let Some(recorder_sink) = self.recorder_sink.as_mut() {
            recorder_sink.start();
        }
    }

    fn flush_recorder(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        self.transmit_deadline = Deadline::Never;
        if let (Some(recorder), Some(peer)) = (self.recorder.as_mut(), peer) {
            match recorder.flush(&mut |batch| peer.send_multiple(batch)) {
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
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        debug!("Connected to remote {peer}");
        // self.start_recording();
        Ok(())
    }
}

impl<E: AudioEncoder> Drop for RtpSessionTx<E> {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::super::rtp_recorder::tests::{TestEncoder, recorder};
    use super::*;
    use std::{cell::Cell, time::Duration};

    struct Peer {
        blocked: Cell<bool>,
        sent: Cell<usize>,
    }

    impl UdpSocketPeer<RtpSessionTx<TestEncoder>> for Peer {
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
        let mut session = RtpSessionTx {
            recorder: Some(recorder),
            recorder_notify: Notify::new(),
            transmit_deadline: Deadline::Never,
            recorder_sink: None,
        };
        let peer = Peer {
            blocked: Cell::new(true),
            sent: Cell::new(0),
        };
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
}
