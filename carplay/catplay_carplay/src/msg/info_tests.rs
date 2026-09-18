use super::*;
use crate::carplay_rx::AirPlayReceiverProfile;
use catplay_plist::PlistSerializable;

fn main_display() -> Display {
    Display {
        uuid: "synthetic-main".into(),
        width_pixels: 1024,
        height_pixels: 480,
        width_physical: 188,
        height_physical: 88,
        max_fps: Some(30),
        ..Default::default()
    }
}

fn alternate() -> Display {
    Display {
        uuid: "AudiAltScreenCluster".into(),
        width_pixels: 1440,
        height_pixels: 450,
        ..main_display()
    }
}

fn info(displays: Vec<Display>) -> InfoMessageResponse {
    InfoMessageResponse {
        displays,
        ..Default::default()
    }
}

fn hid(association: Option<&str>) -> HidDevice {
    HidDevice {
        display_uuid: association.map(Into::into),
        uuid: "synthetic-device".into(),
        hid_descriptor: vec![5, 1, 9, 6].into(),
        ..Default::default()
    }
}

#[test]
fn captured_shape_and_alternate_first_keep_main_and_all_four_main_hids() {
    for displays in [vec![main_display(), alternate()], vec![alternate(), main_display()]] {
        let mut response = info(displays);
        response.hid_devices = vec![hid(Some("synthetic-main")); 4];
        let hids = response.hid_devices.clone();
        response.retain_main_screen_only().unwrap();
        assert_eq!(response.displays, vec![main_display()]);
        assert_eq!(response.hid_devices, hids);
        let once = response.clone();
        response.retain_main_screen_only().unwrap();
        assert_eq!(response, once);
    }
}

#[test]
fn explicit_main_name_beats_legacy_order_and_alternate_name_is_excluded() {
    let main = Display {
        name: Some("mainScreen".into()),
        ..main_display()
    };
    let other = Display {
        uuid: "other".into(),
        ..main_display()
    };
    let mut response = info(vec![other, main.clone()]);
    response.retain_main_screen_only().unwrap();
    assert_eq!(response.displays, vec![main]);
    let alt = Display {
        name: Some("altScreen".into()),
        uuid: "other".into(),
        ..main_display()
    };
    let mut response = info(vec![alt, main_display()]);
    response.retain_main_screen_only().unwrap();
    assert_eq!(response.displays, vec![main_display()]);
}

#[test]
fn legacy_single_and_unlabelled_first_are_preserved_without_geometry_heuristics() {
    for displays in [
        vec![main_display()],
        vec![
            main_display(),
            Display {
                uuid: "second".into(),
                ..alternate()
            },
        ],
    ] {
        let mut response = info(displays);
        response.retain_main_screen_only().unwrap();
        assert_eq!(response.displays, vec![main_display()]);
    }
}

#[test]
fn malformed_profiles_fail_without_mutation_or_promoting_alternate() {
    let named = Display {
        name: Some("main".into()),
        ..main_display()
    };
    for displays in [
        vec![],
        vec![alternate()],
        vec![
            Display {
                width_pixels: 0,
                ..main_display()
            },
            alternate(),
        ],
        vec![Display {
            height_pixels: 0,
            ..main_display()
        }],
        vec![Display {
            uuid: String::new(),
            ..main_display()
        }],
        vec![named.clone(), named],
    ] {
        let mut response = info(displays);
        response.hid_devices = vec![hid(None)];
        let before = response.clone();
        assert!(response.retain_main_screen_only().is_err());
        assert_eq!(response, before);
    }
}

#[test]
fn removes_only_explicit_removed_display_associations() {
    let mut response = info(vec![alternate(), main_display()]);
    let kept = vec![
        hid(None),
        hid(Some("")),
        hid(Some("synthetic-main")),
        hid(Some("unknown")),
        HidDevice {
            // A device's own UUID must never be mistaken for its display association.
            uuid: alternate().uuid,
            ..hid(None)
        },
    ];
    response.hid_devices = kept.clone();
    response.hid_devices.push(hid(Some(&alternate().uuid)));
    response.retain_main_screen_only().unwrap();
    assert_eq!(response.hid_devices, kept);
}

#[test]
fn numeric_string_ids_and_uuid_namespaces_are_distinct() {
    let mut main = main_display();
    main.display_id = Some(DisplayId::String("7".into()));
    let mut alt = alternate();
    alt.display_id = Some(DisplayId::Number(7));
    let kept = vec![
        hid(Some("7")),
        HidDevice {
            display_id: main.display_id.clone(),
            ..hid(None)
        },
        HidDevice {
            display_id: Some(DisplayId::Number(8)),
            ..hid(None)
        },
    ];
    let mut response = info(vec![alt.clone(), main]);
    response.hid_devices = kept.clone();
    response.hid_devices.push(HidDevice {
        display_id: alt.display_id,
        ..hid(None)
    });
    response.retain_main_screen_only().unwrap();
    assert_eq!(response.hid_devices, kept);
}

#[test]
fn duplicate_display_identifiers_do_not_drop_main_input() {
    let main = main_display();
    let mut alt = alternate();
    alt.name = Some("altScreen".into());
    alt.uuid = main.uuid.clone();
    let mut response = info(vec![alt, main]);
    response.hid_devices.push(hid(Some("synthetic-main")));
    response.retain_main_screen_only().unwrap();
    assert_eq!(response.hid_devices.len(), 1);
}

#[test]
fn binary_plist_retains_optional_associations_and_identifier_types() {
    for id in [None, Some(DisplayId::Number(7)), Some(DisplayId::String("7".into()))] {
        let mut response = info(vec![alternate(), main_display()]);
        response.displays[1].display_id = id.clone();
        response.hid_devices = vec![
            hid(None),
            HidDevice {
                display_id: id,
                ..hid(Some("synthetic-main"))
            },
        ];
        let bytes = response.pencode().unwrap();
        assert!(bytes.starts_with(b"bplist00"));
        let mut decoded = InfoMessageResponse::pdecode(&bytes).unwrap();
        assert_eq!(decoded, response);
        decoded.retain_main_screen_only().unwrap();
        assert_eq!(InfoMessageResponse::pdecode(&decoded.pencode().unwrap()).unwrap(), decoded);
        let value: catplay_plist::Value = catplay_plist::from_bytes(&decoded.pencode().unwrap()).unwrap();
        let device = &value.as_dictionary().unwrap()["hidDevices"].as_array().unwrap()[0];
        assert!(!device.as_dictionary().unwrap().contains_key("displayUUID"));
    }
}

#[test]
fn malformed_wire_associations_are_rejected_not_coerced() {
    for (key, value) in [
        ("displayUUID", serde_json::json!(7)),
        ("displayID", serde_json::json!([])),
        ("displayID", serde_json::json!(-1)),
        ("displayID", serde_json::json!(true)),
    ] {
        let mut wire = serde_json::to_value(hid(None)).unwrap();
        wire[key] = value;
        assert!(serde_json::from_value::<HidDevice>(wire).is_err());
    }
}

#[test]
fn profile_constraint_after_patch_leaves_non_carplay_unchanged() {
    let mut response = info(vec![main_display()]);
    // Model the application/sink patch reintroducing the car's complete display list.
    response.displays = vec![alternate(), main_display()];
    let before = response.clone();
    AirPlayReceiverProfile::AppleTV.constrain_info(&mut response).unwrap();
    assert_eq!(response, before);
    AirPlayReceiverProfile::CarPlay.constrain_info(&mut response).unwrap();
    assert_eq!(response.displays, vec![main_display()]);
}
