use async_trait::async_trait;
use catplay_hap::{HomekitStorage, HomekitStorageRef};

use catplay_iap2_client::{
    CsmRemote,
    tokio::{AsyncClient, AsyncClientDrain},
};
use catplay_mfi::MfiDevice;
use catplay_plist::CachingSerializer;
use catplay_tokio::{TcpHelper, UdpHelper};
use catplay_util::{AsyncShutdown, EventReconciler, EventSink, EventSleeper, futures_xordered::FuturesUnordered, mpsc};
use futures::future::BoxFuture;
use log::{debug, error, info, trace, warn};
use macaddr::MacAddr6;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};
use std::{sync::Arc, time::Duration};

use crate::{
    audio::codec::{AudioDecoder, AudioEncoder},
    carplay_rx::{
        AirPlayReceiverHandleRef, AirPlayReceiverProfile, AirPlayReceiverSessionError, AirPlayReceiverSinkBox,
        sink::{AirPlayReceiverHandleImpl, AirPlayServerShared, AirPlaySessionGuard},
    },
    cipher::AirPlayStreamEncryption,
    clock::{MediaClockProxy, TimingClient},
    common::{AIRPLAY_SDK_VERSION, AirPlayBonjourEntry, AirPlayEncryptionType, AirPlayFeature, AirPlayStatus},
    events::EventsClient,
    keep_alive::KeepAliveServer,
    modes::AirPlayModeState,
    msg::{
        Command, CommandError, CommandIApSendMessage, ControllerFeature, ExtendedFeature, HevcInfo, InfoMessage, InfoMessageResponse,
        InfoMessageTxtAirPlayResponse, InitialSetup, InitialSetupResponse, Setup, SetupResponse, StreamDescription, StreamDescriptionAudio,
        StreamDescriptionResponse, StreamDescriptionScreen, StreamType, TeardownPayload,
    },
    pairing::PairingHelperRx,
    rtp::RtpReceiver,
    rtsp_frame::{HttpHeader, HttpStatus, RtspError, RtspMethod::*, RtspQueue, RtspRequest, RtspResponse, RtspResult},
    rtsp_session::{RtspReceiverCallback, RtspReceiverEvent, RtspReceiverEventResult},
    screen::rx::ScreenReceiverSession,
};

#[derive(Default, PartialEq, Debug)]
enum AirPlayReceiverState {
    #[default]
    Pairing,
    InitialSetup,
    PreRecord,
    PostRecord,

    Closed(RtspError),
}

impl AirPlayReceiverState {
    pub fn change(&mut self, next: Self) {
        debug!("State {self:?} -> {next:?}");
        *self = next;
    }
}

#[derive(AsyncShutdown, EventSleeper, EventReconciler)]
#[reconcile_error(RtspError)]
#[reconcile_func(reconcile_close_status, |e: AirPlayReceiverSessionError| RtspError::foreign(e))]
#[reconcile_func(flush_iap2)]
#[reconcile_func(reconcile_event_loop)]
pub struct AirPlayReceiver {
    pub bind_ip: SocketAddr,
    pub peer_ip: SocketAddr,
    iface: String,
    mac_addr: MacAddr6,

    #[sleep]
    event_loop: FuturesUnordered<BoxFuture<'static, ()>>,
    task_queue_tx: mpsc::UnboundedSender<BoxFuture<'static, ()>>,
    #[sleep]
    task_queue_rx: mpsc::UnboundedReceiver<BoxFuture<'static, ()>>,

    #[sleep]
    #[shutdown]
    sink: AirPlayReceiverSinkBox,
    sink_had_init: bool,

    shared: AirPlayServerShared,
    pub lock: Option<AirPlaySessionGuard>,

    pub homekit: HomekitStorageRef,
    pub pairing: PairingHelperRx,

    state: AirPlayReceiverState,
    stream_encryption: AirPlayStreamEncryption,
    #[sleep]
    #[reconcile(|e: AirPlayReceiverSessionError| RtspError::foreign(e))]
    #[shutdown]
    streams: Streams,

    #[sleep]
    close_pending_rx: mpsc::Receiver<RtspError>,
    close_pending_tx: mpsc::Sender<RtspError>,

    wireless: bool,

    modes: AirPlayModeState,
    profile: AirPlayReceiverProfile,
    features: AirPlayFeature,
    serializer: CachingSerializer,
    start: Instant,
}

