use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use catplay_carplay::carplay_rx::{
    AirPlayReceiverHandleRef, AirPlayReceiverProfile, AirPlayReceiverSink,
    sink::{AirPlayServer, AirPlayServerShared},
};
use catplay_hap::HomekitStorageRef;
use catplay_iap2_usb::GadgetError;
use catplay_iap2_usb_gadget::{AccessoryError, AccessoryGadget, AccessoryStatus};
use catplay_mfi::MfiDeficeRef;
use catplay_util::{ArcBox, AsyncShutdown, EventReconciler, EventSleeper, Reconcilable, Reconciler, deadline_after, event_select};
use log::{debug, error, info, trace};
use macaddr::MacAddr6;

use crate::{CarPlayServerSession, CarPlaySessionIdentity};

/// CarPlay HeadUnit (USB)
pub struct CarPlayUsbGadget<T: AirPlayReceiverSink> {
    name: String,

    usb_gadget: Option<Reconciler<AccessoryGadget>>,
    server: Option<AirPlayServer>,
    usb_pinned: bool,
    usb_udc: Option<String>,

    // AirPlay shared state
    shared: AirPlayServerShared,

    // Dependencies
    homekit: HomekitStorageRef,
    mfi: Option<MfiDeficeRef>,

    // AirPlay sink
    sink: Arc<dyn Fn() -> T + Send + Sync + 'static>,
}

impl<T: AirPlayReceiverSink> CarPlayUsbGadget<T> {
    pub fn new<F: Fn() -> T + Send + Sync + 'static>(
        homekit: HomekitStorageRef,
        mfi: Option<MfiDeficeRef>,
        shared: AirPlayServerShared,
        udc: Option<&str>,
        pinned: bool,
        name: &str,

        sink: F,
    ) -> Reconciler<Self> {
        Reconciler::new(
            Self {
                name: name.into(),

                server: None,
                usb_pinned: pinned,
                usb_gadget: None,
                usb_udc: udc.map(|u| u.into()),

                shared,

                homekit,
                mfi,

                sink: Arc::new(sink),
            },
            Ok(LocalState::Initial),
        )
    }
}
pub enum UsbServerStatus {
    NotRequested,
    NotConnected,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CarPlayUsbGadgetError {
    StartingGadget(GadgetError),
    Gadget(AccessoryError),

    Bonjour(ArcBox<io::Error>),
    AirPlayServer(ArcBox<io::Error>),
    // UsbGadgetError(CarPlayPhoneGadgetError),
    UnexpectedState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CarPlayUsbGadgetState {
    Initial,
    WaitingForUdc { udc: Option<String>, pinned: bool },
    StartingGadget { udc: Option<String>, pinned: bool },

    WaitingForCarPlaySession,
    StartingServer { iface: String, mac: MacAddr6 },

    Inviting { timeout: Duration },
    Receiving,
    Passive,
}

type LocalResult<T> = Result<T, LocalError>;
type LocalStatus = LocalResult<LocalState>;
type LocalError = CarPlayUsbGadgetError;
type LocalState = CarPlayUsbGadgetState;

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

impl<T: AirPlayReceiverSink> CarPlayUsbGadget<T> {
    const RESTART_DELAY: Duration = Duration::from_millis(1000);
    const INVITE_TIMEOUT: Duration = Duration::from_millis(30000);

    fn find_active_session(&mut self) -> Option<AirPlayReceiverHandleRef> {
        let gadget = self.usb_gadget.as_mut()?;
        if let Some(session) = self.shared.borrow()
            && let Ok(AccessoryStatus::CarPlaySession { iface, .. }) = gadget.state()
            && iface == session.iface()
        {
            return Some(session);
        }

        None
    }
}

impl<T: AirPlayReceiverSink> AsyncShutdown for CarPlayUsbGadget<T> {
    async fn shutdown(&mut self) {
        if let Some(mut server) = self.server.take() {
            server.shutdown().await;
        }

        if let Some(mut gadget) = self.usb_gadget.take() {
            gadget.shutdown().await;
        }
    }
}

impl<T: AirPlayReceiverSink> EventSleeper for CarPlayUsbGadget<T> {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        event_select!(self.usb_gadget, deadline_after(Duration::from_millis(50)))
    }
}

impl<T: AirPlayReceiverSink> Reconcilable for CarPlayUsbGadget<T> {
    type Output = LocalStatus;

