use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use catplay_bonjour::BonjourEntry;
use catplay_carplay::{
    carplay_tx::{
        AirPlayTransmitterBootstrap, AirPlayTransmitterBootstrapError, AirPlayTransmitterBootstrapResult, AirPlayTransmitterProxy,
        AirPlayTransmitterProxyRef, AirPlayTransmitterSessionError, TeardownGuard,
    },
    common::AirPlayBonjourEntry,
    ctrl::CarPlayCtrlServer,
};
use catplay_hap::HomekitStorageRef;
use catplay_iap2_usb::GadgetResult;
use catplay_iap2_usb_host::{CarPlayPhoneGadget, CarPlayPhoneGadgetError, CarPlayPhoneGadgetStatus};
use catplay_util::{
    ArcBox, AsyncShutdown, EventReconciler, EventSleeper, LazyAsync, Reconcilable, Reconciler, deadline_after, event_select,
    notify::Notify, sleeper,
};
use log::{debug, error, info, warn};

use crate::carplay_client_session::CarPlayClientSession;

/// CarPlay transmitter (iPhone)
pub struct CarPlayUsbClientGadget {
    g: Reconciler<CarPlayPhoneGadget>,
    // Bonjour ID (preferably mac address)
    bonjour_id: String,
    homekit: HomekitStorageRef,

    last_session_id: Option<String>,
    last_announce: Option<Instant>,
    ctrl_server: Option<CarPlayCtrlServer>,

    pending_connect: Option<LazyAsync<AirPlayTransmitterBootstrapResult<AirPlayTransmitterProxyRef>>>,
    pending_error: Option<AirPlayTransmitterBootstrapError>,
    pending_transmitter: Option<AirPlayTransmitterProxyRef>,
    pending_transmitter_error: Option<AirPlayTransmitterSessionError>,

    transmitting: Arc<Mutex<bool>>,
    transmitting_end_notify: Notify,
}

impl CarPlayUsbClientGadget {
    const RESTART_DELAY: Duration = Duration::from_millis(1000);

    pub fn new(
        iphone_instance: &str,
        bonjour_id: &str,
        udc: Option<&str>,
        homekit: HomekitStorageRef,
        pinned: bool,
    ) -> GadgetResult<Reconciler<Self>> {
        let g = CarPlayPhoneGadget::new_with_csm(udc, iphone_instance, pinned, || CarPlayClientSession::default())?;
        Ok(Reconciler::new(
            Self {
                g,
                homekit,
                bonjour_id: bonjour_id.into(),
                last_session_id: None,
                last_announce: None,

                ctrl_server: None,
                pending_connect: None,
                pending_transmitter: None,
                pending_error: None,
                pending_transmitter_error: None,
                transmitting: Default::default(),
                transmitting_end_notify: Default::default(),
            },
            Ok(CarPlayUsbClientGadgetStatus::Initial),
        ))
    }

