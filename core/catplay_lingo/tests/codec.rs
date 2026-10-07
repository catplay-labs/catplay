use catplay_lingo::{
    DecodeContext, DecodeError, EncodeContext, EncodeError, Iap1Decode, Iap1DecodeCounted, Iap1DecodeRemaining, Iap1Encode,
    Iap1EncodeCounted, Iap1Message, PredicateValue, Reader, Writer, iap1, iap1_bitflags, iap1_enum, iap1_enum_open, iap1_enum_tokens,
};
use catplay_reflect::static_strings;

iap1_enum_tokens! {
    strings = TestLingoString;
    pub enum ExampleTokens {
        #[iap1_token(fid_type = 0x12, fid_subtype = 0x34)]
        Word(u16),
        #[iap1_token(fid_type = 0x12, fid_subtype = 0x35)]
        #[deprecated]
        Byte(u8),
    }
}

#[test]
#[allow(deprecated)]
fn token_enum_macro_round_trips_known_and_rejects_unknown_variants() {
    let known = ExampleTokens::Word(0xbeef);
    let mut bytes = Vec::new();
    known.encode(&mut Writer::new(&mut bytes)).unwrap();
    assert_eq!(bytes, [4, 0x12, 0x34, 0xbe, 0xef]);
    assert_eq!(ExampleTokens::decode(&mut Reader::new(&bytes)), Ok(known));
    assert_eq!(
        ExampleTokens::decode(&mut Reader::new(&[3, 0x12, 0x35, 7])),
        Ok(ExampleTokens::Byte(7))
    );

    assert_eq!(
        ExampleTokens::decode(&mut Reader::new(&[4, 0xfe, 0xfd, 0xaa, 0xbb])),
        Err(DecodeError::UnknownToken {
            token: "ExampleTokens",
            fid_type: 0xfe,
            fid_subtype: 0xfd,
        })
    );
    assert_eq!(
        ExampleTokens::decode(&mut Reader::new(&[4, 0x12, 0x35, 7, 8])),
        Err(DecodeError::TrailingData { remaining: 1 })
    );
}

iap1_bitflags! {
    strings = TestLingoString;
    pub struct ExampleFlags: u16 {
        /// A known bit.
        const FIRST = 1 << 0;
        const SECOND = 1 << 8;
    }
}

#[test]
fn bitflags_macro_preserves_unknown_wire_bits() {
    let raw = 0x8101;
    let flags = ExampleFlags::decode(&mut Reader::new(&[0x81, 0x01])).unwrap();
    assert!(flags.contains(ExampleFlags::FIRST));
    assert!(flags.contains(ExampleFlags::SECOND));
    assert_eq!(flags.bits(), raw);
    assert_eq!(flags.predicate_integer(), Some(raw as i128));
    let mut encoded = Vec::new();
    flags.encode(&mut Writer::new(&mut encoded)).unwrap();
    assert_eq!(encoded, [0x81, 0x01]);
}

#[test]
fn generated_mixed_bitfield_exposes_fields_and_preserves_reserved_bits() {
    use catplay_lingo::lingos::lingo_0xe::{
        SetAccessoryControlSysCtl, SetAccessoryControlSysCtlAccGPSRadioPower, SetAccessoryControlSysCtlFlags,
    };

    let mut value = SetAccessoryControlSysCtl::from_raw(0x83);
    value.flags = SetAccessoryControlSysCtlFlags::ASYNCHRONOUS_LOCATION_NOTIFICATIONS_ENABLED;
    assert_eq!(value.raw(), 0x87);
    assert_eq!(value.flags.bits(), 4);
    assert_eq!(value.acc_gps_radio_power, SetAccessoryControlSysCtlAccGPSRadioPower::PowerOn);
    assert_eq!(value.reserved_bits, 0x80);
    value.acc_gps_radio_power = SetAccessoryControlSysCtlAccGPSRadioPower::PowerOff;
    assert_eq!(value.raw(), 0x84);

    let unknown = SetAccessoryControlSysCtl::from_raw(1);
    assert_eq!(unknown.acc_gps_radio_power, SetAccessoryControlSysCtlAccGPSRadioPower::Other(1));
    assert_eq!(unknown.raw(), 1);
    assert_eq!(
        SetAccessoryControlSysCtl::new(
            SetAccessoryControlSysCtlFlags::empty(),
            SetAccessoryControlSysCtlAccGPSRadioPower::PowerOn,
        )
        .raw(),
        3
    );
}