#[derive(Default, AsyncShutdown, EventSleeper, EventReconciler)]
#[reconcile_error(AirPlayReceiverSessionError)]
struct Streams {
    #[sleep]
    #[reconcile(AirPlayReceiverSessionError::KeepAlive)]
    keep_alive_socket: Option<UdpHelper<1500, 0, KeepAliveServer>>,
    #[sleep]
    #[reconcile(AirPlayReceiverSessionError::Events)]
    event_socket: Option<TcpHelper<EventsClient>>,
    #[sleep]
    #[reconcile(AirPlayReceiverSessionError::Timing)]
    timing_socket: Option<UdpHelper<1500, 0, TimingClient>>,
    media_clock: Option<MediaClockProxy>,

    #[sleep]
    #[shutdown(take)]
    // #[reconcile(AirPlayReceiverSessionError::Screen)]
    screen: Option<TcpHelper<ScreenReceiverSession>>,
    #[shutdown(take)]
    #[sleep]
    #[reconcile(AirPlayReceiverSessionError::Rtp)]
    main_audio: Option<RtpReceiver>,
    #[shutdown(take)]
    #[sleep]
    #[reconcile(AirPlayReceiverSessionError::Rtp)]
    alt_audio: Option<RtpReceiver>,

    event_rtsp: Option<RtspQueue>,

    #[sleep]
    #[shutdown(take)]
    #[reconcile]
    iap2: Option<AsyncClient>,
    #[sleep]
    iap2_drain: Option<AsyncClientDrain>,

    handle: Option<AirPlayReceiverHandleRef>,
}

#[async_trait]
impl EventSink<RtspReceiverEvent<'_>, RtspReceiverEventResult> for AirPlayReceiver {
    async fn on_event<'a>(&mut self, event: RtspReceiverEvent<'a>) -> RtspReceiverEventResult {
        #[cfg(debug_assertions)]
        trace!("Received event: {event:?}");

        match event {
            RtspReceiverEvent::SetBindIp(bind_ip) => self.bind_ip = bind_ip,
            RtspReceiverEvent::SetPeerIp(peer_ip) => self.peer_ip = peer_ip,
            RtspReceiverEvent::Request { request, response } => {
                if let Err(err) = self.process_request(request, response).await {
                    return if let Some(key) = self.pairing.transition_pending_to_encrypted() {
                        self.state.change(AirPlayReceiverState::InitialSetup);
                        RtspReceiverEventResult::encrypt(Err(err), key)
                    } else {
                        RtspReceiverEventResult::new(Err(err))
                    };
                }

                return if let Some(key) = self.pairing.transition_pending_to_encrypted() {
                    self.state.change(AirPlayReceiverState::InitialSetup);
                    RtspReceiverEventResult::encrypt_ready(key)
                } else {
                    RtspReceiverEventResult::ready()
                };
            }
            RtspReceiverEvent::Eof(err) => {
                warn!("Observed EOF on RTSP stream! {err:?}");
                self.shutdown().await;
            }
        }

        RtspReceiverEventResult::noop()
    }
}

impl RtspReceiverCallback for AirPlayReceiver {}

impl Drop for AirPlayReceiver {
    fn drop(&mut self) {
        debug!(
            "AirPlayReceiver {}<->{} dropped in state {:?}",
            self.bind_ip, self.peer_ip, self.state
        );
    }
}

impl AirPlayReceiver {
    const ALLOW_PAIR_SETUP_DOWNGRADE: bool = true;
    const TCP_REVERSE_CONN_TIMEOUT: Duration = Duration::from_millis(2000);
    const CARPLAY_FORCE_HEVC: bool = false;
    const APPLETV_FORCE_HEVC: bool = true;

    pub fn new(
        iface: &str,
        mac_addr: MacAddr6,
        homekit: Arc<dyn HomekitStorage>,
        mfi: Option<Arc<dyn MfiDevice>>,
        shared: AirPlayServerShared,
        sink: AirPlayReceiverSinkBox,
        profile: AirPlayReceiverProfile,
    ) -> Self {
        let pairing = PairingHelperRx::new(homekit.clone(), mfi);
        let (close_pending_tx, close_pending_rx) = mpsc::channel(1);

        let event_loop = mpsc::unbounded();

        AirPlayReceiver {
            bind_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            peer_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            iface: iface.into(),
            mac_addr,

            event_loop: Default::default(),
            task_queue_tx: event_loop.0,
            task_queue_rx: event_loop.1,

            shared,
            lock: None,

            homekit,
            pairing,

            streams: Streams::default(),
            sink,
            sink_had_init: false,

            state: AirPlayReceiverState::InitialSetup, /*Pairing*/
            stream_encryption: AirPlayStreamEncryption::Unconfigured,
            close_pending_rx,
            close_pending_tx,
            wireless: true, // TODO
            modes: Default::default(),
            profile,
            features: Self::features(profile),
            serializer: CachingSerializer::default(),
            start: Instant::now(),
        }
    }

