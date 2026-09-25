use std::{
    net::{IpAddr, Ipv6Addr},
    sync::Arc,
    time::{Duration, Instant},
};

use catplay_iap2_client::{
    CsmClient, CsmRemote, CsmSession, CsmSessionCallback, CsmSessionError, CsmSessionStatus,
    tokio::{AsyncClient, AsyncClientStream},
};
use catplay_iap2_usb::{
    GadgetError, GadgetResult, NcmHelper,
    host::{GadgetClient, GadgetHostHelper, RusbHotplugWatcher},
};
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, Reconcilable, Reconciler, deadline_after, event_select};
use log::{debug, error, info, trace, warn};
use macaddr::MacAddr6;
use tokio::io::DuplexStream;
use usb_gadget::Udc;
use uuid::Uuid;

use crate::gadget::{Gadget, GadgetAccessory, GadgetHelper, GadgetStatus, OtgRole};

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum AccessoryError {
    #[error("iPhone was detected but it rejected our role switch request: {0}")]
    PhoneRejectedRoleSwitch(GadgetError),
    #[error("iPhone failed to take interest in our gadget within a reasonable timeout")]
    PhoneFailedToTakeInterestInGadget,
    #[error("iPhone did not connect to iAP2 endpoint within a reasonable timeout")]
    PhoneFailedToInitIAp2,
    /// Note that iPhone suspending USB iAP2 is valid outside of CarPlay context, for example when it dims the screen and enters deep sleep.
    ///
    /// But in CarPlay session it should never happen, and Suspend FFS event is the only (quite reliable, for CarPlay) hint of cable disconnection with ChipIdea UDC.
    ///
    /// (we can't detect VBUS loss, iPhone never provides VBUS to CarPlay headunits)
    #[error("iPhone suspended USB connection or UDC noticed cable disconnection")]
    PhoneSuspendedUSB,

    #[error("iPhone has terminated the iAP2 session: {0:?}")]
    PhoneTerminatedIAp2(CsmSessionStatus),
    #[error("iPhone has terminated the iAP2 session: {0:?}")]
    PhoneTerminatedIAp2V2(CsmSessionError),
    #[error("iPhone has disconnected our accessory from USB bus at status: {0}")]
    PhoneDisconnectedUSB(GadgetStatus),

    #[error("Failed to query USB interface for connected iPhones: {0}")]
    FailedUSBDiscovery(GadgetError),
    #[error("Failed to create gadget: {0}")]
    FailedGadgetCreate(GadgetError),
    #[error("Failed to bind gadget: {0}")]
    FailedGadgetBind(GadgetError),
    #[error("Failed to export CarPlay NCM interface: {0}")]
    FailedNCMExport(GadgetError),

    #[error("Failed to poll USB for hotplugs: {0}")]
    FailedHotplug(GadgetError),

    #[error("Unexpected state")]
    UnexpectedState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AccessoryStatus {
    Initial,
    WaitingForPhone,
    RoleSwitch,
    WaitingForPhoneDisconnect,
    WaitingForEnable {
        pinned: bool,
    },
    WaitingForIAp2Session,

    CarPlaySession {
        iface: String,
        mac_addr: MacAddr6,
        session_id: String,
        ip: IpAddr,
    },
    NonCarPlaySession,
}

pub type AccessoryResult<T> = Result<T, AccessoryError>;
pub type AccessoryState = AccessoryResult<AccessoryStatus>;

impl From<AccessoryError> for AccessoryState {
    fn from(value: AccessoryError) -> Self {
        Err(value)
    }
}

impl From<AccessoryStatus> for AccessoryState {
    fn from(value: AccessoryStatus) -> Self {
        Ok(value)
    }
}

pub struct AccessoryGadget {
    carplay: bool,
    udc: Option<Udc>,

    iphone: Option<GadgetClient>,
    gadget: Option<Gadget>,
    gadget_iap2: Option<AsyncClientStream<DuplexStream>>,

    hotplug: RusbHotplugWatcher,
    hotplug_pending: bool,

    csm: CsmSessionCallback,
    /// Forcefully pin roles in a way that bypasses the role-switch flow.
    ///
    /// This is useful when using a PC host to test interaction with a gadget,
    ///
    /// or wanting to connnect with iPhone as an accessory powered by it over USB-C (when iPhone forcefully assumes itself as a host).
    pinned: bool,
}

impl AccessoryGadget {
    const TIMEOUT_ROLE_SWITCH_REQUEST: Duration = Duration::from_millis(2000);
    const TIMEOUT_PHONE_DISAPPEARANCE: Duration = Duration::from_millis(500);
    /// If you have a non-patched kernel there is a 5000ms OTG FSM penalty to be accounted for (in testing environment, at least)
    const TIMEOUT_GADGET_INTEREST: Duration = Duration::from_millis(8000);
    const TIMEOUT_IAP2_NEGOTIATE: Duration = Duration::from_millis(2000);

    const RESTART_DELAY: Duration = Duration::from_millis(1000);

    const IPV6_LL_IP: Ipv6Addr = Ipv6Addr::new(0xFD00, 0, 0, 0, 0, 0, 1, 2);

    const ENABLE_HOTPLUG_POLLING: bool = true;

    pub fn resolve_udc(udc: Option<&str>, pinned: bool) -> GadgetResult<Option<Udc>> {
        let udc = GadgetHelper::resolve_udc(udc)?;
        if GadgetHelper::requires_udc(false, pinned) && udc.is_none() {
            return Err(GadgetError::MissingUdc);
        }

        Ok(udc)
    }

    pub fn carplay<T: CsmSession, F: Fn() -> T + Send + Sync + 'static>(
        udc: Option<&str>,
        pinned: bool,
        csm: F,
    ) -> GadgetResult<Reconciler<Self>> {
        let udc = Self::resolve_udc(udc, pinned)?;
        let csm: CsmSessionCallback = Arc::new(move || Box::new(csm()));

        Ok(Reconciler::new(
            Self {
                carplay: true,
                hotplug: GadgetHostHelper::create_hotplug_watcher()?,
                hotplug_pending: false,
                udc,

                iphone: None,
                gadget: None,
                gadget_iap2: None,

                csm,
                pinned,
            },
            Ok(AccessoryStatus::Initial),
        ))
    }

    async fn start_iap2(&mut self, socket: DuplexStream) {
        self.shutdown_iap2().await;
        let session = (self.csm)();

        let client = AsyncClient::new(false, CsmRemote::usb_host(), session);
        let pipe = AsyncClientStream::new(client.0, client.1, socket);
        self.gadget_iap2.replace(pipe);
    }

    async fn shutdown_iap2(&mut self) {
        if let Some(mut gadget_iap2) = self.gadget_iap2.take() {
            gadget_iap2.shutdown().await;
        }
    }

    fn detect_phone(&self) -> AccessoryResult<Option<GadgetClient>> {
        Ok(GadgetHostHelper::find_iphones()
            .map_err(AccessoryError::FailedUSBDiscovery)?
            .into_iter()
            .next())
    }

    async fn stop_gadget(&mut self) -> AccessoryResult<()> {
        if let Some(mut gadget) = self.gadget.take() {
            gadget.shutdown().await;
        }

        self.shutdown_iap2().await;

        self.iphone = None;
        Ok(())
    }

    async fn start_gadget(&mut self) -> AccessoryResult<()> {
        if self.gadget.is_some() {
            return Ok(());
        }

        debug!("Creating gadget");

        let udc = self
            .udc
            .as_ref()
            .ok_or(AccessoryError::FailedGadgetCreate(GadgetError::MissingUdc))?;
        let mut gadget = Gadget::new(self.carplay /* is_carplay */, udc, true /* otg */).map_err(AccessoryError::FailedGadgetCreate)?;

        debug!("Starting gadget");
        if let Err(err) = gadget
            .bind()
            .await
            .map_err(AccessoryError::FailedGadgetBind)
        {
            gadget.shutdown().await;
            return Err(err);
        }

        if gadget.has_ncm() {
            // It's fine to always use IPv6 LL when talking to modern iPhones.
            // Only on the reverse side we need to stay flexible talking to different headunits
            // with possible fallback to IPv4 and DHCP.
            let ip = NcmHelper::LINK_LOCAL_IP_CAR;
            if let Err(err) = gadget
                .bind_ncm(ip)
                .await
                .map_err(AccessoryError::FailedNCMExport)
            {
                gadget.shutdown().await;
                return Err(err);
            }
            debug!("Gadget has configured additional NCM interface for CarPlay");
        }
        debug!("Gadget started");

        self.gadget.replace(gadget);
        Ok(())
    }
}

