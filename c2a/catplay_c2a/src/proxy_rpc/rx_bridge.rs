use std::time::Duration;

use async_trait::async_trait;
use catplay_carplay::{
    audio::{AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription},
    carplay_rx::{AirPlayReceiverHandleRef, AirPlayReceiverSink},
    modes::AirPlayModeState,
    msg::{AudioFormat, AudioType, Command, InfoMessageResponse, StreamType},
    rtsp_frame::{RtspError, RtspResponse, RtspResult},
    screen::rx::ScreenReceiverSinkBox,
};
use catplay_iap2_client::CsmSessionBox;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper};

use crate::proxy_rpc::tx_adapter::TxAdapter;

pub struct CarPlayRxSession {
    tx: Option<TxAdapter>,
    handle: Option<AirPlayReceiverHandleRef>,
    reject: bool,
}

impl CarPlayRxSession {
    pub fn new(tx: TxAdapter) -> Self {
        Self {
            tx: Some(tx),
            handle: None,
            reject: false,
        }
    }

    pub fn reject() -> Self {
        Self {
            tx: None,
            handle: None,
            reject: true,
        }
    }

    fn tx(&mut self) -> &mut TxAdapter {
        self.tx.as_mut().expect("missing TX adapter")
    }

    fn reject_if_needed(&mut self) -> RtspResult<()> {
        if self.reject {
            return Err(RtspError::UnexpectedState("no TX session while RX has connected".into()));
        }
        Ok(())
    }
}

#[async_trait]
impl AirPlayReceiverSink for CarPlayRxSession {
    fn init(&mut self, session: AirPlayReceiverHandleRef) -> RtspResult<()> {
        self.reject_if_needed()?;
        self.handle.replace(session.clone());
        self.tx().on_init(session);
        Ok(())
    }

    async fn open_audio(
        &mut self,
        latency: Duration,
        stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,
    ) -> RtspResult<AudioPlayerBox<i16>> {
        self.reject_if_needed()?;
        self.tx()
            .proxy_audio(latency, stream_type, audio_type, audio_format, pcm_format, duplex)
            .await
    }

    async fn open_microphone(
        &mut self,
        stream_type: StreamType,
        audio_type: AudioType,
        pcm_format: AudioStreamBasicDescription,
    ) -> RtspResult<AudioRecorderBox<i16>> {
        self.reject_if_needed()?;
        self.tx()
            .proxy_microphone(stream_type, audio_type, pcm_format)
            .await
    }

    async fn on_record(&mut self) -> RtspResult<()> {
        self.reject_if_needed()?;
        self.tx().proxy_record().await
    }

    async fn open_iap2(&mut self) -> Option<CsmSessionBox> {
        None
    }

    async fn on_info(&mut self, info: &mut InfoMessageResponse) {
        if self.reject {
            return;
        }

        self.tx().proxy_info(info).await
    }

    async fn on_initial_setup(&mut self) -> RtspResult<()> {
        self.reject_if_needed()?;
        // TODO check busy status
        Ok(())
    }
    async fn open_screen(&mut self, latency: Duration) -> RtspResult<ScreenReceiverSinkBox> {
        self.reject_if_needed()?;
        self.tx().proxy_screen(latency).await
    }

    async fn on_command_raw(&mut self, command: &Command) -> RtspResult<RtspResponse> {
        self.reject_if_needed()?;
        self.tx().proxy_cmd_and_spy(command).await
    }

    async fn on_modes(&mut self, modes: &AirPlayModeState) {
        if let Some(tx) = self.tx.as_mut() {
            tx.on_modes(*modes).await
        }
    }
}

impl EventReconciler for CarPlayRxSession {
    type Error = RtspError;
}

impl EventSleeper for CarPlayRxSession {}

impl AsyncShutdown for CarPlayRxSession {
    async fn shutdown(&mut self) {
        if self.reject {
            return;
        }

        self.tx().on_shutdown().await;
    }
}

impl Drop for CarPlayRxSession {
    fn drop(&mut self) {
        if self.reject {
            return;
        }

        self.tx().on_drop();
    }
}
