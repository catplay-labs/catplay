use std::{
    fs, io,
    sync::Arc,
    time::{Duration, Instant},
};

use catplay_bt::BluezManager;
use catplay_carplay::carplay_rx::{
    AirPlayReceiverHandleRef, AirPlayReceiverProfile, AirPlayReceiverSink,
    sink::{AirPlayServer, AirPlayServerShared},
};
use catplay_hap::HomekitStorageRef;
use catplay_iap2_bt::{BluetoothError, BluetoothManager};
use catplay_iap2_usb::{GadgetError, NcmHelper};
use catplay_mfi::MfiDeficeRef;
use catplay_util::{
    AbortOnDropHandle, ArcBox, AsyncShutdown, EventSleeper, Reconcilable, Reconciler, deadline_after, event_select, sleep, spawn,
    spawn_blocking,
};
use log::{debug, error, info, trace, warn};
use macaddr::MacAddr6;

#[path = "device_cycle.rs"]
mod device_cycle;
#[path = "switch_intent.rs"]
mod switch_intent;

use crate::{CarPlayServerSession, CarPlaySessionIdentity};
use catplay_util::handover_trace::{self as trace, Stage};

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
    switch_requested: bool,
    switch_query: Option<AbortOnDropHandle<Result<Option<MacAddr6>, String>>>,
    switch_target: Option<(MacAddr6, device_cycle::SwitchTiming)>,
    switch_bt_task: Option<AbortOnDropHandle<bool>>,
    switch_old_session: Option<AirPlayReceiverHandleRef>,
    switch_cache_load: Option<AbortOnDropHandle<Result<(Option<MacAddr6>, Option<MacAddr6>), String>>>,
    switch_cleanup: Option<AbortOnDropHandle<Result<(), String>>>,
    resume_target: Option<MacAddr6>,
    switch_resuming: bool,
    phone_recorded: bool,
    switch_car_reset_requested: bool,
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
                switch_requested: false,
                switch_query: None,
                switch_target: None,
                switch_bt_task: None,
                switch_old_session: None,
                switch_cache_load: None,
                switch_cleanup: None,
                resume_target: None,
                switch_resuming: false,
                phone_recorded: false,
                switch_car_reset_requested: false,
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
        trace::mark(Stage::BluetoothRead);
        if let Some(bt) = self.bluetooth.as_mut() {
            let peer = bt.get_last_connected();
            trace::mark(Stage::BluetoothReadDone);
            return peer;
        }
        trace::mark(Stage::BluetoothReadDone);
        None
    }

    pub fn set_bt_last_connect_cache_file(&mut self, file: impl Into<String>) {
        self.bt_last_connect_cache_file.replace(file.into());
        self.bt_last_connect_cache_loaded = false;
    }

    async fn load_bt_last_connect(&mut self) -> bool {
        if self.bt_last_connect_cache_loaded {
            return true;
        }

        let Some(file) = self.bt_last_connect_cache_file.as_ref() else {
            self.bt_last_connect_cache_loaded = true;
            return true;
        };
        if self.switch_cache_load.is_none() {
            let file = file.clone();
            self.switch_cache_load = Some(spawn_blocking(move || {
                let previous = fs::read_to_string(&file)
                    .ok()
                    .and_then(|value| value.trim().parse::<MacAddr6>().ok());
                let resume = switch_intent::resume(&switch_intent::path(&file))
                    .map_err(|error| error.to_string())?
                    .and_then(|value| value.parse::<MacAddr6>().ok());
                Ok((previous, resume))
            }));
        }
        if !self.switch_cache_load.as_ref().unwrap().is_finished() {
            return false;
        }
        self.bt_last_connect_cache_loaded = true;
        match self.switch_cache_load.take().unwrap().await {
            Ok(Ok((previous, resume))) => {
                self.last_bt_peer = resume.or(previous);
                self.resume_target = resume;
                if let Some(target) = resume {
                    self.switch_target = Some((target, device_cycle::SwitchTiming::new(Instant::now())));
                    info!("Resuming stored phone switch after restart");
                }
            }
            _ => warn!("Could not load Bluetooth preference or switch intent"),
        }
        true
    }

    fn sync_bt_last_connect(&mut self) {
        let last_connect = self.get_bt_last_connect();
        if let Some(peer) = last_connect {
            self.last_bt_peer = Some(peer);
        }
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
        trace::mark(Stage::ReceiverBorrow);
        let session = self.shared.borrow();
        trace::mark(Stage::ReceiverBorrowDone);
        if let Some(session) = session
            && self.iface == session.iface()
        {
            return Some(session);
        }

        None
    }

    pub fn set_invites_blocked(&mut self, blocked: bool) {
        self.invites_blocked = blocked;
        if blocked {
            self.last_bt_peer_task.take();
            if let Some(server) = self.server.as_mut() {
                server.stop_inviting();
            }
        }
    }

    pub fn set_phone_recorded(&mut self, recorded: bool) {
        self.phone_recorded = recorded;
    }

    /// The owner rebuilds the radio while the explicit Bluetooth switch runs.
    pub fn take_switch_handover_request(&mut self) -> bool {
        std::mem::take(&mut self.switch_car_reset_requested)
    }

    pub fn cancel_device_switch(&mut self) {
        if let Some(server) = self.server.as_mut() {
            server.stop_inviting();
        }
        if let Some((target, _)) = self.switch_target {
            self.clear_switch_intent(target);
        }
        self.switch_target = None;
        self.switch_query.take();
        self.switch_bt_task.take();
        self.switch_requested = false;
        self.switch_car_reset_requested = false;
        self.switch_old_session = None;
    }

    fn clear_switch_intent(&mut self, target: MacAddr6) {
        if let Some(file) = self.bt_last_connect_cache_file.as_ref() {
            let path = switch_intent::path(file);
            let target = target.to_string();
            self.switch_cleanup = Some(spawn_blocking(move || {
                switch_intent::clear(&path, &target).map_err(|error| error.to_string())
            }));
        }
    }

    pub fn watch_previous_phone_release(&self) -> AbortOnDropHandle<bool> {
        let old = self.switch_old_session.clone();
        let mut sessions = self.shared.subscribe();
        spawn(async move {
            loop {
                let still_old = {
                    let session = sessions.borrow();
                    old.as_ref()
                        .zip(session.as_ref())
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                };
                if !still_old {
                    return true;
                }
                if sessions.changed().await.is_err() {
                    return false;
                }
            }
        })
    }

    pub fn close_for_car_restart(&mut self) {
        self.set_invites_blocked(true);
        if let Some(session) = self.find_active_session() {
            session.close(catplay_carplay::rtsp_frame::RtspError::UnexpectedState(
                "Car session restarting".into(),
            ));
        }
    }

    /// Coalesce presses while a switch is pending. Pairings are never removed.
    pub fn request_next_device(&mut self) {
        if self.switch_query.is_none() && self.switch_target.is_none() && self.switch_cleanup.is_none() {
            self.switch_requested = true;
        }
    }

    pub fn is_switching_device(&self) -> bool {
        self.switch_requested || self.switch_query.is_some() || self.switch_target.is_some()
    }

    async fn reconcile_device_switch(&mut self) {
        if self
            .switch_cleanup
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            if !matches!(self.switch_cleanup.take().unwrap().await, Ok(Ok(()))) {
                warn!("Could not remove completed switch intent; boot retry limit remains active");
            }
        }
        if let Some(target) = self.resume_target.take() {
            self.switch_resuming = true;
            self.switch_query = Some(spawn(async move { Ok(Some(target)) }));
        }
        if self.switch_requested {
            self.switch_resuming = false;
            self.switch_requested = false;
            // Never drop an active session unless its Bluetooth identity is known.
            let current = self.get_bt_last_connect().or(self.last_bt_peer);
            if self.find_active_session().is_some() && current.is_none() {
                warn!("Cannot switch device: active phone identity is unknown");
                return;
            }
            let adapter = self.hci.clone();
            let intent = self
                .bt_last_connect_cache_file
                .as_ref()
                .map(|file| switch_intent::path(file));
            self.switch_query = Some(spawn(async move {
                let phones = BluezManager::new()
                    .paired_iphones(&adapter)
                    .await
                    .map_err(|e| e.to_string())?;
                let target = device_cycle::next_device(&phones, current);
                if let Some(target) = target
                    && let Some(path) = intent
                {
                    spawn_blocking(move || switch_intent::save(&path, &target.to_string()))
                        .await
                        .map_err(|_| "Switch intent worker failed".to_owned())?
                        .map_err(|error| error.to_string())?;
                }
                Ok(target)
            }));
        }
        if self
            .switch_query
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            match self.switch_query.take().unwrap().await {
                Ok(Ok(Some(target))) => {
                    let previous = self.get_bt_last_connect().or(self.last_bt_peer);
                    // Cancel the old auto-reconnect before selecting a different phone.
                    self.last_bt_peer_task.take();
                    if let Some(server) = self.server.as_mut() {
                        server.block_incoming_phone();
                    }
                    // The radio owner closes this receiver only after entering its
                    // handover phase. Closing it here races inflight command recovery.
                    let timing = device_cycle::SwitchTiming::new(Instant::now());
                    self.switch_old_session = self.find_active_session();
                    let deadline = timing.bluetooth_deadline();
                    let adapter = self.hci.clone();
                    self.switch_target = Some((target, timing));
                    // Keep this task separate from the normal auto-reconnect task:
                    // loss of radio readiness must not cancel the Bluetooth switch.
                    self.switch_bt_task = Some(spawn(async move {
                        if let Some(previous) = previous {
                            let disconnect = BluezManager::new();
                            if !matches!(
                                tokio::time::timeout(Duration::from_secs(3), disconnect.disconnect_peer(&adapter, previous),).await,
                                Ok(Ok(()))
                            ) {
                                debug!("Previous Bluetooth disconnect failed or timed out; continuing target switch");
                            }
                        }
                        info!("Device switch: starting selected iPhone Bluetooth connection");
                        while Instant::now() < deadline {
                            let attempt = tokio::time::timeout_at(
                                (Instant::now() + Duration::from_secs(5))
                                    .min(deadline)
                                    .into(),
                                BluetoothManager::invite_iphone(&adapter, &target),
                            )
                            .await;
                            if matches!(attempt, Ok(Ok(()))) {
                                info!("Device switch: selected iPhone Bluetooth profile connected");
                                return true;
                            }
                            debug!("Device switch: Bluetooth attempt failed or timed out");
                            tokio::time::sleep_until(
                                (Instant::now() + Duration::from_secs(2))
                                    .min(deadline)
                                    .into(),
                            )
                            .await;
                        }
                        false
                    }));
                    self.switch_car_reset_requested = !self.switch_resuming || self.switch_old_session.is_some();
                    self.switch_resuming = false;
                    info!("Device switch selected; preparing phone handover on existing radio session");
                    return;
                }
                Ok(Ok(None)) => info!("No other paired iPhone; keeping current session"),
                Ok(Err(err)) => warn!("Cannot query paired iPhones: {err}"),
                Err(err) => warn!("Device-switch task failed: {err}"),
            }
        }
        if self
            .switch_bt_task
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            let connected = matches!(self.switch_bt_task.take().unwrap().await, Ok(true));
            if connected && let Some((_, timing)) = self.switch_target.as_mut() {
                timing.bluetooth_connected();
            }
        }
        if let Some((target, mut timing)) = self.switch_target {
            let radio_ready = !self.invites_blocked && !self.switch_car_reset_requested;
            let selected_phone = self.get_bt_last_connect() == Some(target);
            if selected_phone {
                // The incoming profile callback may precede the outgoing DBus reply.
                timing.bluetooth_connected();
                self.switch_bt_task.take();
            }
            let active = self.find_active_session();
            let new_session = active.as_ref().is_some_and(|active| {
                !self
                    .switch_old_session
                    .as_ref()
                    .is_some_and(|old| Arc::ptr_eq(active, old))
            });
            if new_session {
                // The server admitted only the selected control peer's addresses.
                // Fast Wi-Fi reconnect does not require a fresh Bluetooth callback.
                timing.bluetooth_connected();
                self.switch_bt_task.take();
            }
            if radio_ready && new_session && self.phone_recorded {
                self.clear_switch_intent(target);
                self.switch_target = None;
                self.switch_bt_task.take();
                self.switch_old_session = None;
                info!("Device switch connected");
            } else if let Some(phase) = timing.observe(Instant::now(), radio_ready) {
                if let Some(session) = active.filter(|_| new_session && !self.phone_recorded) {
                    session.close(catplay_carplay::rtsp_frame::RtspError::UserClosing);
                }
                self.clear_switch_intent(target);
                self.switch_target = None;
                self.switch_car_reset_requested = false;
                self.switch_bt_task.take();
                self.switch_old_session = None;
                warn!("Device switch timed out in {phase:?}; resuming normal discovery");
            } else {
                self.switch_target = Some((target, timing));
            }
        }
    }
}

