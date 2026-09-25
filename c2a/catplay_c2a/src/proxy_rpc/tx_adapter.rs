use std::time::Duration;

use catplay_carplay::{
    audio::{AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription},
    carplay_rx::AirPlayReceiverHandleRef,
    modes::AirPlayModeState,
    msg::{AudioFormat, AudioType, Command, InfoMessageResponse, StreamType},
    rtsp_frame::{RtspError, RtspResponse, RtspResult},
    screen::rx::ScreenReceiverSinkBox,
};
use catplay_util::{mpsc, oneshot};
use log::{debug, info, warn};

use crate::proxy_rpc::{ScreenProxyOp, proxy_screen::ScreenProxy};

pub struct TxAdapter {
    car: mpsc::UnboundedSender<TxAdapterOp>,
    pending_mic: Option<AudioRecorderBox<i16>>,
    handle: Option<AirPlayReceiverHandleRef>,
}

pub enum TxAdapterOp {
    IphoneConnected {
        handle: AirPlayReceiverHandleRef,
    },
    IphoneDisconnected,
    ScreenChannel {
        channel: mpsc::UnboundedReceiver<ScreenProxyOp>,
    },
    Modes {
        modes: AirPlayModeState,
        consumed: oneshot::Sender<()>,
    },
    Command {
        command: Command,
        response: oneshot::Sender<RtspResponse>,
    },
    PatchInfo {
        info: InfoMessageResponse,
        patched: oneshot::Sender<InfoMessageResponse>,
    },
    SetupAudio {
        latency: Duration,
        stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,

        response: oneshot::Sender<RtspResult<(AudioPlayerBox<i16>, Option<AudioRecorderBox<i16>>)>>,
    },
}

impl TxAdapter {
    pub fn new(car: mpsc::UnboundedSender<TxAdapterOp>) -> Self {
        Self {
            car,
            pending_mic: None,
            handle: None,
        }
    }

    pub async fn proxy_cmd_and_spy(&mut self, command: &Command) -> RtspResult<RtspResponse> {
        // Received over Wi-Fi; blocks RTSP thread
        let os = oneshot::channel();
        let op = TxAdapterOp::Command {
            command: command.clone(),
            response: os.0,
        };
        let _ = self.car.unbounded_send(op);
        os.1.await.map_err(|_| RtspError::Closed)
    }

    pub async fn proxy_audio(
        &mut self,
        latency: Duration,
        stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,
    ) -> RtspResult<AudioPlayerBox<i16>> {
        let os = oneshot::channel();
        let _ = self.car.unbounded_send(TxAdapterOp::SetupAudio {
            latency,
            stream_type,
            audio_type,
            audio_format,
            pcm_format,
            duplex,
            response: os.0,
        });

        let (player, pending_mic) = os.1.await.map_err(|_| RtspError::Closed)??;
        self.pending_mic = pending_mic;
        Ok(player)
    }

    pub async fn proxy_microphone(
        &mut self,
        stream_type: StreamType,
        audio_type: AudioType,
        _pcm_format: AudioStreamBasicDescription,
    ) -> RtspResult<AudioRecorderBox<i16>> {
        match self.pending_mic.take() {
            Some(mic) => Ok(mic),
            None => {
                warn!("Attempted to open microphone stream, but none was pre-initialized! {stream_type:?}, {audio_type:?}");
                Err(RtspError::NotSupported)
            }
        }
    }

    pub async fn proxy_record(&mut self) -> RtspResult<()> {
        info!("iPhone has notified about RECORD (ignored for now)");
        Ok(())
    }

    pub async fn proxy_info(&mut self, info: &mut InfoMessageResponse) {
        let os = oneshot::channel();
        let _ = self.car.unbounded_send(TxAdapterOp::PatchInfo {
            info: info.clone(),
            patched: os.0,
        });
        match os.1.await {
            Err(_) => {
                debug!("Returning unpatched info")
            }
            Ok(v) => {
                *info = v;
            }
        }
    }

    pub async fn proxy_screen(&mut self, _latency: Duration) -> RtspResult<ScreenReceiverSinkBox> {
        let screen = ScreenProxy::new(self.handle.clone().expect("handle missing"));
        let channel = screen.1;
        let _ = self
            .car
            .unbounded_send(TxAdapterOp::ScreenChannel { channel });
        Ok(Box::new(screen.0))
    }

    pub fn on_init(&mut self, handle: AirPlayReceiverHandleRef) {
        // TODO move to initial setup

        let _ = self
            .car
            .unbounded_send(TxAdapterOp::IphoneConnected { handle: handle.clone() });
        self.handle.replace(handle);
    }

    pub async fn on_shutdown(&mut self) {}

    pub fn on_drop(&mut self) {
        let _ = self.car.unbounded_send(TxAdapterOp::IphoneDisconnected {});
    }

    pub async fn on_modes(&mut self, modes: AirPlayModeState) {
        let os = oneshot::channel();
        let _ = self
            .car
            .unbounded_send(TxAdapterOp::Modes { modes, consumed: os.0 });
        let _ = os.1.await;
    }
}
