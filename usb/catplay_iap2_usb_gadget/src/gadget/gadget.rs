use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use catplay_iap2_usb::{GadgetError, GadgetResult, NcmHelper};
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken};
use log::{debug, error, info, trace, warn};
use macaddr::MacAddr6;
use tokio::{io::DuplexStream, select, task::spawn_blocking};
use usb_gadget::Gadget as GadgetExternal;
use usb_gadget::{
    Class, Config, Id, OsDescriptor, RegGadget, Strings, Udc,
    function::{
        Handle,
        custom::{Custom, Endpoint, EndpointDirection, Event, Interface, OsExtCompat},
        net::{Net, NetClass},
    },
};

use crate::gadget::{GadgetAccessory, GadgetBulkPipe, GadgetStatus, OtgRole, OtgRoleBorrow};

pub struct Gadget {
    gadget: Option<RegGadget>,
    status: GadgetStatus,
    event_ready: bool,

    iap2_pipe: Option<GadgetBulkPipe>,
    custom: Custom,
    iap2_pending: Option<DuplexStream>,

    otg_enable: bool,
    otg: Option<OtgRoleBorrow>,
    binding: bool,
    udc: Udc,
    ncm: Arc<Option<Net>>,
}

async fn retry_async<T: Send + 'static, E: Send + 'static, F: Fn() -> Result<T, E> + Send + 'static>(
    interval: Duration,
    retries: usize,
    callback: F,
) -> Result<T, E> {
    let mut tried = 0;
    let callback = Arc::new(Mutex::new(callback));
    let mut ret = (callback.clone().lock().unwrap())();

    while ret.is_err() && tried < retries {
        tried += 1;
        let callback = callback.clone();
        ret = spawn_blocking(move || callback.lock().unwrap()())
            .await
            .unwrap();
        tokio::time::sleep(interval).await;
    }

    ret
}

impl Gadget {
    const _VENDOR_CARPLAY: (u16, u16) = (0x05AC, 0x12FF);
    const VENDOR_CARPLAY: (u16, u16) = (0x25e1, 0x4351);
    const IAP2_OS_EXT_COMPAT: OsExtCompat = OsExtCompat::new(*b"iAP2\0\0\0\0", *b"\0\0\0\0\0\0\0\0");

    // Safe values, don't modify
    const _STRINGS_CARPLAY: (&str, &str, &str) = ("Alpine", "CarPlay Headunit", "CP1234567891");
    const STRINGS_CARPLAY: (&str, &str, &str) = ("Daimler AG", "MB Infotainment", "d2f88445c4f4fc69f");

    const IAP2_PIPE_BUFFER: usize = 1024;

    const CARPLAY_MAC_DEV: MacAddr6 = MacAddr6::new(0x66, 0xf9, 0x7d, 0xf2, 0x3e, 0x2a);
    const CARPLAY_MAC_HOST: MacAddr6 = MacAddr6::new(0x7e, 0x21, 0xb2, 0xcb, 0xd4, 0x51);
    const CARPLAY_NCM_QMULT: u32 = 10;

    pub fn new(is_carplay: bool, udc: &Udc, otg: bool) -> GadgetResult<Self> {
        fn real_iap2(ep1_dir: EndpointDirection, ep2_dir: EndpointDirection) -> (Custom, Handle) {
            Custom::builder()
                .with_interface(
                    Interface::new(Class::vendor_specific(0xF0, 0x00), "iAP Interface")
                        .with_endpoint(Endpoint::bulk(ep1_dir))
                        .with_endpoint(Endpoint::bulk(ep2_dir))
                        .with_os_ext_compat(Gadget::IAP2_OS_EXT_COMPAT),
                )
                .build()
        }

        let (ep1_rx, ep1_dir) = EndpointDirection::host_to_device();
        let (ep2_tx, ep2_dir) = EndpointDirection::device_to_host();

        let (_iap2, iap2_handle) = real_iap2(ep1_dir, ep2_dir);

        let mut builder = Net::builder(NetClass::Ncm);
        builder.dev_addr = Some(Self::CARPLAY_MAC_DEV);
        builder.host_addr = Some(Self::CARPLAY_MAC_HOST);
        builder.qmult = Some(Self::CARPLAY_NCM_QMULT);
        let (_net, ncm_handle) = builder.build();

        let ncm = Some(_net).take_if(|_| is_carplay);

        let iap2_pipe = Some(GadgetBulkPipe::new(ep1_rx, ep2_tx, Self::IAP2_PIPE_BUFFER, Self::IAP2_PIPE_BUFFER)?);

        let description = match is_carplay {
            true => "CarPlay",
            false => "iAP v2 Accessory",
        };

        let mut config = Config::new(description).with_function(iap2_handle);
        if is_carplay {
            config = config.with_function(ncm_handle);
        }

        let (vendor, strings) = (Self::VENDOR_CARPLAY, Self::STRINGS_CARPLAY);
        let g = GadgetExternal::new(
            Class::interface_specific(),
            Id::new(vendor.0, vendor.1),
            Strings::new(strings.0, strings.1, strings.2),
        )
        .with_config(config)
        .with_os_descriptor(OsDescriptor::microsoft());

        let gadget = g
            .register()
            .map_err(|e| GadgetError::FailedGadgetRegister(e.into()))?;

        let handle = Self {
            gadget: Some(gadget),
            status: GadgetStatus::Initial,
            event_ready: false,
            iap2_pipe,
            custom: _iap2,
            iap2_pending: None,

            otg: None,
            otg_enable: otg,
            binding: false,
            udc: udc.clone(),
            ncm: Arc::new(ncm),
        };

        Ok(handle)
    }

