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

        // Modern CarPlay (secondary displays such as an instrument cluster)
        pub uid: Option<String>,
        /// For example "Instrument Cluster".
        pub properties: Option<Vec<String>>,
        /// Stream type used to drive this display (`AltScreen` for secondary displays).
        #[serde(rename = "type")]
        pub stream_type: Option<StreamType>,
        #[serde(rename = "initialURL")]
        pub initial_url: Option<String>,
        pub zoom_factor: Option<f64>,
        /// Index into `view_areas` of the view area that is active when the session starts.
        pub initial_view_area: Option<u64>,
        pub view_areas: Option<Vec<ViewArea>>,
        /// Set when the display has more than one view area to switch between.
        pub view_area_transition_control: Option<FlexBool>,
        pub corner_masks: Option<FlexBool>,
    }
}

// Rectangle within a view area in which CarPlay places important UI elements.
plist_struct! {
    pub struct SafeArea {
        pub origin_x_pixels: i32,
        pub origin_y_pixels: i32,
        pub width_pixels: i32,
        pub height_pixels: i32,
    }
}

// Rectangle of the video stream in which CarPlay draws its UI. The pixel values are signed, because negative values
// have been observed in the safe area of some setups.
plist_struct! {
    pub struct ViewArea {
        pub origin_x_pixels: i32,
        pub origin_y_pixels: i32,
        pub width_pixels: i32,
        pub height_pixels: i32,
        pub safe_area: Option<SafeArea>,
        /// Edge on which CarPlay places the dock (for example, 1 is bottom and 2 is driver side).
        pub view_area_status_bar_edge: Option<u64>,
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

#[cfg(test)]
mod display_tests {
    use super::*;
    use catplay_plist::{Dictionary, Value, from_bytes, to_writer_binary};

    #[test]
    fn alt_display_round_trips_without_losing_keys() {
        // Values observed for the instrument cluster in the /info response of a VW MMX head unit.
        let safe_area = Value::Dictionary(Dictionary::from_iter([
            ("widthPixels", Value::from(320u64)),
            ("originXPixels", Value::from(240u64)),
            ("originYPixels", Value::from(60u64)),
            ("heightPixels", Value::from(340u64)),
        ]));
        let view_areas = Value::Array(vec![Value::Dictionary(Dictionary::from_iter([
            ("widthPixels", Value::from(800u64)),
            ("originXPixels", Value::from(0u64)),
            ("originYPixels", Value::from(0u64)),
            ("heightPixels", Value::from(480u64)),
            ("safeArea", safe_area),
        ]))]);
        let display_entry = Value::Dictionary(Dictionary::from_iter([
            ("widthPixels", Value::from(800u64)),
            ("heightPixels", Value::from(480u64)),
            ("widthPhysical", Value::from(240u64)),
            ("heightPhysical", Value::from(144u64)),
            ("features", Value::from(10u64)),
            ("maxFPS", Value::from(40u64)),
            ("primaryInputDevice", Value::from(3u64)),
            ("properties", Value::Array(vec!["Instrument Cluster".into()])),
            ("zoomFactor", Value::from(1.5)),
            ("initialViewArea", Value::from(0u64)),
            ("uid", "b7e6c5a0-2222-4000-8000-000000000002".into()),
            ("uuid", "b7e6c5a0-2222-4000-8000-000000000002".into()),
            ("initialURL", "maps:/car/instrumentcluster".into()),
            ("type", Value::from(111u64)),
            ("viewAreas", view_areas),
        ]));

        let mut binary = Vec::new();
        to_writer_binary(&mut binary, &display_entry).unwrap();
        let display: Display = from_bytes(&binary).unwrap();
        assert_eq!(display.stream_type, Some(StreamType::AltScreen));
        assert_eq!(display.primary_input_device, Some(PrimaryInputDevice::Knob));
        assert_eq!(display.properties, Some(vec!["Instrument Cluster".to_owned()]));

        // The display is proxied to the iPhone, so it must serialize to exactly what the car has sent.
        binary.clear();
        to_writer_binary(&mut binary, &display).unwrap();
        assert_eq!(from_bytes::<Value>(&binary).unwrap(), display_entry);
    }

    #[test]
    fn main_display_with_view_areas_round_trips_without_losing_keys() {
        // Keys that are reported for head units with a split screen, including a negative safe area width.
        let safe_area = Value::Dictionary(Dictionary::from_iter([
            ("widthPixels", Value::from(-10i64)),
            ("originXPixels", Value::from(0u64)),
            ("originYPixels", Value::from(0u64)),
            ("heightPixels", Value::from(720u64)),
        ]));
        let view_areas = Value::Array(vec![
            Value::Dictionary(Dictionary::from_iter([
                ("widthPixels", Value::from(1920u64)),
                ("originXPixels", Value::from(0u64)),
                ("originYPixels", Value::from(0u64)),
                ("heightPixels", Value::from(720u64)),
                ("viewAreaStatusBarEdge", Value::from(1u64)),
            ])),
            Value::Dictionary(Dictionary::from_iter([
                ("widthPixels", Value::from(1280u64)),
                ("originXPixels", Value::from(0u64)),
                ("originYPixels", Value::from(0u64)),
                ("heightPixels", Value::from(720u64)),
                ("safeArea", safe_area),
                ("viewAreaStatusBarEdge", Value::from(2u64)),
            ])),
        ]);
        let display_entry = Value::Dictionary(Dictionary::from_iter([
            ("widthPixels", Value::from(1920u64)),
            ("heightPixels", Value::from(720u64)),
            ("widthPhysical", Value::from(292u64)),
            ("heightPhysical", Value::from(110u64)),
            ("features", Value::from(2u64)),
            ("uuid", "e5f7a68d-7b0f-4305-984b-974f677a150b".into()),
            ("initialViewArea", Value::from(1u64)),
            ("viewAreas", view_areas),
            ("viewAreaTransitionControl", Value::Boolean(true)),
            ("cornerMasks", Value::Boolean(true)),
        ]));

        let mut binary = Vec::new();
        to_writer_binary(&mut binary, &display_entry).unwrap();
        let display: Display = from_bytes(&binary).unwrap();
        let view_areas = display.view_areas.as_ref().unwrap();
        assert_eq!(view_areas.len(), 2);
        assert_eq!(view_areas[0].safe_area, None);
        assert_eq!(view_areas[1].safe_area.as_ref().unwrap().width_pixels, -10);
        assert_eq!(view_areas[1].view_area_status_bar_edge, Some(2));

        binary.clear();
        to_writer_binary(&mut binary, &display).unwrap();
        assert_eq!(from_bytes::<Value>(&binary).unwrap(), display_entry);
    }

    #[test]
    fn main_display_does_not_gain_alt_display_keys() {
        let mut binary = Vec::new();
        to_writer_binary(&mut binary, &Display::default()).unwrap();
        let Value::Dictionary(display_entry) = from_bytes::<Value>(&binary).unwrap() else {
            panic!("expected a dictionary");
        };

        for key in [
            "uid",
            "properties",
            "type",
            "initialURL",
            "zoomFactor",
            "initialViewArea",
            "viewAreas",
            "viewAreaTransitionControl",
            "cornerMasks",
        ] {
            assert!(display_entry.get(key).is_none(), "unexpected key {key}");
        }
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
