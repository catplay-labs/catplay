use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use catplay_carplay::{
    carplay_rx::sink::AirPlayServerShared,
    carplay_tx::{AirPlayTransmitterImpl, AirPlayTransmitterSessionError, TeardownGuard},
};
use catplay_carplay_rx_gadget::{CarPlayWirelessGadget, CarPlayWirelessGadgetError, CarPlayWirelessGadgetState};
use catplay_carplay_tx_gadget::client::{CarPlayUsbClientGadget, CarPlayUsbClientGadgetError, CarPlayUsbClientGadgetStatus};
use catplay_hap::HomekitStorageRef;
use catplay_iap2_usb::GadgetError;
use catplay_mfi::MfiDeficeRef;
use catplay_util::handover_trace::{self as trace, Stage};
use catplay_util::{
    AbortOnDropHandle, AsyncShutdown, EventReconciler, EventSleeper, EventToken, Reconcilable, Reconciler, deadline_after, event_select,
    mpsc, oneshot, spawn,
};
use log::{debug, error, info, warn};
use tokio::sync::Mutex as TokioMutex;

use crate::proxy_rpc::{CarManager, CarPlayRxSession, PhoneGate, TxAdapter, TxAdapterOp};

pub struct ProdGadgetConfig {
    pub homekit_rx: HomekitStorageRef,
    pub homekit_tx: HomekitStorageRef,
    pub mfi: Option<MfiDeficeRef>,

    // Car side UDC. None = autodiscover only UDC, Some = UDC name
    pub udc_tx: Option<String>,
    // Extra UDC for optional USB receive. None = disabled, Some = UDC name
    pub udc_rx: Option<String>,

    pub bt_name: String,
    pub hci: String,
    pub iface: String,
    pub wpa: bool,
    pub ssid: String,
    pub channel: Option<u8>,
    pub passphrase: String,

    pub bt_cache_file: String,
    pub persist_dir: Option<PathBuf>,
}

pub struct ProdGadget {
    cfg: ProdGadgetConfig,

    rx: Option<Reconciler<CarPlayWirelessGadget<CarPlayRxSession>>>,
    // rx_usb: Option<Reconciler<CarPlayUsbGadget<CarPlayRxSession>>>,
    tx: Option<Reconciler<CarPlayUsbClientGadget>>,
    pending_transmitter: Arc<TokioMutex<Option<TeardownGuard<AirPlayTransmitterImpl>>>>,

    last_tx: Arc<Mutex<Option<ProdGadgetSession>>>,
    accepting_phone: Arc<AtomicBool>,
    car_task: Option<AbortOnDropHandle<()>>,
    phone_handover: Option<AbortOnDropHandle<catplay_carplay::rtsp_frame::RtspResult<()>>>,
    trace_heartbeat: Option<AbortOnDropHandle<()>>,
    trace_initialized: bool,
    burst_wakeups: bool,
}