    async fn reconcile_event_loop(&mut self) -> RtspResult<()> {
        if let Some(task) = self.task_queue_rx.take() {
            self.event_loop.push(task);
        }

        self.event_loop.take();
        Ok(())
    }

    async fn reconcile_close_status(&mut self) -> Result<(), AirPlayReceiverSessionError> {
        if let Some(err) = self.close_pending_rx.take() {
            warn!("Closing receiver as requested: {err}");
            self.state = AirPlayReceiverState::Closed(err);
        }

        if let AirPlayReceiverState::Closed(err) = &self.state {
            return Err(AirPlayReceiverSessionError::Sink(err.clone()));
        }

        Ok(())
    }

    pub async fn process_request(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        let (method, url) = (req.method, req.url.clone());

        if !Self::ALLOW_PAIR_SETUP_DOWNGRADE && self.features.contains(AirPlayFeature::CAR) && !self.pairing.allow_unpaired(req) {
            return Err(RtspError::NotEncrypted);
        }

        match (method, url.as_str()) {
            (Options, _) => self.respond_options(req, resp),
            (GetParameter, _) => self.respond_get_parameter(req, resp)?,
            (SetParameter, _) => self.respond_set_parameter(req, resp)?,

            (Flush, _) => self.respond_flush(req, resp)?,
            (Record, _) => self.respond_record(req, resp).await?,
            (Setup, _) => self.respond_setup(req, resp).await?,
            (Teardown, _) => self.respond_teardown(req, resp).await?,

            // Gets
            (Get, "/info") => self.respond_info(req, resp).await?,
            (Get, "/log") => Err(HttpStatus::NotFound)?,
            (Get, _) => Err(HttpStatus::NotFound)?,

            // Posts
            (Post, "/pair-setup") => self.pairing.respond_pair_setup(req, resp).await?,
            (Post, "/pair-verify") => self.pairing.respond_pair_verify(req, resp).await?,
            (Post, "/auth-setup") => self.pairing.respond_auth_setup(req, resp).await?,
            (Post, "/fp-setup") => self.pairing.respond_fp_setup(req, resp)?,

            (Post, "/command") => self.respond_command(req, resp).await?,
            (Post, "/diag-info") => Err(HttpStatus::NotFound)?,
            (Post, "/feedback") => Err(HttpStatus::Ok)?,
            (Post, "/info") => self.respond_info(req, resp).await?,
            (Post, "/audio-mode") => Err(HttpStatus::Ok)?,
            (Post, _) => Err(HttpStatus::NotFound)?,

            (_, _) => Err(HttpStatus::NotFound)?,
        };

        Ok(())
    }

    fn respond_get_parameter(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        if req.get_header(&HttpHeader::ContentType) == Some("text/parameters") {
            let body = String::from_utf8_lossy(&req.payload);
            let mut out = String::new();

            for line in body.lines().map(str::trim).filter(|line| !line.is_empty()) {
                match line {
                    "volume" => out.push_str("volume: 0.000000\r\n"),
                    _ => {}
                }
            }

            if !out.is_empty() {
                resp.payload.clear();
                resp.payload.extend_from_slice(out.as_bytes());
                resp.set_header(HttpHeader::ContentType, "text/parameters");
            }
        }

        Ok(())
    }

    fn respond_set_parameter(&mut self, req: &RtspRequest, _resp: &mut RtspResponse) -> RtspResult<()> {
        if req.get_header(&HttpHeader::ContentType) == Some("text/parameters") {
            let body = String::from_utf8_lossy(&req.payload);
            warn!("Ignoring AirPlay SET_PARAMETER payload: {}", body.trim_end());
        }

        Ok(())
    }

