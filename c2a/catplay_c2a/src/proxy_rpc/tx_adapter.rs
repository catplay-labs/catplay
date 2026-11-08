use std::time::Duration;

use catplay_carplay::{
    audio::{AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription},
    carplay_rx::AirPlayReceiverHandleRef,
    modes::AirPlayModeState,
    msg::{AudioFormat, AudioType, Command, ControllerFeature, InfoMessageResponse, StreamType},
    rtsp_frame::{RtspError, RtspResponse, RtspResult},
    screen::rx::ScreenReceiverSinkBox,
};
use catplay_util::{mpsc, oneshot};
use log::{debug, info, warn};

use super::PhoneLease;
use crate::proxy_rpc::{ScreenProxyOp, proxy_screen::ScreenProxy};

pub struct TxAdapter {
    car: mpsc::UnboundedSender<TxAdapterOp>,
    lease: PhoneLease,
    pending_mic: Option<AudioRecorderBox<i16>>,
    handle: Option<AirPlayReceiverHandleRef>,
}

pub enum TxAdapterOp {
    Phone { lease: PhoneLease, op: PhoneOp },
    BeginHandover { ready: oneshot::Sender<RtspResult<()>> },
}

pub enum PhoneOp {
    IphoneConnected {
        handle: AirPlayReceiverHandleRef,
    },
    IphoneDisconnected,
    ScreenChannel {
        channel: mpsc::UnboundedReceiver<ScreenProxyOp>,
    },
    AltScreenChannel {
        channel: mpsc::UnboundedReceiver<ScreenProxyOp>,
        uuid: Option<String>,
    },
    NegotiateFeatures {
        offered: Vec<ControllerFeature>,
        enabled: oneshot::Sender<Vec<ControllerFeature>>,
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
    pub(crate) fn new(car: mpsc::UnboundedSender<TxAdapterOp>, lease: PhoneLease) -> Self {
        Self {
            car,
            lease,
            pending_mic: None,
            handle: None,
        }
    }

    pub fn live(&self) -> bool {
        self.lease.live()
    }

    fn send(&self, op: PhoneOp) {
        let _ = self.car.unbounded_send(TxAdapterOp::Phone {
            lease: self.lease.clone(),
            op,
        });
    }

    pub async fn proxy_cmd_and_spy(&mut self, command: &Command) -> RtspResult<RtspResponse> {
        // Received over Wi-Fi; blocks RTSP thread
        let os = oneshot::channel();
        let op = PhoneOp::Command {
            command: command.clone(),
            response: os.0,
        };
        self.send(op);
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
        self.send(PhoneOp::SetupAudio {
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
        if !self.lease.live() {
            return Err(RtspError::Closed);
        }
        self.lease.record();
        info!("iPhone RECORD confirmed for active session");
        Ok(())
    }

    pub async fn proxy_info(&mut self, info: &mut InfoMessageResponse) {
        let os = oneshot::channel();
        self.send(PhoneOp::PatchInfo {
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
        self.send(PhoneOp::ScreenChannel { channel });
        Ok(Box::new(screen.0))
    }

    pub async fn proxy_alt_screen(&mut self, _latency: Duration, uuid: Option<String>) -> RtspResult<ScreenReceiverSinkBox> {
        let screen = ScreenProxy::new(self.handle.clone().expect("handle missing"));
        let channel = screen.1;
        self.send(PhoneOp::AltScreenChannel { channel, uuid });
        Ok(Box::new(screen.0))
    }

    pub async fn proxy_features(&mut self, offered: &[ControllerFeature]) -> Vec<ControllerFeature> {
        let os = oneshot::channel();
        self.send(PhoneOp::NegotiateFeatures {
            offered: offered.into(),
            enabled: os.0,
        });
        os.1.await.unwrap_or_default()
    }

    pub fn on_init(&mut self, handle: AirPlayReceiverHandleRef) {
        // TODO move to initial setup

        self.send(PhoneOp::IphoneConnected { handle: handle.clone() });
        self.handle.replace(handle);
    }

    pub async fn on_shutdown(&mut self) {}

    pub fn on_drop(&mut self) {
        self.lease.end();
        self.send(PhoneOp::IphoneDisconnected);
    }

    pub async fn on_modes(&mut self, modes: AirPlayModeState) {
        let os = oneshot::channel();
        self.send(PhoneOp::Modes { modes, consumed: os.0 });
        let _ = os.1.await;
    }
}
