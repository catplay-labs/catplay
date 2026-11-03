use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum CommandDirection {
    FromAccessory,
    FromDevice,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    DuckAudio,
    UnduckAudio,
    DisableBluetooth,
    ChangeModes,
    ModesChanged,
    ForceKeyFrame,
    HidSendReport,
    HidSetInputMode,
    RequestSiri,
    RequestUI,
    SetNightMode,
    SetLimitedUI,
    IApSendMessage,
    ChangeMapZoomLevel,
    // updateVehicleInformation
    // flushAudio
    // performHapticFeedback
    // hidSetReport
    // updateDisplayPanels
    // updateVocoderInfo
    // updateViewArea
    // changeUIContext
    // uiAppearanceUpdate
    // mapAppearanceUpdate
}

impl std::fmt::Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl CommandType {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret: CommandType = match s {
            "duckAudio" => CommandType::DuckAudio,
            "unduckAudio" => CommandType::UnduckAudio,
            "disableBluetooth" => CommandType::DisableBluetooth,
            "changeModes" => CommandType::ChangeModes,
            "modesChanged" => CommandType::ModesChanged,
            "forceKeyFrame" => CommandType::ForceKeyFrame,
            "hidSendReport" => CommandType::HidSendReport,
            "hidSetInputMode" => CommandType::HidSetInputMode,
            "requestSiri" => CommandType::RequestSiri,
            "requestUI" => CommandType::RequestUI,
            "setNightMode" => CommandType::SetNightMode,
            "setLimitedUI" => CommandType::SetLimitedUI,
            "iAPSendMessage" => CommandType::IApSendMessage,
            "changeMapZoomLevel" => CommandType::ChangeMapZoomLevel,

            _ => return Err(()),
        };
        Ok(ret)
    }
}

impl AsRef<str> for CommandType {
    fn as_ref(&self) -> &'static str {
        match self {
            CommandType::DuckAudio => "duckAudio",
            CommandType::UnduckAudio => "unduckAudio",
            CommandType::DisableBluetooth => "disableBluetooth",
            CommandType::ChangeModes => "changeModes",
            CommandType::ModesChanged => "modesChanged",
            CommandType::ForceKeyFrame => "forceKeyFrame",
            CommandType::HidSendReport => "hidSendReport",
            CommandType::HidSetInputMode => "hidSetInputMode",
            CommandType::RequestSiri => "requestSiri",
            CommandType::RequestUI => "requestUI",
            CommandType::SetNightMode => "setNightMode",
            CommandType::SetLimitedUI => "setLimitedUI",
            CommandType::IApSendMessage => "iAPSendMessage",
            CommandType::ChangeMapZoomLevel => "changeMapZoomLevel",
        }
    }
}

impl CommandType {
    pub fn direction(&self) -> CommandDirection {
        match self {
            CommandType::DuckAudio
            | CommandType::UnduckAudio
            | CommandType::DisableBluetooth
            | CommandType::ModesChanged
            | CommandType::HidSetInputMode => CommandDirection::FromDevice,

            CommandType::ChangeModes
            | CommandType::ForceKeyFrame
            | CommandType::HidSendReport
            | CommandType::RequestSiri
            | CommandType::RequestUI
            | CommandType::SetNightMode
            | CommandType::SetLimitedUI
            | CommandType::ChangeMapZoomLevel => CommandDirection::FromAccessory,

            CommandType::IApSendMessage => CommandDirection::Bidirectional,
        }
    }
}
