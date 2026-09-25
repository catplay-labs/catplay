use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use catplay_iap2_client::{
    CsmClient, CsmRemote, CsmSession, CsmSessionCallback, CsmSessionError, CsmSessionStatus,
    tokio::{AsyncClient, AsyncClientStream, IAP2Fd},
};
use catplay_iap2_usb::{NcmHelper, UdcHelper};
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, Reconcilable, Reconciler, deadline_after, event_select};
use log::{debug, error, info};

use catplay_iap2_usb::{GadgetError, GadgetResult};
use uuid::Uuid;

use crate::{AccessoryData, GadgetStatus, phone_gadget::PhoneGadget};

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum CarPlayPhoneGadgetError {
    #[error("Accessory failed to appear after the role switch within a reasonable timeout")]
    AccessoryFailedRoleSwitch,
    #[error("Accessory failed to negotiate iAP2 session")]
    AccessoryFailedToInitIAp2,
    #[error("Accessory claimed to offer CarPlay NCM interface, but it failed to bind: {0}")]
    AccessoryFailedNCM(GadgetError),
    #[error("Accessory has failed to notice our gadget, and all USB reset attempts have been exhausted")]
    AccessoryFailedToNoticeOurGadget,
    #[error("Accessory failed to answer basic USB query")]
    AccessoryFailedUSBQuery(GadgetError),

    #[error("Accessory has disconnected from the USB bus: {0}")]
    AccessoryDisconnectedUSB(GadgetError),
    #[error("Accessory has disconnected from the USB iAP2 bulk endpoints: {0}")]
    AccessoryDisconnectedUSBBulk(GadgetError),
    #[error("Accessory has terminated the iAP2 session: {0:?}")]
    AccessoryDisconnectedIAp2(CsmSessionError),
    #[error("Accessory has disconnected our phone gadget without a role switch at status: {0}")]
    AccessoryDisconnectedWithoutRoleSwitch(GadgetStatus),
    #[error("Accessory has suspended our phone gadget or cable was disconnected")]
    AccessorySuspendedPhoneGadget,

    #[error("Failed to claim USB interface of the accessory: {0}")]
    FailedUSBInterfaceClaim(GadgetError),
    #[error("Failed to query USB interface for accessories: {0}")]
    FailedUSBDiscovery(GadgetError),
    #[error("Failed to create gadget: {0}")]
    FailedGadgetCreate(GadgetError),
    #[error("Failed to bind gadget: {0}")]
    FailedGadgetBind(GadgetError),
    #[error("Failed to request soft-connect: {0}")]
    FailedSoftConnect(GadgetError),
    #[error("Failed to request switch into host mode: {0}")]
    FailedHostSwitch(GadgetError),

    #[error("Failed to poll USB for hotplugs: {0}")]
    FailedHotplug(GadgetError),

    #[error("State machine has entered invalid state")]
    UnexpectedState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CarPlayPhoneGadgetStatus {
    Initial,
    GadgetStarted,
    GadgetFailedToBeNoticedByHost,

    ReceivedEnable,
    ReceivedRoleSwitch,
    WaitingForAccessory {
        pinned: bool,
    },
    DetectedAccessory,
    WaitingForIAp2Session,
    ConfiguringNcm,
    WaitingForStableMulticast,
    MulticastStable,

    CarPlaySession {
        session_id: String,
        iface: String,
        ipv6_ll: IpAddr,
    },

    /// Connected to accessory without CarPlay support.
    NonCarPlaySession,
}

pub type PhoneResult<T> = Result<T, CarPlayPhoneGadgetError>;
pub type PhoneState = PhoneResult<CarPlayPhoneGadgetStatus>;

impl<T> From<CarPlayPhoneGadgetError> for PhoneResult<T> {
    fn from(value: CarPlayPhoneGadgetError) -> Self {
        Err(value)
    }
}

impl From<CarPlayPhoneGadgetStatus> for PhoneState {
    fn from(value: CarPlayPhoneGadgetStatus) -> Self {
        Ok(value)
    }
}

pub struct CarPlayPhoneGadget {
    iphone_instance: String,

    /// Forcefully pin roles in a way that bypasses the role-switch flow.
    ///
    /// This is useful when using a PC host to test interaction with a gadget,
    ///
    /// or wanting to connnect with iPhone as an accessory powered by it over USB-C (when iPhone forcefully assumes itself as a host).
    pinned: bool,

    udc: Option<String>,

    gadget: Option<PhoneGadget>,

    accessory: Option<AccessoryData>,

    accessory_iap2: Option<AsyncClientStream<IAP2Fd>>,

    csm: CsmSessionCallback,
    burst_wakeups: bool,
    status_now: CarPlayPhoneGadgetStatus,
    state_since: Instant,
    last_hb: Option<Instant>,
}

impl CarPlayPhoneGadget {
    const RESTART_DELAY: Duration = Duration::from_millis(1000);
    const ACCESSORY_DETECT_TIMEOUT: Duration = Duration::from_millis(5000);
    const TIMEOUT_IAP2_NEGOTIATE: Duration = Duration::from_millis(8000);
    const WAIT_HEARTBEAT: Duration = Duration::from_secs(30);

    pub fn new_with_csm<T: CsmSession, F: Fn() -> T + Send + Sync + 'static>(
        udc: Option<&str>,
        iphone_instance: &str,
        pinned: bool,
        cb: F,
    ) -> GadgetResult<Reconciler<Self>> {
        let udc = UdcHelper::resolve_udc_name(udc)?;
        if UdcHelper::requires_udc(true, pinned) && udc.is_none() {
            return Err(GadgetError::MissingUdc);
        }

        let csm: CsmSessionCallback = Arc::new(move || Box::new(cb()));
        let me = Self {
            iphone_instance: iphone_instance.into(),
            pinned,
            udc,
            gadget: None,

            accessory: None,

            accessory_iap2: None,
            csm,
            burst_wakeups: false,
            status_now: CarPlayPhoneGadgetStatus::Initial,
            state_since: Instant::now(),
            last_hb: None,
        };
        Ok(Reconciler::new(me, Ok(CarPlayPhoneGadgetStatus::Initial)))
    }

    async fn start_iap2(&mut self) {
        self.shutdown_iap2().await;

        let session = (self.csm)();
        let client = AsyncClient::new(true, CsmRemote::usb_gadget(), session);
        // self.accessory_iap2_status.replace(client.0.subscribe());
        info!("Opening iAP2 FD /dev/iap2-0 for car session");
        let fd = IAP2Fd::open("/dev/iap2-0").expect("/dev/iap2-0 not openable (g_iphone/iap2_char driver?)");

        let pipe = AsyncClientStream::new(client.0, client.1, fd);
        self.accessory_iap2.replace(pipe);
    }

    async fn shutdown_iap2(&mut self) {
        if let Some(mut iap2) = self.accessory_iap2.take() {
            iap2.shutdown().await;
        }
    }
}

