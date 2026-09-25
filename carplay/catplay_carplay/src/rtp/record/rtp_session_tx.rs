use std::{net::SocketAddr, num::NonZero};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{EventSleeper, notify::Notify};
use log::{debug, trace, warn};

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
    recorder_sink: Option<AudioRecorderBox<E::Sample>>,
}

impl<E: AudioEncoder> RtpSessionTx<E> {
    pub fn new(
        encryption: AirPlayStreamEncryption,
        stream_connection_id: u64,
        stream_type: StreamType,

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
        Self::new_with_cipher(cipher, stream_type, codec, recoder)
    }

    fn new_with_cipher(cipher: RtpCipher, stream_type: StreamType, codec: E, mut recoder: AudioRecorderBox<E::Sample>) -> RtspResult<Self> {
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
        let (recorder, ring) = RtpRecorder::new(codec, stream_type, samples_per_packet, frame_size_in_samples, cipher);

        // RtpSink -> owns audio ring producer side and provides data to AudioRecorder (sink)
        let sink = {
            let encoder_wakeup = encoder_wakeup.clone();
            RtpSink::new(ring, samples_per_packet, move || encoder_wakeup.notify())
        };

        recoder.init(Box::new(sink))?;

        Ok(Self {
            recorder: Some(recorder),
            recorder_notify: encoder_wakeup,
            recorder_sink: Some(recoder),
        })
    }

    pub fn start_recording(&mut self) {
        if let Some(recorder_sink) = self.recorder_sink.as_mut() {
            recorder_sink.start();
        }
    }

    fn flush_recorder(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> RtspResult<()> {
        if let Some(recorder) = self.recorder.as_mut()
            && let Some(peer) = peer
        {
            let ret = recorder.drain(move |batch| {
                let packets_prepared = batch.len();
                if packets_prepared == 0 {
                    return;
                }

                match peer.send_multiple(batch) {
                    Ok(n) if n < packets_prepared => warn!("Lost some outgoing RTP packets from batch: sent {n}/{packets_prepared}"),
                    Err(err) => warn!("Lost outgoing RTP packets due to UDP error: {err}"),
                    _ => trace!("Sent {packets_prepared} RTP packets"),
                }
            });

            if let Err(err) = ret {
                warn!("RTP recorder crash: {err:?}");
                return Err(RtspError::DisconnectNow);
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
