use std::{
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_hap::{HomekitStorageRef, auth_setup::MfiSapSuccess};
use catplay_tokio::{TcpHelper, UdpHelper};
use catplay_util::{AsyncShutdown, EventReconciler, EventSink, EventSleeper, LazyAsync};
use futures::{TryFutureExt, channel::oneshot};
use log::{debug, info, warn};
use rand::Rng;
use uuid::Uuid;

use crate::{
    carplay_tx::{
        AirPlayRtspClient, AirPlayTransmitterBootstrap, AirPlayTransmitterBootstrapError, AirPlayTransmitterBootstrapResult,
        AirPlayTransmitterSessionError,
    },
    cipher::AirPlayStreamEncryption,
    clock::{MediaClockProxy, TimingServer},
    common::{AIRPLAY_TX_IOS_BUILD, AIRPLAY_TX_IOS_MODEL, AIRPLAY_TX_IOS_VERSION, AIRPLAY_TX_SDK_VERSION, AirPlayEncryptionType},
    events::{CommandQueue, CommandQueueDrain, EventsServer},
    keep_alive::KeepAliveClient,
    msg::{ControllerFeature, InfoMessageResponse, InitialSetup},
    rtsp_frame::{RtspError, RtspQueue},
    rtsp_session::{RtspTransmitter, RtspTransmitterCallback, RtspTransmitterEvent, RtspTransmitterHandle},
};

#[derive(EventSleeper)]
pub struct AirPlayTransmitterBootstrapSession {
    homekit: HomekitStorageRef,
    remote_uuid: Option<Uuid>,

    encrypt_callback: Option<RtspTransmitterHandle>,
    rtsp_queue: Option<RtspQueue>,
    client: Option<AirPlayRtspClient>,

    bind_ip: SocketAddr,
    peer_ip: SocketAddr,

    #[sleep]
    setup_task: SetupTask,
    #[sleep]
    pair_task: PairTask,
    #[sleep]
    auth_setup_task: AuthSetupTask,

    result_tx: Option<BootstrapResultSender>,

    /// Modern features
    controller_features: Vec<ControllerFeature>,

    shared_secret: Option<[u8; 32]>,
    aes_key: Option<[u8; 16]>,
    aes_key_encrypted: Option<[u8; 16]>,
    aes_iv: Option<[u8; 16]>,

    wants_pair_setup: bool,
    wants_auth_setup: bool,
}

type BootstrapResult = AirPlayTransmitterBootstrapResult<AirPlayTransmitterBootstrapStreams>;
type BootstrapResultSender = oneshot::Sender<BootstrapResult>;
type PairTask = LazyAsync<Option<BootstrapResult>>;
type SetupTask = LazyAsync<BootstrapResult>;
type AuthSetupTask = LazyAsync<AirPlayTransmitterBootstrapResult<MfiSapSuccess>>;

#[allow(dead_code)]
const BOOTSTRAP_FLOW_CHART: &str = r#"
+----------------+
| RTSP connected |
+-------+--------+
        |
        v
+-------------------------+
| pair gate               |
+-----------+-------------+
            |
            v
+-------------------------+
| wants_pair_setup?       |
+-----------+-------------+
   | yes                | no
   v                    v
goto /pair-setup      goto auth gate
   |
   v
+-------------------------+
| /pair-setup             |
+-----------+-------------+
            |
   +--------+--------+
   |                 |
   v ok              v error
goto auth gate      fail bootstrap

+-------------------------+
| auth gate               |
+-----------+-------------+
            |
            v
+-------------------------+
| wants_auth_setup?       |
+-----------+-------------+
   | yes                | no
   v                    v
goto /auth-setup     goto SETUP
   |
   v
+-------------------------+
| /auth-setup             |
+-----------+-------------+
            |
   +--------+--------+
   |                 |
   v ok              v error
goto SETUP          fail bootstrap

+-------------------------+
| SETUP                   |
+-----------+-------------+
            |
   +--------+--------+
   |                 |
   v ok              v error
goto /info   +----------------------+
             | pair-setup already?  |
             +----------+-----------+
                        |
              +---------+---------+
              |                   |
              v no                v yes
 set wants_pair_setup=yes   fail bootstrap
              |
              v
        goto pair gate

+-------------------------+
| /info                   |
+-----------+-------------+
            |
            v
+-------------------------+
| connect event sockets   |
| timing, keep-alive      |
+-----------+-------------+
            |
            v
+-------------------------+
| finish bootstrap        |
+-------------------------+
"#;

#[derive(AsyncShutdown, EventSleeper, EventReconciler)]
#[reconcile_error(AirPlayTransmitterSessionError)]
pub struct AirPlayTransmitterBootstrapStreams {
    pub peer_ip: SocketAddr,
    pub bind_ip: SocketAddr,

    pub info: Option<InfoMessageResponse>,
    /// Modern features (matched)
    pub enabled_features: Vec<ControllerFeature>,

    #[sleep]
    #[reconcile(AirPlayTransmitterSessionError::EventsDisconnected)]
    #[shutdown]
    pub events: Option<TcpHelper<EventsServer<CommandQueue>>>,
    #[sleep]
    #[reconcile(AirPlayTransmitterSessionError::KeepAliveDisconnected)]
    #[shutdown]
    pub keep_alive: Option<UdpHelper<1500, 0, KeepAliveClient>>,
    #[sleep]
    pub cmd_drain: Option<CommandQueueDrain>,
    #[sleep]
    #[reconcile(AirPlayTransmitterSessionError::TimingDisconnected)]
    #[shutdown]
    pub timing: Option<UdpHelper<1500, 0, TimingServer>>,
    pub media_clock: Option<MediaClockProxy>,

    /// Self reference (moved here at a later stage post-bootstrap to avoid disconnection)
    #[sleep]
    #[reconcile(AirPlayTransmitterSessionError::Disconnected)]
    #[shutdown]
    pub rtsp: Option<TcpHelper<RtspTransmitter<AirPlayTransmitterBootstrapSession>>>,
    pub client: Option<AirPlayRtspClient>,
    pub cipher: AirPlayStreamEncryption,
}

impl Default for AirPlayTransmitterBootstrapStreams {
    fn default() -> Self {
        Self {
            bind_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            peer_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),

            info: Default::default(),
            enabled_features: Default::default(),
            events: Default::default(),
            keep_alive: Default::default(),
            cmd_drain: Default::default(),
            timing: Default::default(),
            media_clock: Default::default(),
            rtsp: Default::default(),
            client: Default::default(),
            cipher: AirPlayStreamEncryption::Unconfigured,
        }
    }
}

