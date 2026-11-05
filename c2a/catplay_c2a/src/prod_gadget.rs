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
use catplay_util::{
    AbortOnDropHandle, AsyncShutdown, EventReconciler, EventSleeper, EventToken, Reconcilable, Reconciler, deadline_after, event_select,
    mpsc,
};
use log::{debug, error, info};
use tokio::sync::Mutex as TokioMutex;

use crate::proxy_rpc::{CarManager, CarPlayRxSession, TxAdapter, TxAdapterOp};

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
    car_restart_after: Option<Instant>,
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
                car_restart_after: None,
                burst_wakeups: false,
            },
            Ok(ProdGadgetState::Initial),
        )
    }

    async fn restart_car_session(&mut self) {
        // Reject new receivers before tearing down the worker they would use.
        self.accepting_phone.store(false, Ordering::Release);
        self.last_tx.lock().unwrap().take();
        if let Some(rx) = self.rx.as_mut() {
            rx.child_mut().close_for_car_restart();
        }
        if let Some(task) = self.car_task.take() {
            task.abort();
            let _ = task.await;
        }
        self.pending_transmitter.lock().await.take();
        if let Some(mut tx) = self.tx.take() {
            tx.shutdown().await;
        }
        // Give the HU time to observe the logical USB disconnect.
        self.car_restart_after = Some(Instant::now() + Duration::from_secs(1));
        info!("Car session released; rebuilding USB connection before phone invitation");
    }
}

impl AsyncShutdown for ProdGadget {
    async fn shutdown(&mut self) {
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
}

impl EventSleeper for ProdGadget {
    async fn sleep(&mut self) -> Option<EventToken> {
        let mut pending_transmitter = self.pending_transmitter.lock().await;
        let fallback = if self.burst_wakeups {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(500)
        };

        event_select!(
            pending_transmitter.as_mut(),
            self.tx,
            // self.rx_usb,
            self.rx,
            deadline_after(fallback)
        )
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
        let _ = self.tx.reconcile().await;
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
            let ready = self.last_tx.lock().unwrap().is_some();
            rx.child_mut().set_invites_blocked(!ready);
            self.accepting_phone
                .store(ready && rx.child_mut().current_phone_matches_switch_target(), Ordering::Release);
        }
        let _ = self.rx.reconcile().await;
        // let _ = self.rx_usb.reconcile().await;
        if self
            .rx
            .as_mut()
            .is_some_and(|rx| rx.child_mut().take_switch_car_reset_request())
        {
            self.restart_car_session().await;
            return LocalState::WaitingForCar.into();
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

                        let sess = TxAdapter::new(car.proxy_tx.clone());
                        CarPlayRxSession::new(sess)
                    }
                },
            ));
        }

        if self.tx.is_none() {
            if self
                .car_restart_after
                .is_some_and(|deadline| Instant::now() < deadline)
            {
                return LocalState::WaitingForCar.into();
            }
            self.car_restart_after = None;
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
            let car_state = ProdGadgetSession { proxy_tx: proxy_chan.0 };

            let transmitter2 = CarManager::new(transmitter, proxy_chan.1, self.cfg.persist_dir.clone())
                .await
                .map_err(LocalError::CarSessionStart)?;
            self.car_task = Some(transmitter2.background());
            self.last_tx.lock().unwrap().replace(car_state);
        }

        rx.child_mut()
            .set_invites_blocked(self.last_tx.lock().unwrap().is_none());
        self.accepting_phone.store(
            self.last_tx.lock().unwrap().is_some() && rx.child_mut().current_phone_matches_switch_target(),
            Ordering::Release,
        );

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
