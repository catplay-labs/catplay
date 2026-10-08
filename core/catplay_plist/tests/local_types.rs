#![cfg(feature = "serde")]

use catplay_plist::bplist::wire::decode::{Document, Integer as WireInteger, Object};
use catplay_plist::{Date, Dictionary, Error, Integer, Uid, Value, from_bytes, from_value, to_value, to_writer_binary};
use serde::Deserialize;

#[test]
fn local_value_round_trips_binary_and_exposes_wire_objects() {
    let date = Date::from_xml_format("2024-01-01T00:00:00Z").unwrap();
    let mut map = Dictionary::new();
    map.insert("name".into(), Value::String("CarPlay".into()));
    map.insert("date".into(), Value::Date(date));
    map.insert("uid".into(), Value::Uid(Uid::new(65_000)));
    map.insert("large".into(), Value::Integer(Integer::from(u64::MAX)));
    map.insert("data".into(), Value::Data(vec![0, 1, 255]));
    let value = Value::Dictionary(map);

    let mut bytes = Vec::new();
    to_writer_binary(&mut bytes, &value).unwrap();
    assert_eq!(from_bytes::<Value>(&bytes).unwrap(), value);
    assert_eq!(to_value(&value).unwrap(), value);
    let doc = Document::parse(&bytes).unwrap();
    let large = doc.dictionary_get(doc.root(), "large").unwrap().unwrap();
    assert!(matches!(
        doc.object(large).unwrap(),
        Object::Integer(WireInteger::Unsigned(u64::MAX))
    ));
    let data = doc.dictionary_get(doc.root(), "data").unwrap().unwrap();
    assert!(matches!(doc.object(data).unwrap(), Object::Data([0, 1, 255])));
}

#[test]
fn local_value_reads_xml() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>name</key><string>CarPlay</string><key>date</key><date>2024-01-01T00:00:00Z</date></dict></plist>"#;
    let value: Value = from_bytes(xml).unwrap();
    let map = value.as_dictionary().unwrap();
    assert_eq!(map["name"].as_string(), Some("CarPlay"));
    assert_eq!(map["date"].as_date().unwrap().to_xml_format(), "2024-01-01T00:00:00Z");

    let mut output = Vec::new();
    value.to_writer_xml(&mut output).unwrap();
    assert_eq!(Value::from_xml_bytes(output.as_slice()).unwrap(), value);
}

#[derive(Debug, Deserialize, PartialEq)]
struct Payload {
    name: String,
    omitted: Option<u64>,
    present: Option<u64>,
}

#[test]
fn from_value_preserves_struct_field_option_semantics() {
    let mut map = Dictionary::new();
    map.insert("name".into(), "CarPlay".into());
    map.insert("present".into(), 17u64.into());
    let parsed: Payload = from_value(&Value::Dictionary(map)).unwrap();
    assert_eq!(
        parsed,
        Payload {
            name: "CarPlay".into(),
            omitted: None,
            present: Some(17)
        }
    );
}

#[test]
fn explicit_none_requires_string_sentinel() {
    let mut map = Dictionary::new();
    map.insert("None".into(), Value::String(String::new()));
    let value = Value::Array(vec![Value::Dictionary(map)]);
    assert_eq!(from_value::<Vec<Option<u64>>>(&value).unwrap(), vec![None]);

    let mut malformed = Dictionary::new();
    malformed.insert("None".into(), 1u64.into());
    assert!(from_value::<Vec<Option<u64>>>(&Value::Array(vec![Value::Dictionary(malformed)])).is_err());
}

#[test]
fn public_error_is_local() {
    let err = Error::message("broken");
    assert_eq!(err.to_string(), "broken");
}
