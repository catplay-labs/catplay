use crate::{
    common::{AirPlayFeature, AirPlayStatus},
    modes::ChangeModes,
    msg::{AudioFormat, AudioType, DisplayFeature, ExtendedFeature, LimitedUIElement, PrimaryInputDevice, StreamType},
};
use catplay_plist::{FlexBool, PlistByteArray, plist_struct};

#[cfg(test)]
#[path = "info_tests.rs"]
mod tests;

plist_struct! {
    pub struct InfoMessage {
       pub qualifier: Option<Vec<String>>
    }
}

plist_struct! {
    pub struct InfoMessageTxtAirPlayResponse {
        #[serde(rename = "txtAirPlay")]
        pub txt_airplay: PlistByteArray,
    }
}

plist_struct! {
    pub struct HevcInfo {}
}

plist_struct! {
    pub struct InfoMessageResponse {
        pub audio_formats: Vec<AudioFormatStruct>,
        pub audio_latencies: Vec<AudioLatency>,
        #[serde(rename = "bluetoothIDs", default)]
        pub bluetooth_ids: Vec<String>,
        #[serde(rename = "deviceID")]
        pub device_id: String,
        pub displays: Vec<Display>,
        #[serde(default)]
        pub extended_features: Vec<ExtendedFeature>,
        pub features: AirPlayFeature,
        #[serde(default)]
        pub firmware_revision: String,
        #[serde(default)]
        pub hardware_revision: String,
        #[serde(default)]
        pub hid_devices: Vec<HidDevice>,
        #[serde(default)]
        pub hid_languages: Vec<String>,
        #[serde(default)]
        pub keep_alive_low_power: bool,
        #[serde(default)]
        pub keep_alive_send_stats_as_body: bool,
        #[serde(rename = "limitedUIElements", default)]
        pub limited_ui_elements: Vec<LimitedUIElement>,
        #[serde(rename = "limitedUI", default)]
        pub limited_ui: FlexBool,
        pub manufacturer: String,
        pub model: String,
        pub modes: ChangeModes,
        /// This must be set to "CarPlay"
        pub name: String,
        pub night_mode: Option<FlexBool>,
        pub oem_icon: Option<PlistByteArray>,
        #[serde(default)]
        pub oem_icons: Vec<OemIcon>,
        pub oem_icon_label: Option<String>,
        #[serde(default)]
        pub oem_icon_visible: FlexBool,
        #[serde(rename = "OSInfo")]
        pub os_info: Option<String>,
        /// 1.0
        pub protocol_version: Option<String>,
        #[serde(default)]
        pub right_hand_drive: FlexBool,
        /// SDK version
        pub source_version: String,

        pub status_flags: AirPlayStatus,

        // Modern CarPlay
        pub hevc_info: Option<HevcInfo>
        // pub vehicle_information: VehicleInformation,
    }
}

plist_struct! {
    pub struct OemIcon {
        pub image_data: PlistByteArray,
        pub height_pixels: u32,
        pub width_pixels: u32,
        pub prerendered: FlexBool
    }
}

plist_struct! {
    pub struct AudioFormatStruct {
        pub audio_input_formats: Option<AudioFormat>,
        pub audio_output_formats: Option<AudioFormat>,
        #[serde(rename = "type")]
        pub stream_type: StreamType,
        /// Absent in v210.81
        pub audio_type: Option<AudioType>
    }
}

plist_struct! {
    pub struct AudioLatency {
        #[serde(rename = "type")]
        /// Absent in v210.81
        pub stream_type: Option<StreamType>,
        pub audio_type: Option<AudioType>,
        pub sr: Option<u64>,
        pub ss: Option<u64>,
        pub ch: Option<u64>,
        pub input_latency_micros: Option<u64>,
        pub output_latency_micros: Option<u64>
    }
}

plist_struct! {
    pub struct Display {
        pub name: Option<String>,
        #[serde(rename = "displayID")]
        pub display_id: Option<DisplayId>,
        pub edid: Option<PlistByteArray>,
        pub features: DisplayFeature,
        #[serde(rename = "maxFPS")]
        pub max_fps: Option<u32>,
        pub height_pixels: u32,
        pub width_pixels: u32,
        pub height_physical: u32,
        pub width_physical: u32,
        pub uuid: String,
        /// Absent in v210.81
        pub primary_input_device: Option<PrimaryInputDevice>,
    }
}

