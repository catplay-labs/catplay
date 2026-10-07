use std::{
    error::Error,
    fs::{self, File},
    io,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};

use catplay_util::spawn_blocking;
use log::{debug, trace};

use catplay_iap2_usb::{GadgetError, GadgetResult};

use crate::{AccessoryData, GadgetStatus};

fn read_as_string(path: &str) -> io::Result<String> {
    Ok(String::from_utf8_lossy(&fs::read(path)?).trim().to_string())
}

fn verify_function(udc: &str, path: &str) -> GadgetResult<()> {
    let function = read_as_string(path).map_err(|e| GadgetError::FailedUdcStatusCheck(e.into()))?;
    let function = function.trim();

    if !function.is_empty() && !function.starts_with("iphone") {
        return Err(GadgetError::UnexpectedGadgetFunction {
            udc: udc.into(),
            expected: "iphone".into(),
            found: function.into(),
        });
    }
    Ok(())
}

pub struct PhoneGadgetDriver {
    udc: String,
    ownership: bool,
    device_name: String,
    function_path: String,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum PhoneGadgetDriverError {
    #[error("Failed to load iPhone kernel driver: {0}")]
    DriverLoad(GadgetError),
    // #[error("Failed to unload iPhone kernel driver: {0}")]
    // DriverUnload(GadgetError),
    #[error("Failed to create new iPhone device: {0}")]
    Create(GadgetError),
    #[error("Failed to remove iPhone device: {0}")]
    Remove(GadgetError),

    #[error("Failed to bind iPhone device: {0}")]
    Bind(GadgetError),
    #[error("Failed to unbind iPhone device: {0}")]
    Unbind(GadgetError),
}

impl PhoneGadgetDriverError {
    fn into_arc(self) -> Arc<dyn Error + Send + Sync + 'static> {
        Arc::new(self) as _
    }
}

impl From<PhoneGadgetDriverError> for GadgetError {
    fn from(value: PhoneGadgetDriverError) -> Self {
        GadgetError::Other(value.into_arc().into())
    }
}

pub type PhoneGadgetDriverResult<T> = Result<T, PhoneGadgetDriverError>;

impl PhoneGadgetDriver {
    // const IPHONE_SERIAL: &str = "00008130000E044E384B1D3ADDDDDDDDDDDDDDDA";
    const PATH_SYS_CLASS_IPHONE: &str = "/sys/class/iphone";
    const PATH_SYS_CLASS_IPHONE_CREATE: &str = "/sys/class/iphone/create";
    const PATH_SYS_CLASS_IPHONE_REMOVE: &str = "/sys/class/iphone/remove";
    const PATH_SYS_CLASS_UDC: &str = "/sys/class/udc";
    const PATH_SEP: char = '/';

    fn iphone_param_path(device_name: &str, param: &str) -> String {
        let mut path = String::with_capacity(Self::PATH_SYS_CLASS_IPHONE.len() + device_name.len() + param.len() + 2);
        path.push_str(Self::PATH_SYS_CLASS_IPHONE);
        path.push(Self::PATH_SEP);
        path.push_str(device_name);
        path.push(Self::PATH_SEP);
        path.push_str(param);
        path
    }

    fn udc_function_path(udc: &str) -> String {
        let suffix = "function";
        let mut path = String::with_capacity(Self::PATH_SYS_CLASS_UDC.len() + udc.len() + suffix.len() + 2);
        path.push_str(Self::PATH_SYS_CLASS_UDC);
        path.push(Self::PATH_SEP);
        path.push_str(udc);
        path.push(Self::PATH_SEP);
        path.push_str(suffix);
        path
    }

    pub fn create(device_name: &str) -> GadgetResult<()> {
        let mut opts = File::options();
        opts.write(true);

        Ok(opts
            .open(Path::new(Self::PATH_SYS_CLASS_IPHONE_CREATE))?
            .write(device_name.as_bytes())
            .map(|_| ())?)
    }

    pub fn remove(device_name: &str) -> GadgetResult<()> {
        let mut opts = File::options();
        opts.write(true);

        Ok(opts
            .open(Path::new(Self::PATH_SYS_CLASS_IPHONE_REMOVE))?
            .write(device_name.as_bytes())
            .map(|_| ())?)
    }

