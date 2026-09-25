use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
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
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken, Reconcilable, Reconciler, deadline_after, event_select, mpsc};
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
    burst_wakeups: bool,
    car_wait_since: Option<Instant>,
    last_hb: Option<Instant>,
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

    pub fn new(cfg: ProdGadgetConfig) -> Reconciler<Self> {
        Reconciler::new(
            Self {
                cfg,
                // rx_usb: None,
                rx: None,
                tx: None,
                pending_transmitter: Default::default(),
                last_tx: Default::default(),
                burst_wakeups: false,
                car_wait_since: None,
                last_hb: None,
            },
            Ok(ProdGadgetState::Initial),
        )
    }
}

impl AsyncShutdown for ProdGadget {
    async fn shutdown(&mut self) {
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
            self.pending_transmitter.lock().await.take();
            if let Some(rx) = self.rx.as_mut() {
                rx.child_mut().set_invites_blocked(true)
            };
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
        let _ = self.rx.reconcile().await;
        // let _ = self.rx_usb.reconcile().await;
        let _ = self.tx.reconcile().await;

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

                    move || {
                        let Some(car) = car.lock().unwrap().clone() else {
                            return CarPlayRxSession::reject();
                        };

                        let sess = TxAdapter::new(car.proxy_tx.clone());
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

            let transmitter2 = CarManager::new(transmitter, proxy_chan.1, self.cfg.persist_dir.clone());
            transmitter2.await.expect("TODO").background();
            self.last_tx.lock().unwrap().replace(car_state);
        }

        // TODO: this doesn't properly go back to `blocked` after transmit session is closed
        rx.child_mut().set_invites_blocked(self.last_tx.lock().unwrap().is_none());

        match _status {
            ProdGadgetState::Initial => {
                ProdGadgetState::WaitingForUdc.into()
            }
            ProdGadgetState::WaitingForUdc => {
                ProdGadgetState::WaitingForUsbTransmitterGadget.into()
            }
            ProdGadgetState::WaitingForUsbTransmitterGadget => {
                ProdGadgetState::WaitingForWirelessCarPlayGadget.into()
            }
            ProdGadgetState::WaitingForWirelessCarPlayGadget => {
                ProdGadgetState::WaitingForCar.into()
            }
            ProdGadgetState::WaitingForCar => {
                let car_state = match tx.state() {
                    Ok(CarPlayUsbClientGadgetStatus::Transmitting | CarPlayUsbClientGadgetStatus::TransmitterReadyForPickup) => {
                        self.car_wait_since = None;
                        ProdGadgetState::WaitingForIPhone.into()
                    }
                    _ if pending_transmitter.is_some() => {
                        self.car_wait_since = None;
                        ProdGadgetState::WaitingForIPhone.into()
                    }
                    _ => {
                        let since = *self.car_wait_since.get_or_insert(update);
                        if since.elapsed() >= Duration::from_secs(30)
                            && self.last_hb.map_or(true, |t| t.elapsed() >= Duration::from_secs(30))
                        {
                            self.last_hb = Some(Instant::now());
                            info!(
                                "Still WaitingForCar after {}s: tx={:?} rx={:?}",
                                since.elapsed().as_secs_f32(),
                                tx.state(),
                                rx.state()
                            );
                        }
                        ProdGadgetState::WaitingForCar.into()
                    }
                };
                car_state
            }
            ProdGadgetState::WaitingForIPhone => match rx.state() {
                Ok(CarPlayWirelessGadgetState::Receiving) => ProdGadgetState::Running.into(),
                _ => ProdGadgetState::WaitingForIPhone.into(),
            },
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
