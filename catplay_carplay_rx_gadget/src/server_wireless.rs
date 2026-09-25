use std::{
    fs, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use catplay_carplay::carplay_rx::{
    AirPlayReceiverHandleRef, AirPlayReceiverProfile, AirPlayReceiverSink,
    sink::{AirPlayServer, AirPlayServerShared},
};
use catplay_hap::HomekitStorageRef;
use catplay_iap2_bt::{BluetoothError, BluetoothManager};
use catplay_iap2_usb::{GadgetError, NcmHelper};
use catplay_mfi::MfiDeficeRef;
use catplay_util::{AbortOnDropHandle, ArcBox, AsyncShutdown, EventSleeper, Reconcilable, Reconciler, deadline_after, event_select, sleep, spawn};
use log::{debug, error, info, trace, warn};
use macaddr::MacAddr6;

use crate::{CarPlayServerSession, CarPlaySessionIdentity};

/// Rate-limits warn-level reports from the StartingBluetoothPower retry loop.
static BT_POWER_FAIL_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Rate-limits warn-level reports from the WaitingForBluetooth (query_mac) retry loop.
static BT_QUERY_FAIL_COUNT: AtomicUsize = AtomicUsize::new(0);

/// CarPlay HeadUnit (Wireless)
pub struct CarPlayWirelessGadget<T: AirPlayReceiverSink> {
    // AirPlay server
    server: Option<AirPlayServer>,
    // Bluetooth manager
    bluetooth: Option<BluetoothManager>,
    last_bt_peer: Option<MacAddr6>,
    last_bt_peer_task: Option<AbortOnDropHandle<()>>,
    name: String,

    hci: String,
    iface: String,
    ssid: String,
    pass: Option<String>,
    channel: Option<u8>,
    wpa: bool,

    // AirPlay shared state
    shared: AirPlayServerShared,

    // Dependencies
    homekit: HomekitStorageRef,
    mfi: Option<MfiDeficeRef>,

    // AirPlay sink
    sink: Arc<dyn Fn() -> T + Send + Sync + 'static>,

    invites_blocked: bool,
    last_mac: Option<MacAddr6>,

    // BT "last-connect" cache file
    bt_last_connect_cache_file: Option<String>,
    bt_last_connect_cache_loaded: bool,

    burst_wakeups: bool,
}

impl<T: AirPlayReceiverSink> CarPlayWirelessGadget<T> {
    pub fn new(
        homekit: HomekitStorageRef,
        mfi: Option<MfiDeficeRef>,
        shared: AirPlayServerShared,
        name: &str,

        hci: &str,
        iface: &str,
        ssid: &str,
        pass: Option<&str>,
        wpa: bool,
        channel: Option<u8>,

        bt_last_connect_cache_file: Option<&str>,
        sink: impl Fn() -> T + Send + Sync + 'static,
    ) -> Reconciler<Self> {
        Reconciler::new(
            Self {
                last_bt_peer: None,
                last_bt_peer_task: None,
                name: name.into(),
                server: None,
                bluetooth: None,

                hci: hci.into(),
                iface: iface.into(),
                ssid: ssid.into(),
                pass: pass.map(|p| p.into()),
                wpa,
                channel,

                homekit,
                mfi,
                shared,
                sink: Arc::new(sink),
                invites_blocked: false,
                last_mac: None,
                bt_last_connect_cache_file: bt_last_connect_cache_file.map(|p| p.into()),
                bt_last_connect_cache_loaded: false,
                burst_wakeups: false,
            },
            Ok(LocalState::Initial),
        )
    }
}

#[derive(thiserror::Error, Clone, PartialEq, Debug)]
pub enum CarPlayWirelessGadgetError {
    #[error("AirPlayServer: {0}")]
    AirPlayServer(ArcBox<io::Error>),
    #[error("Bonjour: {0}")]
    Bonjour(ArcBox<io::Error>),
    #[error("Bluetooth: {0}")]
    Bluetooth(ArcBox<BluetoothError>),

    #[error("MacAddressChanged: {prev} vs {current}")]
    MacAddressChanged { prev: MacAddr6, current: MacAddr6 },
    #[error("InterfaceGone: {0}")]
    InterfaceGone(GadgetError),
    #[error("Unexpected state")]
    UnexpectedState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CarPlayWirelessGadgetState {
    Initial,
    WaitingForBluetooth { hci: String },
    StartingBluetoothPower { hci: String, mac: MacAddr6 },
    StartingBluetooth { hci: String, mac: MacAddr6 },
    StartingLastConnect { hci: String, mac: MacAddr6, peer_mac: MacAddr6 },

    WaitingForInterface { iface: String },
    WaitingForInterfaceRunning { iface: String, mac: MacAddr6 },
    WaitingForStableMulticast { iface: String, mac: MacAddr6 },
    StartingServer { iface: String, mac: MacAddr6 },

    Inviting,
    Receiving,
    Passive,
}

type LocalResult<T> = Result<T, LocalError>;
type LocalStatus = LocalResult<LocalState>;
type LocalError = CarPlayWirelessGadgetError;
type LocalState = CarPlayWirelessGadgetState;

impl<T> From<LocalError> for LocalResult<T> {
    fn from(value: LocalError) -> Self {
        Err(value)
    }
}

impl From<LocalState> for LocalStatus {
    fn from(value: LocalState) -> Self {
        Ok(value)
    }
}

impl<T: AirPlayReceiverSink> CarPlayWirelessGadget<T> {
    const RESTART_DELAY: Duration = Duration::from_millis(1000);

    pub fn get_bt_last_connect(&mut self) -> Option<MacAddr6> {
        if let Some(bt) = self.bluetooth.as_mut() {
            return bt.get_last_connected();
        }
        None
    }

    pub fn set_bt_last_connect_cache_file(&mut self, file: impl Into<String>) {
        self.bt_last_connect_cache_file.replace(file.into());
        self.bt_last_connect_cache_loaded = false;
    }

    fn load_bt_last_connect(&mut self) {
        if self.bt_last_connect_cache_loaded {
            return;
        }

        let Some(file) = self.bt_last_connect_cache_file.as_ref() else {
            return;
        };

        self.bt_last_connect_cache_loaded = true;

        let Ok(current) = fs::read_to_string(file) else {
            return;
        };

        match current.trim().parse::<MacAddr6>() {
            Ok(mac) => {
                self.last_bt_peer.replace(mac);
            }
            Err(err) => {
                warn!("Failed to parse Bluetooth last-connect cache file {file}: {err:?}");
            }
        }
    }

    fn sync_bt_last_connect(&mut self) {
        let last_connect = self.get_bt_last_connect();
        if let Some(file) = self.bt_last_connect_cache_file.as_ref() {
            let Some(last_connect) = last_connect else {
                return;
            };

            let mac = last_connect.to_string();
            if fs::read_to_string(file).is_ok_and(|current| current.trim() == mac) {
                return;
            }

            let content = format!("{mac}\n");
            if let Err(err) = fs::write(file, content) {
                warn!("Failed to sync Bluetooth last-connect cache file {file}: {err:?}");
            }
        }
    }

    fn find_active_session(&mut self) -> Option<AirPlayReceiverHandleRef> {
        if let Some(session) = self.shared.borrow()
            && self.iface == session.iface()
        {
            return Some(session);
        }

        None
    }

    pub fn set_invites_blocked(&mut self, blocked: bool) {
        if self.invites_blocked != blocked {
            if blocked {
                info!("iPhone invites blocked (no active car session)");
            } else {
                info!("iPhone invites enabled (car session active)");
            }
        }
        self.invites_blocked = blocked;
    }
}

impl<T: AirPlayReceiverSink> AsyncShutdown for CarPlayWirelessGadget<T> {
    async fn shutdown(&mut self) {
        self.server.take().shutdown().await;

        self.bluetooth.take();
        self.last_bt_peer_task.take();
    }
}

impl<T: AirPlayReceiverSink> EventSleeper for CarPlayWirelessGadget<T> {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        event_select!(deadline_after(if self.burst_wakeups {
            Duration::from_millis(50)
        } else {
            // AirPlay does not provide a callback when a session is released,
            // so we periodically check find_active_session(). This fallback
            // exists primarily to detect that session release.
            Duration::from_millis(500)
        }))
    }
}

impl<T: AirPlayReceiverSink> Reconcilable for CarPlayWirelessGadget<T> {
    type Output = LocalStatus;

    async fn on_update(&mut self, new: LocalStatus) -> LocalStatus {
        match new {
            Ok(ref ok) => info!("Progressing -> {ok:?}"),
            Err(ref err) => error!("Entered error state: {err:?}"),
        };

        if new.is_err() {
            debug!("Cleaning up due to error state");
            if let Some(mut server) = self.server.take() {
                server.shutdown().await;
            }
            self.bluetooth.take();
            self.last_mac.take();
        }

        self.burst_wakeups = match new {
            Err(_) => true,
            Ok(LocalState::Inviting | LocalState::Receiving | LocalState::Passive) => false,
            Ok(_) => true,
        };

        new
    }

    async fn render(&mut self, prev: LocalStatus, update: Instant) -> LocalStatus {
        let Ok(_status) = prev.as_ref() else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return LocalState::Initial.into();
            }

            return err.into();
        };

        self.load_bt_last_connect();
        self.sync_bt_last_connect();

        // Healthcheck - AirPlay server
        if let Some(_server) = self.server.as_ref() {
            // TODO - detect accept() failures
        }

        if let Some(last_mac) = self.last_mac {
            let mac = NcmHelper::find_mac_address(&self.iface);
            match mac {
                Ok(v) if v != last_mac => {
                    return Err(LocalError::MacAddressChanged {
                        prev: last_mac,
                        current: v,
                    });
                }
                Err(err) => {
                    return Err(LocalError::InterfaceGone(err));
                }
                _ => {}
            }
        }

        match _status {
            LocalState::Initial => LocalState::WaitingForBluetooth { hci: self.hci.clone() }.into(),
            LocalState::WaitingForBluetooth { hci } => {
                let mac = match BluetoothManager::query_mac(hci).await {
                    Err(err) => {
                        // Polled ~every 500ms; warn early and periodically so a
                        // BlueZ that never comes up is visible in the log.
                        let attempt = BT_QUERY_FAIL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                        if attempt <= 5 || attempt % 60 == 0 {
                            warn!("BlueZ not reachable on {hci} (attempt {attempt}): {err:?}");
                        } else {
                            trace!("BlueZ is down: {err:?}");
                        }
                        return LocalState::WaitingForBluetooth { hci: hci.into() }.into();
                    }
                    Ok(v) => {
                        BT_QUERY_FAIL_COUNT.store(0, Ordering::Relaxed);
                        v
                    }
                };

                info!("Discovered Bluetooth MAC {mac} on {hci}");
                LocalState::StartingBluetooth { hci: hci.clone(), mac }.into()
            }
            LocalState::StartingBluetooth { hci, mac } => {
                let mfi = self.mfi.clone();
                let identity = CarPlaySessionIdentity {
                    display_name: self.name.clone(),
                    ncm_iface: None,
                    bt_mac: Some(*mac),
                    wifi_ssid: Some(self.ssid.clone()),
                    wifi_passphrase: self.pass.clone(),
                    wifi_is_wpa: self.wpa,
                    wifi_channel: self.channel,
                    is_usb_transport: false,
                    has_gps: true,
                    wants_now_playing: false,
                    ..CarPlaySessionIdentity::default()
                };

                let cb = move || CarPlayServerSession::new(mfi.clone(), identity.clone()).0;
                let mut mgr = BluetoothManager::new(false, true, true, hci, cb);
                mgr.start().await.map_err(|e| LocalError::Bluetooth(e.into()))?;

                self.bluetooth.replace(mgr);

                LocalState::StartingBluetoothPower {
                    hci: hci.into(),
                    mac: *mac,
                }
                .into()
            }
            LocalState::StartingBluetoothPower { hci, mac } => {
                // If possible, our app will be the first to set Powered = true;
                // this isn't strictly necessary, but reduces chances of a race where iOS caches SDP profiles without iAP2

                if let Err(err) = BluetoothManager::start_power(hci, &self.name).await {
                    let attempt = BT_POWER_FAIL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                    // This state is re-polled ~every 500ms; warn only on early
                    // failures and periodically, so a persistent failure stays
                    // visible in catplay.log without flooding the 128K log.
                    if attempt <= 5 || attempt % 60 == 0 {
                        warn!("Could not power-up Bluetooth on {hci} (attempt {attempt}): {err}");
                    } else {
                        debug!("Could not power-up Bluetooth on {hci} (attempt {attempt}): {err}");
                    }

                    // Repeated Powered = true attempts also automate unglitching process of some Realtek chips
                    return LocalState::StartingBluetoothPower {
                        hci: hci.into(),
                        mac: *mac,
                    }
                    .into();
                }
                BT_POWER_FAIL_COUNT.store(0, Ordering::Relaxed);
                info!("Bluetooth {hci} powered on (alias: {})", self.name);

                LocalState::WaitingForInterface { iface: self.iface.clone() }.into()
            }
            LocalState::WaitingForInterface { iface } => {
                let mac = NcmHelper::find_mac_address(iface);
                match mac {
                    Ok(mac) => {
                        self.last_mac.replace(mac);
                        LocalState::WaitingForInterfaceRunning {
                            iface: iface.to_string(),
                            mac,
                        }
                    }
                    .into(),
                    Err(_) => LocalState::WaitingForInterface { iface: iface.to_string() }.into(),
                }
            }
            LocalState::WaitingForInterfaceRunning { iface, mac } => {
                let stable = NcmHelper::is_iface_running(iface);

                match stable {
                    Ok(true) => LocalState::WaitingForStableMulticast {
                        iface: iface.to_string(),
                        mac: *mac,
                    }
                    .into(),
                    _ => LocalState::WaitingForInterfaceRunning {
                        iface: iface.to_string(),
                        mac: *mac,
                    }
                    .into(),
                }
            }

            LocalState::WaitingForStableMulticast { iface, mac } => {
                let stable = NcmHelper::is_mdns_v6_stable(iface);

                match stable {
                    Ok(_) => LocalState::StartingServer {
                        iface: iface.to_string(),
                        mac: *mac,
                    }
                    .into(),
                    Err(_) => LocalState::WaitingForStableMulticast {
                        iface: iface.to_string(),
                        mac: *mac,
                    }
                    .into(),
                }
            }

            LocalState::StartingServer { iface, mac } => {
                const SERVER_BIND_PORT: u16 = 5000;

                let sink = self.sink.clone();
                let mut server = AirPlayServer::new(
                    SERVER_BIND_PORT,
                    *mac,
                    iface,
                    self.homekit.clone(),
                    self.mfi.clone(),
                    self.shared.clone(),
                    AirPlayReceiverProfile::CarPlay,
                    move || (sink)(),
                );

                self.last_mac.replace(*mac);
                server.bind().await.map_err(|err| LocalError::AirPlayServer(err.into()))?;
                server.start_advertise().map_err(|err| LocalError::Bonjour(err.into()))?;
                self.server.replace(server);

                if let Some(last_bt_peer) = self.last_bt_peer {
                    return LocalState::StartingLastConnect {
                        hci: self.hci.clone(),
                        mac: *mac,
                        peer_mac: last_bt_peer,
                    }
                    .into();
                }

                match self.invites_blocked {
                    true => LocalState::Passive.into(),
                    false => LocalState::Inviting.into(),
                }
            }
            LocalState::StartingLastConnect { hci, peer_mac, .. } => {
                if self.last_bt_peer_task.is_none() {
                    let hci = hci.clone();
                    let peer_mac = *peer_mac;
                    let task = spawn(async move {
                        const INVITE_DEADLINE: Duration = Duration::from_millis(60000);
                        const INVITE_RETRY: Duration = Duration::from_millis(100);

                        let invite_deadline = Instant::now() + INVITE_DEADLINE;
                        let mut last_err = None;

                        loop {
                            match BluetoothManager::invite_iphone(&hci, &peer_mac).await {
                                Ok(_) => {
                                    info!("Successful reconnect to iPhone peer {peer_mac}");
                                    return;
                                }
                                Err(err) => {
                                    last_err.replace(err);
                                }
                            }

                            let remaining = invite_deadline.saturating_duration_since(Instant::now());
                            if remaining.is_zero() {
                                break;
                            }

                            sleep(remaining.min(INVITE_RETRY)).await;
                        }

                        if let Some(err) = last_err {
                            warn!("Failed to reconnect to iPhone peer {peer_mac} before invite deadline: {err:?}");
                        }
                    });
                    self.last_bt_peer_task.replace(task);
                }
                match self.invites_blocked {
                    true => LocalState::Passive.into(),
                    false => LocalState::Inviting.into(),
                }
            }
            LocalState::Inviting {} => {
                if self.find_active_session().is_some() {
                    return LocalState::Receiving.into();
                }

                let Some(server) = self.server.as_mut() else {
                    return LocalError::UnexpectedState.into();
                };

                server.start_inviting();

                match self.invites_blocked {
                    true => LocalState::Passive.into(),
                    false => LocalState::Inviting.into(),
                }
            }
            LocalState::Receiving | LocalState::Passive => {
                let Some(server) = self.server.as_mut() else {
                    return LocalError::UnexpectedState.into();
                };

                if _status == &LocalState::Receiving {
                    self.last_bt_peer_task.take(); // Stop last connect task now
                }

                server.stop_inviting();

                match self.find_active_session() {
                    None if self.invites_blocked => LocalState::Passive.into(),
                    None => LocalState::Inviting.into(),
                    Some(_) => LocalState::Receiving.into(),
                }
            }
        }
    }
}