    pub fn features(profile: AirPlayReceiverProfile) -> AirPlayFeature {
        let mut features: AirPlayFeature = AirPlayFeature::empty();

        match profile {
            AirPlayReceiverProfile::CarPlay => {
                features |= AirPlayFeature::AUDIO;
                // features |= AirPlayFeature::REDUNDANT_AUDIO;
                features |= AirPlayFeature::AUDIO_AES_128_MFI_SAP_V1;
                features |= AirPlayFeature::AUDIO_PCM;
                features |= AirPlayFeature::AUDIO_UNENCRYPTED;
                features |= AirPlayFeature::SCREEN;
                features |= AirPlayFeature::ROTATE;
                features |= AirPlayFeature::UNIFIED_BONJOUR;
                features |= AirPlayFeature::CAR;
                features |= AirPlayFeature::CARPLAY_CONTROL;
                features |= AirPlayFeature::HK_PAIRING_AND_ENCRYPT;

                features |= AirPlayFeature::AUDIO_AAC_LC;
            }
            AirPlayReceiverProfile::AppleTV => {
                features |= AirPlayFeature::AUDIO;
                // features |= AirPlayFeature::REDUNDANT_AUDIO;
                // features |= AirPlayFeature::AUDIO_AES_128_MFI_SAP_V1;
                features |= AirPlayFeature::AUDIO_PCM;
                features |= AirPlayFeature::AUDIO_UNENCRYPTED;
                features |= AirPlayFeature::SCREEN;
                features |= AirPlayFeature::ROTATE;
                features |= AirPlayFeature::UNIFIED_BONJOUR;
                // features |= AirPlayFeature::CAR;
                // features |= AirPlayFeature::CARPLAY_CONTROL;
                // features |= AirPlayFeature::HK_PAIRING_AND_ENCRYPT; // can be enabled for testing

                features |= AirPlayFeature::AUDIO_AAC_LC;
                features |= AirPlayFeature::AUDIO_AAC_ELD;
                features |= AirPlayFeature::AUDIO_ALAC;
                // features |= AirPlayFeature::FPSAP_V2PT5_AES_GCM;
                features |= AirPlayFeature::VIDEO_FAIRPLAY;
                features |= AirPlayFeature::AUTHENTICATION4;

                if Self::APPLETV_FORCE_HEVC {
                    features |= AirPlayFeature::SUPPORTS_SCREEN_MULTI_CODEC; // can be enabled for testing
                }
            }
        }

        features
    }

    pub fn bonjour(profile: AirPlayReceiverProfile, homekit: HomekitStorageRef, device_id: MacAddr6) -> AirPlayBonjourEntry {
        pub const AIRPLAY_MODEL_BRAND_CARPLAY: &str = "CarPlay";
        pub const AIRPLAY_MODEL_BRAND_MIRROR: &str = "AppleTV3,2";

        pub const AIRPLAY_PROTO_VERS: &str = "1.0";

        AirPlayBonjourEntry {
            srcvers: AIRPLAY_SDK_VERSION.into(),
            pi: Some(homekit.device_id()),
            pk: homekit.public_key_as_hex(),
            protovers: AIRPLAY_PROTO_VERS.into(),
            model: match profile {
                AirPlayReceiverProfile::CarPlay => AIRPLAY_MODEL_BRAND_CARPLAY,
                AirPlayReceiverProfile::AppleTV => AIRPLAY_MODEL_BRAND_MIRROR,
            }
            .into(),
            features: AirPlayReceiver::features(profile),
            flags: AirPlayStatus::AUDIO,
            deviceid: match profile {
                AirPlayReceiverProfile::CarPlay => format!("{}", device_id),
                AirPlayReceiverProfile::AppleTV => format!("{}_atv", device_id),
            },

            vv: match profile {
                AirPlayReceiverProfile::CarPlay => "".into(),
                AirPlayReceiverProfile::AppleTV => "2".into(),
            },
            pw: match profile {
                AirPlayReceiverProfile::CarPlay => "".into(),
                AirPlayReceiverProfile::AppleTV => "false".into(),
            },
            // ..Default::default()
        }
    }

    // RTSP handlers start

    fn respond_options(&self, _req: &RtspRequest, resp: &mut RtspResponse) {
        resp.set_header(
            HttpHeader::Public,
            "ANNOUNCE, SETUP, RECORD, PAUSE, FLUSH, TEARDOWN, OPTIONS, POST, GET, PUT, GET_PARAMETER, SET_PARAMETER",
        );
    }

    fn respond_flush(&mut self, _req: &RtspRequest, _resp: &mut RtspResponse) -> RtspResult<()> {
        if self.state != AirPlayReceiverState::PostRecord {
            return Err(HttpStatus::NotAcceptable)?;
        }

        Err(HttpStatus::NotAcceptable.into())
    }

    async fn respond_record(&mut self, _req: &RtspRequest, _resp: &mut RtspResponse) -> RtspResult<()> {
        if self.state != AirPlayReceiverState::PreRecord {
            return Err(HttpStatus::NotAcceptable)?;
        }

        self.state.change(AirPlayReceiverState::PostRecord);
        // Session has formally started
        info!("Session starting as RECORD was requested");
        self.sink.on_record().await
    }