impl AsyncShutdown for AccessoryGadget {
    async fn shutdown(&mut self) {
        let _ = self.stop_gadget().await;
    }
}

impl EventSleeper for AccessoryGadget {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        if !Self::ENABLE_HOTPLUG_POLLING {
            debug!("force hotplug");
            self.hotplug_pending = true;
        }

        event_select!(
            self.hotplug,
            self.gadget_iap2,
            self.gadget,
            deadline_after(Duration::from_millis(100))
        )
    }
}

impl Reconcilable for AccessoryGadget {
    type Output = AccessoryState;

    async fn on_update(&mut self, new: AccessoryState) -> AccessoryState {
        match new {
            Ok(ref ok) => info!("Progressing -> {ok:?}"),
            Err(ref err) => error!("Entered error state: {err}"),
        };

        if new.is_err() {
            self.stop_gadget().await?;
        }

        new
    }

    async fn render(&mut self, prev: AccessoryState, update: Instant) -> AccessoryState {
        let Ok(status) = prev else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return AccessoryStatus::WaitingForPhone.into();
            }

            return err.into();
        };

        // Healthcheck - hotplug
        if let Err(err) = self.hotplug.reconcile().await {
            return Err(AccessoryError::FailedHotplug(err.into()));
        }

        if self.hotplug.take_pending_hotplug() {
            debug!("set hotplug_pending = true");
            self.hotplug_pending = true;
        }

        // Healthcheck - iAP2 session
        if let Some(live_gadget_iap2) = self.gadget_iap2.as_mut() {
            let status = live_gadget_iap2.client_mut().status();
            if status.is_final() {
                let err = AccessoryError::PhoneTerminatedIAp2(status);
                return err.into();
            }

            if let Err(err) = live_gadget_iap2.reconcile().await {
                // TODO rewrite this
                let err = AccessoryError::PhoneTerminatedIAp2V2(err);
                return err.into();
            }
            // TODO: verify CSM session status too
        }

        // Healthcheck - CarPlay gadget
        if let Some(gadget) = self.gadget.as_mut() {
            if let Err(err) = gadget.reconcile().await {
                return AccessoryError::PhoneDisconnectedUSB(GadgetStatus::Error(err)).into();
            }

            let status = gadget.status();
            if status.is_final() {
                return AccessoryError::PhoneDisconnectedUSB(status).into();
            }

            if status == GadgetStatus::Suspended {
                return AccessoryError::PhoneSuspendedUSB.into();
            }

            if let Some(socket) = gadget.accept_iap2() {
                info!("Accepted iAP2 connection from iPhone");
                self.start_iap2(socket).await;
            }
        }

        match status {
            AccessoryStatus::Initial => {
                if let Some(udc) = self.udc.as_ref() {
                    // Best effort
                    debug!("Performing OTG role reset to 'host'");
                    let _ = GadgetHelper::change_usb_otg_role(&udc.name().to_string_lossy(), OtgRole::Host);
                }

                if self.pinned {
                    return AccessoryStatus::WaitingForEnable { pinned: true }.into();
                }

                AccessoryStatus::WaitingForPhone.into()
            }

            AccessoryStatus::WaitingForPhone => {
                self.stop_gadget().await?;

                if !self.hotplug_pending {
                    trace!("Not checking device list, because of no hotplug");
                    return AccessoryStatus::WaitingForPhone.into();
                }

                self.hotplug_pending = false;
                debug!("reset hotplug_pending to false");
                let Some(iphone) = self.detect_phone()? else {
                    debug!("hotplug but no phone found?");
                    return AccessoryStatus::WaitingForPhone.into();
                };

                self.iphone.replace(iphone);
                AccessoryStatus::RoleSwitch.into()
            }
            AccessoryStatus::RoleSwitch => {
                let Some(iphone) = self.iphone.as_ref() else {
                    return AccessoryError::PhoneRejectedRoleSwitch(GadgetError::Timeout).into();
                };

                if let Err(err) = iphone.open() {
                    return AccessoryError::PhoneRejectedRoleSwitch(err).into();
                }

                debug!("iPhone detected, sending role switch");

                // Possibly do iphone.get_capabilities() as well
                iphone
                    .offer_power_capability(500, Self::TIMEOUT_ROLE_SWITCH_REQUEST)
                    .map_err(AccessoryError::PhoneRejectedRoleSwitch)?;
                iphone
                    .role_switch(true, Self::TIMEOUT_ROLE_SWITCH_REQUEST)
                    .map_err(AccessoryError::PhoneRejectedRoleSwitch)?;

                debug!("iPhone accepted role switch");
                AccessoryStatus::WaitingForPhoneDisconnect.into()
            }
            AccessoryStatus::WaitingForPhoneDisconnect => {
                // To be compliant with spec, we should NOT start our gadget until iPhone disconnects!
                if update.elapsed() > Self::TIMEOUT_PHONE_DISAPPEARANCE {
                    warn!("iPhone took too long to disappear post role-switch, continuing anyway");
                    return AccessoryStatus::WaitingForIAp2Session.into();
                }

                let Some(iphone) = self.iphone.as_ref() else {
                    return AccessoryError::PhoneRejectedRoleSwitch(GadgetError::Timeout).into();
                };

                if self.hotplug_pending && iphone.open().is_err() {
                    self.hotplug_pending = false;
                    debug!("iPhone has properly disconnected, continuing with gadget flow");
                    return AccessoryStatus::WaitingForEnable { pinned: false }.into();
                } else if self.hotplug_pending {
                    debug!("hotplug event but phone is still here?");
                }

                AccessoryStatus::WaitingForPhoneDisconnect.into()
            }
            AccessoryStatus::WaitingForEnable { pinned } => {
                // Ensure gadget is started
                self.start_gadget().await?;

                if !pinned && update.elapsed() > Self::TIMEOUT_GADGET_INTEREST {
                    return AccessoryError::PhoneFailedToTakeInterestInGadget.into();
                }

                // Wait for any signs of USB activity and progress to next state
                if let Some(gadget) = self.gadget.as_ref()
                    && gadget.status() == GadgetStatus::Enabled
                {
                    debug!("iPhone has noticed our gadget, continuing");
                    return AccessoryStatus::WaitingForIAp2Session.into();
                }

                AccessoryStatus::WaitingForEnable { pinned }.into()
            }

            AccessoryStatus::WaitingForIAp2Session => {
                // Ensure gadget is started
                self.start_gadget().await?;

                if update.elapsed() > Self::TIMEOUT_IAP2_NEGOTIATE {
                    return AccessoryError::PhoneFailedToInitIAp2.into();
                }

                if let Some(iap2) = self.gadget_iap2.as_mut()
                    && iap2.client_mut().status() == CsmSessionStatus::Writable
                {
                    debug!("iPhone has properly negotiated iAP2 link");
                } else {
                    return AccessoryStatus::WaitingForIAp2Session.into();
                }

                let Some(gadget) = self.gadget.as_ref() else {
                    return AccessoryError::UnexpectedState.into();
                };

                if !gadget.has_ncm() {
                    return AccessoryStatus::NonCarPlaySession.into();
                }

                let iface = gadget.ncm_name().map_err(AccessoryError::FailedNCMExport)?;
                let mac_addr = gadget
                    .mac_address()
                    .map_err(AccessoryError::FailedNCMExport)?;

                if let Some(iface) = iface
                    && let Some(mac_addr) = mac_addr
                {
                    debug!("iPhone has properly negotiated CarPlay link");
                    return AccessoryStatus::CarPlaySession {
                        iface,
                        mac_addr,
                        session_id: Uuid::new_v4().into(),
                        ip: Self::IPV6_LL_IP.into(),
                    }
                    .into();
                }

                AccessoryError::UnexpectedState.into()
            }
            AccessoryStatus::CarPlaySession {
                iface,
                mac_addr,
                session_id,
                ip,
            } => AccessoryStatus::CarPlaySession {
                iface,
                mac_addr,
                session_id,
                ip,
            }
            .into(),
            AccessoryStatus::NonCarPlaySession => AccessoryStatus::NonCarPlaySession.into(),
        }
    }
}
