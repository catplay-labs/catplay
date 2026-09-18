use catplay_tokio::TcpHelper;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, deadline, mpsc};
use futures::{FutureExt, StreamExt};
use log::{debug, info, trace, warn};
use std::time::{Duration, Instant};

use crate::{
    audio::{AudioPlayerBox, AudioRecorderBox, codec::AudioEncoder},
    carplay_tx::{
        AirPlayTransmitterBootstrap, AirPlayTransmitterBootstrapError, AirPlayTransmitterBootstrapSession, AirPlayTransmitterSessionError,
        StreamState, TeardownGuard, transmitter::AirPlayTransmitterBootstrapStreams,
    },
    cipher::AirPlayStreamEncryption,
    events::CommandPending,
    msg::{
        AudioFormat, AudioType, Command, FeedbackPayload, Setup, StreamDescriptionAudio, StreamDescriptionScreen, StreamType,
    },
    rtp::RtpReceiver,
    rtsp_frame::{RtspError, RtspFuture, RtspResult},
    screen::tx::{ScreenTransmitProxy, ScreenTransmitSession},
};

#[derive(EventSleeper, EventReconciler, AsyncShutdown)]
#[reconcile_error(AirPlayTransmitterSessionError)]
#[reconcile_func(send_feedback)]
#[reconcile_func(reconcile)]
#[sleep(deadline(self.next_feedback))]
pub struct AirPlayTransmitterImpl {
    #[sleep]
    #[reconcile]
    #[shutdown]
    streams: AirPlayTransmitterBootstrapStreams,
    #[sleep]
    #[reconcile]
    #[shutdown]
    media: MediaStreams,

    #[sleep]
    teardown_queue: mpsc::UnboundedReceiver<StreamType>,
    teardown_queue_tx: mpsc::UnboundedSender<StreamType>,
    teardown_error: Option<RtspError>,

    next_feedback: Instant,
}

#[derive(EventSleeper, EventReconciler, AsyncShutdown)]
#[reconcile_error(AirPlayTransmitterSessionError)]
struct MediaStreams {
    command_pending: Option<CommandPending>,

    #[sleep]
    #[reconcile(|err| AirPlayTransmitterSessionError::StreamDisconnected(self.screen.stream_type(), err))]
    #[shutdown]
    screen: StreamState<TcpHelper<ScreenTransmitSession>>,

    #[sleep]
    #[reconcile(|err| AirPlayTransmitterSessionError::StreamDisconnected(self.main_audio.stream_type(), err))]
    #[shutdown]
    main_audio: StreamState<RtpReceiver>,
    #[sleep]
    #[reconcile(|err| AirPlayTransmitterSessionError::StreamDisconnected(self.alt_audio.stream_type(), err))]
    #[shutdown]
    alt_audio: StreamState<RtpReceiver>,
}

impl Default for MediaStreams {
    fn default() -> Self {
        Self {
            command_pending: Default::default(),
            main_audio: StreamState::new(StreamType::MainAudio),
            alt_audio: StreamState::new(StreamType::AltAudio),
            screen: StreamState::new(StreamType::Screen),
        }
    }
}

impl AirPlayTransmitterImpl {
    const TCP_CONN_TIMEOUT_SCREEN: Duration = Duration::from_millis(2000);
    const FEEDBACK_INTERVAL: Duration = Duration::from_millis(1000);

    pub fn new(streams: AirPlayTransmitterBootstrapStreams) -> Self {
        let (teardown_queue_tx, teardown_queue) = mpsc::unbounded();

        Self {
            streams,
            media: MediaStreams::default(),
            teardown_queue,
            teardown_queue_tx,
            teardown_error: Default::default(),
            next_feedback: Instant::now(),
        }
    }

    pub async fn connect(bootstrap: AirPlayTransmitterBootstrap) -> Result<Self, AirPlayTransmitterBootstrapError> {
        let b = AirPlayTransmitterBootstrapSession::connect(bootstrap).await?;
        Ok(Self::new(b))
    }

    pub fn raw_streams(&mut self) -> &mut AirPlayTransmitterBootstrapStreams {
        &mut self.streams
    }

    pub fn pop_command(&mut self) -> Option<CommandPending> {
        self.media.command_pending.take()
    }

