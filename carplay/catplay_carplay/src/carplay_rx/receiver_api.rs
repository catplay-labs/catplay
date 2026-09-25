use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_csm::decoder::CsmPacketBox;
use catplay_hap::HomekitStorageRef;
use catplay_iap2_client::{CsmSessionBox, CsmSessionError};
use catplay_mfi::MfiDeficeRef;
use catplay_util::{AsyncShutdownDyn, EventReconcilerDyn, EventSleeperDyn};

use crate::{
    audio::{AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription},
    carplay_rx::sink::AirPlayServerShared,
    clock::MediaClockBox,
    modes::{AirPlayModeState, AppState, ChangeModes, Resource, ResourceConstraint, ResourceID, ResourceTransferPriority},
    msg::{AudioFormat, AudioType, Command, HidDevice, InfoMessageResponse, StreamType},
    rtp::RtpReceiverError,
    rtsp_frame::{HttpStatus, RtspError, RtspFuture, RtspResponse, RtspResult},
    screen::rx::ScreenReceiverSinkBox,
};

#[derive(thiserror::Error, Debug)]
pub enum AirPlayReceiverSessionError {
    #[error("User sink terminated: {0}")]
    Sink(RtspError),
    #[error("RTSP socket terminated: {0}")]
    Rtsp(RtspError),
    #[error("Timing socket terminated: {0}")]
    Timing(RtspError),
    #[error("Keep-alive socket terminated: {0}")]
    KeepAlive(RtspError),
    #[error("Event socket terminated: {0}")]
    Events(RtspError),

    #[error("Screen session terminated: {0}")]
    Screen(RtspError),
    #[error("RTP stream terminated: {0}")]
    Rtp(#[from] RtpReceiverError),
    #[error("iAP2 session terminated: {0}")]
    IAp2(#[from] CsmSessionError),

    #[error("iAP2 sanity violation: {0}")]
    IAp2Sanity(RtspError),
}

pub enum SessionConflictPolicy {
    Replace,
    Reject,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AirPlayReceiverProfile {
    CarPlay,
    AppleTV,
}

/// Informations needed to bootstrap AirPlay receiver session.
pub struct AirPlayReceiverBootstrap {
    pub iface: String,
    pub homekit: HomekitStorageRef,
    pub mfi: Option<MfiDeficeRef>,
    pub shared: AirPlayServerShared,
    pub sink: AirPlayReceiverSinkBox,
    pub conflict: SessionConflictPolicy,
}

#[async_trait]
pub trait AirPlayReceiverHandle: Send + Sync {
    fn media_clock(&self) -> MediaClockBox;

    /// Send a generic command.
    fn send_command(&self, cmd: Command) -> RtspResult<RtspFuture>;

    /// Send a generic command (best-effort).
    fn send_command_and_forget(&self, cmd: Command);

    /// Send a HID report (best-effort).
    fn send_hid_report(&self, ts: Instant, device: &HidDevice, report: &[u8]);

    /// Send raw iAP2 packet.
    fn send_iap2(&self, csm: CsmPacketBox);

    /// Request keyframe after video decoder error (best-effort).
    fn request_keyframe(&self);

    /// Request UI to be shown. Required after screen Untake, among other things.
    fn request_ui(&self, url: &str);

    /// Request session termination due to unexpected state or user request.
    ///
    /// [RtspError::UserClosing] may be treated as a hint that session is terminated on user's request and that active reconnection attempts
    /// should be paused until user decides otherwise.
    ///
    /// The session is guaranteed to be terminated shortly after this call, but asynchronously.
    fn close(&self, err: RtspError);

    /// Get network interface name on which the session is running.
    fn iface(&self) -> &str;

    /// True if wireless session, false if USB session.
    fn is_wireless(&self) -> bool;

    async fn change_modes(&self, modes: ChangeModes) -> RtspResult<AirPlayModeState>;

    async fn change_resource_mode(&self, resource: Resource) -> RtspResult<AirPlayModeState>;

    async fn change_app_state(&self, app_state: AppState) -> RtspResult<AirPlayModeState>;

    // Screen helpers

    async fn take_screen(
        &self,
        priority: ResourceTransferPriority,
        take: ResourceConstraint,
        borrow: ResourceConstraint,
    ) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::take(ResourceID::MainScreen, priority, take, borrow))
            .await
    }

    async fn borrow_screen(&self, priority: ResourceTransferPriority, unborrow: ResourceConstraint) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::borrow(ResourceID::MainScreen, priority, unborrow))
            .await
    }

    async fn untake_screen(&self) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::untake(ResourceID::MainScreen))
            .await
    }

    async fn unborrow_screen(&self) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::unborrow(ResourceID::MainScreen))
            .await
    }

    // Audio helpers

    async fn take_audio(
        &self,
        priority: ResourceTransferPriority,
        take: ResourceConstraint,
        borrow: ResourceConstraint,
    ) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::take(ResourceID::MainAudio, priority, take, borrow))
            .await
    }

    async fn borrow_audio(&self, priority: ResourceTransferPriority, unborrow: ResourceConstraint) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::borrow(ResourceID::MainAudio, priority, unborrow))
            .await
    }

    async fn untake_audio(&self) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::untake(ResourceID::MainAudio))
            .await
    }

    async fn unborrow_audio(&self) -> RtspResult<AirPlayModeState> {
        self.change_resource_mode(Resource::unborrow(ResourceID::MainAudio))
            .await
    }

    fn is_car(&self) -> bool;

    fn is_mirroring(&self) -> bool;
}