    pub fn exists(device_name: &str) -> bool {
        Self::get_status(device_name).is_ok()
    }

    pub fn driver_loaded() -> bool {
        Path::new(Self::PATH_SYS_CLASS_IPHONE).exists()
    }

    pub fn set_param(device_name: &str, param: &str, value: &str) -> GadgetResult<()> {
        let mut opts = File::options();
        opts.write(true);

        debug!("set_param {device_name} {param} {value}");

        let path = Self::iphone_param_path(device_name, param);
        opts.open(path)
            .map_err(|e| GadgetError::FailedGadgetStatusCheck(e.into()))?
            .write(value.as_bytes())
            .map_err(|e| GadgetError::FailedGadgetStatusCheck(e.into()))
            .map(|_| ())
    }

    pub fn get_param(device_name: &str, param: &str) -> GadgetResult<String> {
        trace!("get_param {device_name} {param}");

        let path = Self::iphone_param_path(device_name, param);
        read_as_string(&path).map_err(|e| GadgetError::FailedGadgetStatusCheck(e.into()))
    }

    pub fn set_serial(device_name: &str, serial: &str) -> GadgetResult<()> {
        Self::set_param(device_name, "serial", serial)
    }

    pub fn set_binding(device_name: &str, bind: bool) -> GadgetResult<()> {
        Self::set_param(device_name, "bind", if bind { "1" } else { "0" })
    }

    pub fn set_udc(device_name: &str, udc: &str) -> GadgetResult<()> {
        Self::set_param(device_name, "udc", udc)
    }