    async fn reconcile_event(&mut self) -> GadgetResult<()> {
        enum GadgetEventAction {
            None,
            Enable,
            Disable,
            Unbind,
        }

        let action = {
            let event = self.custom.event().map_err(GadgetError::from)?;

            debug!("Gadget event: {event:?}");
            match event {
                Event::Enable => GadgetEventAction::Enable,
                Event::Disable => GadgetEventAction::Disable,
                Event::Suspend => {
                    info!("iAP2 flow: suspended");
                    self.status = GadgetStatus::Suspended;
                    GadgetEventAction::None
                }
                Event::Resume => {
                    info!("iAP2 flow: unsuspended");
                    self.status = GadgetStatus::Enabled;
                    GadgetEventAction::None
                }
                Event::Bind => {
                    info!("Observed bind event");
                    self.status = GadgetStatus::Bind;
                    GadgetEventAction::None
                }
                Event::Unbind => {
                    // Kernel has forcefully unbound this device.
                    // It doesn't seem to be triggered by user-controlled unbind.
                    info!("Observed unbind event");
                    self.status = GadgetStatus::Unbind;
                    GadgetEventAction::Unbind
                }
                _ => GadgetEventAction::None,
            }
        };

        match action {
            GadgetEventAction::None => {}
            GadgetEventAction::Enable => {
                if let Some(iap2_pipe) = self.iap2_pipe.as_mut() {
                    debug!("Starting/resetting iAP2 pipe");
                    let socket = iap2_pipe.start().await;
                    self.iap2_pending = Some(socket);
                }

                info!("iAP2 flow: enabled");
                self.status = GadgetStatus::Enabled;
            }
            GadgetEventAction::Disable => {
                if let Some(iap2_pipe) = self.iap2_pipe.as_mut() {
                    debug!("Stopping iAP2 pipe because remote has lost interest");
                    iap2_pipe.shutdown().await;
                }

                info!("iAP2 flow: disabled");
                self.status = GadgetStatus::Disabled;
            }
            GadgetEventAction::Unbind => {
                self.shutdown_pipe().await;
            }
        }

        Ok(())
    }

    async fn shutdown_pipe(&mut self) {
        if let Some(iap2_pipe) = self.iap2_pipe.as_mut() {
            iap2_pipe.shutdown().await;
        }
        self.iap2_pending = None;
    }
}

impl GadgetAccessory for Gadget {
    fn has_ncm(&self) -> bool {
        self.ncm.is_some()
    }

    fn ncm_name(&self) -> GadgetResult<Option<String>> {
        match self.ncm.as_ref() {
            Some(ncm) => Ok(ncm.ifname()?.into_string().ok()),
            _ => Ok(None),
        }
    }

    fn mac_address(&self) -> GadgetResult<Option<MacAddr6>> {
        match self.ncm.as_ref() {
            Some(ncm) => Ok(Some(ncm.dev_addr()?)),
            _ => Ok(None),
        }
    }

    fn accept_iap2(&mut self) -> Option<DuplexStream> {
        self.iap2_pending.take()
    }

    async fn unbind_ncm(&mut self) -> GadgetResult<()> {
        if !self.has_ncm() {
            return Err(GadgetError::Unsupported);
        }

        let ncm = self.ncm.clone();

        retry_async(NcmHelper::CDC_NCM_RETRY_INTERVAL, NcmHelper::CDC_NCM_RETRIES, move || {
            let ncm = ncm.as_ref().as_ref().unwrap();

            let iface = ncm.ifname()?.into_string().ok().unwrap();
            NcmHelper::disable_interface(&iface)
        })
        .await
    }

