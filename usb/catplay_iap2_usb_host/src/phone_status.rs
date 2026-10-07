use std::fmt;

use catplay_iap2_usb::GadgetError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessoryData {
    pub vid: String,
    pub pid: String,
    pub manufacturer: String,
    pub product: String,
    pub ncm: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GadgetStatus {
    Initial,
    Bind,
    Enabled,
    Disabled,
    Suspended,

    Accessory(AccessoryData),
    // Final states
    RoleSwitch { is_carplay: bool },
    RoleSwitchFailed,
    Unbind,
    Shutdown,
    Error(GadgetError),
}

impl Default for GadgetStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl GadgetStatus {
    pub fn new() -> Self {
        GadgetStatus::Initial
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            GadgetStatus::Unbind | GadgetStatus::Shutdown | GadgetStatus::Error(_) | GadgetStatus::RoleSwitch { .. }
        )
    }

    pub fn is_role_switch(&self) -> bool {
        matches!(self, GadgetStatus::RoleSwitch { .. })
    }

    pub fn has_accessory(&self) -> bool {
        matches!(self, GadgetStatus::Accessory { .. })
    }

    pub fn as_accessory(&self) -> Option<AccessoryData> {
        if let GadgetStatus::Accessory(acc) = self {
            return Some(acc.clone());
        };

        None
    }

    pub fn is_role_switch_carplay(&self) -> bool {
        match self {
            GadgetStatus::RoleSwitch { is_carplay } => *is_carplay,
            _ => false,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{self:?}")
    }
}

impl fmt::Display for GadgetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