impl AirPlayTransmitterBootstrapSession {
    const TCP_CONN_TIMEOUT_RTSP: Duration = Duration::from_millis(2000);
    const TCP_CONN_TIMEOUT_EVENTS: Duration = Duration::from_millis(2000);
    // This is required for batch command proxying when there is a spam of latency-sensitive HID commands
    const COMMAND_QUEUE_INFLIGHTS: usize = 1024;

    pub async fn connect(
        bootstrap: AirPlayTransmitterBootstrap,
    ) -> Result<AirPlayTransmitterBootstrapStreams, AirPlayTransmitterBootstrapError> {
        let (result_tx, result_rx) = oneshot::channel();

        // Initialize TcpHelper with RtspTransmitter<AirPlayTransmitterBootstrapSession>
        let session = RtspTransmitter::new(AirPlayTransmitterBootstrapSession::new(
            bootstrap.homekit,
            bootstrap.remote_homekit_id,
            result_tx,
            bootstrap.controller_features,
        ));
        let helper = TcpHelper::connect_timeout(bootstrap.peer_ip, Self::TCP_CONN_TIMEOUT_RTSP, session)
            // Failed to initialize FDs
            .map_err(AirPlayTransmitterBootstrapError::FailedToConnect)?;

        // Run the bootstrap process - TCP connect, HomeKit pairing, SETUP, /info and wait for result; return early, if failed
        debug!("Polling for transmitter initialization...");
        let mut ret = result_rx.await.expect("sender was dropped")?;
        debug!("Polling complete");

        // Take ownership over initialized helper streams like media clock, event socket etc.
        // Also, hide our TcpHelper reference of main RTSP stream within the result to avoid disconnection when this function returns.
        // This completes the bootstrap process and the target AirPlay transmitter can take over via RtspQueue.

        ret.rtsp.replace(helper);
        Ok(ret)
    }