impl AsyncShutdown for CarPlayPhoneGadget {
    async fn shutdown(&mut self) {
        let _ = self.stop_gadget().await;
        self.shutdown_iap2().await;
        self.accessory = None;
    }
}

impl CarPlayPhoneGadget {
    fn is_waiting(st: &CarPlayPhoneGadgetStatus) -> bool {
        !matches!(
            st,
            CarPlayPhoneGadgetStatus::Initial | CarPlayPhoneGadgetStatus::CarPlaySession { .. }
        )
    }

    pub async fn stop_gadget(&mut self) -> PhoneResult<()> {
        if let Some(mut gadget) = self.gadget.take() {
            let _ = gadget.unbind().await;
        }
        Ok(())
    }

    pub async fn start_gadget(&mut self) -> PhoneResult<()> {
        if self.gadget.is_some() {
            return Ok(());
        }

        debug!("Creating gadget");

        let udc = self
            .udc
            .as_ref()
            .ok_or(CarPlayPhoneGadgetError::FailedGadgetCreate(GadgetError::MissingUdc))?;

        let mut gadget = PhoneGadget::new(&self.iphone_instance, true, udc).map_err(CarPlayPhoneGadgetError::FailedGadgetCreate)?;
        debug!("Starting gadget");

        if let Err(err) = gadget
            .bind()
            .await
            .map_err(CarPlayPhoneGadgetError::FailedGadgetBind)
        {
            gadget.shutdown().await;
            return Err(err);
        }

        debug!("Gadget started");

        self.gadget.replace(gadget);

        Ok(())
    }
}

impl EventSleeper for CarPlayPhoneGadget {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        let waiting = Self::is_waiting(&self.status_now);
        event_select!(
            self.gadget,
            self.accessory_iap2,
            deadline_after(if self.burst_wakeups {
                Duration::from_millis(20)
            } else if waiting {
                Self::WAIT_HEARTBEAT
            } else {
                Duration::MAX
            })
        )
    }
}

impl Reconcilable for CarPlayPhoneGadget {
    type Output = PhoneState;

