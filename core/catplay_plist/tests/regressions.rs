#![cfg(feature = "serde")]

use catplay_plist::{
    Date, Dictionary, Value, from_bytes, from_value, from_xml_bytes, plist_bitflags, to_value, to_writer_binary, to_writer_xml,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum OptionalVariant {
    X(Option<u64>),
}

fn dict(key: &str, value: Value) -> Value {
    Value::Dictionary(Dictionary::from_iter([(key, value)]))
}

fn check_representation<T>(value: &T, expected: Value)
where
    T: Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let mut bytes = Vec::new();
    to_writer_binary(&mut bytes, &expected).unwrap();
    assert_eq!(&from_bytes::<T>(&bytes).unwrap(), value);
    assert_eq!(&from_value::<T>(&expected).unwrap(), value);
    bytes.clear();
    to_writer_binary(&mut bytes, value).unwrap();
    assert_eq!(from_bytes::<Value>(&bytes).unwrap(), expected);
    assert_eq!(to_value(value).unwrap(), expected);
}

#[test]
fn enum_option_preserves_parent_context() {
    // plist 1.8 encodes a root newtype variant's Some without an extra wrapper.
    check_representation(&OptionalVariant::X(Some(42)), dict("X", 42u64.into()));
    // Sequence/map entries use explicit options, including None.
    let entries = vec![OptionalVariant::X(Some(42)), OptionalVariant::X(None)];
    check_representation(
        &entries,
        Value::Array(vec![dict("X", dict("Some", 42u64.into())), dict("X", dict("None", "".into()))]),
    );
    let map = std::collections::BTreeMap::from([("entry".to_owned(), OptionalVariant::X(Some(42)))]);
    check_representation(&map, dict("entry", dict("X", dict("Some", 42u64.into()))));
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Field {
        entry: Option<OptionalVariant>,
    }
    check_representation(
        &Field {
            entry: Some(OptionalVariant::X(Some(42))),
        },
        dict("entry", dict("X", dict("Some", 42u64.into()))),
    );
}

#[test]
fn fixed_sequences_reject_trailing_elements_in_binary_value_and_xml() {
    let value = Value::Array(vec![1u64.into(), 2u64.into(), 3u64.into()]);
    let mut binary = Vec::new();
    to_writer_binary(&mut binary, &value).unwrap();
    let xml = b"<array><integer>1</integer><integer>2</integer><integer>3</integer></array>";
    assert!(from_bytes::<(u64, u64)>(&binary).is_err());
    assert!(from_value::<(u64, u64)>(&value).is_err());
    assert!(from_xml_bytes::<(u64, u64)>(xml).is_err());
    assert!(from_bytes::<[u64; 2]>(&binary).is_err());
    assert!(from_value::<[u64; 2]>(&value).is_err());
    assert_eq!(from_bytes::<[u64; 3]>(&binary).unwrap(), [1, 2, 3]);
    assert_eq!(from_value::<(u64, u64, u64)>(&value).unwrap(), (1, 2, 3));
}

#[test]
fn value_and_xml_preserve_nanosecond_dates() {
    let date = Date::from_xml_format("2026-09-24T12:34:56.123456789Z").unwrap();
    // Exercise nesting and the public Date serializer as well as Value itself.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Dated {
        dates: Vec<Date>,
    }
    let original = Dated { dates: vec![date] };
    let expected = dict("dates", Value::Array(vec![Value::Date(date)]));
    assert_eq!(to_value(&original).unwrap(), expected);
    assert_eq!(to_value(&expected).unwrap(), expected);
    let mut xml = Vec::new();
    to_writer_xml(&mut xml, &original).unwrap();
    assert_eq!(from_xml_bytes::<Dated>(&xml).unwrap(), original);
}

#[derive(Debug)]
struct FirstEntry;
impl<'de> Deserialize<'de> for FirstEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FirstEntry;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a dictionary")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> Result<FirstEntry, M::Error> {
                let _: Option<(String, u64)> = map.next_entry()?;
                Ok(FirstEntry)
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}

#[test]
fn map_visitors_must_consume_all_entries() {
    let value = Value::Dictionary(Dictionary::from_iter([("one", 1u64), ("two", 2u64)]));
    let mut bytes = Vec::new();
    to_writer_binary(&mut bytes, &value).unwrap();
    assert!(from_bytes::<FirstEntry>(&bytes).is_err());
    assert!(from_value::<FirstEntry>(&value).is_err());
    // Normal derived structs still consume unknown fields through IgnoredAny.
    #[derive(Deserialize)]
    struct Known {
        one: u64,
    }
    assert_eq!(from_bytes::<Known>(&bytes).unwrap().one, 1);
    assert_eq!(from_value::<Known>(&value).unwrap().one, 1);
}

#[test]
fn legacy_u64_audio_format_adapter_truncates_wide_flags() {
    plist_bitflags! {
        struct AudioFormat: u128 {
            const LOW = 1 << 4;
            const HIGH = 1 << 80;
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AudioFormatFields {
        #[serde(with = "catplay_plist::downcast_enum_to_legacy_u64")]
        audio_format: AudioFormat,
        #[serde(
            default,
            with = "catplay_plist::downcast_enum_to_legacy_u64::option",
            skip_serializing_if = "Option::is_none"
        )]
        audio_input_formats: Option<AudioFormat>,
    }

    let fields = AudioFormatFields {
        audio_format: AudioFormat::LOW | AudioFormat::HIGH,
        audio_input_formats: Some(AudioFormat::HIGH),
    };
    let expected = Value::Dictionary(Dictionary::from_iter([
        ("audioFormat", Value::from(16u64)),
        ("audioInputFormats", Value::from(0u64)),
    ]));
    assert_eq!(to_value(&fields).unwrap(), expected);
    let mut binary = Vec::new();
    to_writer_binary(&mut binary, &fields).unwrap();
    assert_eq!(from_bytes::<Value>(&binary).unwrap(), expected);
    assert_eq!(
        from_bytes::<AudioFormatFields>(&binary)
            .unwrap()
            .audio_format,
        AudioFormat::LOW
    );
    assert_eq!(
        from_value::<AudioFormatFields>(&expected)
            .unwrap()
            .audio_input_formats,
        Some(AudioFormat::empty())
    );

    let missing = AudioFormatFields {
        audio_format: AudioFormat::LOW,
        audio_input_formats: None,
    };
    assert_eq!(from_value::<AudioFormatFields>(&to_value(&missing).unwrap()).unwrap(), missing);
}
