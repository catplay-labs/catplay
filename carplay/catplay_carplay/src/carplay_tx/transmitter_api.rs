use std::{net::SocketAddr, time::Duration};

use uuid::Uuid;

use crate::{
    audio::{AudioPlayerBox, AudioRecorderBox},
    carplay_tx::TeardownGuard,
    clock::MediaClockBox,
    events::CommandPending,
    modes::AirPlayModeState,
    msg::{AudioFormat, AudioType, Command, ControllerFeature, InfoMessageResponse, StreamType},
    pairing::PairingErrorTx,
    rtp::RtpReceiverError,
    rtsp_frame::{RtspError, RtspFuture, RtspResult},
    screen::tx::ScreenTransmitProxy,
};
use catplay_hap::HomekitStorageRef;

/// Informations needed to bootstrap AirPlay transmitter session.
pub struct AirPlayTransmitterBootstrap {
    /// HomeKit database
    pub homekit: HomekitStorageRef,
    /// IP of the remote since this is an outgoing connection.
    pub peer_ip: SocketAddr,
    /// A set of optional features that may be matched in response by the target device.
    pub controller_features: Vec<ControllerFeature>,
    /// HomeKit UUID of the remote(from Bonjour); used for fast-path in the pairing flow.
    pub remote_homekit_id: Option<Uuid>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum AirPlayTransmitterBootstrapError {
    #[error("Failed to bind local socket: {0}")]
    FailedToBind(RtspError),

    #[error("Failed to connnect TCP: {0}")]
    FailedToConnect(RtspError),
    #[error("Unexpected response to initial SETUP: {0}")]
    FailedInitialSetup(RtspError),
    #[error("Unexpected response to /info: {0}")]
    FailedInfo(RtspError),

    #[error("Failed to pair: {0}")]
    Pair(#[from] PairingErrorTx),
    #[error("Failed to auth-setup: {0}")]
    AuthSetup(PairingErrorTx),
}

pub type AirPlayTransmitterBootstrapResult<T> = Result<T, AirPlayTransmitterBootstrapError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum StreamStateError {
    #[error("Socket: {0}")]
    Socket(#[from] RtspError),
    #[error("{0}")]
    Rtp(#[from] RtpReceiverError),
    #[error("Teardown error: {0}")]
    Teardown(RtspError),
}

impl From<Option<RtspError>> for StreamStateError {
    fn from(value: Option<RtspError>) -> Self {
        match value {
            None => Self::Socket(RtspError::Closed),
            Some(err) => Self::Socket(err),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum AirPlayTransmitterSessionError {
    #[error("Failed teardown: {0}")]
    Teardown(RtspError),

    #[error("RTSP stream has disconnected: {0}")]
    Disconnected(RtspError),
    #[error("Events stream has disconnected: {0}")]
    EventsDisconnected(RtspError),
    #[error("Keep alive has disconnected: {0}")]
    KeepAliveDisconnected(RtspError),
    #[error("Timing socket has disconnected: {0}")]
    TimingDisconnected(RtspError),

    #[error("Media stream {0:?} has disconnected: {1}")]
    StreamDisconnected(StreamType, StreamStateError),
}

/// Represents an AirPlay session ready for opening user-defined set of media streams.
///
/// Such session already passed stages of TCP connect, HomeKit pairing, initial setup and `/info`.
#[allow(async_fn_in_trait)]
pub trait AirPlayTransmitter: Send + 'static {
    /// Wait for transmitter to get closed, or return instantly with a cached error.
    async fn closed(&self) -> AirPlayTransmitterSessionError;

    /// Close the transmitter if not closed already and wait for the background tasks to terminate.
    async fn shutdown(&self);

    fn info_cached(&self) -> &InfoMessageResponse;

    /// Controller features enabled by the remote in response to the initial SETUP.
    fn enabled_features(&self) -> &[ControllerFeature];

    fn media_clock(&self) -> MediaClockBox;

    /// Pop next queued incoming command, if any.
    ///
    /// A response should be sent (preferably - within current async call, but it's not a hard requirement) using a provided callback.
    ///
    /// Dropping the callback will result in sending a generic error response.
    async fn pop_command(&self) -> Option<CommandPending>;

    /// Send a command to remote (response payload and error code can be parsed from `RtspResponse`).
    fn send_command_noresp(&self, cmd: &Command, timeout: Duration) -> RtspResult<RtspFuture>;

    /// Setup screen stream.
    async fn setup_screen(&self, latency: Duration) -> RtspResult<TeardownGuard<ScreenTransmitProxy>>;

    /// Setup alt screen stream (for example, an instrument cluster) for the display with the given `uuid`.
    ///
    /// Requires the [ControllerFeature::AltScreen] feature to be enabled by the remote.
    async fn setup_alt_screen(&self, latency: Duration, uuid: Option<String>) -> RtspResult<TeardownGuard<ScreenTransmitProxy>>;

    /// Setup audio stream.
    async fn setup_audio(
        &self,
        latency: Duration,
        stream_type: StreamType,
        audio_format: AudioFormat,
        audio_type: AudioType,
        // Player (optional)
        player: Option<AudioPlayerBox<i16>>,
        // Recorder (required)
        recorder: AudioRecorderBox<i16>,
    ) -> RtspResult<TeardownGuard<StreamType>>;

    /// Formally request session's RECORD after streams were setup.
    async fn record(&self) -> RtspResult<()>;

    async fn assert_modes(&self, modes: AirPlayModeState) -> RtspResult<()>;

    /// Drain TEARDOWN queue populated by dropping stream guards/proxies.
    async fn drain_teardown_queue(&self) -> RtspResult<()>;
}

pub type AirPlayTransmitterBox = Box<dyn AirPlayTransmitter>;