    async fn on_update(&mut self, new: PhoneState) -> PhoneState {
        match new {
            Ok(ref ok) => {
                info!("Progressing -> {ok:?}");
                self.status_now = ok.clone();
                self.state_since = Instant::now();
            }
            Err(ref err) => {
                error!("Entered error state: {err}");
                self.status_now = CarPlayPhoneGadgetStatus::Initial;
            }
        };

        if new.is_err() {
            self.shutdown_iap2().await;
            self.stop_gadget().await?;
            self.accessory = None;
        }

        self.burst_wakeups = new.is_err()
            || matches!(
                new,
                Ok(CarPlayPhoneGadgetStatus::WaitingForStableMulticast) | Ok(CarPlayPhoneGadgetStatus::WaitingForAccessory { .. })
            );

        new
    }

    async fn render(&mut self, prev: PhoneState, update: Instant) -> PhoneState {
        let Ok(status) = prev else {
            let err = prev.err().unwrap();

            if update.elapsed() > Self::RESTART_DELAY {
                return CarPlayPhoneGadgetStatus::Initial.into();
            }

            return err.into();
        };

        if let Some(pipe) = self.accessory_iap2.as_mut()
            && let Err(err) = pipe.reconcile().await
        {
            return CarPlayPhoneGadgetError::AccessoryDisconnectedIAp2(err).into();
        }

        if let Some(gadget) = self.gadget.as_mut() {
            if let Err(err) = gadget.reconcile().await {
                return CarPlayPhoneGadgetError::FailedHotplug(err).into();
            }

            let gstatus = gadget.status();
            if Self::is_waiting(&status)
                && update.elapsed() >= Self::WAIT_HEARTBEAT
                && self.last_hb.map_or(true, |t| t.elapsed() >= Self::WAIT_HEARTBEAT)
            {
                self.last_hb = Some(Instant::now());
                info!(
                    "Car USB: still waiting in {status:?} for {}s (raw gadget status: {gstatus:?})",
                    self.state_since.elapsed().as_secs_f32()
                );
            }
            if gstatus.is_final() && !gstatus.is_role_switch() {
                return CarPlayPhoneGadgetError::AccessoryDisconnectedWithoutRoleSwitch(gstatus).into();
            }

            // if gstatus == GadgetStatus::Suspended {
            //     return CarPlayPhoneGadgetError::AccessorySuspendedPhoneGadget.into();
            // }

            if gstatus.is_role_switch()
                && status != CarPlayPhoneGadgetStatus::ReceivedRoleSwitch
                && !matches!(status, CarPlayPhoneGadgetStatus::WaitingForAccessory { pinned: false })
            {
                return CarPlayPhoneGadgetStatus::ReceivedRoleSwitch.into();
            }

            if gstatus.has_accessory()
                && status != CarPlayPhoneGadgetStatus::DetectedAccessory
                && status != CarPlayPhoneGadgetStatus::WaitingForIAp2Session
                && status != CarPlayPhoneGadgetStatus::ConfiguringNcm
                && status != CarPlayPhoneGadgetStatus::WaitingForStableMulticast
                && status != CarPlayPhoneGadgetStatus::MulticastStable
            // TODO ugly
                && !matches!(status, CarPlayPhoneGadgetStatus::CarPlaySession { .. })
            {
                let acc = gstatus.as_accessory().unwrap();
                info!("Car USB accessory detected: vid={} pid={} ncm={:?}", acc.vid, acc.pid, acc.ncm);
                self.accessory.replace(acc);
                return CarPlayPhoneGadgetStatus::DetectedAccessory.into();
            }

            if gstatus == GadgetStatus::Enabled && status != CarPlayPhoneGadgetStatus::ReceivedEnable {
                return CarPlayPhoneGadgetStatus::ReceivedEnable.into();
            }
        };

        if status == CarPlayPhoneGadgetStatus::Initial {
            if self.pinned {
                return CarPlayPhoneGadgetStatus::WaitingForAccessory { pinned: true }.into();
            }

            self.start_gadget().await?;
            return CarPlayPhoneGadgetStatus::GadgetStarted.into();
        }

        if status == CarPlayPhoneGadgetStatus::GadgetStarted {
            return CarPlayPhoneGadgetStatus::GadgetStarted.into();
        }

        if status == CarPlayPhoneGadgetStatus::ReceivedEnable {
            return CarPlayPhoneGadgetStatus::ReceivedEnable.into();
        }

        if status == CarPlayPhoneGadgetStatus::ReceivedRoleSwitch {
            return CarPlayPhoneGadgetStatus::WaitingForAccessory { pinned: false }.into();
        }

        if let CarPlayPhoneGadgetStatus::WaitingForAccessory { pinned } = status {
            if pinned {
                return CarPlayPhoneGadgetStatus::DetectedAccessory.into();
            }

            if update.elapsed() > Self::ACCESSORY_DETECT_TIMEOUT {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            }

            return CarPlayPhoneGadgetStatus::WaitingForAccessory { pinned }.into();
        }

        if status == CarPlayPhoneGadgetStatus::DetectedAccessory {
            let Some(acc) = self.accessory.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            // if accessory.has_ncm() {
            //     accessory.bind_ncm().map_err(CarPlayPhoneGadgetError::AccessoryFailedNCM)?;
            //     let _iface = accessory
            //         .configure_ncm(NcmHelper::LINK_LOCAL_IP_PHONE)
            //         .map_err(CarPlayPhoneGadgetError::AccessoryFailedNCM)?;
            // }

            // let (iap2_pipe, socket) = {
            //     let Some(accessory) = self.accessory.as_ref() else {
            //         return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            //     };

            //     accessory
            //         .open_iap2_pipe(self.use_usb_reset)
            //         .map_err(CarPlayPhoneGadgetError::FailedUSBInterfaceClaim)?
            // };

            self.start_iap2().await;
            return CarPlayPhoneGadgetStatus::WaitingForIAp2Session.into();
        }

        if status == CarPlayPhoneGadgetStatus::WaitingForIAp2Session {
            let Some(accessory) = self.accessory.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            let Some(iap2) = self.accessory_iap2.as_mut() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            if update.elapsed() > Self::TIMEOUT_IAP2_NEGOTIATE {
                return CarPlayPhoneGadgetError::AccessoryFailedToInitIAp2.into();
            }

            if iap2.client_mut().status() != CsmSessionStatus::Writable {
                return CarPlayPhoneGadgetStatus::WaitingForIAp2Session.into();
            }

            if accessory.ncm.is_none() {
                return CarPlayPhoneGadgetStatus::NonCarPlaySession.into();
            }

            return CarPlayPhoneGadgetStatus::ConfiguringNcm.into();
        }

        if status == CarPlayPhoneGadgetStatus::ConfiguringNcm {
            let Some(acc) = self.accessory.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            if let Some(ncm) = acc.ncm.as_ref() {
                NcmHelper::configure_for_carplay(ncm, NcmHelper::LINK_LOCAL_IP_PHONE)
                    .map_err(CarPlayPhoneGadgetError::AccessoryFailedNCM)?;
            };

            return CarPlayPhoneGadgetStatus::WaitingForStableMulticast.into();
        }

        if status == CarPlayPhoneGadgetStatus::WaitingForStableMulticast {
            let Some(accessory) = self.accessory.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            let Some(ncm) = accessory.ncm.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            // if !NcmHelper::has_ipv6_addr(ncm, NcmHelper::LINK_LOCAL_IP_PHONE).map_err(CarPlayPhoneGadgetError::AccessoryFailedNCM)? {
            //     debug!("NCM interface {ncm} lost {}, reconfiguring", NcmHelper::LINK_LOCAL_IP_PHONE);
            //     return CarPlayPhoneGadgetStatus::ConfiguringNcm.into();
            // }

            return match NcmHelper::is_mdns_v6_stable(ncm) {
                Ok(_) => CarPlayPhoneGadgetStatus::MulticastStable.into(),
                Err(err) => {
                    debug!("Multicast unstable at {ncm}: {err:?}");
                    CarPlayPhoneGadgetStatus::WaitingForStableMulticast.into()
                }
            };
        }

        if status == CarPlayPhoneGadgetStatus::MulticastStable {
            let Some(accessory) = self.accessory.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            let Some(ncm) = accessory.ncm.as_ref() else {
                return CarPlayPhoneGadgetError::AccessoryFailedRoleSwitch.into();
            };

            return CarPlayPhoneGadgetStatus::CarPlaySession {
                session_id: Uuid::new_v4().into(),
                iface: ncm.into(),
                ipv6_ll: NcmHelper::LINK_LOCAL_IP_PHONE_RAW.parse().unwrap(),
            }
            .into();
        }

        if let CarPlayPhoneGadgetStatus::CarPlaySession {
            iface: _,
            ipv6_ll: _,
            session_id: _,
            ..
        } = status.clone()
        {
            return status.into();
        }

        if status == CarPlayPhoneGadgetStatus::NonCarPlaySession {
            return status.into();
        }

        CarPlayPhoneGadgetError::UnexpectedState.into()
    }
}
