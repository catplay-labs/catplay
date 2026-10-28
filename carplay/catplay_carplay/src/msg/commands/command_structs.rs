use crate::modes::{ChangeModes, ModesChanged};
use catplay_plist::{PlistByteArray, plist_enum_repr, plist_struct};

plist_struct! {
    pub struct CommandDuckAudio {
        /// Number of milliseconds the ramp down should last.
        pub duration_ms: f64,
        /// Recommended final dB attenuation of the audio at the end of the duck (-144 to 0dB).
        pub volume: f64
    }
}

plist_struct! {
    pub struct CommandUnduckAudio {
        /// Number of milliseconds the ramp up should last.
        pub duration_ms: f64,
        pub volume: f64
    }
}

plist_struct! {
    pub struct CommandDisableBluetooth {
        /// MAC address of Bluetooth device to disable connectivity to (that is, the iOS device).
        #[serde(rename = "deviceID")]
        pub device_id: String,
    }
}

plist_struct! {
    pub struct CommandChangeModes(pub ChangeModes);
}

plist_struct! {
    pub struct CommandModesChanged(pub ModesChanged);
}

plist_struct! {
    pub struct CommandForceKeyFrame {
        /// UUID of the display that needs a keyframe. If omitted, the main display is implied.
        pub uuid: Option<String>
    }
}

plist_struct! {
    pub struct CommandHidSendReport {
        /// USB-formatted HID report.
        pub hid_report: PlistByteArray,
        /// NTP timestamp when the event occurred (synchronized to the device's clock).
        pub timestamp: Option<u64>,
        /// UUID to uniquely identify the HID device.
        pub uuid: String
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum HidInputMode {
        #[default]
        Default = 0,
        Character = 1,
        Scrolling = 2,
        ScrollingWithCharacters = 3,
        DialPad = 4
    }
}

plist_struct! {
    pub struct CommandHidSetInputMode {
        hid_input_mode: HidInputMode,
        /// UUID to uniquely identify the HID device.
        uuid: String
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum SiriAction {
        /// Indicate that the device should begin preparing Siri. At this point, no audio or video resources will be taken.
        #[default]
        Prewarm = 1,
        /// Indicate to the device that the Siri button has been depressed.
        ButtonDown = 2,
        /// Indicate to the device that the Siri button has been released.
        ButtonUp = 3
    }
}

plist_struct! {
    pub struct CommandRequestSiri {
        pub siri_action: SiriAction,
    }
}

plist_struct! {
    pub struct CommandRequestUI {
        pub url: Option<String>,
    }
}

plist_struct! {
    pub struct CommandSetNightMode {
        /// True if it is dark outside, false otherwise.
        pub night_mode: bool,
    }
}

plist_struct! {
    pub struct CommandSetLimitedUI {
        /// True if certain UI elements should be limited.
        #[serde(rename = "limitedUI")]
        pub limited_ui: bool,
    }
}

plist_struct! {
    pub struct CommandChangeMapZoomLevel {
        /// Direction to zoom the map in (observed value 1 when zooming via the instrument cluster).
        pub zoom_direction: i64,
        /// UUID of the display showing the map.
        pub uuid: String
    }
}

plist_struct! {
    pub struct CommandUpdateVehicleInformation {
        // TODO
    }
}

plist_struct! {
    pub struct CommandIApSendMessage {
        pub data: PlistByteArray
    }
}

plist_struct! {
    pub struct CommandFlushAudio {}
}

plist_struct! {
    pub struct CommandPerformHapticFeedback {
        pub haptic_feedback_type: u8,
        pub uuid: String
    }
}