    fn new(
        homekit: HomekitStorageRef,
        remote_uuid: Option<Uuid>,
        result_tx: BootstrapResultSender,
        controller_features: Vec<ControllerFeature>,
    ) -> Self {
        let _features = [
            ControllerFeature::UiContext,
            ControllerFeature::ViewAreas,
            ControllerFeature::CornerMasks,
            ControllerFeature::FocusTransfer,
            ControllerFeature::H264Level51,
            ControllerFeature::MainBuffered,
            ControllerFeature::AltScreen,
            ControllerFeature::EnhancedSiri,
            ControllerFeature::Hevc,
            ControllerFeature::SessionManagement,
            ControllerFeature::LogTransfer,
            ControllerFeature::IApChannel,
        ];

        Self {
            homekit,
            remote_uuid,

            rtsp_queue: None,
            encrypt_callback: None,
            client: None,

            bind_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            peer_ip: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            shared_secret: None,
            result_tx: Some(result_tx),

            pair_task: Default::default(),
            setup_task: Default::default(),
            auth_setup_task: Default::default(),
            controller_features,

            // Mercedes advertises HK_PAIRING_AND_ENCRYPT, but stores identity in RAM and it's lost when you exit the car.
            // Try to bypass /pair-setup when possible for faster connection.
            wants_pair_setup: false,

            wants_auth_setup: true,

            aes_key: None,
            aes_key_encrypted: None,
            aes_iv: None,
        }
    }

    fn send_result(&mut self, resp: BootstrapResult) {
        if let Some(result_tx) = self.result_tx.take() {
            assert!(result_tx.send(resp).is_ok(), "receiver was dropped");
        }
    }

    fn handle_pair_failure(&mut self, resp: BootstrapResult) {
        self.send_result(resp);
    }

    fn handle_setup_result(&mut self, resp: BootstrapResult) {
        if !self.wants_pair_setup
            && let Err(err) = resp
        {
            self.wants_pair_setup = true;
            warn!("Failed first SETUP, attempting crypto upgrade: {err}");
            self.start_pair_task();
            return;
        }
        self.send_result(resp);
    }

