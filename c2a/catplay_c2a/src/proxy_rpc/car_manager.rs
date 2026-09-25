use catplay_bt::BluezManager;
use catplay_carplay::{
    carplay_rx::AirPlayReceiverHandleRef,
    carplay_tx::{AirPlayTransmitter, AirPlayTransmitterProxyRef, AirPlayTransmitterSessionError, TeardownGuard},
    clock::{MediaClockBox, NtpU64},
    events::CommandPending,
    modes::{ChangeModesResponse, ResourceTransferPriority},
    msg::{Command, CommandType, InfoMessageResponse},
    rtsp_frame::{HttpStatus, RtspError, RtspResponse, RtspResult},
    screen::tx::ScreenTransmitProxy,
    video::Pts,
};
use catplay_tracing::ainfo;
use catplay_util::{EventToken, LazyAsync, sleep, spawn};
use catplay_util::{event_select, filling_slot, filling_slot_value, futures_xordered::FuturesOrdered, mpsc, oneshot};
use futures::{FutureExt, future::BoxFuture};
use log::{debug, error, info, warn};
use macaddr::MacAddr6;
use std::{
    collections::VecDeque,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{
    audiov2::AudioProxyUtil,
    proxy_rpc::{ModesArbiter, TxAdapterOp, car_to_iphone_command_task::CarToIphoneCommandTask, overlay_manager::OverlayManager},
};

pub struct CarManager {
    // TODO dont wrap the ref with TeardownGuard...
    car_state: TeardownGuard<AirPlayTransmitterProxyRef>,
    car_had_record: bool,
    car_media_clock: MediaClockBox,

    iphone_peer: Option<IphonePeer>,
    cmd_next: Option<CommandPending>,

    tx_closed: Option<AirPlayTransmitterSessionError>,

    overlay: OverlayManager,

    modes: ModesArbiter,
    initial_screen_steal_attempted: bool,
    night_mode: bool,

    tx_current: Option<LazyAsync<RtspResult<()>>>,
    tx_queue: VecDeque<BoxFuture<'static, RtspResult<()>>>,
    pending_screen_setup: Option<oneshot::Receiver<TeardownGuard<ScreenTransmitProxy>>>,

    error: Option<CarManagerError>,
    proxy_rx: mpsc::UnboundedReceiver<TxAdapterOp>,
}

struct IphonePeer {
    car_to_iphone_proxy: FuturesOrdered<CarToIphoneCommandTask>,
    iphone: AirPlayReceiverHandleRef,
}

#[derive(Debug, Clone, thiserror::Error)]
enum CarManagerError {
    #[error("iPhone did not respond to command {0}, violating TX session sanity")]
    IphoneCommandTimeout(CommandType),
    #[error("iPhone disconnected while command proxies were inflight")]
    IphoneDisconnectedDuringProxy,

    #[error("Car failed to respond to RTSP command: {0}")]
    CarFailedToRespond(RtspError),
    #[error("Car failed screen SETUP: {0}")]
    CarFailedScreenSetup(RtspError),

    #[error("Car disconnected from RTSP session: {0}")]
    TxDisconnect(#[from] AirPlayTransmitterSessionError),
}

impl CarManager {
    pub async fn new(
        car: TeardownGuard<AirPlayTransmitterProxyRef>,
        proxy_rx: mpsc::UnboundedReceiver<TxAdapterOp>,
        persist_dir: Option<PathBuf>,
    ) -> RtspResult<Self> {
        let info = car.info_cached();
        let car_media_clock = car.media_clock();
        let screen = info.displays.first().ok_or(RtspError::Unknown)?;

        let (width, height, fps, dpi) = (
            screen.width_pixels,
            screen.height_pixels,
            screen.max_fps.unwrap_or(30),
            screen.dpi(),
        );
        let overlay = OverlayManager::new(width, height, fps, dpi, persist_dir);

        Ok(Self {
            car_media_clock,
            car_had_record: false,
            iphone_peer: None,
            cmd_next: None,
            overlay,
            tx_closed: None,

            tx_queue: Default::default(),
            night_mode: *info.night_mode.clone().unwrap_or_default(),
            modes: ModesArbiter::new(&info.modes),
            initial_screen_steal_attempted: false,
            error: None,
            pending_screen_setup: Default::default(),
            car_state: car,
            proxy_rx,
            tx_current: None,
        })
    }

    pub fn tx_idle(&self) -> bool {
        match self.tx_current.as_ref() {
            None => true,
            Some(v) if v.is_ready() => true,
            _ => false,
        }
    }

    pub fn recover_inflight(cmd: &mut CommandPending) {
        match cmd.command().get_type() {
            CommandType::ForceKeyFrame
            | CommandType::RequestSiri
            | CommandType::RequestUI
            | CommandType::SetNightMode
            | CommandType::SetLimitedUI
            | CommandType::HidSendReport => {
                warn!("Recovering inflight {} by returning 200 OK", cmd.command().get_type());
                cmd.respond_ok();
            }
            CommandType::ChangeModes => {
                warn!("Recovering inflight ChangeModes by returning an error to the car");
                let _ = cmd.respond_plist(HttpStatus::Ok, ChangeModesResponse::error(2));
            }
            _ => {}
        }
    }

    pub fn background(self) {
        tokio::spawn(async move { self.worker().await });
    }

    pub async fn worker(mut self) {
        debug!("worker start");

        loop {
            if let Err(err) = self.reconcile().await {
                error!("Exiting CarManager because of err: {err}");
                return;
            }

            self.sleep().await.unwrap();
        }
    }

    async fn sleep(&mut self) -> Option<EventToken> {
        let tx = self.car_state.clone();

        if self.tx_idle()
            && let Some(task) = self.tx_queue.pop_front()
        {
            self.tx_current.replace(LazyAsync::new(move || task));
        }

        event_select!(
            self.overlay,
            self.proxy_rx,
            self.pending_screen_setup,
            filling_slot(&mut self.cmd_next, tx.pop_command()),
            filling_slot_value(&mut self.tx_closed, tx.closed()),
            self.iphone_peer
                .as_mut()
                .map(|p| &mut p.car_to_iphone_proxy),
            self.tx_current
        )
    }

    async fn reconcile(&mut self) -> Result<(), CarManagerError> {
        if !self.car_had_record {
            self.car_had_record = true;
            let tx = self.car_state.clone();
            self.tx_queue.push_back(
                async move {
                    let ret = tx.record().boxed().await;
                    catplay_util::sleep(Duration::from_millis(50)).await;

                    ret
                }
                .boxed(),
            );
        }

        if let Some(err) = self.tx_closed.clone() {
            return Err(err)?;
        }

        if let Some(err) = self.error.clone() {
            return Err(err)?;
        }

        if let Some(tx_current) = self.tx_current.as_mut()
            && let Some(Err(err)) = tx_current.as_ref()
        {
            return Err(CarManagerError::CarFailedToRespond(err.clone()));
        }

        if let Some(err) = self.tx_closed.take() {
            return Err(CarManagerError::TxDisconnect(err));
        }

        if let Some(os) = self.pending_screen_setup.as_mut()
            && let Some(Ok(screen)) = os.take()
        {
            self.pending_screen_setup.take();
            self.overlay.set_car_peer(screen);
        }

        if let Some(peer) = self.iphone_peer.as_mut()
            && let Some(mut completed) = peer.car_to_iphone_proxy.take()
        {
            if !completed.cmd.is_complete() {
                warn!(
                    "Recovering car command {:?} after iPhone proxy ended without response",
                    completed.command
                );
                Self::recover_inflight(&mut completed.cmd);

                // TODO: if recover_inflight did nothing, throw sanity error here
            } else if let Command::ChangeModes(ref _v) = completed.command
                && let Some(resp) = completed.response
                && let Ok(p) = resp.get_plist()
            {
                // TODO: if command completed without response, there probably should be a seperate path
                // that clears FIFO state
                warn!("FIFO on_peer_change_modes_end {p:?}");
                self.modes.on_peer_change_modes_end(&p);
            }
        }

        if let Some(op) = self.proxy_rx.take() {
            self.dispatch_tx_op(op);
        }

        if let Some(cmd) = self.cmd_next.take() {
            self.dispatch_car_command(cmd).await;
        }

        self.flush_modes();

        // Don't attempt to forcefully steal Screen if we have a peer and arbitration is active as that will be glitchy
        if self.iphone_peer.is_none() && !self.initial_screen_steal_attempted {
            self.initial_screen_steal_attempted = true;
            self.modes
                .try_steal_screen(ResourceTransferPriority::NiceToHave);
        }
        self.flush_modes();

        self.overlay.reconcile().await;

        Ok(())
    }

    pub fn flush_modes(&mut self) {
        // This enforces very important ordering
        // iPhone takes screen: modesChanged, SETUP
        // Accessory takes screen: modesChanged, TEARDOWN

        if self.modes.modes_dirty() {
            self.modes.clear_dirty();

            let modes = self.modes.modes_mut().serialize_to_state();
            info!("Modes dirty, scheduling modesChanged {modes:?}");

            let tx = self.car_state.clone();
            self.tx_queue
                .push_back(async move { tx.assert_modes(modes).await }.boxed());
        }

        if self.modes.has_screen() && !self.overlay.has_car_peer() && self.pending_screen_setup.is_none() {
            info!("Scheduling screen SETUP");
            // Ensure screen is SETUP
            let tx = self.car_state.clone();

            let (otx, orx) = oneshot::channel();
            self.pending_screen_setup.replace(orx);
            let modes = self.modes.modes_mut().serialize_to_state();

            self.tx_queue.push_back(
                async move {
                    tx.assert_modes(modes).await?;

                    warn!("Screen SETUP start");
                    let screen = tx.setup_screen(Duration::from_millis(25)).await?;
                    warn!("Screen SETUP end");
                    otx.send(screen).map_err(|_| RtspError::Closed)
                }
                .boxed(),
            );
        } else if !self.modes.has_screen() && self.overlay.has_car_peer() && self.pending_screen_setup.is_none() {
            // Ensure screen is TEARDOWN
            info!("Scheduling screen TEARDOWN");

            let screen = self.overlay.drop_car_peer();
            let tx = self.car_state.clone();
            let modes = self.modes.modes_mut().serialize_to_state();

            self.tx_queue.push_back(
                async move {
                    drop(screen);
                    tx.drain_teardown_queue().await?;
                    tx.assert_modes(modes).await?;
                    Ok(())
                }
                .boxed(),
            );
        }
    }

    fn sanity_error(&mut self, error: CarManagerError) {
        if self.error.is_none() {
            self.error.replace(error);
        }
    }

    pub fn dispatch_tx_op(&mut self, op: TxAdapterOp) {
        match op {
            TxAdapterOp::IphoneConnected { handle } => {
                self.on_peer_connected(handle);
            }
            TxAdapterOp::IphoneDisconnected => {
                self.on_peer_disconnect();
            }
            TxAdapterOp::ScreenChannel { channel } => {
                debug!("iPhone: screen");
                self.overlay.set_iphone_peer_screen(Some(channel));
            }
            TxAdapterOp::Modes { modes, .. } => {
                warn!("Peer modes updated: {modes:?}");
                self.modes.on_peer_modes_changed(&modes.serialize());
            }
            TxAdapterOp::Command { command, response } => {
                self.on_iphone_command(&command, response);
            }
            TxAdapterOp::PatchInfo { mut info, patched } => {
                // TODO reject conn if needed
                self.on_patch_info(&mut info);
                let _ = patched.send(info);
            }
            TxAdapterOp::SetupAudio {
                latency,
                stream_type,
                audio_type,
                audio_format,
                pcm_format,
                duplex,
                response,
            } => {
                let tx = self.car_state.clone();
                let fut = AudioProxyUtil::open_audio(tx, latency, stream_type, audio_type, audio_format, pcm_format, duplex);
                self.tx_queue.push_back(
                    async move {
                        // Audio setup may fail, but don't treat it as full sanity error - let iPhone decide what it wants to do next
                        let _ = response.send(fut.await);
                        Ok(())
                    }
                    .boxed(),
                );
            }
        }
    }

    pub fn on_peer_connected(&mut self, handle: AirPlayReceiverHandleRef) {
        debug!("iPhone: peer");

        self.iphone_peer.replace(IphonePeer {
            car_to_iphone_proxy: Default::default(),
            iphone: handle.clone(),
        });
        self.overlay.set_iphone_peer(Some(handle.clone()));
    }

    pub fn on_peer_disconnect(&mut self) {
        debug!("iPhone gone?");
        if let Some(peer) = self.iphone_peer.take() {
            // If iPhone happened to disconnect while we were proxying commands
            // treat this as a hard sanity error
            if !peer.car_to_iphone_proxy.is_empty() {
                // TODO: this should use recover_inflight
                // but for now there's no API to deconstruct FuturesOrdered into parts to access the tasks
                self.sanity_error(CarManagerError::IphoneDisconnectedDuringProxy);
            }
            warn!("modes.on_peer_disconnect()");
            self.modes.on_peer_disconnect();
        }
        self.overlay.set_iphone_peer_screen(None);
        self.overlay.set_iphone_peer(None);
    }

    pub fn on_patch_info(&mut self, info: &mut InfoMessageResponse) {
        let car = self.car_state.clone();

        let info_patched = {
            let other = car.info_cached().clone();
            info!("CarPlay info of the car: {other:?}");

            // Sync data based on car probe
            // Time-sensitive fields: modes, night_mode (best-effort initial sync)
            // Use cached info; protocol says /info could be called more than once, but that never happens in practice.
            let info_clone = info.clone();

            InfoMessageResponse {
                hid_devices: other.hid_devices,
                hid_languages: other.hid_languages,
                right_hand_drive: other.right_hand_drive,

                night_mode: Some(self.night_mode.into()),
                modes: self.modes.modes_mut().serialize_to_info_request(),

                oem_icon: other.oem_icon,
                oem_icon_label: Some("CatPlay".into()),
                oem_icon_visible: other.oem_icon_visible,
                oem_icons: other.oem_icons,

                manufacturer: other.manufacturer,
                model: other.model,
                displays: other.displays,

                audio_latencies: other.audio_latencies,
                features: other.features,

                ..info_clone
            }
        };
        *info = info_patched;
        info!("CarPlay info returned to iPhone (merged): {info:?}");
    }

    pub fn on_iphone_command(&mut self, command: &Command, response: oneshot::Sender<RtspResponse>) {
        // Received over Wi-Fi; blocks RX RTSP thread

        const TIMEOUT: Duration = Duration::from_secs(10);
        let command = command.clone();
        let car = self.car_state.clone();

        let proxy = match command.get_type() {
            CommandType::DuckAudio | CommandType::UnduckAudio | CommandType::HidSetInputMode | CommandType::RequestUI => true,
            CommandType::DisableBluetooth => {
                let command = command.clone();
                let task = spawn(async move {
                    let Command::DisableBluetooth(cmd) = command else {
                        return;
                    };

                    if let Ok(id) = MacAddr6::from_str(&cmd.device_id) {
                        warn!("Disconnecting BT peer: {id}");
                        let _ = BluezManager::new().disconnect_peer("hci0", id).await;
                    }
                });
                task.detach();
                true
            }
            _ => false,
        };

        if proxy {
            info!("Proxying iPhone command {command:?}");

            self.tx_queue.push_back(
                async move {
                    // Send command over USB; blocks with 10s timeout
                    let start = Instant::now();

                    match car.send_command_noresp(&command, TIMEOUT)?.await {
                        Ok(resp) => {
                            info!("Car responded after {:?} with {resp:?}", Instant::now() - start);
                            let _ = response.send(resp);
                            Ok(())
                        }
                        Err(err) => {
                            // Error is Closed/Timeout: critical to session's sanity
                            error!("Car did not respond to command {command:?}: {err}");
                            let _ = response.send(RtspResponse::new(None, HttpStatus::InternalServerError));
                            Err(err)
                        }
                    }
                }
                .boxed(),
            );
        } else {
            if command.get_type() != CommandType::ModesChanged {
                warn!("Ignored iPhone command type {}", command.get_type());
            }

            let _ = response.send(RtspResponse::new(None, HttpStatus::Ok));
        }
    }

    async fn dispatch_car_command(&mut self, cmd: CommandPending) {
        // Received over USB; this function can run in background outside of RTSP TX thread
        // [starts synchronously, but the async task will run in background]
        // This is also latency-sensitive (like hid events) so it makes sense to have a few inflight commands at a time
        // in this direction (and that's implemented already)

        match self.iphone_peer.as_mut() {
            Some(_) => self.on_car_command_connected(cmd),
            None => self.on_car_command_unconnected(cmd).await,
        }
    }

    fn on_car_command_connected(&mut self, mut cmd: CommandPending) {
        let peer = self.iphone_peer.as_mut().expect("iPhone peer missing");
        let phone = peer.iphone.clone();
        let cmd_type = cmd.command().get_type();

        if let Command::HidSendReport(hid) = cmd.command_mut() {
            let car_media_clock = &mut self.car_media_clock;

            let decoded = hid
                .timestamp
                .map(|t| car_media_clock.decode_local(NtpU64(t)));
            if let Some(decoded) = decoded {
                debug!("HID event delta: {}", Pts(decoded));
            }
            let encoded = decoded.and_then(|t| phone.media_clock().encode_remote(t).map(|t| t.0));
            hid.timestamp = encoded;

            if !self.overlay.on_hid_interact() {
                cmd.respond_ok();
                return;
            }
        }

        if let Command::SetNightMode(mode) = cmd.command() {
            self.night_mode = mode.night_mode;
        }

        if let Command::ChangeModes(cm) = cmd.command() {
            self.modes.on_peer_change_modes_start(cm);
        }

        #[derive(PartialEq, Eq)]
        enum ProxyType {
            None,
            Proxy,
            ProxyFastAck,
        }

        pub fn proxy_type(cmd: &Command) -> ProxyType {
            match cmd.get_type() {
                CommandType::ChangeModes
                | CommandType::ForceKeyFrame // TODO
                | CommandType::RequestSiri
                | CommandType::RequestUI
                | CommandType::SetNightMode
                | CommandType::SetLimitedUI => ProxyType::Proxy,
                // Some cars won't queue more than one HID command at a time, and waiting for iPhone's response
                // generates too much latency.
                CommandType::HidSendReport => ProxyType::ProxyFastAck,
                _ => ProxyType::None,
            }
        }

        let proxy = proxy_type(cmd.command());

        match proxy {
            ProxyType::None => {
                info!("Ignored car command type {cmd_type}");
                cmd.respond_ok();
            }
            ProxyType::Proxy | ProxyType::ProxyFastAck => {
                let fast_ack = proxy == ProxyType::ProxyFastAck;
                if fast_ack {
                    cmd.respond_ok();
                }

                let command = cmd.command().clone();
                let command0 = cmd.command().clone();
                ainfo!("Proxying car command {:?}", command0.clone());

                let task = CarToIphoneCommandTask::new(phone, cmd, command);
                peer.car_to_iphone_proxy.push_back(task);
            }
        }
    }

    async fn on_car_command_unconnected(&mut self, mut cmd: CommandPending) {
        match cmd.command() {
            Command::ChangeModes(v) => {
                // With no iPhone peer connected, we accept all requested mode changes (and adjust SETUP/TEARDOWN of screen soon after)
                // unless the request makes no sense, then we reject it.
                // Don't treat such rejection as a hard sanity error; let the car decide what it wants to do.
                info!("Attempting to accept car's ChangeModes as-is: {v:?}");
                let resp = self.modes.process_car_request(v);

                if resp.is_error() {
                    warn!("Rejecting car's ChangeModes: {v:?}");
                }

                sleep(Duration::from_millis(10)).await;

                if let Err(_err) = cmd.respond_plist(HttpStatus::Ok, resp) {
                    cmd.respond_err();
                }

                // Will immediately send modesChanged (because dirty flag is active)
            }
            Command::ForceKeyFrame(_) => {
                self.overlay.force_keyframe();
                cmd.respond_ok();
            }
            Command::HidSendReport(_) => {
                self.overlay.on_hid_interact();
                // Generic UI interaction/clicked any button
                cmd.respond_ok();
            }
            Command::SetNightMode(v) => {
                self.night_mode = v.night_mode;
                cmd.respond_ok();
            }
            Command::RequestUI(_) => {
                warn!("requestUI during overlay!");
                self.modes
                    .try_steal_screen(ResourceTransferPriority::UserInitiated);
                self.modes.mark_dirty();

                sleep(Duration::from_millis(100)).await;
                cmd.respond_ok();
            }
            _ => cmd.respond_ok(),
        };
    }
}

impl Drop for CarManager {
    fn drop(&mut self) {
        // TODO ensure closed in all cases
        if let Some(phone) = self.iphone_peer.take() {
            phone
                .iphone
                .close(RtspError::UnexpectedState("TX proxy is closing".into()));
        }
    }
}