impl Display {
    // Legacy MHI2Q uses the UUID itself as the alternate display's name.
    // Do not infer roles from dimensions, array order, or numeric identifiers.
    fn is_alternate(&self) -> bool {
        fn alternate(name: &str) -> bool {
            name.eq_ignore_ascii_case("alt")
                || name.eq_ignore_ascii_case("alternate")
                || name.eq_ignore_ascii_case("cluster")
                || name.as_bytes().windows(9).any(|s| s.eq_ignore_ascii_case(b"altscreen"))
        }
        alternate(&self.uuid) || self.name.as_deref().is_some_and(alternate)
    }

    fn is_named_main(&self) -> bool {
        fn main(name: &str) -> bool {
            name.eq_ignore_ascii_case("main") || name.eq_ignore_ascii_case("mainScreen") || name.eq_ignore_ascii_case("primary")
        }
        !self.is_alternate() && (main(&self.uuid) || self.name.as_deref().is_some_and(main))
    }

    pub fn dpi(&self) -> f32 {
        const FALLBACK_DPI: f32 = 160.0;
        const MIN_DPI: f32 = 60.0;
        const MAX_DPI: f32 = 300.0;

        let dpi = if self.width_physical > 0 && self.width_pixels > 0 {
            self.width_pixels as f32 / (self.width_physical as f32 / 25.4)
        } else {
            FALLBACK_DPI
        };

        dpi.clamp(MIN_DPI, MAX_DPI)
    }
}

plist_struct! {
    pub struct HidDevice {
        #[serde(rename = "displayUUID")]
        pub display_uuid: Option<String>,
        #[serde(rename = "displayID")]
        pub display_id: Option<DisplayId>,
        pub hid_country_code: u16,
        pub hid_descriptor: PlistByteArray,
        #[serde(rename = "hidProductID")]
        pub hid_product_id: u16,
        #[serde(rename = "hidVendorID")]
        pub hid_vendor_id: u16,
        pub name: String,
        pub uuid: String,
    }
}

/// Keep identifier types and namespaces intact: displayID 7, displayID "7",
/// displayUUID "7", and a HID's own uuid are not interchangeable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DisplayId {
    String(String),
    Number(u64),
}

impl InfoMessageResponse {
    /// Restrict this advertisement to the receiver's single main-screen transport.
    /// Call after all sink/application patches, and only for the CarPlay profile.
    /// This does not negotiate or implement altScreen.
    pub fn retain_main_screen_only(&mut self) -> Result<(), &'static str> {
        let mut named = self.displays.iter().enumerate().filter(|(_, d)| d.is_named_main());
        let main = named.next().map(|(i, _)| i);
        if named.next().is_some() {
            return Err("ambiguous main displays");
        }
        // Legacy /info has no role metadata: retain the first non-alternate entry.
        // In particular, an alternate-first list must not select the cluster.
        let index = main
            .or_else(|| self.displays.iter().position(|d| !d.is_alternate()))
            .ok_or("missing main display")?;
        let selected = &self.displays[index];
        if selected.uuid.is_empty() || selected.width_pixels == 0 || selected.height_pixels == 0 {
            return Err("invalid main display identity or dimensions");
        }

        self.hid_devices.retain(|hid| {
            // Keep unassociated and unknown associations. Remove only a known
            // removed-display association, never the HID's own device identity.
            // A duplicated display identifier is ambiguous: favor preserving input.
            let removed_uuid = hid
                .display_uuid
                .as_deref()
                .filter(|id| !id.is_empty())
                .is_some_and(|id| id != selected.uuid && self.displays.iter().any(|d| d.uuid == id));
            let removed_id = hid.display_id.as_ref().is_some_and(|id| {
                selected.display_id.as_ref() != Some(id) && self.displays.iter().any(|d| d.display_id.as_ref() == Some(id))
            });
            !removed_uuid && !removed_id
        });
        self.displays.swap(0, index);
        self.displays.truncate(1);
        Ok(())
    }
}