#[derive(Clone, PartialEq, Debug, thiserror::Error)]
pub enum ProdGadgetError {
    #[error("Rx error: {0}")]
    Rx(#[from] CarPlayWirelessGadgetError),
    #[error("Tx error: {0}")]
    Tx(#[from] CarPlayUsbClientGadgetError),

    #[error("Failed to setup usb client, missing udc? {0}")]
    FailedToSetupUsbClient(GadgetError),

    #[error("Transmitter reached eof before pickup: {0}")]
    UnpickedTransmitterEof(#[from] AirPlayTransmitterSessionError),

    #[error("Failed to start car session: {0}")]
    CarSessionStart(catplay_carplay::rtsp_frame::RtspError),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProdGadgetState {
    Initial,
    WaitingForUdc,
    WaitingForUsbTransmitterGadget,
    WaitingForWirelessCarPlayGadget,

    WaitingForCar,
    WaitingForIPhone,

    Running,
}

type LocalResult<T> = Result<T, LocalError>;
type LocalStatus = LocalResult<LocalState>;
type LocalError = ProdGadgetError;
type LocalState = ProdGadgetState;

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

impl ProdGadget {
    const RESTART_DELAY: Duration = Duration::from_millis(1000);

    /// Request the next paired phone without depending on a particular input device.
    pub fn request_next_device(&mut self) {
        if self
            .last_tx
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|car| !car.proxy_tx.is_closed())
        {
            trace::arm();
            if let Some(rx) = self.rx.as_mut() {
                rx.child_mut().request_next_device();
            }
        }
    }

    pub fn new(cfg: ProdGadgetConfig) -> Reconciler<Self> {
        Reconciler::new(
            Self {
                cfg,
                // rx_usb: None,
                rx: None,
                tx: None,
                pending_transmitter: Default::default(),
                last_tx: Default::default(),
                accepting_phone: Arc::new(AtomicBool::new(false)),
                car_task: None,
                phone_handover: None,
                trace_heartbeat: None,
                trace_initialized: false,
                burst_wakeups: false,
            },
            Ok(ProdGadgetState::Initial),
        )
    }

    async fn restart_car_session(&mut self) {
        trace::mark(Stage::ParentRadioRestart);
        // Reject new receivers before tearing down the worker they would use.
        self.accepting_phone.store(false, Ordering::Release);
        self.phone_handover.take();
        self.last_tx.lock().unwrap().take();
        if let Some(rx) = self.rx.as_mut() {
            rx.child_mut().close_for_car_restart();
        }
        if let Some(task) = self.car_task.take() {
            task.abort();
            let _ = task.await;
        }
        self.pending_transmitter.lock().await.take();
        if let Some(tx) = self.tx.as_mut() {
            tx.child_mut().restart_session().await;
        }
        info!("Car session released; renewing radio session without USB disconnect");
    }
}

impl AsyncShutdown for ProdGadget {
    async fn shutdown(&mut self) {
        self.phone_handover.take();
        trace::stop();
        self.trace_heartbeat.take();
        self.accepting_phone.store(false, Ordering::Release);
        self.last_tx.lock().unwrap().take();
        if let Some(task) = self.car_task.take() {
            task.abort();
            let _ = task.await;
        }
        self.pending_transmitter.lock().await.take();

        if let Some(mut rx) = self.rx.take() {
            rx.shutdown().await;
        }
        // if let Some(mut rx_usb) = self.rx_usb.take() {
        //     rx_usb.shutdown().await;
        // }
        if let Some(mut tx) = self.tx.take() {
            tx.shutdown().await;
        }
    }
}

#[derive(Clone)]
pub struct ProdGadgetSession {
    pub proxy_tx: mpsc::UnboundedSender<TxAdapterOp>,
    pub(crate) phones: PhoneGate,
}

impl EventSleeper for ProdGadget {
    async fn sleep(&mut self) -> Option<EventToken> {
        trace::mark(Stage::SleepLock);
        let mut pending_transmitter = self.pending_transmitter.lock().await;
        let fallback = if self.burst_wakeups || self.phone_handover.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(500)
        };

        trace::mark(Stage::SleepSelect);
        let event = event_select!(
            pending_transmitter.as_mut(),
            self.tx,
            // self.rx_usb,
            self.rx,
            deadline_after(fallback)
        );
        trace::mark(Stage::SleepReady);
        event
    }
}

impl Reconcilable for ProdGadget {
    type Output = LocalStatus;

    async fn on_update(&mut self, new: LocalStatus) -> LocalStatus {
        match new {
            Ok(ref ok) => info!("Progressing -> {ok:?}"),
            Err(ref err) => error!("Entered error state: {err:?}"),
        };

        if new.is_err() {
            debug!("Cleaning up due to error state");
            self.restart_car_session().await;
        }

        self.burst_wakeups = new.is_err()
            || matches!(
                new,
                Ok(ProdGadgetState::Initial)
                    | Ok(ProdGadgetState::WaitingForUdc)
                    | Ok(ProdGadgetState::WaitingForUsbTransmitterGadget)
                    | Ok(ProdGadgetState::WaitingForWirelessCarPlayGadget)
            );

        new
    }

    async fn render(&mut self, prev: LocalStatus, update: Instant) -> LocalStatus {
        if self
            .phone_handover
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            let done = self.phone_handover.take().unwrap().await;
            if matches!(done, Ok(Ok(()))) {
                info!("Phone handover: media release confirmed; radio session preserved");
            } else {
                warn!("Phone media release failed or stalled; recovering radio and retaining target");
                self.restart_car_session().await;
            }
        }
        if !self.trace_initialized {
            self.trace_initialized = true;
            if trace::start_from_env() {
                self.trace_heartbeat = Some(catplay_util::spawn(async {
                    while trace::enabled() {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        trace::runtime_tick();
                    }
                }));
            }
        }
        trace::parent_tick();
        let stale_car = self
            .last_tx
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|car| car.proxy_tx.is_closed())
            || self
                .car_task
                .as_ref()
                .is_some_and(|task| task.is_finished());
        if stale_car {
            info!("Car worker ended; discarding stale session");
            self.restart_car_session().await;
        }
        // Resolve radio health before allowing a phone to start connecting.
        trace::mark(Stage::ParentTx);
        let _ = self.tx.reconcile().await;
        trace::mark(Stage::ParentAfterTx);
        if self.last_tx.lock().unwrap().is_some()
            && !self
                .tx
                .as_ref()
                .is_some_and(|tx| matches!(tx.state(), Ok(CarPlayUsbClientGadgetStatus::Transmitting)))
        {
            info!("Car USB link ended; clearing its receiver before accepting another phone");
            self.restart_car_session().await;
        }
        if let Some(rx) = self.rx.as_mut() {
            trace::mark(Stage::ParentAdmission);
            let ready = self
                .last_tx
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|car| !car.phones.paused())
                && self.phone_handover.is_none();
            let recorded = self
                .last_tx
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|car| car.phones.recorded());
            rx.child_mut().set_phone_recorded(recorded);
            rx.child_mut().set_invites_blocked(!ready);
            self.accepting_phone.store(ready, Ordering::Release);
        }
        trace::mark(Stage::ParentRx);
        let _ = self.rx.reconcile().await;
        trace::mark(Stage::ParentAfterRx);
        // let _ = self.rx_usb.reconcile().await;
        if self
            .rx
            .as_mut()
            .is_some_and(|rx| rx.child_mut().take_switch_handover_request())
        {
            self.accepting_phone.store(false, Ordering::Release);
            let car = self.last_tx.lock().unwrap().clone();
            if let Some(car) = car {
                car.phones.pause();
                let (ready, wait) = oneshot::channel();
                if car
                    .proxy_tx
                    .unbounded_send(TxAdapterOp::BeginHandover { ready })
                    .is_ok()
                {
                    let released = self
                        .rx
                        .as_ref()
                        .unwrap()
                        .child()
                        .watch_previous_phone_release();
                    self.phone_handover = Some(spawn(async move {
                        let handover = async move {
                            let result = wait
                                .await
                                .map_err(|_| catplay_carplay::rtsp_frame::RtspError::Closed)?;
                            result?;
                            if released.await.unwrap_or(false) {
                                Ok(())
                            } else {
                                Err(catplay_carplay::rtsp_frame::RtspError::Closed)
                            }
                        };
                        match tokio::time::timeout(Duration::from_secs(8), handover).await {
                            Ok(result) => result,
                            _ => Err(catplay_carplay::rtsp_frame::RtspError::Timeout),
                        }
                    }));
                } else {
                    warn!("Radio worker ended before handover; retaining switch target");
                }
            } else {
                warn!("Radio disappeared before handover; retaining switch target");
            }
            return LocalState::WaitingForIPhone.into();
        }

        let Ok(_status) = prev.as_ref() else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return LocalState::Initial.into();
            }

            return err.into();
        };

        let shared = AirPlayServerShared::new();

        if self.rx.is_none() {
            self.rx.replace(CarPlayWirelessGadget::new(
                self.cfg.homekit_rx.clone(),
                self.cfg.mfi.clone(),
                shared.clone(),
                &self.cfg.bt_name,
                &self.cfg.hci,
                &self.cfg.iface,
                &self.cfg.ssid,
                Some(&self.cfg.passphrase),
                self.cfg.wpa,
                self.cfg.channel,
                Some(&self.cfg.bt_cache_file),
                {
                    let car = self.last_tx.clone();
                    let accepting = self.accepting_phone.clone();

                    move || {
                        if !accepting.load(Ordering::Acquire) {
                            return CarPlayRxSession::reject();
                        }
                        let Some(car) = car
                            .lock()
                            .unwrap()
                            .clone()
                            .filter(|car| !car.proxy_tx.is_closed())
                        else {
                            return CarPlayRxSession::reject();
                        };

                        let Some(lease) = car.phones.acquire() else {
                            return CarPlayRxSession::reject();
                        };
                        let sess = TxAdapter::new(car.proxy_tx.clone(), lease);
                        CarPlayRxSession::new(sess)
                    }
                },
            ));
        }

        if self.tx.is_none() {
            let gadget = CarPlayUsbClientGadget::new(
                "default",
                "aa:bb:cc:dd:ee:ff",
                self.cfg.udc_tx.as_deref(),
                self.cfg.homekit_tx.clone(),
                false,
            )
            .map_err(|err| LocalError::FailedToSetupUsbClient(err))?;
            self.tx.replace(gadget);
        }
        trace::mark(Stage::ParentPendingLock);
        let mut pending_transmitter = self.pending_transmitter.lock().await;

        // Transmitter reached error status after successful handshake, but before being picked up by the receiver.
        if let Err(err) = pending_transmitter.reconcile().await {
            return Err(err)?;
        }

        let tx = self.tx.as_mut().unwrap();
        let rx = self.rx.as_mut().unwrap();
        if pending_transmitter.is_none()
            && let Some(transmitter) = tx.child_mut().pop_transmitter()
        {
            debug!("Got a transmitter");
            // pending_transmitter.replace(transmitter);

            let proxy_chan = mpsc::unbounded();
            let phones = PhoneGate::default();
            let car_state = ProdGadgetSession {
                proxy_tx: proxy_chan.0,
                phones: phones.clone(),
            };

            trace::mark(Stage::ParentManagerNew);
            let transmitter2 = CarManager::new(transmitter, proxy_chan.1, self.cfg.persist_dir.clone(), phones)
                .await
                .map_err(LocalError::CarSessionStart)?;
            self.car_task = Some(transmitter2.background());
            self.last_tx.lock().unwrap().replace(car_state);
        }

        let ready = self
            .last_tx
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|car| !car.phones.paused())
            && self.phone_handover.is_none();
        rx.child_mut().set_invites_blocked(!ready);
        self.accepting_phone.store(ready, Ordering::Release);

        trace::mark(Stage::ParentReturn);
        match _status {
            ProdGadgetState::Initial => ProdGadgetState::WaitingForUdc.into(),
            ProdGadgetState::WaitingForUdc => ProdGadgetState::WaitingForUsbTransmitterGadget.into(),
            ProdGadgetState::WaitingForUsbTransmitterGadget => ProdGadgetState::WaitingForWirelessCarPlayGadget.into(),
            ProdGadgetState::WaitingForWirelessCarPlayGadget => ProdGadgetState::WaitingForCar.into(),
            ProdGadgetState::WaitingForCar | ProdGadgetState::WaitingForIPhone => {
                if self.last_tx.lock().unwrap().is_none() {
                    ProdGadgetState::WaitingForCar.into()
                } else if matches!(rx.state(), Ok(CarPlayWirelessGadgetState::Receiving)) {
                    ProdGadgetState::Running.into()
                } else {
                    ProdGadgetState::WaitingForIPhone.into()
                }
            }
            ProdGadgetState::Running => {
                let still_transmitting = matches!(tx.state(), Ok(CarPlayUsbClientGadgetStatus::Transmitting));
                let still_receiving = matches!(rx.state(), Ok(CarPlayWirelessGadgetState::Receiving));

                match still_receiving && still_transmitting {
                    true => ProdGadgetState::Running.into(),
                    false => ProdGadgetState::WaitingForCar.into(),
                }
            }
        }
    }
}
