use std::sync::Arc;

use catplay_util::{
    AsyncShutdown, EventReconciler, EventSleeper,
    io::{Ready as IoReady, SysfsNotify},
};
use log::{info, warn};

use catplay_iap2_usb::{GadgetError, GadgetResult, NcmHelper};

use crate::{GadgetStatus, phone_gadget_driver::PhoneGadgetDriver};

pub struct PhoneGadget {
    status: GadgetStatus,
    status_notify: SysfsNotify,
    phone_bridge: Arc<PhoneGadgetDriver>,
    dirty: bool,

    udc: String,
    binding: bool,
}

impl PhoneGadget {
    pub fn status(&self) -> GadgetStatus {
        self.status.clone()
    }

    pub fn is_binding(&self) -> bool {
        self.binding
    }

    pub async fn bind(&mut self) -> GadgetResult<()> {
        info!("Binding phone bridge gadget");
        self.phone_bridge.bind_async().await?;
        self.binding = true;

        Ok(())
    }

    pub async fn unbind(&mut self) -> GadgetResult<()> {
        info!("Forcing gadget unbind");
        if let Err(err) = self.phone_bridge.unbind_async().await {
            warn!("Failed to unbind gadget: {}", err);
        }
        self.binding = false;

        Ok(())
    }
}

impl PhoneGadget {
    pub fn new(device_name: &str, take_over: bool, udc: &str) -> GadgetResult<Self> {
        let phone_bridge = Arc::new(PhoneGadgetDriver::new(device_name, take_over, udc)?);
        let status_notify =
            SysfsNotify::open(phone_bridge.status_path()).map_err(|err| GadgetError::FailedGadgetStatusCheck(err.into()))?;

        let handle = Self {
            phone_bridge,
            status: GadgetStatus::Initial,
            status_notify,
            dirty: true,

            binding: false,
            udc: udc.into(),
        };

        Ok(handle)
    }
}

impl AsyncShutdown for PhoneGadget {
    async fn shutdown(&mut self) {
        self.status = GadgetStatus::Shutdown;
    }
}

impl EventSleeper for PhoneGadget {
    async fn sleep(&mut self) -> Option<catplay_util::EventToken> {
        let token = self.status_notify.sleep().await?;
        self.dirty = true;
        Some(token)
    }
}

impl EventReconciler for PhoneGadget {
    type Error = GadgetError;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        if !self.dirty {
            return Ok(());
        }

        let ready = self.status_notify.io().pending_ready();
        if ready.is_priority() || ready.is_error() {
            self.status_notify
                .reconcile()
                .await
                .map_err(|err| GadgetError::FailedGadgetStatusCheck(err.into()))?;
            self.status_notify
                .io_mut()
                .clear_ready(IoReady::PRIORITY | IoReady::ERROR)
                .await
                .map_err(|err| GadgetError::FailedGadgetStatusCheck(err.into()))?;
        }

        let new_status = self.phone_bridge.status();
        if new_status != self.status {
            info!("New phone status: {new_status:?}");

            if let Some(accessory) = new_status.as_accessory()
                && let Some(interface) = accessory.ncm
            {
                let _ = NcmHelper::release_from_network_manager(&interface);
            }

            self.status = new_status;
        }
        self.dirty = false;

        Ok(())
    }
}