    fn handle_auth_setup_result(&mut self, resp: Result<MfiSapSuccess, AirPlayTransmitterBootstrapError>) {
        match resp {
            Ok(mut mfi_sap) => {
                let mut key = [0u8; 16];
                let mut iv = [0u8; 16];

                let mut rng = rand::rng();
                rng.fill(&mut key);
                rng.fill(&mut iv);

                self.aes_iv.replace(iv);
                self.aes_key.replace(key);

                let key_encrypted = mfi_sap.xcrypt_audio_key(&key).expect("invalid key size");
                self.aes_key_encrypted.replace(key_encrypted);
                info!("Finished /auth-setup");
                debug!("MFi-SAP response {:?}", mfi_sap.response);
                self.start_setup_task();
            }
            Err(err) => self.send_result(Err(err)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn do_initial_setup(
        bind_ip: SocketAddr,
        peer_ip: SocketAddr,
        client: AirPlayRtspClient,
        shared_secret: Option<[u8; 32]>,
        aes_key: Option<[u8; 16]>,
        aes_key_encrypted: Option<[u8; 16]>,
        aes_iv: Option<[u8; 16]>,

        controller_features: Vec<ControllerFeature>,
    ) -> AirPlayTransmitterBootstrapResult<AirPlayTransmitterBootstrapStreams> {
        let mut streams = AirPlayTransmitterBootstrapStreams::default();

        let mut timing_ip = bind_ip;
        timing_ip.set_port(0);

        let (timing_server, media_clock) = TimingServer::new();
        let timing_server = UdpHelper::bind(timing_ip, timing_server).map_err(AirPlayTransmitterBootstrapError::FailedToBind)?;

        // We deal with receivers in 3 categories depending on automotive era and OEM configuration:
        // 1) pre-HomeKit: has no /pair-setup and no HK_PAIRING_AND_ENCRYPT Bonjour flag (v210 era +/-)
        // 2) supports /pair-setup, advertises HK_PAIRING_AND_ENCRYPT but allows skipping it (v280 era +/-)
        // 3) enforces /pair-setup: SETUP without it is guaranteed to result in 403

        // Note that either way we don't deal with /auth-setup as it's useless for us and was never enforced on receiver side
        // to get a functional session.

        // It doesn't hurt to attempt crypto downgrade to None, since we only transmit over USB and want to save some CPU.

        let mut req = InitialSetup {
            device_id: "ff:ee:dd:cc:bb:aa".into(),
            features: controller_features.clone(),
            mac_address: "aa:bb:cc:dd:ee:ff".into(),
            model: AIRPLAY_TX_IOS_MODEL.into(),
            name: "iPhone".into(),
            os_name: Some("iPhone OS".into()),
            os_build_version: AIRPLAY_TX_IOS_BUILD.into(),
            os_version: Some(AIRPLAY_TX_IOS_VERSION.into()),

            session_uuid: "7762C714-AB2D-4AEC-BC29-430A0919EBB6".into(),
            session_correlation_uuid: Some("8C641C2C-3371-4F8B-90C6-920CB8EE3300".into()),
            source_version: AIRPLAY_TX_SDK_VERSION.into(),
            streams: None,
            stats_collection_enabled: false,
            timing_port: timing_server.1.port(),
            update_session_request: false,
            keep_alive_low_power: true,
            ..Default::default()
        };

        // /pair-setup always overrides /auth-setup for streams encryption, if both were used

        let cipher = {
            if let Some(shared_secret) = shared_secret {
                req.et = None;
                AirPlayStreamEncryption::ChaCha { shared_secret }
            } else if let Some(aes_key) = aes_key
                && let Some(aes_iv) = aes_iv
                && let Some(aes_key_encrypted) = aes_key_encrypted
            {
                debug!("MFi-SAP keys: key:{aes_key:02X?} iv:{aes_iv:02X?} key_encrypted:{aes_key_encrypted:02X?}");
                req.et.replace(AirPlayEncryptionType::MFI_SAPv1);
                req.ekey.replace(aes_key_encrypted.to_vec().into());
                req.eiv.replace(aes_iv.to_vec().into());
                AirPlayStreamEncryption::Aes { key: aes_key, iv: aes_iv }
            } else {
                req.et.replace(AirPlayEncryptionType::NONE);
                AirPlayStreamEncryption::None
            }
        };

        let resp = client
            .initial_setup(req)
            .map_err(AirPlayTransmitterBootstrapError::FailedInitialSetup)
            .await?;

        let info_start = Instant::now();
        // Get /info
        debug!("Getting /info ...");
        let info = client
            .info()
            .map_err(AirPlayTransmitterBootstrapError::FailedInfo)
            .await?;
        debug!("Got info: {info:?}");
        info!("Received /info in {:?}", Instant::now() - info_start);
        streams.info.replace(info);

        let mut timing_ip = peer_ip;
        timing_ip.set_port(resp.timing_port);

        // **DO NOT** perform two-way UDP binding
        // Receiver will re-create it's own side of the socket during idle transition, changing port, and next calls to screen SETUP will timeout trying to sync time!
        // timing_server.0.connect_finish(timing_ip).map_err(AirPlayTransmitterBootstrapError::FailedToBind)?;

        streams.enabled_features = resp.enabled_features;
        streams
            .enabled_features
            .retain(|f| controller_features.contains(f));

        let mut event_ip = peer_ip;
        event_ip.set_port(resp.event_port);

        let (cmd_queue, cmd_drain) = CommandQueue::new();

        let events = TcpHelper::connect_timeout(
            event_ip,
            Self::TCP_CONN_TIMEOUT_EVENTS,
            match cipher {
                AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None | AirPlayStreamEncryption::Aes { .. } => {
                    EventsServer::unencrypted(cmd_queue, Self::COMMAND_QUEUE_INFLIGHTS)
                }
                AirPlayStreamEncryption::ChaCha { shared_secret } => {
                    EventsServer::new(shared_secret, cmd_queue, Self::COMMAND_QUEUE_INFLIGHTS)
                }
            },
        )
        .map_err(AirPlayTransmitterBootstrapError::FailedToBind)?;

        // .await
        // .map_err(|e| RtspError::Code(format!("event port was not listening at time of setup ({})", e)))?;

        let keep_alive = if let Some(keep_alive_port) = resp.keep_alive_port {
            let mut keep_alive_ip = peer_ip;
            keep_alive_ip.set_port(keep_alive_port);
            Some(
                UdpHelper::connect(keep_alive_ip, KeepAliveClient::new())
                    .map_err(AirPlayTransmitterBootstrapError::FailedToBind)?
                    .0,
            )
        } else {
            None
        };

        streams.events.replace(events);
        streams.keep_alive = keep_alive;
        streams.timing.replace(timing_server.0);
        streams.cmd_drain.replace(cmd_drain);
        streams.media_clock.replace(media_clock);
        streams.cipher = cipher;

        streams.client.replace(client.clone());
        streams.bind_ip = bind_ip;
        streams.peer_ip = peer_ip;

        Ok(streams)
    }

    fn start_pair_task(&mut self) {
        let client = self.client.as_ref().expect("client missing").clone();
        self.pair_task.reset(move || async move {
            client
                .pair()
                .await
                .err()
                .map(|err| Err(AirPlayTransmitterBootstrapError::Pair(err)))
        });
    }

    fn start_auth_setup_task(&mut self) {
        let client = self.client.as_ref().expect("client missing").clone();
        self.auth_setup_task.reset(move || async move {
            client
                .auth_setup()
                .await
                .map_err(AirPlayTransmitterBootstrapError::AuthSetup)
        });
    }

    fn start_setup_task(&mut self) {
        let task = Self::do_initial_setup(
            self.bind_ip,
            self.peer_ip,
            self.client.as_mut().unwrap().clone(),
            self.shared_secret,
            self.aes_key,
            self.aes_key_encrypted,
            self.aes_iv,
            self.controller_features.clone(),
        );
        self.setup_task.reset(move || task);
    }
}

#[async_trait]
impl EventSink<RtspTransmitterEvent, ()> for AirPlayTransmitterBootstrapSession {
    async fn on_event<'a>(&mut self, event: RtspTransmitterEvent) {
        match event {
            RtspTransmitterEvent::Init { queue, handle } => {
                debug!("Received Init");

                self.rtsp_queue.replace(queue.clone());
                self.encrypt_callback.replace(handle.clone());

                let client = AirPlayRtspClient::new(queue.clone(), handle.clone(), self.homekit.clone(), self.remote_uuid);
                self.client.replace(client);
            }

            RtspTransmitterEvent::SetBindIp(ip) => self.bind_ip = ip,
            RtspTransmitterEvent::SetPeerIp(ip) => self.peer_ip = ip,
            RtspTransmitterEvent::Connected => {
                debug!("Connected - starting pairing flow");
                self.start_setup_task();
                // self.start_auth_setup_task();
                // self.start_pair_task();
            }
            RtspTransmitterEvent::Encrypted { shared_secret } => {
                debug!("Paired - starting SETUP/info flow");
                self.shared_secret.replace(shared_secret);
                self.start_setup_task();
            }
            RtspTransmitterEvent::ConnectionFailed => {
                self.send_result(Err(AirPlayTransmitterBootstrapError::FailedToConnect(RtspError::Closed)));
            }
            RtspTransmitterEvent::Eof(err) => {
                self.send_result(Err(AirPlayTransmitterBootstrapError::FailedToConnect(err)));
            }
        }
    }
}

impl EventReconciler for AirPlayTransmitterBootstrapSession {
    type Error = RtspError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        if let Some(Some(resp)) = self.pair_task.take() {
            self.handle_pair_failure(resp);
        }

        if let Some(resp) = self.setup_task.take() {
            self.handle_setup_result(resp);
        }

        if let Some(mfi_sap) = self.auth_setup_task.take() {
            self.handle_auth_setup_result(mfi_sap);
        }

        Ok(())
    }
}

impl AsyncShutdown for AirPlayTransmitterBootstrapSession {
    async fn shutdown(&mut self) {}
}

impl Drop for AirPlayTransmitterBootstrapSession {
    fn drop(&mut self) {
        debug!("AirPlayTransmitter was dropped!");
    }
}

impl RtspTransmitterCallback for AirPlayTransmitterBootstrapSession {}