impl<T: AirPlayReceiverSink> AsyncShutdown for CarPlayWirelessGadget<T> {
    async fn shutdown(&mut self) {
        self.server.take().shutdown().await;

        self.bluetooth.take();
        self.last_bt_peer_task.take();
        self.switch_query.take();
        self.switch_bt_task.take();
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
            if self.switch_target.is_some() {
                self.bt_last_connect_cache_loaded = false;
                self.switch_cache_load.take();
                self.resume_target = None;
            }
            if let Some(mut server) = self.server.take() {
                server.shutdown().await;
            }
            self.bluetooth.take();
            self.last_mac.take();
            self.switch_query.take();
            self.switch_target = None;
            self.switch_old_session = None;
            self.switch_bt_task.take();
            self.switch_car_reset_requested = false;
            self.switch_requested = false;
            self.last_bt_peer_task.take();
        }

        self.burst_wakeups = match new {
            Err(_) => true,
            Ok(LocalState::Inviting | LocalState::Receiving | LocalState::Passive) => false,
            Ok(_) => true,
        };

        new
    }

    async fn render(&mut self, prev: LocalStatus, update: Instant) -> LocalStatus {
        trace::mark(Stage::WirelessRender);
        let Ok(_status) = prev.as_ref() else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return LocalState::Initial.into();
            }

            return err.into();
        };

        trace::mark(Stage::WirelessCache);
        if !self.load_bt_last_connect().await {
            return prev;
        }
        self.sync_bt_last_connect();

        trace::mark(Stage::WirelessSwitch);
        if self.server.is_some() && self.bluetooth.is_some() {
            self.reconcile_device_switch().await;
        }
        let active = self.find_active_session();
        let selected_session_started = active.as_ref().is_some_and(|active| {
            !self
                .switch_old_session
                .as_ref()
                .is_some_and(|old| Arc::ptr_eq(active, old))
        });
        if let Some((target, _)) = self.switch_target
            && self.server.is_some()
        {
            if let Some(server) = self.server.as_mut() {
                if selected_session_started {
                    server.pause_inviting();
                } else if !self.invites_blocked && !self.switch_car_reset_requested {
                    server.start_inviting_target(Some(target));
                } else {
                    server.block_incoming_phone();
                }
            }
            // Invite only this phone, including when its initial early attempt was rejected.
            return LocalState::Passive.into();
        }

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
                        trace!("BlueZ is down: {err:?}");
                        return LocalState::WaitingForBluetooth { hci: hci.into() }.into();
                    }
                    Ok(v) => v,
                };

                debug!("Discovered Bluetooth MAC: {mac}");
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
                mgr.start()
                    .await
                    .map_err(|e| LocalError::Bluetooth(e.into()))?;

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
                    debug!("Could not power-up Bluetooth: {err:?}");

                    // Repeated Powered = true attempts also automate unglitching process of some Realtek chips
                    return LocalState::StartingBluetoothPower {
                        hci: hci.into(),
                        mac: *mac,
                    }
                    .into();
                }

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
                if self.switch_target.is_some() {
                    server.block_incoming_phone();
                }
                server
                    .bind()
                    .await
                    .map_err(|err| LocalError::AirPlayServer(err.into()))?;
                server
                    .start_advertise()
                    .map_err(|err| LocalError::Bonjour(err.into()))?;
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
                // Do not spend the reconnect window before the radio is usable.
                if self.invites_blocked {
                    return prev;
                }
                if self.find_active_session().is_some() {
                    return LocalState::Receiving.into();
                }
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
                if self.invites_blocked {
                    return LocalState::Passive.into();
                }
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
                    None if self.last_bt_peer_task.is_none() && self.last_bt_peer.is_some() => LocalState::StartingLastConnect {
                        hci: self.hci.clone(),
                        mac: self.last_mac.unwrap(),
                        peer_mac: self.last_bt_peer.unwrap(),
                    }
                    .into(),
                    None => LocalState::Inviting.into(),
                    Some(_) => LocalState::Receiving.into(),
                }
            }
        }
    }
}
