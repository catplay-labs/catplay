#![cfg(feature = "serde")]

use catplay_plist::{PlistByteArray, from_bytes, from_xml_bytes, plist_struct, to_writer_binary, to_writer_xml};
#[cfg(debug_assertions)]
use insta::assert_debug_snapshot;

plist_struct! {
    struct InfoMessageResponse {
        pub oem_icon: Option<PlistByteArray>,
    }
}

plist_struct! {
    struct InfoMessage {
        pub device_name: String,
        pub supported_modes: Vec<String>,
        pub response: InfoMessageResponse,
    }
}

#[derive(Debug)]
struct Snapshot {
    case: &'static str,
    decoded: String,
    binary: Vec<u8>,
    xml: String,
}

fn round_trip<T>(case: &'static str, value: &T) -> Snapshot
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let mut binary = Vec::new();
    to_writer_binary(&mut binary, value).unwrap();
    let decoded_binary: T = from_bytes(&binary).unwrap();
    assert_eq!(&decoded_binary, value, "binary round trip: {case}");

    let mut xml_bytes = Vec::new();
    to_writer_xml(&mut xml_bytes, value).unwrap();
    let decoded_xml: T = from_xml_bytes(&xml_bytes).unwrap();
    assert_eq!(&decoded_xml, value, "XML round trip: {case}");
    assert_eq!(from_bytes::<T>(&xml_bytes).unwrap(), *value, "XML autodetection: {case}");

    Snapshot {
        case,
        decoded: format!("{decoded_binary:?}"),
        binary,
        xml: String::from_utf8(xml_bytes).unwrap(),
    }
}

#[test]
fn plist_struct_binary_and_xml_round_trips() {
    let with_icon = InfoMessageResponse {
        oem_icon: Some([0, 1, 2, 0xfe, 0xff].into()),
    };
    let without_icon = InfoMessageResponse::default();
    let nested = InfoMessage {
        device_name: "CatPlay & Car".into(),
        supported_modes: vec!["wired".into(), "wireless".into()],
        response: with_icon.clone(),
    };

    let snapshots = vec![
        round_trip("InfoMessageResponse with icon", &with_icon),
        round_trip("InfoMessageResponse without icon", &without_icon),
        round_trip("nested InfoMessage", &nested),
    ];

    assert_eq!(snapshots[0].case, "InfoMessageResponse with icon");
    assert_eq!(snapshots[0].decoded, format!("{with_icon:?}"));
    assert!(
        snapshots
            .iter()
            .all(|snapshot| snapshot.binary.starts_with(b"bplist00"))
    );
    assert!(snapshots[0].xml.contains("<key>oemIcon</key>"));
    assert!(!snapshots[1].xml.contains("oemIcon"));
    assert!(snapshots[2].xml.contains("<key>deviceName</key>"));
    assert!(snapshots[2].xml.contains("CatPlay &amp; Car"));

    #[cfg(debug_assertions)]
    assert_debug_snapshot!(snapshots);
}
