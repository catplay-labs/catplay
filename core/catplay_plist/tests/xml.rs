#![cfg(feature = "serde")]

use catplay_plist::{
    Date, Dictionary, PlistByteArray, PlistError, Value, XmlWriteOptions, from_bytes, from_xml_bytes, to_writer_xml,
    to_writer_xml_with_options,
};

#[test]
fn borrowed_byte_slice_and_plist_byte_array_have_different_xml_types() {
    let payload = [0u8, 1, 2, 255];

    let mut slice_xml = Vec::new();
    to_writer_xml(&mut slice_xml, &payload.as_slice()).unwrap();
    let slice_text = std::str::from_utf8(&slice_xml).unwrap();
    assert!(
        slice_text.contains(
            "<array>\n\t<integer>0</integer>\n\t<integer>1</integer>\n\t<integer>2</integer>\n\t<integer>255</integer>\n</array>"
        )
    );
    assert_eq!(from_xml_bytes::<Vec<u8>>(&slice_xml).unwrap(), payload);
    assert!(from_xml_bytes::<PlistByteArray>(&slice_xml).is_err());

    let mut data_xml = Vec::new();
    to_writer_xml(&mut data_xml, &PlistByteArray::from(payload)).unwrap();
    let data_text = std::str::from_utf8(&data_xml).unwrap();
    assert!(data_text.contains("<data>AAEC/w==</data>"));
    assert_eq!(
        from_xml_bytes::<PlistByteArray>(&data_xml)
            .unwrap()
            .as_ref(),
        payload
    );
    assert!(from_xml_bytes::<Vec<u8>>(&data_xml).is_err());
}

#[test]
fn xml_value_round_trip_with_entities_data_and_date() {
    let mut dict = Dictionary::new();
    dict.insert("<key>&".into(), Value::String("a<&\"'ż".into()));
    dict.insert("data".into(), Value::Data((0..=255).collect()));
    dict.insert("date".into(), Value::Date(Date::from_xml_format("2024-01-02T03:04:05Z").unwrap()));
    dict.insert(
        "array".into(),
        Value::Array(vec![Value::Boolean(true), Value::Boolean(false), Value::Integer(u64::MAX.into())]),
    );
    let value = Value::Dictionary(dict);

    let mut bytes = Vec::new();
    value.to_writer_xml(&mut bytes).unwrap();
    assert_eq!(from_bytes::<Value>(&bytes).unwrap(), value);
    assert_eq!(from_xml_bytes::<Value>(bytes.as_slice()).unwrap(), value);

    let xml = std::str::from_utf8(&bytes).unwrap();
    assert!(xml.contains("<key>&lt;key&gt;&amp;</key>"));
    assert!(xml.contains("<date>2024-01-02T03:04:05Z</date>"));
    assert!(xml.contains("<data>AAECAwQ"));
}

#[test]
fn xml_options_and_malformed_input() {
    let mut bytes = Vec::new();
    to_writer_xml_with_options(
        &mut bytes,
        &"hello",
        &XmlWriteOptions::default()
            .root_element(false)
            .indent(b' ', 2),
    )
    .unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), "<string>hello</string>");
    assert_eq!(from_bytes::<String>(&bytes).unwrap(), "hello");
    assert_eq!(
        from_bytes::<String>(b"\xef\xbb\xbf<plist><string>hello</string></plist>").unwrap(),
        "hello"
    );
    assert_eq!(
        from_bytes::<Value>(b"<plist><array><true/><string/><data/></array></plist>").unwrap(),
        Value::Array(vec![Value::Boolean(true), Value::String(String::new()), Value::Data(Vec::new())])
    );
    assert_eq!(
        from_bytes::<String>(b"<plist><string><![CDATA[a<b]]>&amp;&#x17c;</string></plist>").unwrap(),
        "a<b&ż"
    );
    assert!(from_bytes::<String>(b"<plist><string>&bogus;</string></plist>").is_err());
    assert!(from_bytes::<Value>(b"<plist><array><string>x</array></plist>").is_err());
    assert!(matches!(from_bytes::<Value>(b"{ answer = 42; }"), Err(PlistError::Decode(_))));
}
