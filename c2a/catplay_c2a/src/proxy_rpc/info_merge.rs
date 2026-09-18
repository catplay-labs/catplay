use catplay_carplay::msg::InfoMessageResponse;

/// Copy car-owned presentation/input metadata into the phone-facing receiver's /info.
pub(super) fn merge_car_info(local: &mut InfoMessageResponse, car: InfoMessageResponse) {
    // Features describe this receiver, not the separate transmitter-to-car connection.
    // In particular, RX AAC is decoded to PCM before TX, so a PCM-only car does not
    // remove local AAC support. Preserve the incoming mask, including unknown bits.
    local.hid_devices = car.hid_devices;
    local.hid_languages = car.hid_languages;
    local.right_hand_drive = car.right_hand_drive;
    local.oem_icon = car.oem_icon;
    local.oem_icon_label = Some("CatPlay".into());
    local.oem_icon_visible = car.oem_icon_visible;
    local.oem_icons = car.oem_icons;
    local.manufacturer = car.manufacturer;
    local.model = car.model;
    local.displays = car.displays;
    local.audio_latencies = car.audio_latencies;
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_carplay::{
        common::AirPlayFeature,
        modes::ChangeModes,
        msg::{AudioFormatStruct, AudioLatency, Display, HevcInfo, HidDevice, OemIcon},
        rtsp_frame::{HttpStatus, RtspResponse},
    };

    const LOCAL: u64 = 0x6144540380;
    const AUDI: u64 = 0x144440b80;

    fn info(features: u64) -> InfoMessageResponse {
        InfoMessageResponse {
            features: AirPlayFeature::from_bits_retain(features),
            ..Default::default()
        }
    }

    #[test]
    fn audi_mask_does_not_replace_local_receiver_features() {
        let mut local = info(LOCAL);
        merge_car_info(&mut local, info(AUDI));
        assert_eq!(local.features.bits(), LOCAL);
        assert!(
            local
                .features
                .contains(AirPlayFeature::HK_PAIRING_AND_ENCRYPT | AirPlayFeature::CARPLAY_CONTROL | AirPlayFeature::AUDIO_AAC_LC)
        );
        assert!(!local.features.contains(AirPlayFeature::REDUNDANT_AUDIO));
    }

    #[test]
    fn car_only_unknown_bits_and_hevc_are_not_imported() {
        let mut local = info(LOCAL);
        let mut car = info(AUDI | (1 << 63) | AirPlayFeature::SUPPORTS_SCREEN_MULTI_CODEC.bits());
        car.hevc_info = Some(HevcInfo::default());
        merge_car_info(&mut local, car);
        assert_eq!(local.features.bits(), LOCAL);
        assert_eq!(local.hevc_info, None);
    }

    #[test]
    fn reduced_local_mask_is_not_widened() {
        for mask in [
            0,
            AirPlayFeature::SCREEN.bits(),
            LOCAL & !AirPlayFeature::AUDIO_AAC_LC.bits(),
            LOCAL | (1 << 63),
        ] {
            let mut local = info(mask);
            merge_car_info(&mut local, info(u64::MAX));
            assert_eq!(local.features.bits(), mask);
        }
    }

    #[test]
    fn car_metadata_is_copied_while_receiver_and_runtime_fields_stay_local() {
        let mut local = InfoMessageResponse {
            protocol_version: Some("1.0".into()),
            source_version: "local-sdk".into(),
            device_id: "local-device".into(),
            audio_formats: vec![AudioFormatStruct::default()],
            keep_alive_low_power: true,
            keep_alive_send_stats_as_body: true,
            night_mode: Some(false.into()),
            modes: ChangeModes::initial(),
            ..info(LOCAL)
        };
        let car = InfoMessageResponse {
            manufacturer: "Audi".into(),
            model: "MHI2Q".into(),
            protocol_version: Some("car-protocol".into()),
            source_version: "car-sdk".into(),
            hid_devices: vec![HidDevice {
                name: "car-input".into(),
                ..Default::default()
            }],
            hid_languages: vec!["en".into()],
            right_hand_drive: true.into(),
            oem_icon: Some(vec![1, 2, 3].into()),
            oem_icon_label: Some("car-label".into()),
            oem_icon_visible: true.into(),
            oem_icons: vec![OemIcon {
                width_pixels: 24,
                ..Default::default()
            }],
            displays: vec![Display {
                width_pixels: 1440,
                height_pixels: 450,
                ..Default::default()
            }],
            audio_latencies: vec![AudioLatency {
                output_latency_micros: Some(12345),
                ..Default::default()
            }],
            night_mode: Some(true.into()),
            ..info(AUDI)
        };
        let before = local.clone();
        merge_car_info(&mut local, car.clone());
        assert_eq!(local.hid_devices, car.hid_devices);
        assert_eq!(local.hid_languages, car.hid_languages);
        assert_eq!(local.right_hand_drive, car.right_hand_drive);
        assert_eq!(local.oem_icon, car.oem_icon);
        assert_eq!(local.oem_icon_label.as_deref(), Some("CatPlay"));
        assert_eq!(local.oem_icon_visible, car.oem_icon_visible);
        assert_eq!(local.oem_icons, car.oem_icons);
        assert_eq!(local.manufacturer, car.manufacturer);
        assert_eq!(local.model, car.model);
        assert_eq!(local.displays, car.displays);
        assert_eq!(local.audio_latencies, car.audio_latencies);
        assert_eq!(local.features, before.features);
        assert_eq!(local.protocol_version, before.protocol_version);
        assert_eq!(local.source_version, before.source_version);
        assert_eq!(local.device_id, before.device_id);
        assert_eq!(local.audio_formats, before.audio_formats);
        assert_eq!(local.keep_alive_low_power, before.keep_alive_low_power);
        assert_eq!(local.keep_alive_send_stats_as_body, before.keep_alive_send_stats_as_body);
        assert_eq!(local.modes, before.modes);
        assert_eq!(local.night_mode, before.night_mode);
    }

    #[test]
    fn merged_features_survive_binary_plist_u64_roundtrip() {
        for mask in [LOCAL, LOCAL | AirPlayFeature::SUPPORTS_SCREEN_MULTI_CODEC.bits()] {
            let mut local = info(mask);
            merge_car_info(&mut local, info(AUDI));
            let mut response = RtspResponse::new(None, HttpStatus::Ok);
            response.set_plist(local.clone()).unwrap();
            assert!(response.payload.starts_with(b"bplist00"));
            let decoded: InfoMessageResponse = response.get_plist().unwrap();
            assert_eq!(decoded.features.bits(), mask);
            assert_eq!(decoded, local);
        }
    }
}