    /// Binds and configures CarPlay NCM interface. Only possible after successful gadget bind.
    async fn bind_ncm(&mut self, ip_with_mask: &str) -> GadgetResult<String> {
        if !self.has_ncm() {
            return Err(GadgetError::Unsupported);
        }

        let ncm = self.ncm.clone();
        let ip_with_mask = ip_with_mask.to_string();

        retry_async(NcmHelper::CDC_NCM_RETRY_INTERVAL, NcmHelper::CDC_NCM_RETRIES, move || {
            let ncm = ncm.as_ref().as_ref().unwrap();

            let iface = ncm.ifname()?.into_string().ok().unwrap();
            NcmHelper::disable_ipv6_dad(&iface)?;
            NcmHelper::disable_ipv6_ra(&iface)?;
            NcmHelper::fix_ipv6_neigh_timeouts(&iface)?;

            NcmHelper::set_interface_ip(&iface, &ip_with_mask)?;
            Ok(iface)
        })
        .await
    }

    fn status(&self) -> GadgetStatus {
        self.status.clone()
    }

    fn is_binding(&self) -> bool {
        self.binding
    }

    async fn bind(&mut self) -> GadgetResult<()> {
        let udc = &self.udc;

        if let Some(gadget) = self.gadget.as_ref() {
            info!(
                "Binding USB gadget {} to {}",
                gadget.name().to_string_lossy(),
                udc.name().to_string_lossy()
            );
            gadget.bind(Some(udc))?;
        }

        if let Some(mut otg) = self.otg.take() {
            otg.shutdown().await;
        }

        self.otg = if self.otg_enable {
            Some(OtgRoleBorrow::with_next(udc, OtgRole::Gadget, OtgRole::Host)?)
        } else {
            None
        };

        self.binding = true;

        Ok(())
    }

    async fn set_soft_connect(&mut self, connect: bool) {
        if let Err(err) = self.udc.set_soft_connect(connect) {
            warn!("Gadget soft-connect was rejected: {}", err)
        }
    }

    async fn unbind(&mut self) -> GadgetResult<()> {
        info!("Forcing gadget unbind");
        if let Some(gadget) = self.gadget.as_ref()
            && let Err(err) = gadget.bind(None)
        {
            warn!("Failed to unbind gadget: {}", err);
        }
        self.binding = false;

        Ok(())
    }
}

impl EventSleeper for Gadget {
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.status.is_final() {
            return None;
        }

        select! {
            ret = self.custom.wait_event() => {
                match ret {
                    Ok(()) => {
                        trace!("Gadget event ready");
                        self.event_ready = true;
                        Some(EventToken(1))
                    }
                    Err(err) => {
                        self.status = GadgetStatus::Error(err.into());
                        Some(EventToken(2))
                    }
                }
            }
            Some(_) = self.iap2_pipe.sleep() => {
                Some(EventToken(3))
            }
        }
    }
}

impl EventReconciler for Gadget {
    type Error = GadgetError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        if self.status.is_final() {
            self.shutdown_pipe().await;
            if let GadgetStatus::Error(err) = &self.status {
                return Err(err.clone());
            }

            return Ok(());
        }

        if let Some(iap2_pipe) = self.iap2_pipe.as_mut()
            && let Err(err) = iap2_pipe.reconcile().await
        {
            let err = GadgetError::from(err);
            self.status = GadgetStatus::Error(err.clone());
            self.shutdown_pipe().await;
            return Err(err);
        }

        if !self.event_ready {
            return Ok(());
        }

        self.event_ready = false;
        if let Err(err) = self.reconcile_event().await {
            self.status = GadgetStatus::Error(err.clone());
            self.shutdown_pipe().await;
            return Err(err);
        }

        Ok(())
    }
}

impl AsyncShutdown for Gadget {
    /// Performs async shutdown.
    ///
    /// The gadget should not be dropped without shutdown, to make sure file descriptors are closed when it's ultimately dropped and not being closed asynchronously.
    async fn shutdown(&mut self) {
        self.status = GadgetStatus::Shutdown;
        self.shutdown_pipe().await;

        let _ = self.unbind().await;

        if let Some(mut otg) = self.otg.take() {
            let _ = otg.shutdown().await;
        }
    }
}

impl Drop for Gadget {
    fn drop(&mut self) {
        if !self.status.is_final() {
            error!("Gadget was dropped without async shutdown first!");
        }
    }
}