    async fn respond_teardown(&mut self, req: &RtspRequest, _resp: &mut RtspResponse) -> RtspResult<()> {
        let data: TeardownPayload = req.get_plist()?;

        let teardown_session = data.streams.is_empty();
        if teardown_session {
            info!("Session TEARDOWN was requested!");
            let _ = self.close_pending_tx.try_send(RtspError::Teardown);
            return Ok(());
        }

        for stream in data.streams {
            info!("Performing teardown of {:?}", stream.stream_type);

            let mut matched = false;
            match stream.stream_type {
                StreamType::GeneralAudio | StreamType::MainAudio | StreamType::MainHighAudio => {
                    if let Some(mut audio) = self.streams.main_audio.take() {
                        audio.shutdown().await;
                        matched = true;
                    }
                }
                StreamType::AltAudio => {
                    if let Some(mut audio) = self.streams.alt_audio.take() {
                        audio.shutdown().await;
                        matched = true;
                    }
                }
                StreamType::Screen => {
                    if let Some(mut screen) = self.streams.screen.take() {
                        screen.shutdown().await;
                        matched = true;
                    }
                }
                _ => {}
            }

            if !matched {
                warn!("No match for stream type {:?} during teardown", stream.stream_type);
            }
        }

        Ok(())
    }

    async fn respond_info(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        let bon = Self::bonjour(self.profile, self.homekit.clone(), self.mac_addr);

        if let Ok(info) = req.get_plist::<InfoMessage>()
            && info.qualifier.as_ref().is_some_and(|q| q.iter().any(|v| v == "txtAirPlay"))
        {
            return resp.set_plist(InfoMessageTxtAirPlayResponse {
                txt_airplay: bon.to_rtsp_info_string().into(),
            });
        }

        let mut r = InfoMessageResponse {
            status_flags: AirPlayStatus::AUDIO,
            features: self.features,
            extended_features: if self.profile == AirPlayReceiverProfile::CarPlay {
                vec![ExtendedFeature::EnhancedRequestCarUI, ExtendedFeature::VocoderInfo]
            } else {
                vec![]
            },
            device_id: bon.deviceid,
            firmware_revision: "1.0".into(),
            hardware_revision: "1.0".into(),
            keep_alive_low_power: true,
            manufacturer: "unknown".into(),
            model: if self.profile == AirPlayReceiverProfile::CarPlay {
                "CatPlay".into()
            } else {
                "AppleTV3,2".into()
            },
            name: if self.profile == AirPlayReceiverProfile::CarPlay {
                "CarPlay".into()
            } else {
                "CatPlay AirPlay".into()
            },
            source_version: AIRPLAY_SDK_VERSION.into(),

            ..Default::default()
        };

        r.modes = self.sink.init_modes();

        Self::setup_audio_defaults(&mut r);

        self.sink.on_info(&mut r).await;
        let before = r.displays.len();
        self.profile.constrain_info(&mut r).map_err(RtspError::ProtocolViolation)?;
        if self.profile == AirPlayReceiverProfile::CarPlay {
            let display = &r.displays[0];
            info!(
                "CarPlay RX /info: displays={}->1 main={}x{} altScreen=false",
                before, display.width_pixels, display.height_pixels
            );
        }
        self.modes = AirPlayModeState::new(&r.modes);

        if self.features.contains(AirPlayFeature::CAR) && Self::CARPLAY_FORCE_HEVC {
            r.hevc_info.replace(HevcInfo::default());
        }
        resp.set_plist(r)
    }

    async fn respond_setup(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        match self.state {
            AirPlayReceiverState::InitialSetup => return self.respond_setup_initial(req, resp).await,
            AirPlayReceiverState::PreRecord | AirPlayReceiverState::PostRecord => {}
            _ => return Err(HttpStatus::BadRequest)?,
        }

        let data: Setup = req.get_plist()?;
        debug!("plist data: {data:?}");

        let mut streams_resp = SetupResponse::default();
        for stream in data.streams {
            let sdr = self.setup_stream(stream).await?;
            streams_resp.streams.push(sdr);
        }

        resp.set_plist(streams_resp)
    }

    async fn init_iap2(&mut self, _handle: AirPlayReceiverHandleRef) {
        let Some(_sink) = self.sink.open_iap2().await else {
            return;
        };

        if !self.wireless {
            return;
        }

        let (client, drain) = AsyncClient::new(false, CsmRemote::airplay(), _sink);
        self.streams.iap2_drain.replace(drain);
        self.streams.iap2.replace(client);
    }

    async fn flush_iap2(&mut self) -> RtspResult<()> {
        if let Some(handle) = self.streams.handle.as_mut()
            && let Some(drain) = self.streams.iap2_drain.as_mut()
            && let Some(packet) = drain.take()
        {
            debug!("Forwarding iAP2 packet");
            let fut = handle.send_command(Command::IApSendMessage(CommandIApSendMessage {
                data: packet.into_inner().into(),
            }))?;
        }

        Ok(())
    }

