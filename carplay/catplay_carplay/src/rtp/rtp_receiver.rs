use std::{net::SocketAddr, time::Duration};

use catplay_tokio::UdpHelper;
use catplay_util::{ArcBox, AsyncShutdown, EventReconciler, EventSleeper};
use log::{debug, warn};

use crate::{
    audio::{
        AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription,
        codec::{
            AudioDecoderFactory, AudioEncoderFactory,
            runtime::{AudioDecoderDispatch, AudioEncoderDispatch},
        },
    },
    cipher::AirPlayStreamEncryption,
    msg::{StreamDescriptionAudio, StreamDescriptionResponse, StreamType},
    rtp::{RTP_BUFFER_PAD, RTP_PACKET_MAX, RtcpSession, play::RtpSession, record::RtpSessionTx},
    rtsp_frame::{RtspError, RtspResult},
};

#[derive(EventSleeper, AsyncShutdown)]
pub struct RtpReceiver {
    _stream: StreamDescriptionAudio,

    #[sleep]
    #[shutdown(take)]
    socket_rx: Option<UdpHelper<RTP_PACKET_MAX, RTP_BUFFER_PAD, RtpSession<AudioDecoderDispatch>>>,
    #[sleep]
    #[shutdown(take)]
    socket_tx: Option<UdpHelper<RTP_PACKET_MAX, RTP_BUFFER_PAD, RtpSessionTx<AudioEncoderDispatch>>>,
    #[sleep]
    #[shutdown(take)]
    socket_ctrl: Option<UdpHelper<RTP_PACKET_MAX, RTP_BUFFER_PAD, RtcpSession>>,

    pub local_port_rtp: u16,
    pub local_port_rtcp: u16,
    pub latency: Duration,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum RtpReceiverError {
    #[error("Stream closed: {stream_type:?} receiver={receiver}: {error}")]
    StreamClosed {
        stream_type: StreamType,
        receiver: bool,
        #[source]
        error: ArcBox<RtspError>,
    },
}

impl RtpReceiver {
    fn effective_audio_encryption(encryption: AirPlayStreamEncryption, stream_type: StreamType, receiver: bool) -> AirPlayStreamEncryption {
        if stream_type.supports_legacy_audio_aes() {
            return encryption;
        }

        match encryption {
            AirPlayStreamEncryption::Aes { .. } => {
                warn!(
                    "[{}] Downgrading {:?} encryption from AES to None",
                    if receiver { "RX" } else { "TX" },
                    stream_type
                );
                AirPlayStreamEncryption::None
            }
            encryption => encryption,
        }
    }

    pub fn init_codec(desc: &StreamDescriptionAudio) -> RtspResult<AudioDecoderDispatch> {
        let asbd = AudioStreamBasicDescription::try_from(desc.audio_format).map_err(|_| RtspError::NotSupported)?;

        let codec = match AudioDecoderDispatch::new(asbd) {
            Err(err) => {
                debug!("Failed to initialize audio codec: {err:?}");
                return Err(RtspError::NotSupported);
            }
            Ok(v) => v,
        };

        Ok(codec)
    }