    fn connect_bonjour(&mut self, invite: &BonjourEntry<AirPlayBonjourEntry>) -> ClientResult<()> {
        let peer_ip = *invite
            .meta
            .addrs
            .first()
            .ok_or(CarPlayUsbClientGadgetError::UnexpectedState)?;

        let bootstrap = AirPlayTransmitterBootstrap {
            homekit: self.homekit.clone(),
            peer_ip,
            controller_features: vec![],
            remote_homekit_id: invite.data.pi,
        };

        self.pending_connect
            .replace(LazyAsync::new(move || AirPlayTransmitterProxy::connect(bootstrap)));
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum CarPlayUsbClientGadgetStatus {
    Initial,
    WaitingForUsbAccessory,
    WaitingForInvite,
    Invited { invite: BonjourEntry<AirPlayBonjourEntry> },
    Connecting,
    TransmitterReadyForPickup,
    Transmitting,
}

#[derive(Clone, PartialEq, Debug, thiserror::Error)]
pub enum CarPlayUsbClientGadgetError {
    #[error("CtrlServer {0:?}")]
    CtrlServer(ArcBox<io::Error>),
    #[error("Bonjour {0:?}")]
    Bonjour(ArcBox<io::Error>),
    #[error("UsbGadgetError {0:?}")]
    UsbGadgetError(CarPlayPhoneGadgetError),
    #[error("Failed to connect: {0}")]
    Connect(#[from] AirPlayTransmitterBootstrapError),
    #[error("Transmitter reached Eof status before being picked up: {0}")]
    Eof(#[from] AirPlayTransmitterSessionError),

    // TODO
    #[error("Unexpected state")]
    UnexpectedState,
}

pub type ClientResult<T> = Result<T, CarPlayUsbClientGadgetError>;
pub type ClientState = ClientResult<CarPlayUsbClientGadgetStatus>;

impl<T> From<CarPlayUsbClientGadgetError> for ClientResult<T> {
    fn from(value: CarPlayUsbClientGadgetError) -> Self {
        Err(value)
    }
}

impl From<CarPlayUsbClientGadgetStatus> for ClientState {
    fn from(value: CarPlayUsbClientGadgetStatus) -> Self {
        Ok(value)
    }
}

impl CarPlayUsbClientGadget {
    const ANNOUNCE_INTERVAL: Duration = Duration::from_millis(1000);

    pub fn has_transmitter(&mut self) -> bool {
        self.pending_transmitter.is_some()
    }

    pub fn force_unlock(&mut self) {
        *self.transmitting.lock().unwrap() = false;
        self.transmitting_end_notify.notify();
    }

    pub fn pop_transmitter(&mut self) -> Option<TeardownGuard<AirPlayTransmitterProxyRef>> {
        let t = self.pending_transmitter.take()?;

        let mut transmitting = self.transmitting.lock().unwrap();

        if *transmitting {
            debug!("??? Already transmitting during pop_transmitter");
        }

        *transmitting = true;

        Some(TeardownGuard::new(t, {
            let transmitting = self.transmitting.clone();
            let transmitting_end_notify = self.transmitting_end_notify.clone();
            move || {
                debug!("Dropping transmitting guard!");
                *transmitting.lock().unwrap() = false;
                transmitting_end_notify.notify();
            }
        }))
    }
}

impl AsyncShutdown for CarPlayUsbClientGadget {
    async fn shutdown(&mut self) {
        if let Some(mut ctrl_server) = self.ctrl_server.take() {
            ctrl_server.shutdown().await;
        }

        self.pending_connect.take();
        self.pending_transmitter.take();
        self.g.shutdown().await;
    }
}

impl EventSleeper for CarPlayUsbClientGadget {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        event_select!(
            self.ctrl_server,
            self.pending_connect,
            sleeper(self.g.sleep()),
            deadline_after(Duration::from_millis(100))
        )
    }
}

impl Reconcilable for CarPlayUsbClientGadget {
    type Output = ClientState;

    async fn on_update(&mut self, new: ClientState) -> ClientState {
        match new {
            Ok(ref ok) => info!("Progressing -> {ok:?}"),
            Err(ref err) => error!("Entered error state: {err:?}"),
        };

        if new.is_err() {
            debug!("Cleaning up due to error state");
            if let Some(mut ctrl_server) = self.ctrl_server.take() {
                ctrl_server.shutdown().await;
            }

            self.pending_connect.take();
            self.last_announce = None;
            self.last_session_id = None;
        }

        new
    }

    async fn render(&mut self, prev: ClientState, update: Instant) -> ClientState {
        let Ok(_status) = prev.as_ref() else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return CarPlayUsbClientGadgetStatus::Initial.into();
            }

            return err.into();
        };

        // Reconcile USB gadget
        let _ = self.g.reconcile().await;

        // Health check - USB gadget
        let Ok(g_status) = self.g.state().clone() else {
            let err = self.g.state().as_ref().err().unwrap();
            return CarPlayUsbClientGadgetError::UsbGadgetError(err.clone()).into();
        };

        // Invite turned into connection attempt and that connection attempt turned into failure.
        // Likely a timeout (overloaded HU) or too old CarPlay version and a failed handshake.
        // If HU sends another invite then try again; there's likely no use to forcefully
        // turn one invite into multiple reconnect attempts.
        if let Some(connect) = self.pending_connect.as_mut()
            && let Some(result) = connect.take()
        {
            debug!("Resolved pending connect");
            self.pending_connect.take();
            match result {
                Ok(transmitter) => {
                    self.pending_transmitter.replace(transmitter);
                }
                Err(err) => {
                    self.pending_error.replace(err);
                }
            };
        }

        if let Some(err) = self.pending_error.take() {
            return Err(err)?;
        }

        // Transmitter reached error status after successful handshake, but before being picked up by the user.
        // TODO: fixme
        // if let Some(a) = self.pending_transmitter.as_mut()
        //     && let Err(err) = a().await
        // {
        //     return Err(err)?;
        // }

        let CarPlayPhoneGadgetStatus::CarPlaySession {
            iface,
            session_id,
            ipv6_ll,
            ..
        } = g_status
        else {
            // CarPlay gadget inactive
            if let Some(mut ctrl_server) = self.ctrl_server.take() {
                debug!("Shutting down old CarPlayCtrlServer");
                ctrl_server.shutdown().await;
            }

            return Ok(CarPlayUsbClientGadgetStatus::WaitingForUsbAccessory);
        };

        if self.last_session_id.as_ref() != Some(&session_id) {
            debug!("Cleaning up old session");
            if let Some(mut ctrl_server) = self.ctrl_server.take() {
                ctrl_server.shutdown().await;
            }

            self.last_session_id.replace(session_id);
        }

        if self.ctrl_server.is_none() {
            let port = 60000;
            info!("Starting advertisement of {ipv6_ll:?} using CarPlayCtrlServer");

            let ctrl_server =
                CarPlayCtrlServer::new(&self.bonjour_id, port, &iface).map_err(|e| CarPlayUsbClientGadgetError::CtrlServer(e.into()))?;

            self.ctrl_server.replace(ctrl_server);
        }

        if let Some(ctrl_server) = self.ctrl_server.as_mut()
            && let Some(invite) = ctrl_server.pop_invite()
        {
            match *_status {
                CarPlayUsbClientGadgetStatus::WaitingForInvite => {
                    return Ok(CarPlayUsbClientGadgetStatus::Invited { invite: invite.entry });
                }
                CarPlayUsbClientGadgetStatus::Invited { .. } | CarPlayUsbClientGadgetStatus::Connecting => {
                    warn!("Droping excess invite; already connecting")
                }
                CarPlayUsbClientGadgetStatus::TransmitterReadyForPickup => warn!("Dropping excess invite; transmitter ready for pick up"),
                CarPlayUsbClientGadgetStatus::Transmitting => warn!("Droping excess invite; already transmitting"),

                _ => {
                    warn!("Droping excess invite")
                }
            }
        }

        let now = Instant::now();
        if (self.last_announce.is_none() || now - self.last_announce.unwrap() > Self::ANNOUNCE_INTERVAL)
            && let Some(ctrl_server) = self.ctrl_server.as_mut()
        {
            self.last_announce.replace(now);
            debug!("Forcing ctrl-server announcment");
            if let Err(err) = ctrl_server.force_announce() {
                warn!("Failed to announce? Error: {err:?}");
            }
        }

        if let CarPlayUsbClientGadgetStatus::Invited { invite } = _status {
            info!("Creating transmitter in response to invite: {invite:?}");
            self.connect_bonjour(invite)?;
            return Ok(CarPlayUsbClientGadgetStatus::Connecting);
        }

        match () {
            _ if self.pending_transmitter.is_some() => Ok(CarPlayUsbClientGadgetStatus::TransmitterReadyForPickup),
            _ if self.pending_connect.is_some() => Ok(CarPlayUsbClientGadgetStatus::Connecting),
            _ if *self.transmitting.lock().unwrap() => Ok(CarPlayUsbClientGadgetStatus::Transmitting),
            _ => Ok(CarPlayUsbClientGadgetStatus::WaitingForInvite),
        }
    }
}
