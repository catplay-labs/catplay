use std::{fs, io, path::Path};

use log::{debug, warn};
use usb_gadget::{Udc, default_udc, remove_all, udcs, unbind_all};

use crate::gadget::OtgRole;

pub struct GadgetHelper {}

impl GadgetHelper {
    pub fn resolve_udc(udc: Option<&str>) -> io::Result<Option<Udc>> {
        let ret = match udc {
            None => default_udc().ok(),
            Some(udc) => udcs()?.into_iter().find(|u| *u.name() == *udc),
        };
        Ok(ret)
    }

    pub fn requires_udc(phone: bool, pinned: bool) -> bool {
        (phone && !pinned) || (!phone && pinned)
    }

    /// Not needed for DWC2 OTG, but required for ci_hdrc
    pub fn change_usb_otg_role(udc: &str, gadget: OtgRole) -> io::Result<bool> {
        let role = match gadget {
            OtgRole::Gadget => "device",
            OtgRole::Host => "host",
            OtgRole::None => "none",
        };
        let path = Path::new("/sys/class/usb_role")
            .join(format!("{}-role-switch", udc))
            .join("role");

        debug!("Starting OTG role switch -> {role}");
        match path.exists() {
            true => {
                fs::write(path, role).inspect_err(|err| warn!("Role switch to {role} resulted in error: {err}"))?;
                warn!("Role switch to {role} was successful");
                Ok(true)
            }
            false => {
                warn!("Role switch to {role} was impossible (likely auto-managed by kernel)");
                Ok(false)
            }
        }
    }

    pub fn get_usb_otg_role(udc: &str) -> io::Result<Option<Option<bool>>> {
        let path = Path::new("/sys/class/usb_role")
            .join(format!("{}-role-switch", udc))
            .join("role");

        match path.exists() {
            true => {
                let role = String::from_utf8_lossy(&fs::read(path)?).to_string();
                match role.as_str().trim() {
                    "device" => Ok(Some(Some(true))),
                    "host" => Ok(Some(Some(false))),
                    "none" => Ok(Some(None)),
                    _ => Err(io::Error::other(format!("unexpected OTG role: {}", role))),
                }
            }
            false => Ok(None),
        }
    }

    /// Cleanup gadgets in case the app crashed and restarted.
    pub fn cleanup_once() -> io::Result<()> {
        unbind_all()?;
        remove_all()
    }
}
