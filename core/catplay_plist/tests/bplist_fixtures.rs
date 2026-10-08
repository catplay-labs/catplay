#![cfg(feature = "serde")]

use catplay_plist::{Value, bplist::wire::decode::Document, from_bytes};

const FOUNDATION_MIXED: &[u8] = include_bytes!("fixtures/bplist/foundation_mixed.plist");
const COREFOUNDATION_KEYED_ARCHIVE: &[u8] = include_bytes!("fixtures/bplist/corefoundation_keyed_archive.plist");
const UPSTREAM_MIXED: &[u8] = include_bytes!("fixtures/bplist/upstream_mixed.plist");
const UPSTREAM_UTF16: &[u8] = include_bytes!("fixtures/bplist/upstream_utf16.plist");
const UPSTREAM_THREE_BYTE_OFFSETS: &[u8] = include_bytes!("fixtures/bplist/upstream_three_byte_offsets.plist");
const UPSTREAM_CIRCULAR_ARRAY: &[u8] = include_bytes!("fixtures/bplist/upstream_circular_array.plist");

#[test]
fn foundation_mixed_types() {
    let value: Value = from_bytes(FOUNDATION_MIXED).unwrap();
    let root = value.as_dictionary().unwrap();
    assert_eq!(root["item1"].as_string(), Some("value1"));
    let array = root["array1"].as_array().unwrap();
    assert_eq!(array.len(), 5);
    assert_eq!(array[0].as_string(), Some("arr0"));
    assert_eq!(array[1].as_signed_integer(), Some(42));
    assert_eq!(array[2].as_boolean(), Some(false));
    assert_eq!(array[3].as_date().unwrap().to_xml_format(), "1976-04-01T12:00:00Z");
    assert_eq!(array[4].as_data(), Some(&[0xaa, 0xbb, 0xcc, 0xdd, 0, 0x11, 0x22, 0x33][..]));
}

#[test]
fn corefoundation_keyed_archive_preserves_uids() {
    let value: Value = from_bytes(COREFOUNDATION_KEYED_ARCHIVE).unwrap();
    let root = value.as_dictionary().unwrap();
    assert_eq!(root["$archiver"].as_string(), Some("NSKeyedArchiver"));
    assert_eq!(root["$objects"].as_array().unwrap().len(), 23);
    assert_eq!(
        root["$top"].as_dictionary().unwrap()["root"]
            .as_uid()
            .unwrap()
            .get(),
        1
    );
    let archive = root["$objects"].as_array().unwrap();
    let first_dictionary = archive[1].as_dictionary().unwrap();
    assert_eq!(
        first_dictionary["NS.keys"].as_array().unwrap()[0]
            .as_uid()
            .unwrap()
            .get(),
        2
    );
}

#[test]
fn upstream_mixed_integer_extremes_and_empty_collections() {
    let value: Value = from_bytes(UPSTREAM_MIXED).unwrap();
    let root = value.as_dictionary().unwrap();
    assert_eq!(root["SmallestNumber"].as_signed_integer(), Some(i64::MIN));
    assert_eq!(root["BiggestNumber"].as_unsigned_integer(), Some(u64::MAX));
    assert!(root["EmptyArray"].as_array().unwrap().is_empty());
    assert!(root["EmptyDictionary"].as_dictionary().unwrap().is_empty());
    assert_eq!(root["Birthdate"].as_date().unwrap().to_xml_format(), "1981-05-16T11:32:06Z");
    assert_eq!(root["Data"].as_data().unwrap().len(), 15);
}

#[test]
fn upstream_utf16_decodes_long_unicode_string() {
    let value: Value = from_bytes(UPSTREAM_UTF16).unwrap();
    let root = value.as_dictionary().unwrap();
    assert_eq!(root["name"].as_string(), Some("★ or better"));
    let long_text = root["longText"].as_string().unwrap();
    assert!(long_text.starts_with("The sun was shining on the sea"));
    assert!(long_text.ends_with('★'));
}

#[test]
fn upstream_large_three_byte_offset_table() {
    let doc = Document::parse(UPSTREAM_THREE_BYTE_OFFSETS).unwrap();
    assert_eq!(doc.object_count(), 10_575);
    let value: Value = from_bytes(UPSTREAM_THREE_BYTE_OFFSETS).unwrap();
    let root = value.as_dictionary().unwrap();
    let data = root["data"].as_dictionary().unwrap();
    assert_eq!(data.len(), 2_199);
    let entry = data["1838"].as_dictionary().unwrap();
    assert_eq!(
        entry["platformName"].as_dictionary().unwrap()["name"].as_string(),
        Some("SiriKit Cloud Media")
    );
}

#[test]
fn upstream_circular_array_with_invalid_trailer_is_rejected() {
    assert!(Document::parse(UPSTREAM_CIRCULAR_ARRAY).is_err());
    assert!(from_bytes::<Value>(UPSTREAM_CIRCULAR_ARRAY).is_err());
}