    pub async fn drain_teardown_queue(&mut self) -> RtspResult<()> {
        // This method is cancel-safe
        let media = &mut self.media;

        loop {
            let client = self.streams.client.clone().unwrap();

            while let Some(stream_type) = self.teardown_queue.next().now_or_never().flatten() {
                debug!("Received from TEARDOWN queue: {stream_type:?}");
                if let Some(stream) = Self::find_audio_stream(media, stream_type) {
                    stream.teardown_audio(client.clone(), false);
                } else if stream_type == StreamType::Screen {
                    media.screen.teardown_screen(client.clone(), true);
                } else {
                    debug!("Unknown stream? {stream_type:?}");
                }
            }

            let mut drained = false;

            if let Some(result) = media.main_audio.drain_teardown().await {
                result?;
                drained = true;
            }
            if let Some(result) = media.alt_audio.drain_teardown().await {
                result?;
                drained = true;
            }
            if let Some(result) = media.screen.drain_teardown().await {
                result?;
                drained = true;
            }

            if !drained {
                return Ok(());
            }
        }
    }

    pub async fn is_idle(&mut self) -> bool {
        let media = &mut self.media;

        for stream in [&mut media.main_audio, &mut media.alt_audio] {
            if stream.is_connected() {
                return false;
            }
        }

        if media.screen.is_connected() {
            return false;
        }

        true
    }

    fn find_audio_stream(s: &mut MediaStreams, stream_type: StreamType) -> Option<&mut StreamState<RtpReceiver>> {
        Some(match stream_type {
            StreamType::MainAudio | StreamType::MainHighAudio => &mut s.main_audio,
            StreamType::AltAudio => &mut s.alt_audio,
            _ => return None,
        })
    }

    pub fn command(&mut self, cmd: &Command, timeout: Duration) -> RtspResult<RtspFuture> {
        let client = self.streams.client.as_ref().unwrap();
        client.with_timeout(timeout).command_unchecked(cmd)
    }

    pub async fn do_record(&mut self) -> RtspResult<()> {
        let client = self.streams.client.as_ref().unwrap();
        client.record().await
    }

    pub async fn do_setup_audio(
        &mut self,
        mut latency: Duration,
        stream_type: StreamType,
        audio_format: AudioFormat,
        audio_type: AudioType,
        // Player (optional)
        player: Option<AudioPlayerBox<i16>>,
        // Recorder (required)
        recorder: AudioRecorderBox<i16>,
    ) -> RtspResult<TeardownGuard<StreamType>> {
        // Sanity: clamp to 32..2000ms to prevent insane memory allocations
        latency = latency.clamp(Duration::from_millis(32), Duration::from_millis(2000));

        let stream = Self::find_audio_stream(&mut self.media, stream_type).ok_or(RtspError::ProtocolViolationGeneric)?;
        match stream {
            StreamState::Unconnected { .. } => {}
            StreamState::Stream { stream_type, .. } => return Err(RtspError::StreamConflict(*stream_type)),
            StreamState::Teardown { .. } => {
                if let Some(Err(err)) = stream.drain_teardown().await {
                    return Err(err);
                }
            }
        }

        let id = match self.streams.cipher {
            AirPlayStreamEncryption::ChaCha { .. } => rand::random::<u64>().saturating_add(1),
            _ => 0,
        };
        let client = self.streams.client.as_ref().ok_or(RtspError::Unknown)?;

        let mut desc = StreamDescriptionAudio::new(
            id,
            latency,
            stream_type,
            audio_format,
            audio_type,
            None,
            None,
            player.is_some(),
            false,
            true,
        );

        let encoder = RtpReceiver::init_codec_record(&desc)?;
        let _sample_rate = encoder.output_type().sample_rate;

        let mut player_pair = None;
        if let Some(player) = player {
            let decoder = RtpReceiver::init_codec(&desc)?;
            player_pair.replace((decoder, player));
        }

        let mut rtp = RtpReceiver::new_tx(
            self.streams.bind_ip,
            self.streams.peer_ip,
            self.streams.cipher,
            &mut desc,
            player_pair,
        )
        .unwrap();
        let setup = Setup::new(&[desc.into()]);

        let guard = Self::create_guard(stream_type, stream_type, self.teardown_queue_tx.clone());

        info!("Audio setup request to send: {setup:?}");

        let resp = client.setup(setup).await?;
        let resp = resp.streams.first().ok_or(RtspError::ProtocolViolationGeneric)?;
        if resp.stream_type != stream_type {
            return Err(RtspError::ProtocolViolationGeneric);
        }

        info!("Audio setup response received: {resp:?}");
        // self.feedback_drift.register_stream(resp.stream_connection_id, stream_type, sample_rate);

        rtp.finish_tx(
            self.streams.bind_ip,
            self.streams.peer_ip,
            self.streams.cipher,
            resp,
            (encoder, recorder),
        )?;

        stream.connect(rtp, latency);
        Ok(guard)
    }