pub type AirPlayReceiverHandleRef = Arc<dyn AirPlayReceiverHandle>;

#[allow(unused)]
#[async_trait]
pub trait AirPlayReceiverSink: AsyncShutdownDyn + EventReconcilerDyn<Error = RtspError> + EventSleeperDyn + Send + 'static {
    /// Called in "initial setup" phase.
    ///
    /// This is guaranteed to be the first callback to be received by the user, which can be used
    /// to prepare data for next setup-related callbacks.
    fn init(&mut self, session: AirPlayReceiverHandleRef) -> RtspResult<()> {
        Ok(())
    }

    /// Allows rejecting incoming sessions if Error is returned here.
    async fn on_initial_setup(&mut self) -> RtspResult<()> {
        Ok(())
    }

    /// Allows raw response to incoming AirPlay commands; this is called before additional convenience callbacks for given command, if any.
    async fn on_command_raw(&mut self, command: &Command) -> RtspResult<RtspResponse> {
        Ok(RtspResponse::new(None, HttpStatus::Ok))
    }

    /// Perform final overrides on the `Info` message.
    ///
    /// Important keys: `bluetooth_ids`, `model`, `manufacturer`, `hardware_revision`, `firmware_revision`, `audio_latencies`.
    async fn on_info(&mut self, info: &mut InfoMessageResponse) {}

    /// Open audio sink for PCM-decoded audio (for example, ALSA).
    ///
    /// In case of `MainAudio`/`MainHighAudio` any conflicting audio sources, like HU's radio stream, should be closed before this function returns.
    async fn open_audio(
        &mut self,
        latency: Duration,
        stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,
    ) -> RtspResult<AudioPlayerBox<i16>> {
        Err(RtspError::NotSupported)
    }

    /// Open screen sink for encoded video frames with presentation timestamps.
    ///
    /// It is user's responsibility to decode them and schedule for presentation
    /// as close as possible to the timestamp.
    ///
    /// Note that in some cases the session may continue with a closed video sink.
    async fn open_screen(&mut self, latency: Duration) -> RtspResult<ScreenReceiverSinkBox> {
        Err(RtspError::NotSupported)
    }

    /// Open microphone recorder sink for PCM-encoded audio (for example, ALSA).
    async fn open_microphone(
        &mut self,
        stream_type: StreamType,
        audio_type: AudioType,
        pcm_format: AudioStreamBasicDescription,
    ) -> RtspResult<AudioRecorderBox<i16>> {
        Err(RtspError::NotSupported)
    }

    /// Initialize borrow status of system resources like screen and audio at session's start.
    ///
    /// For example - a radio is already playing and iPhone shouldn't attempt to immediately take over with it's own audio stream.
    ///
    /// From that point, until end of session, iPhone has the authority over future borrow status of each resource.
    ///
    /// A change request can be sent to iPhone, but until confirmed by iPhone it shouldn't be considered as approved - until
    /// for example there's an emergency warning sound to be played.
    fn init_modes(&mut self) -> ChangeModes {
        ChangeModes::initial()
    }

    /// Informs about session start.
    async fn on_record(&mut self) -> RtspResult<()> {
        Ok(())
    }

    /// Called to open extra iAP2 session - only on wireless connection.
    async fn open_iap2(&mut self) -> Option<CsmSessionBox> {
        None
    }

    async fn on_modes(&mut self, modes: &AirPlayModeState) {}
}

pub type AirPlayReceiverSinkBox = Box<dyn AirPlayReceiverSink>;
pub type AirPlayReceiverSinkCallback = Arc<dyn Fn() -> AirPlayReceiverSinkBox + Send + Sync + 'static>;