#[test]
fn generated_group_only_bitfield_preserves_other_values() {
    use catplay_lingo::lingo_0x0::{IdentifyDeviceLingoesOptions, IdentifyDeviceLingoesOptionsAuthenticationControlBits};

    let mut value = IdentifyDeviceLingoesOptions::from_raw(0x10f);
    assert_eq!(
        value.authentication_control_bits,
        IdentifyDeviceLingoesOptionsAuthenticationControlBits::Other(3)
    );
    value.authentication_control_bits = IdentifyDeviceLingoesOptionsAuthenticationControlBits::ImmediateAuthentication;
    assert_eq!(value.raw(), 0x10e);
    assert_eq!(value.reserved_bits, 0x100);
    let mut encoded = Vec::new();
    value.encode(&mut Writer::new(&mut encoded)).unwrap();
    assert_eq!(encoded, [0, 0, 1, 0x0e]);
    assert_eq!(IdentifyDeviceLingoesOptions::decode(&mut Reader::new(&encoded)), Ok(value));
}

iap1_enum! {
    strings = TestLingoString;
    pub enum ExampleClosedEnum: u16 {
        /// The first wire value.
        First = 0x1234,
        /// The second wire value.
        Second = 0x5678,
    }
}

#[test]
fn closed_enum_macro_reuses_scalar_codec_and_rejects_unknown_values() {
    let names = <ExampleClosedEnum as catplay_lingo::Iap1Enum<u16>>::NAMES;
    assert_eq!(names.iter().map(|name| name.get()).collect::<Vec<_>>(), ["First", "Second"]);
    let mut reader = Reader::new(&[0x12, 0x34]);
    let value = ExampleClosedEnum::decode(&mut reader).unwrap();
    reader.finish().unwrap();
    assert_eq!(value, ExampleClosedEnum::First);
    assert_eq!(value.raw(), 0x1234);
    assert_eq!(value.predicate_integer(), Some(0x1234));
    let mut bytes = Vec::new();
    value.encode(&mut Writer::new(&mut bytes)).unwrap();
    assert_eq!(bytes, [0x12, 0x34]);
    assert_eq!(
        ExampleClosedEnum::decode(&mut Reader::new(&[0x00, 0x01])),
        Err(DecodeError::InvalidEnumValue { value: 1 })
    );
}

iap1_enum_open! {
    strings = TestLingoString;
    #[iap1_enum_open(storage = u16, ranges = [1..=3, 0x100..=0x102])]
    pub enum ExampleOpenEnum {
        /// A named value inside the first permitted range.
        #[deprecated]
        #[allow(deprecated)]
        Named = 2,
        Zero = 0,
    }
}

#[test]
#[allow(deprecated)]
fn open_enum_macro_preserves_named_values_and_checks_each_range() {
    assert_eq!(format!("{:?}", ExampleOpenEnum::Named), "Named");
    assert_eq!(format!("{:?}", ExampleOpenEnum::Other(3)), "Other(3)");
    assert_eq!(format!("{:?}", ExampleOpenEnum::Other(2)), "Other(2)");
    let names = <ExampleOpenEnum as catplay_lingo::Iap1Enum<u16>>::NAMES;
    assert_eq!(names.iter().map(|name| name.get()).collect::<Vec<_>>(), ["Named", "Zero"]);
    assert_eq!(ExampleOpenEnum::decode(&mut Reader::new(&[0, 2])), Ok(ExampleOpenEnum::Named));
    assert_eq!(ExampleOpenEnum::decode(&mut Reader::new(&[0, 3])), Ok(ExampleOpenEnum::Other(3)));
    assert_eq!(
        ExampleOpenEnum::decode(&mut Reader::new(&[1, 1])),
        Ok(ExampleOpenEnum::Other(0x101))
    );
    assert_eq!(
        ExampleOpenEnum::decode(&mut Reader::new(&[0, 4])),
        Err(DecodeError::InvalidEnumValue { value: 4 })
    );
    assert_eq!(ExampleOpenEnum::Named.raw(), 2);
    assert_eq!(ExampleOpenEnum::Other(0x101).predicate_integer(), Some(0x101));
    let mut bytes = Vec::new();
    ExampleOpenEnum::Other(0x101)
        .encode(&mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [1, 1]);
    assert_eq!(
        ExampleOpenEnum::Other(4).encode(&mut Writer::new(&mut Vec::new())),
        Err(EncodeError::InvalidEnumValue { value: 4 })
    );
    assert_eq!(
        ExampleOpenEnum::Other(0).encode(&mut Writer::new(&mut Vec::new())),
        Err(EncodeError::InvalidEnumValue { value: 0 })
    );
    assert_eq!(ExampleOpenEnum::Zero.raw(), 0);
}