    fn create_guard<T>(stream: T, stream_type: StreamType, teardown_queue: mpsc::UnboundedSender<StreamType>) -> TeardownGuard<T> {
        TeardownGuard::new(stream, move || {
            debug!("TeardownGuard dropped at {stream_type:?}");
            debug!("Adding TEARDOWN to queue: {stream_type:?}");
            if teardown_queue.unbounded_send(stream_type).is_err() {
                debug!("Failed to add TEARDOWN to queue: {stream_type:?}");
            }
        })
    }

    pub async fn do_setup_video(&mut self, stream_type: StreamType, latency: Duration) -> RtspResult<TeardownGuard<ScreenTransmitProxy>> {
        self.drain_teardown_queue().await?;

        loop {
            match &mut self.media.screen {
                StreamState::Unconnected { .. } => break,
                StreamState::Stream { .. } => {
                    let client = self.streams.client.as_ref().ok_or(RtspError::ProtocolViolation("client missing"))?;
                    self.media.screen.teardown_screen(client.clone(), true);
                }
                StreamState::Teardown { .. } => {
                    if let Some(result) = self.media.screen.drain_teardown().await {
                        result?;
                    }
                }
            }
        }

        let info = self.streams.info.as_ref().ok_or(RtspError::ProtocolViolation("missing info"))?;
        let display = info.displays.first().ok_or(RtspError::ProtocolViolation("missing display"))?;
        let media_clock = self.streams.media_clock.as_ref().ok_or(RtspError::ProtocolViolation("missing clock"))?;
        let client = self.streams.client.as_ref().ok_or(RtspError::ProtocolViolation("client missing"))?;
        let id = rand::random::<u64>().saturating_add(1);

        let (session, sink) = ScreenTransmitSession::new(
            self.streams.cipher,
            id,
            media_clock.boxed(),
            ScreenTransmitSession::DEFAULT_MAX_PENDING_FRAMES,
            ScreenTransmitSession::DEFAULT_KEEP_ALIVE,
            latency,
        );
        let session = session.with_keep_alive_send_stats_as_body(info.keep_alive_send_stats_as_body);

        let display_uuid = display.uuid.clone();
        let setup = Setup::new(&[StreamDescriptionScreen::new(id, latency, &display_uuid).into()]);
        let guard = Self::create_guard(sink, stream_type, self.teardown_queue_tx.clone());

        info!("Sending screen SETUP: {setup:?}");
        let resp = client.setup(setup).await?;
        let resp = resp.streams.first().ok_or(RtspError::ProtocolViolation("invalid SETUP response"))?;
        if resp.stream_type != StreamType::Screen {
            return Err(RtspError::ProtocolViolation("invalid SETUP response #2"));
        }
        info!("Screen setup response received: {resp:?}");

        let mut video_ip = self.streams.peer_ip;
        video_ip.set_port(resp.data_port);

        // TODO: await connection and reconcile it with media stream state lifecycle.
        let socket = TcpHelper::connect_timeout(video_ip, Self::TCP_CONN_TIMEOUT_SCREEN, session)?;
        self.media.screen.connect_with_uuid(socket, latency, display_uuid);

        Ok(guard)
    }

    fn handle_feedback_payload(&mut self, payload: FeedbackPayload) {
        warn!("Stream feedback: {payload:?}");
    }

    async fn send_feedback(&mut self) -> Result<(), AirPlayTransmitterSessionError> {
        let now = Instant::now();

        if now < self.next_feedback {
            return Ok(());
        }

        self.next_feedback = now + Self::FEEDBACK_INTERVAL;

        if self.is_idle().await {
            return Ok(());
        }

        debug!("Sending /feedback");

        match self.streams.client.as_ref().unwrap().feedback_payload().await {
            Ok(Some(payload)) => self.handle_feedback_payload(payload),
            Ok(None) => trace!("Received /feedback without payload"),
            Err(err) => warn!("Failed to fetch /feedback payload: {err:?}"),
        }

        Ok(())
    }

    async fn reconcile(&mut self) -> Result<(), AirPlayTransmitterSessionError> {
        if let Some(err) = self.teardown_error.as_ref() {
            return Err(AirPlayTransmitterSessionError::Teardown(err.clone()));
        }

        if let Some(Err(err)) = self.drain_teardown_queue().now_or_never() {
            self.teardown_error.replace(err.clone());
            return Err(AirPlayTransmitterSessionError::Teardown(err));
        }

        // Only consume one command per reconcile() to ensure fairness
        if self.media.command_pending.is_none()
            && let Some(cmd) = self.streams.cmd_drain.as_mut().unwrap().pop()
        {
            debug!("Saving command for pickup: {cmd:?}");
            self.media.command_pending.replace(cmd);
        }

        Ok(())
    }
}