    fn new_common(
        mut bind_ip: SocketAddr,
        peer_ip: SocketAddr,
        d: &StreamDescriptionAudio,
        session: RtpSession<AudioDecoderDispatch>,
        mut session_tx: Option<RtpSessionTx<AudioEncoderDispatch>>,
    ) -> RtspResult<Self> {
        let mut socket_tx = None;
        let session_ctrl = RtcpSession {};

        bind_ip.set_port(0);
        let socket_rx = UdpHelper::bind(bind_ip, session)?;
        debug!("Bind RTP: {}", socket_rx.1);
        let socket_ctrl = UdpHelper::bind(bind_ip, session_ctrl)?;
        debug!("Bind RTCP: {}", socket_ctrl.1);

        if let Some(control_port) = d.control_port {
            let mut rtcp_remote = peer_ip;
            rtcp_remote.set_port(control_port);
            debug!("Bind RTCP reverse: {rtcp_remote}");
            socket_ctrl.0.connect_finish(rtcp_remote)?;
        }

        if let Some(mut session_tx) = session_tx.take() {
            debug!("Attaching recorder sink to RTP session");
            let Some(data_port) = d.data_port else {
                return Err(RtspError::ProtocolViolationGeneric);
            };

            let mut rtp_remote = peer_ip;
            rtp_remote.set_port(data_port);
            session_tx.start_recording();

            let (socket, rtp_remote_bind, _) = UdpHelper::connect(rtp_remote, session_tx)?;
            debug!("RTP connect tx: {rtp_remote_bind} -> {rtp_remote}");
            socket_tx.replace(socket);
        }

        Ok(Self {
            _stream: d.clone(),
            socket_rx: Some(socket_rx.0),
            socket_tx,
            socket_ctrl: Some(socket_ctrl.0),

            local_port_rtp: socket_rx.1.port(),
            local_port_rtcp: socket_ctrl.1.port(),
            latency: Duration::from_millis(d.audio_latency_ms),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bind_ip: SocketAddr,
        peer_ip: SocketAddr,
        mut encryption: AirPlayStreamEncryption,
        d: &StreamDescriptionAudio,

        // Player
        codec: AudioDecoderDispatch,
        sink: AudioPlayerBox<i16>,
        // Recorder (optional)
        codec_record: Option<AudioEncoderDispatch>,
        sink_record: Option<AudioRecorderBox<i16>>,
    ) -> RtspResult<Self> {
        encryption = Self::effective_audio_encryption(encryption, d.stream_type, true);

        let latency = Duration::from_millis(d.audio_latency_ms);
        let stream_connection_id = d.stream_connection_id;
        let session = RtpSession::new(encryption, stream_connection_id, latency, codec, sink, true)?;
        let session_tx = match (encryption, codec_record, sink_record) {
            (encryption, Some(codec_record), Some(sink_record)) => Some(RtpSessionTx::new(
                encryption,
                stream_connection_id,
                d.stream_type,
                d.audio_latency_ms,
                codec_record,
                sink_record,
                true,
            )?),
            _ => None,
        };

        Self::new_common(bind_ip, peer_ip, d, session, session_tx)
    }

    pub fn new_tx(
        mut bind_ip: SocketAddr,
        _peer_ip: SocketAddr,
        mut encryption: AirPlayStreamEncryption,
        d: &mut StreamDescriptionAudio,

        // Player (optional)
        player: Option<(AudioDecoderDispatch, AudioPlayerBox<i16>)>,
    ) -> RtspResult<Self> {
        encryption = Self::effective_audio_encryption(encryption, d.stream_type, false);

        let latency = Duration::from_millis(d.audio_latency_ms);
        let stream_connection_id = d.stream_connection_id;
        let mut socket_rx = None;
        let session_ctrl = RtcpSession {};

        bind_ip.set_port(0);

        if let Some((codec, sink)) = player {
            debug!("Attaching player sink to RTP TX session");
            let session = RtpSession::new(encryption, stream_connection_id, latency, codec, sink, false)?;
            let socket = UdpHelper::bind(bind_ip, session)?;
            d.data_port.replace(socket.1.port());
            socket_rx.replace(socket.0);
        }

        let socket_ctrl = UdpHelper::bind(bind_ip, session_ctrl)?;
        debug!("Bind RTCP: {}", socket_ctrl.1);
        d.control_port.replace(socket_ctrl.1.port());

        Ok(Self {
            _stream: d.clone(),
            socket_rx,
            socket_tx: None,
            socket_ctrl: Some(socket_ctrl.0),

            local_port_rtp: 0,
            local_port_rtcp: 0,
            latency,
        })
    }

    pub fn finish_tx(
        &mut self,
        _bind_ip: SocketAddr,
        peer_ip: SocketAddr,

        mut encryption: AirPlayStreamEncryption,

        d: &StreamDescriptionResponse,
        // Recorder (required)
        recorder: (AudioEncoderDispatch, AudioRecorderBox<i16>),
    ) -> RtspResult<()> {
        debug!("Attaching recorder sink to RTP TX session");

        encryption = Self::effective_audio_encryption(encryption, d.stream_type, false);

        let mut session = RtpSessionTx::new(
            encryption,
            d.stream_connection_id,
            d.stream_type,
            self._stream.audio_latency_ms,
            recorder.0,
            recorder.1,
            false,
        )?;
        let mut rtp_remote = peer_ip;
        rtp_remote.set_port(d.data_port);

        session.start_recording();

        if let Some(socket_ctrl) = self.socket_ctrl.as_mut()
            && let Some(control_port) = d.control_port
        {
            let mut rtcp_remote = peer_ip;
            rtcp_remote.set_port(control_port);
            debug!("Bind RTCP reverse: {rtcp_remote}");
            socket_ctrl.connect_finish(rtcp_remote)?;
        }

        let (socket, rtp_remote_bind, _) = UdpHelper::connect(rtp_remote, session)?;
        debug!("RTP connect tx: {rtp_remote_bind} -> {rtp_remote}");

        self.socket_tx.replace(socket);
        Ok(())
    }

    // Recorder side

    pub fn choose_input_pcm(output: AudioStreamBasicDescription) -> Option<AudioStreamBasicDescription> {
        // Decide which PCM variant should be used by the recorder
        // This is primarly just choosing PCM i16 + exact copy of sample rate and channel count from the requested output ASBD
        let input = AudioStreamBasicDescription::fill_pcm(output.sample_rate, 16, 16, output.channels(), false);
        Some(input)
    }

    pub fn init_codec_record(desc: &StreamDescriptionAudio) -> RtspResult<AudioEncoderDispatch> {
        let output = AudioStreamBasicDescription::try_from(desc.audio_format).map_err(|_| RtspError::NotSupported)?;
        let input = Self::choose_input_pcm(output).ok_or(RtspError::NotSupported)?;

        let codec = match AudioEncoderDispatch::new(input, output) {
            Err(err) => {
                debug!("Failed to initialize audio codec: {err:?}");
                return Err(RtspError::NotSupported);
            }
            Ok(v) => v,
        };

        Ok(codec)
    }
}

impl EventReconciler for RtpReceiver {
    type Error = RtpReceiverError;

    async fn reconcile(&mut self) -> Result<(), RtpReceiverError> {
        if let Some(socket_rx) = self.socket_rx.as_mut()
            && let Err(rx_status) = socket_rx.reconcile().await
        {
            let ret = rx_status.unwrap_or(RtspError::Closed);
            return Err(RtpReceiverError::StreamClosed {
                stream_type: self._stream.stream_type,
                receiver: true,
                error: ret.into(),
            });
        }

        if let Some(socket_tx) = self.socket_tx.as_mut()
            && let Err(tx_status) = socket_tx.reconcile().await
        {
            let ret = tx_status.unwrap_or(RtspError::Closed);
            return Err(RtpReceiverError::StreamClosed {
                stream_type: self._stream.stream_type,
                receiver: false,
                error: ret.into(),
            });
        }

        Ok(())
    }
}