    pub fn get_udc(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "udc")
    }

    pub fn get_serial(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "serial")
    }

    pub fn get_status(device_name: &str) -> GadgetResult<GadgetStatus> {
        let status = Self::get_param(device_name, "status")?;
        Ok(match status.trim() {
            "initial" => GadgetStatus::Initial,
            "bind" => GadgetStatus::Bind,
            "enabled" => GadgetStatus::Enabled,
            "disabled" => GadgetStatus::Disabled,
            "roleswitch" => GadgetStatus::RoleSwitch { is_carplay: true },
            "roleswitchfailed" => GadgetStatus::RoleSwitchFailed,
            "unbind" => GadgetStatus::Unbind,
            "suspended" => GadgetStatus::Suspended,
            "accessory" => GadgetStatus::Accessory(AccessoryData {
                vid: Self::get_iap2_vendor_id(device_name)?,
                pid: Self::get_iap2_product_id(device_name)?,
                manufacturer: Self::get_iap2_manufacturer(device_name)?,
                product: Self::get_iap2_product(device_name)?,
                ncm: Some(Self::get_iap2_ifname(device_name)?),
            }),
            _ => GadgetStatus::Error(format!("unexpected g_iphone status: {}", status.trim()).into()),
        })
    }

    pub fn status_path(&self) -> String {
        Self::iphone_param_path(&self.device_name, "status")
    }

    // Accessory child nodes
    pub fn get_iap2_path(device_name: &str) -> GadgetResult<String> {
        let link = Self::iphone_param_path(device_name, "iap2_devnode");
        let target = fs::read_link(link).map_err(|e| GadgetError::FailedGadgetStatusCheck(e.into()))?;
        let name = target
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| name.starts_with("iap2-") && name[5..].bytes().all(|byte| byte.is_ascii_digit()) && name.len() > 5)
            .ok_or_else(|| {
                GadgetError::FailedGadgetStatusCheck(io::Error::new(io::ErrorKind::InvalidData, "invalid iAP2 sysfs link").into())
            })?;
        Ok(format!("/dev/{name}"))
    }

    pub fn get_iap2_ifname(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "iap2_accessory/ifname")
    }

    pub fn get_iap2_vendor_id(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "iap2_accessory/vendor_id")
    }

    pub fn get_iap2_product_id(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "iap2_accessory/product_id")
    }

    pub fn get_iap2_manufacturer(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "iap2_accessory/manufacturer")
    }

    pub fn get_iap2_product(device_name: &str) -> GadgetResult<String> {
        Self::get_param(device_name, "iap2_accessory/product")
    }

    pub fn modprobe(device_name: &str, serial: &str, udc: &str) -> GadgetResult<()> {
        let mut cmd = Command::new("/sbin/modprobe");
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd.arg("g_iphone");

        if !device_name.is_empty() {
            cmd.arg(format!("device_name={:?}", device_name));
        }
        if !udc.is_empty() {
            cmd.arg(format!("udc_name={:?}", udc));
        }
        if !serial.is_empty() {
            cmd.arg(format!("iphone_serial={:?}", serial));
        }

        let out = cmd.spawn()?.wait_with_output()?;
        debug!("modprobe stdout: {}", String::from_utf8_lossy(&out.stdout));
        debug!("modprobe stderr: {}", String::from_utf8_lossy(&out.stderr));

        if !out.status.success() {
            return Err(io::Error::other(format!(
                "unexpected modprobe exit code {}:\n{}",
                out.status.code().unwrap_or_default(),
                String::from_utf8_lossy(&out.stderr)
            ))
            .into());
        }

        Ok(())
    }

    pub fn rmmod() -> GadgetResult<()> {
        let mut cmd = Command::new("/sbin/rmmod");
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd.arg("g_iphone");

        let out = cmd.spawn()?.wait_with_output()?;

        debug!("rmmod stdout: {}", String::from_utf8_lossy(&out.stdout));
        debug!("rmmod stderr: {}", String::from_utf8_lossy(&out.stderr));

        if !out.status.success() {
            return Err(io::Error::other(format!(
                "unexpected rmmod exit code {}:\n{}",
                out.status.code().unwrap_or_default(),
                String::from_utf8_lossy(&out.stderr)
            ))
            .into());
        }

        Ok(())
    }

    pub fn new(device_name: &str, take_over: bool, udc: &str) -> GadgetResult<Self> {
        let mut ownership = true;

        if Self::exists(device_name) && !take_over {
            Self::remove(device_name).map_err(PhoneGadgetDriverError::Remove)?;
        }

        if Self::exists(device_name) {
            debug!("Taking over device that already exists: {device_name}");
            ownership = false;
        } else {
            if !Self::driver_loaded() {
                debug!("Attempting modprobe");
                Self::modprobe("", "", "").map_err(PhoneGadgetDriverError::DriverLoad)?;
            }
            debug!("Attempting to create device: {device_name}");
            Self::create(device_name).map_err(PhoneGadgetDriverError::Create)?;
            Self::set_udc(device_name, udc).map_err(PhoneGadgetDriverError::Create)?;
        }

        let me = Self {
            udc: udc.into(),
            ownership,
            device_name: device_name.into(),
            function_path: Self::udc_function_path(udc),
        };
        Ok(me)
    }

    pub fn status(&self) -> GadgetStatus {
        if let Err(err) = verify_function(&self.udc, &self.function_path) {
            return GadgetStatus::Error(err);
        }

        match Self::get_status(&self.device_name) {
            Err(err) => GadgetStatus::Error(err),
            Ok(v) => v,
        }
    }

    pub fn unbind(&self) -> GadgetResult<()> {
        Ok(Self::set_binding(&self.device_name, false).map_err(PhoneGadgetDriverError::Unbind)?)
    }

    pub fn bind(&self) -> GadgetResult<()> {
        Ok(Self::set_binding(&self.device_name, true).map_err(PhoneGadgetDriverError::Bind)?)
    }

    pub fn unregister(&self) -> GadgetResult<()> {
        debug!("Unregistering the device");
        Ok(Self::remove(&self.device_name).map_err(PhoneGadgetDriverError::Remove)?)
    }

    pub async fn unbind_async(self: &Arc<Self>) -> GadgetResult<()> {
        let me = self.clone();
        spawn_blocking(move || me.unbind()).await.unwrap()
    }

    pub async fn bind_async(self: &Arc<Self>) -> GadgetResult<()> {
        let me = self.clone();
        spawn_blocking(move || me.bind()).await.unwrap()
    }
}

impl Drop for PhoneGadgetDriver {
    fn drop(&mut self) {
        debug!("PhoneGadgetDriver was dropped!");

        if self.ownership {
            let _ = self.unregister();
        }
    }
}