    async fn on_update(&mut self, new: LocalStatus) -> LocalStatus {
        match new {
            Ok(ref ok) => info!("Progressing -> {ok:?}"),
            Err(ref err) => error!("Entered error state: {err:?}"),
        };

        if new.is_err() {
            debug!("Cleaning up due to error state");
            self.server.take().shutdown().await;
            self.usb_gadget.take().shutdown().await;
        }

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

        // Healthcheck - USB gadget
        if let Some(gadget) = self.usb_gadget.as_mut() {
            if let Err(err) = gadget.state() {
                return LocalError::Gadget(err.clone()).into();
            }

            let _ = gadget.reconcile().await;
        }

        // Healthcheck - AirPlay server
        if let Some(_server) = self.server.as_ref() {
            // TODO - detect accept() failures
        }

        match _status {
            LocalState::Initial => LocalState::WaitingForUdc {
                udc: self.usb_udc.clone(),
                pinned: self.usb_pinned,
            }
            .into(),
            LocalState::WaitingForUdc { udc, pinned } => {
                let resolved = AccessoryGadget::resolve_udc(udc.as_deref(), *pinned);
                match resolved {
                    Err(err) => {
                        trace!("Still waiting for UDC: {err:?}");
                        LocalState::WaitingForUdc {
                            udc: udc.clone(),
                            pinned: *pinned,
                        }
                        .into()
                    }
                    Ok(_) => LocalState::StartingGadget {
                        udc: udc.clone(),
                        pinned: *pinned,
                    }
                    .into(),
                }
            }
            LocalState::StartingGadget { udc, pinned } => {
                let mfi = self.mfi.clone();

                let gadget_hu = AccessoryGadget::carplay(udc.as_deref(), *pinned, move || {
                    let identity = CarPlaySessionIdentity {
                        display_name: "CatPlay".into(),
                        ncm_iface: Some(1),
                        is_usb_transport: true,
                        has_gps: true,
                        wants_now_playing: true,
                        ..CarPlaySessionIdentity::default()
                    };

                    CarPlayServerSession::new(mfi.clone(), identity).0
                })
                .map_err(LocalError::StartingGadget)?;
                self.usb_gadget.replace(gadget_hu);

                LocalState::WaitingForCarPlaySession.into()
            }
            LocalState::WaitingForCarPlaySession => {
                let Some(gadget) = self.usb_gadget.as_mut() else {
                    return LocalError::UnexpectedState.into();
                };

                match gadget.state() {
                    Ok(AccessoryStatus::CarPlaySession { iface, mac_addr, .. }) => LocalState::StartingServer {
                        iface: iface.clone(),
                        mac: *mac_addr,
                    }
                    .into(),
                    _ => LocalState::WaitingForCarPlaySession.into(),
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
                    move || sink(),
                );

                server
                    .bind()
                    .await
                    .map_err(|err| LocalError::AirPlayServer(err.into()))?;
                server
                    .start_advertise()
                    .map_err(|err| LocalError::Bonjour(err.into()))?;
                self.server.replace(server);

                LocalState::Inviting {
                    timeout: Self::INVITE_TIMEOUT,
                }
                .into()
            }
            LocalState::Inviting { timeout } => {
                if self.find_active_session().is_some() {
                    return LocalState::Receiving.into();
                }

                let Some(server) = self.server.as_mut() else {
                    return LocalError::UnexpectedState.into();
                };

                if update.elapsed() > *timeout {
                    return LocalState::Passive.into();
                }

                server.start_inviting();

                LocalState::Inviting {
                    timeout: Self::INVITE_TIMEOUT,
                }
                .into()
            }
            LocalState::Receiving | LocalState::Passive => {
                let Some(server) = self.server.as_mut() else {
                    return LocalError::UnexpectedState.into();
                };

                server.stop_inviting();

                match self.find_active_session() {
                    None => LocalState::Passive.into(),
                    Some(_) => LocalState::Receiving.into(),
                }
            } // _ => Ok(_status.clone()),
        }
    }
}