    async fn setup_screen(&mut self, stream: StreamDescriptionScreen) -> RtspResult<StreamDescriptionResponse> {
        if self.streams.screen.is_some() {
            error!("Attempted to create video sink without TEARDOWN of previous one!");
            return Err(HttpStatus::BadRequest)?;
        }

        info!("Opening screen stream with latency {:?}", stream.latency_ms);

        let latency = Duration::from_millis(stream.latency_ms.unwrap_or(0));
        let mut screen_sink = match self.sink.open_screen(latency).await {
            Ok(sink) => sink,
            Err(err) => {
                error!("Failed to open video sink: {err}");
                return Err(RtspError::Code(HttpStatus::InternalServerError));
            }
        };

        if let Err(err) = screen_sink.init().await {
            error!("Failed to initialize video sink: {err}");
            return Err(RtspError::Code(HttpStatus::InternalServerError));
        }

        let Some(media_clock) = self.streams.media_clock.as_ref() else {
            return Err(RtspError::Code(HttpStatus::InternalServerError));
        };

        let mut bind_ip = self.bind_ip;
        bind_ip.set_port(0);

        let recv = ScreenReceiverSession::new(
            self.stream_encryption,
            stream.stream_connection_id,
            latency,
            screen_sink,
            media_clock.boxed(),
        );

        let (helper, local) = TcpHelper::accept_timeout(bind_ip, Self::TCP_REVERSE_CONN_TIMEOUT, recv)?;
        self.streams.screen.replace(helper);

        Ok(StreamDescriptionResponse::new_screen(local.port()))
    }

    async fn setup_audio(&mut self, mut stream: StreamDescriptionAudio) -> RtspResult<StreamDescriptionResponse> {
        let audio_stream = match stream.stream_type {
            StreamType::GeneralAudio | StreamType::MainAudio | StreamType::MainHighAudio => &mut self.streams.main_audio,
            StreamType::AltAudio => &mut self.streams.alt_audio,
            _ => return Err(RtspError::Code(HttpStatus::NotAcceptable)),
        };

        if audio_stream.is_some() {
            error!("Attempted to start audio stream without TEARDOWN of previous one!");
            return Err(RtspError::Code(HttpStatus::NotAcceptable));
        }

        let audio_type = stream.audio_type;
        let mut record_codec = None;
        let mut record_sink = None;

        // Sanity: clamp to 32..2000ms to prevent insane memory allocations
        stream.audio_latency_ms = stream.audio_latency_ms.clamp(32, 2000);
        let latency = Duration::from_millis(stream.audio_latency_ms);

        let codec = match RtpReceiver::init_codec(&stream) {
            Ok(v) => v,
            Err(err) => {
                error!("Failed to init audio codec: {err}");
                return Err(err);
            }
        };

        let sink = match self
            .sink
            .open_audio(
                latency,
                stream.stream_type,
                audio_type,
                stream.audio_format,
                codec.output_type(),
                stream.input,
            )
            .await
        {
            Ok(sink) => sink,
            Err(err) => {
                error!("Failed to open audio sink: {err}");
                return Err(RtspError::Code(HttpStatus::InternalServerError));
            }
        };

        if stream.input {
            warn!("Setting up microphone stream, format {:?}", stream.audio_format);
            let codec = match RtpReceiver::init_codec_record(&stream) {
                Ok(v) => v,
                Err(err) => {
                    error!("Failed to init mic audio codec: {err}");
                    return Err(err);
                }
            };

            record_sink.replace(
                match self.sink.open_microphone(stream.stream_type, audio_type, codec.input_type()).await {
                    Err(err) => {
                        error!("Failed to open microphone sink: {err}");
                        return Err(RtspError::Code(HttpStatus::InternalServerError));
                    }
                    Ok(sink) => sink,
                },
            );
            record_codec.replace(codec);
        }

        let receiver = match RtpReceiver::new(
            self.bind_ip,
            self.peer_ip,
            self.stream_encryption,
            &stream,
            codec,
            sink,
            record_codec,
            record_sink,
        ) {
            Ok(v) => v,
            Err(err) => {
                error!("Failed to start RTP receiver: {err}");
                return Err(RtspError::Code(HttpStatus::InternalServerError));
            }
        };

        debug!("Created RtpReceiver");

        let resp = StreamDescriptionResponse::new_audio(
            stream.stream_type,
            stream.stream_connection_id,
            receiver.local_port_rtp,
            receiver.local_port_rtcp,
            false,
            true,
        );
        audio_stream.replace(receiver);

        info!(
            "Opening audio stream with type {:?} latency {:?}",
            stream.stream_type, stream.audio_latency_ms
        );

        Ok(resp)
    }

