use crate::{
    common::{AirPlayFeature, AirPlayStatus},
    modes::ChangeModes,
    msg::{AudioFormat, AudioType, DisplayFeature, ExtendedFeature, LimitedUIElement, PrimaryInputDevice, StreamType},
};
use catplay_plist::{FlexBool, PlistByteArray, downcast_enum_to_legacy_u64, plist_struct};

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
        #[serde(default, with = "downcast_enum_to_legacy_u64::option")]
        pub audio_input_formats: Option<AudioFormat>,
        #[serde(default, with = "downcast_enum_to_legacy_u64::option")]
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

#[cfg(test)]
mod audio_format_tests {
    use super::*;
    use catplay_plist::{Dictionary, Value, from_bytes, to_writer_binary};

    #[test]
    fn info_audio_formats_keep_only_low_64_bits_on_wire() {
        let formats = AudioFormatStruct {
            audio_input_formats: Some(AudioFormat::PCM_48000_STEREO | AudioFormat::APAC_48000_STEREO),
            audio_output_formats: Some(AudioFormat::APAC_48000_STEREO),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Default),
        };
        let mut binary = Vec::new();
        to_writer_binary(&mut binary, &formats).unwrap();
        let decoded: AudioFormatStruct = from_bytes(&binary).unwrap();
        assert_eq!(decoded.audio_input_formats, Some(AudioFormat::PCM_48000_STEREO));
        assert_eq!(decoded.audio_output_formats, Some(AudioFormat::empty()));

        // Values observed in the accessory's /info response.
        let response_entry = Value::Dictionary(Dictionary::from_iter([
            ("audioInputFormats", Value::from(84u64)),
            ("audioOutputFormats", Value::from(34_900u64)),
            ("type", Value::from(100u64)),
        ]));
        binary.clear();
        to_writer_binary(&mut binary, &response_entry).unwrap();
        let decoded: AudioFormatStruct = from_bytes(&binary).unwrap();
        assert_eq!(decoded.audio_input_formats.unwrap().bits(), 84);
        assert_eq!(decoded.audio_output_formats.unwrap().bits(), 34_900);
        assert_eq!(decoded.stream_type, StreamType::MainAudio);
    }
}

plist_struct! {
    pub struct HidDevice {
        #[serde(rename = "displayUUID")]
        pub display_uuid: String,
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