static_strings! {
    pub struct TestLingoString(u16) {
        S1 = "bytes",
        S2 = "chapterCount",
        S3 = "chapter_count",
        S4 = "count",
        S5 = "currentChapterIndex",
        S6 = "current_chapter_index",
        S7 = "first",
        S8 = "infoType",
        S9 = "info_type",
        S10 = "second",
        S11 = "setting",
        S12 = "text",
        S13 = "transactionData",
        S14 = "transaction_data",
        S15 = "First",
        S16 = "Second",
        S17 = "ContextFields",
        S18 = "CountedBytes",
        S19 = "CountedUtf8",
        S20 = "ReturnCurrentPlayingTrackChapterInfo",
        S21 = "WithEarlierField",
        S22 = "ExampleTokens",
        S23 = "Word",
        S24 = "Byte",
        S25 = "ExampleOpenEnum",
        S26 = "FIRST",
        S27 = "SECOND",
        S28 = "Named",
        S29 = "Zero",
    }
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0x04, command = 0x0003, source = device,
        response = true, ack = false, deprecated = false, transaction_id = permitted)]
    pub struct ReturnCurrentPlayingTrackChapterInfo {
        fields {
            required pub current_chapter_index: i32 => { bit: 0, key: "currentChapterIndex", when: true, predicate_value: false },
            required pub chapter_count: i32 => { bit: 1, key: "chapterCount", when: true, predicate_value: false },
        }
        steps {
            1 => required current_chapter_index: i32 => { bit: 0, key: "currentChapterIndex", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => required chapter_count: i32 => { bit: 1, key: "chapterCount", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

#[test]
fn fixed_layout_round_trip() {
    let value = ReturnCurrentPlayingTrackChapterInfo {
        current_chapter_index: -2,
        chapter_count: 0x1234,
    };
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [0xff, 0xff, 0xff, 0xfe, 0, 0, 0x12, 0x34]);
    assert_eq!(
        ReturnCurrentPlayingTrackChapterInfo::decode(&DecodeContext::default(), &bytes).unwrap(),
        value
    );
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0x00, command = 0x0008, source = accessory,
        response = false, ack = false, deprecated = false, transaction_id = required)]
    pub struct WithEarlierField {
        fields {
            required pub info_type: u8 => { bit: 0, key: "infoType", when: true, predicate_value: true },
            optional pub setting: u16 => { bit: 1, key: "setting", when: ::catplay_lingo::predicate_eval::eq(&info_type, &7u64), predicate_value: false },
        }
        steps {
            1 => required info_type: u8 => { bit: 0, key: "infoType", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => optional setting: u16 => { bit: 1, key: "setting", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ::catplay_lingo::predicate_eval::eq(&info_type, &7u64), program: [::catplay_lingo::FlatPredicateToken::Eq, ::catplay_lingo::FlatPredicateToken::Field(0), ::catplay_lingo::FlatPredicateToken::Integer(7u8)] } },
        }
    }
}

#[test]
fn conditional_field_uses_earlier_value() {
    let value = WithEarlierField {
        info_type: 7,
        setting: Some(0x1234),
    };
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [7, 0x12, 0x34]);
    assert_eq!(WithEarlierField::decode(&DecodeContext::default(), &bytes).unwrap(), value);
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0x04, command = 0x0004, source = device,
        response = false, ack = false, deprecated = false, transaction_id = permitted)]
    pub struct ContextFields {
        fields {
            required pub first: u8 => { bit: 0, key: "first", when: true, predicate_value: false },
            optional pub second: u8 => { bit: 1, key: "second", when: ::catplay_lingo::predicate_eval::gt(&ctx.adjusted_payload_length, &1u64), predicate_value: false },
            optional pub transaction_data: u16 => { bit: 2, key: "transactionData", when: ::catplay_lingo::predicate_eval::eq(&ctx.has_transaction_id, &true), predicate_value: false },
        }
        steps {
            1 => required first: u8 => { bit: 0, key: "first", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => optional second: u8 => { bit: 1, key: "second", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ::catplay_lingo::predicate_eval::gt(&ctx.adjusted_payload_length, &1u64), program: [::catplay_lingo::FlatPredicateToken::Gt, ::catplay_lingo::FlatPredicateToken::AdjustedPayloadLength, ::catplay_lingo::FlatPredicateToken::Integer(1u8)] } },
            3 => optional transaction_data: u16 => { bit: 2, key: "transactionData", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ::catplay_lingo::predicate_eval::eq(&ctx.has_transaction_id, &true), program: [::catplay_lingo::FlatPredicateToken::Eq, ::catplay_lingo::FlatPredicateToken::HasTransactionId, ::catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

#[test]
fn context_predicates_and_option_validation() {
    let ctx = DecodeContext {
        adjusted_payload_length: 2,
        has_transaction_id: true,
    };
    let value = ContextFields::decode(&ctx, &[1, 2, 0x12, 0x34]).unwrap();
    assert_eq!(
        value,
        ContextFields {
            first: 1,
            second: Some(2),
            transaction_data: Some(0x1234)
        }
    );
    let mut bytes = Vec::new();
    value
        .encode(
            &EncodeContext {
                adjusted_payload_length: 2,
                has_transaction_id: true,
            },
            &mut Writer::new(&mut bytes),
        )
        .unwrap();
    assert_eq!(bytes, [1, 2, 0x12, 0x34]);

    let missing = ContextFields {
        first: 1,
        second: None,
        transaction_data: None,
    };
    assert_eq!(
        missing.encode(
            &EncodeContext {
                adjusted_payload_length: 2,
                has_transaction_id: false
            },
            &mut Writer::new(&mut Vec::new())
        ),
        Err(EncodeError::MissingConditionalField { field: "second" })
    );
    let unexpected = ContextFields {
        first: 1,
        second: Some(2),
        transaction_data: None,
    };
    assert_eq!(
        unexpected.encode(&EncodeContext::default(), &mut Writer::new(&mut Vec::new())),
        Err(EncodeError::UnexpectedConditionalField { field: "second" })
    );
    assert_eq!(
        ContextFields::decode(&DecodeContext::default(), &[1, 2]).map_err(|error| error.cause().clone()),
        Err(DecodeError::TrailingData { remaining: 1 })
    );
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0x00, command = 0x0040, source = accessory,
        response = false, ack = false, deprecated = false, transaction_id = prohibited)]
    pub struct CountedBytes {
        fields {
            required pub count: u8 => { bit: 0, key: "count", when: true, predicate_value: true },
            required pub bytes: Vec<u8> => { bit: 1, key: "bytes", when: true, predicate_value: false },
        }
        steps {
            1 => required count: u8 => { bit: 0, key: "count", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => required bytes: Vec<u8> => { bit: 1, key: "bytes", wire: [bytes(count)], project_wire: ::catplay_lingo::Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

#[test]
fn length_from_earlier_field() {
    let value = CountedBytes {
        count: 3,
        bytes: vec![0xa1, 0xb2, 0xc3],
    };
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [3, 0xa1, 0xb2, 0xc3]);
    assert_eq!(CountedBytes::decode(&DecodeContext::default(), &bytes), Ok(value));
    let bad = CountedBytes {
        count: 2,
        bytes: vec![0xa1],
    };
    assert_eq!(
        bad.encode(&EncodeContext::default(), &mut Writer::new(&mut Vec::new())),
        Err(EncodeError::InvalidFieldValue { field: "bytes" })
    );
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0x00, command = 0x0041, source = accessory,
        response = false, ack = false, deprecated = false, transaction_id = prohibited)]
    pub struct CountedUtf8 {
        fields {
            required pub count: u8 => { bit: 0, key: "count", when: true, predicate_value: true },
            required pub text: String => { bit: 1, key: "text", when: true, predicate_value: false },
        }
        steps {
            1 => required count: u8 => { bit: 0, key: "count", wire: [scalar], project_wire: ::catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => required text: String => { bit: 1, key: "text", wire: [utf8(count)], project_wire: ::catplay_lingo::Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: true, program: [::catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

#[test]
fn counted_utf8_uses_byte_length_and_preserves_decode_errors() {
    let value = CountedUtf8 {
        count: 3,
        text: "ż".to_owned(),
    };
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [3, 0xc5, 0xbc, 0]);
    assert_eq!(CountedUtf8::decode(&DecodeContext::default(), &bytes), Ok(value));
    assert_eq!(
        CountedUtf8::decode(&DecodeContext::default(), &[2, 0xff, 0]).map_err(|error| error.cause().clone()),
        Err(DecodeError::InvalidUtf8)
    );
    assert_eq!(
        CountedUtf8::decode(&DecodeContext::default(), &[2, b'a']).map_err(|error| error.cause().clone()),
        Err(DecodeError::UnexpectedEof { needed: 2, remaining: 1 })
    );
}

#[test]
fn string_counted_and_remaining_codecs_use_utf8_byte_lengths() {
    let value = "ż".to_owned();
    let mut bytes = Vec::new();
    value
        .encode_counted(&mut Writer::new(&mut bytes), 3, "text")
        .unwrap();
    assert_eq!(bytes, [0xc5, 0xbc, 0]);
    assert_eq!(String::decode_counted(&mut Reader::new(&bytes), 3), Ok(value.clone()));
    assert_eq!(String::decode_remaining(&mut Reader::new(&bytes), "text"), Ok(value.clone()));
    assert_eq!(
        value.encode_counted(&mut Writer::new(&mut Vec::new()), 1, "text"),
        Err(EncodeError::InvalidFieldValue { field: "text" })
    );
    assert_eq!(
        String::decode_remaining(&mut Reader::new(&[0xff, 0]), "text"),
        Err(DecodeError::InvalidUtf8)
    );
}

#[test]
fn byte_vec_counted_and_remaining_codecs_use_the_same_dispatch() {
    let value = vec![0x00, 0xff, 0x7f];
    let mut bytes = Vec::new();
    value
        .encode_counted(&mut Writer::new(&mut bytes), 3, "data")
        .unwrap();
    assert_eq!(bytes, value);
    assert_eq!(Vec::<u8>::decode_counted(&mut Reader::new(&bytes), 3), Ok(value.clone()));
    assert_eq!(Vec::<u8>::decode_remaining(&mut Reader::new(&bytes), "data"), Ok(value.clone()));
    assert_eq!(Vec::<u8>::decode(&mut Reader::new(&bytes)), Ok(value.clone()));
    assert_eq!(
        value.encode_counted(&mut Writer::new(&mut Vec::new()), 2, "data"),
        Err(EncodeError::InvalidFieldValue { field: "data" })
    );
    assert_eq!(
        Vec::<u8>::decode_counted(&mut Reader::new(&bytes), 4),
        Err(DecodeError::UnexpectedEof { needed: 1, remaining: 0 })
    );
}

#[test]
fn borrowed_string_and_slices_encode_without_owned_values() {
    let text: &str = "ż";
    let numbers: &[u16] = &[0x1234, 0xabcd];
    let bytes: &[u8] = &[0, 0xff];
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output);

    text.encode_counted(&mut writer, 3, "text").unwrap();
    numbers.encode_counted(&mut writer, 2, "numbers").unwrap();
    bytes.encode(&mut writer).unwrap();
    assert_eq!(output, [0xc5, 0xbc, 0, 0x12, 0x34, 0xab, 0xcd, 0, 0xff]);

    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output);
    assert_eq!(
        text.encode_counted(&mut writer, 1, "text"),
        Err(EncodeError::InvalidFieldValue { field: "text" })
    );
    assert_eq!(
        numbers.encode_counted(&mut writer, 1, "numbers"),
        Err(EncodeError::InvalidFieldValue { field: "numbers" })
    );
    assert!(output.is_empty());
}