    async fn setup_stream(&mut self, stream: StreamDescription) -> RtspResult<StreamDescriptionResponse> {
        match stream {
            StreamDescription::AudioLegacy(v) => self.setup_audio(v.as_modern()).await,
            StreamDescription::Audio(v) => self.setup_audio(v).await,
            StreamDescription::Screen(v) => self.setup_screen(v).await,
            _ => Err(RtspError::Code(HttpStatus::NotAcceptable)),
        }
    }

    async fn respond_setup_initial(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        if self.state != AirPlayReceiverState::InitialSetup {
            return Err(RtspError::ProtocolViolation("initial setup in invalid state"));
        }

        self.stream_encryption = AirPlayStreamEncryption::Unconfigured;

        let data: InitialSetup = req.get_plist()?;
        debug!("Initial Setup: {data:?}");

        if let Some(et) = data.et {
            // Fun fact about mirroring: iOS enforces FairPlay, but if you add HomeKit pairing to the mix...
            // ...they suddenly cancel out after initial /fp-setup handshake and /pair-verify
            // and the key for the stream is just the the HomeKit shared secret, like a normal modern CarPlay session :|
            // So you can skip FairPlay key descrambler, but then Apple will add 5s connection penalty before even attempting to connect...
            // which likely is fixable by some better Bonjour meta

            if self.pairing.shared_secret().is_some() {
                return Err(RtspError::ProtocolViolation(
                    "after /pair-verify streams are always encrypted using ChaCha shared secret",
                ));
            }

            if et == AirPlayEncryptionType::NONE {
                self.stream_encryption = AirPlayStreamEncryption::None;
                warn!("Encryption of media streams is disabled");
            } else if let Some(ekey) = data.ekey
                && let Some(eiv) = data.eiv
            {
                let audio_iv: [u8; 16] = eiv
                    .as_ref()
                    .try_into()
                    .map_err(|_| RtspError::ProtocolViolation("expected 16-byte eiv in initial setup"))?;

                match et {
                    AirPlayEncryptionType::MFI_SAPv1 => {
                        // AES key scrambled by /auth-setup MFi session
                        if let Err(err) = self.pairing.decrypt_mfi_sap_key(ekey.as_ref(), &audio_iv) {
                            return Err(RtspError::ProtocolViolationString(format!("failed to decrypt MFi SAP keys: {err}")));
                        }

                        let Some((key, iv)) = self.pairing.aes_keys() else {
                            return Err(RtspError::ProtocolViolation("failed to obtain AES keys after MFi SAP setup"));
                        };
                        info!("Using MFi SAP for media streams encryption");
                        self.stream_encryption = AirPlayStreamEncryption::Aes { key, iv };
                    }
                    AirPlayEncryptionType::FAIRPLAY => {
                        // AES key scrambled by /fp-setup scrambler
                        if let Err(err) = self.pairing.decrypt_fairplay_key(ekey.as_ref(), &audio_iv) {
                            return Err(RtspError::ProtocolViolationString(format!(
                                "Failed to decrypt FAIRPLAY keys: {err}"
                            )));
                        }

                        let Some((key, iv)) = self.pairing.aes_keys() else {
                            return Err(RtspError::ProtocolViolation("failed to obtain AES keys after FAIRPLAY setup"));
                        };
                        self.stream_encryption = AirPlayStreamEncryption::Aes { key, iv };
                        info!("Using FairPlay for media streams encryption");
                    }

                    _ => {}
                }
            }
        }

        if matches!(self.stream_encryption, AirPlayStreamEncryption::Unconfigured)
            && let Some(shared_secret) = self.pairing.shared_secret()
        {
            self.stream_encryption = AirPlayStreamEncryption::ChaCha { shared_secret };
            info!("Using HomeKit ChaCha for media streams encryption");
        }

        let mut bind_ip = self.bind_ip;
        bind_ip.set_port(0);

        let mut timing_remote = self.peer_ip;
        timing_remote.set_port(data.timing_port);

        let (timing_client, media_clock) = TimingClient::new();
        let (timing_client, timing_bind, _) = UdpHelper::connect(timing_remote, timing_client)?;

        let mut events = None;
        if self.features.contains(AirPlayFeature::CAR) {
            events.replace(self.do_setup_events().await?);
        }

        let handle = Arc::new(AirPlayReceiverHandleImpl::new(
            self.streams.event_rtsp.clone(),
            media_clock.clone(),
            self.task_queue_tx.clone(),
            &self.iface,
            self.wireless,
            self.close_pending_tx.clone(),
        ));

        self.sink.init(handle.clone())?;
        self.sink_had_init = true;

        let lock = self.shared.lock(handle.clone());
        if lock.is_none() {
            warn!("Rejected incoming AirPlay connection, because session is already active!");
            return Err(RtspError::Code(HttpStatus::NotEnoughBandwidth));
        }

        self.lock.replace(lock.unwrap());

        self.sink.on_initial_setup().await?;

        if self.features.contains(AirPlayFeature::CAR) && self.wireless {
            self.init_iap2(handle.clone()).await;
        }

        let (keep_alive_socket, keep_alive_bind) = UdpHelper::bind(bind_ip, KeepAliveServer::new())?;

        let mut r = InitialSetupResponse {
            event_port: events.unwrap_or(0),
            keep_alive_port: Some(keep_alive_bind.port()),
            streams: None,
            timing_port: timing_bind.port(),
            enabled_features: vec![/*ControllerFeature::H264Level51*/],
        };

        if self.features.contains(AirPlayFeature::CAR) && Self::CARPLAY_FORCE_HEVC {
            r.enabled_features.push(ControllerFeature::Hevc);
        }

        if self.profile == AirPlayReceiverProfile::CarPlay {
            info!(
                "CarPlay RX initialSetup: altScreen={} hevc={}",
                r.enabled_features.contains(&ControllerFeature::AltScreen),
                r.enabled_features.contains(&ControllerFeature::Hevc)
            );
        }

        self.streams.keep_alive_socket.replace(keep_alive_socket);
        self.streams.timing_socket.replace(timing_client);
        self.streams.media_clock.replace(media_clock);
        self.streams.handle.replace(handle);

        resp.set_plist(r)?;
        self.state.change(AirPlayReceiverState::PreRecord);
        Ok(())
    }

    async fn do_setup_events(&mut self) -> RtspResult<u16> {
        let secret = self.pairing.shared_secret();

        let mut bind_ip = self.bind_ip;
        bind_ip.set_port(0);

        let event_session = match secret {
            Some(shared_secret) => EventsClient::new(shared_secret),
            None => EventsClient::unencrypted(),
        };

        let event_rtsp = event_session.rtsp_client();

        let (event_socket, event_bind) = TcpHelper::accept_timeout(bind_ip, Self::TCP_REVERSE_CONN_TIMEOUT, event_session)?;
        self.streams.event_socket.replace(event_socket);
        self.streams.event_rtsp.replace(event_rtsp);

        Ok(event_bind.port())
    }

    async fn respond_command(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        match Command::deserialize_caching(&req.payload, &mut self.serializer) {
            Err(err) => {
                warn!("Failed to deserialize command: {err:?}");
                let status = match err {
                    CommandError::UnknownCommand(_) => HttpStatus::NotImplemented,
                    CommandError::InvalidPayload(_) => HttpStatus::BadRequest,
                    CommandError::FailedDeserialize(_, _) => HttpStatus::BadRequest,
                };
                resp.status = status;
                resp.headers.clear();
                resp.payload.clear();
                resp.cseq = req.cseq;
                Ok(())
            }
            Ok(v) => {
                warn!("Received command: {v:?}");
                let next = self.handle_command(v).await?;
                *resp = next;
                resp.cseq = req.cseq; // User does not have to know the correct CSeq so we have to correct it
                Ok(())
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command) -> RtspResult<RtspResponse> {
        self.sink.on_command_raw(&cmd).await?;

        match cmd {
            Command::ModesChanged(v) => {
                let _old_modes = self.modes;
                self.modes.feed(&v.0);

                // Call on_modes even if modes are unchanged.
                // Looks like iPhone likes asserting modes again in certain situations
                // to be more resilent to racy OEM implementations.
                warn!("Received updated modes: {:?}", self.modes);
                self.sink.on_modes(&self.modes).await;

                Ok(RtspResponse::new(None, HttpStatus::Ok))
            }
            Command::IApSendMessage(cmd) => {
                if let Some(iap2) = self.streams.iap2.as_mut() {
                    iap2.read_frame_buf(&cmd.data);
                } else {
                    warn!("Ignoring incoming iAP2 data because pipe is inactive");
                }

                Ok(RtspResponse::new(None, HttpStatus::Ok))
            }
            _ => Ok(RtspResponse::new(None, HttpStatus::Ok)),
        }
    }

    fn is_idle(&self) -> bool {
        let s = &self.streams;
        s.main_audio.is_none() && s.alt_audio.is_none() && s.screen.is_none()
    }
}
