use catplay_lingo::{DecodeContext, DecodeError, EncodeContext, EncodeError, Iap1Decode, Iap1Encode, Iap1Message, Reader, Writer};
use catplay_lingo::{iap1_disjoint, iap1_record};
use catplay_reflect::static_strings;

static_strings! {
    pub struct TestLingoString(u16) {
        S1 = "length",
        S2 = "marker",
        S3 = "selector",
        S4 = "tail",
        S5 = "value",
        S6 = "decode_bytes",
        S7 = "decode_word",
        S8 = "DynamicChild",
        S9 = "Scheduled",
        S10 = "Bytes",
        S11 = "Word",
    }
}

iap1_record! {
    strings = TestLingoString;
    struct Scheduled {
        fields {
            required pub selector: u8 => { bit: 0, key: "selector", when: true, predicate_value: true },
            required pub value: u16 => { bit: 1, key: "value", when: true, predicate_value: false },
            required pub marker: u8 => { bit: 2, key: "marker", when: true, predicate_value: false },
            required pub tail: u8 => { bit: 3, key: "tail", when: true, predicate_value: false },
        }
        steps {
            1 => required selector: u8 => { bit: 0, key: "selector", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => required value: u16 => { bit: 1, key: "value", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&selector, &0u64), program: [catplay_lingo::FlatPredicateToken::Eq, catplay_lingo::FlatPredicateToken::Field(0), catplay_lingo::FlatPredicateToken::Integer(0u8)] } },
            3 => required marker: u8 => { bit: 2, key: "marker", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            4 => required value: u16 => { bit: 1, key: "value", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ne(&selector, &0u64), program: [catplay_lingo::FlatPredicateToken::Ne, catplay_lingo::FlatPredicateToken::Field(0), catplay_lingo::FlatPredicateToken::Integer(0u8)] } },
            5 => required tail: u8 => { bit: 3, key: "tail", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

#[test]
fn exhaustive_disjoin_keeps_both_virtual_positions() {
    for payload in [[0, 0x12, 0x34, 0x56, 0x78], [1, 0x56, 0x12, 0x34, 0x78]] {
        let mut reader = Reader::new(&payload);
        let value = Scheduled::decode(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(value.value, 0x1234);
        assert_eq!(value.marker, 0x56);
        assert_eq!(value.tail, 0x78);
        let mut encoded = Vec::new();
        value.encode(&mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
    }
}

#[test]
fn rds_same_enum_is_shared_in_both_branches_and_absent_in_gap() {
    use catplay_lingo::lingos::lingo_0x7::{RetRdsData, RetRdsDataCharSet};
    for payload in [
        &[4, 1, b'A', b'B'][..],
        &[30, 1, b'A', b'B'][..],
        &[5, 0, 1, 0, 2, 0, 3, 0, 4, 0][..],
    ] {
        let value = <RetRdsData as Iap1Message>::decode(&DecodeContext::default(), payload).unwrap();
        assert_eq!(value.char_set.is_some(), payload[0] != 5);
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &EncodeContext::default(), &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
        let mut invalid = value;
        invalid.char_set = if payload[0] == 5 {
            Some(RetRdsDataCharSet::LatinBasedLanguages)
        } else {
            None
        };
        let error = Iap1Message::encode(&invalid, &EncodeContext::default(), &mut Writer::new(&mut Vec::new())).unwrap_err();
        assert!(matches!(
            error,
            EncodeError::MissingConditionalField { field: "charSet" } | EncodeError::UnexpectedConditionalField { field: "charSet" }
        ));
    }
    assert!(matches!(
        <RetRdsData as Iap1Message>::decode(&DecodeContext::default(), &[30]).map_err(|error| error.cause().clone()),
        Err(DecodeError::UnexpectedEof { .. })
    ));
}

#[test]
fn location_refresh_interval_disjoint_fixes_selector_binding_without_tags() {
    use catplay_lingo::lingos::lingo_0xe::{RetAccessoryData, RetAccessoryDataDataTypeDisjoint, RetAccessoryDataUnknownDataType};
    for (data_type, section) in [(3, 0u16), (4, 0), (3, 1), (4, 1)] {
        let mut payload = vec![2, data_type];
        payload.extend(section.to_be_bytes());
        payload.extend(1u16.to_be_bytes());
        if section == 0 {
            payload.extend(4u32.to_be_bytes());
        }
        payload.extend(3600u32.to_be_bytes());
        let mut value = <RetAccessoryData as Iap1Message>::decode(&DecodeContext::default(), &payload).unwrap();
        assert_eq!(value.max_required_refresh_interval, (data_type == 3).then_some(3600));
        assert_eq!(value.recommended_refresh_interval, (data_type == 4).then_some(3600));
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &EncodeContext::default(), &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
        value.data_type = RetAccessoryDataDataTypeDisjoint::UnknownDataType(RetAccessoryDataUnknownDataType(data_type));
        assert_eq!(
            Iap1Message::encode(&value, &EncodeContext::default(), &mut Writer::new(&mut Vec::new())),
            Err(EncodeError::InvalidFieldValue { field: "dataType" })
        );
    }
}

#[test]
fn different_width_challenges_share_storage_and_ignore_encode_length_hint() {
    use catplay_lingo::lingo_0x0::{GetAccessoryAuthenticationSignature, GetAccessoryAuthenticationSignatureChallengeDisjoint};
    for width in [16usize, 20] {
        let mut payload = vec![0x5a; width];
        payload.push(1);
        let ctx = DecodeContext {
            adjusted_payload_length: payload.len(),
            has_transaction_id: false,
        };
        let mut value = <GetAccessoryAuthenticationSignature as Iap1Message>::decode(&ctx, &payload).unwrap();
        assert_eq!(value.authentication_retry_counter, Some(1));
        let ctx = EncodeContext {
            adjusted_payload_length: payload.len(),
            has_transaction_id: false,
        };
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &ctx, &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
        value.challenge = Some(if width == 16 {
            GetAccessoryAuthenticationSignatureChallengeDisjoint::Value20ByteChallenge([0; 20])
        } else {
            GetAccessoryAuthenticationSignatureChallengeDisjoint::Value16ByteChallenge([0; 16])
        });
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &ctx, &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded.len(), if width == 16 { 21 } else { 17 });
        let decoded = GetAccessoryAuthenticationSignature::decode(
            &DecodeContext {
                adjusted_payload_length: encoded.len(),
                has_transaction_id: false,
            },
            &encoded,
        )
        .unwrap();
        assert_eq!(decoded, value);
    }
}

#[test]
fn preference_classes_select_semantically_named_variants() {
    use catplay_lingo::lingo_0x0::RetiPodPreferences;
    for (class, variant) in [
        (0x00, "VideoOutSetting"),
        (0x01, "ScreenConfiguration"),
        (0x02, "VideoSignalFormat"),
        (0x03, "LineOutUsage"),
        (0x08, "VideoOutConnection"),
        (0x09, "ClosedCaptioning"),
        (0x0a, "VideoMonitorAspectRatio"),
        (0x0c, "Subtitles"),
        (0x0d, "VideoAlternateAudioChannel"),
        (0x0f, "PauseOnPowerRemoval"),
        (0x14, "VoiceOver"),
        (0x16, "AssistiveTouch"),
    ] {
        let payload = [class, 1];
        let value = <RetiPodPreferences as Iap1Message>::decode(&DecodeContext::default(), &payload).unwrap();
        assert!(format!("{:?}", value.preference_setting_id.as_ref().unwrap()).starts_with(&format!("{variant}(")));
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &EncodeContext::default(), &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
    }
}

#[test]
fn disjoint_options_preserve_unknown_lingoes_and_all_unknown_bits() {
    use catplay_lingo::PredicateValue;
    use catplay_lingo::lingo_0x0::RetiPodOptionsForLingo;
    let bits = 0x8100_0000_0000_0001u64;
    for (lingo, variant) in [
        (0, "GeneralOptions"),
        (5, "AccessoryPowerOptions"),
        (8, "AccessoryEqualizerOptions"),
        (11, "TestOptions"),
        (14, "LocationOptions"),
        (15, "UnknownLingoOptions"),
        (255, "UnknownLingoOptions"),
    ] {
        let mut payload = vec![lingo];
        payload.extend(bits.to_be_bytes());
        let value = <RetiPodOptionsForLingo as Iap1Message>::decode(&DecodeContext::default(), &payload).unwrap();
        assert_eq!(value.option_bits.predicate_integer(), Some(bits as i128));
        assert!(format!("{:?}", value.option_bits).starts_with(&format!("{variant}(")));
        let mut encoded = Vec::new();
        Iap1Message::encode(&value, &EncodeContext::default(), &mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
    }
}

iap1_disjoint! {
    strings = TestLingoString;
    #[iap1_disjoint(predicate = opaque)]
    enum CountedOrScalar {
        #[iap1_disjoint(decode = decode_bytes, encode = encode_bytes)]
        Bytes(Vec<u8>),
        #[iap1_disjoint(decode = decode_word, encode = encode_word)]
        Word(u16),
    }
}

#[test]
fn disjoint_debug_uses_names_from_string_pool() {
    assert_eq!(format!("{:?}", CountedOrScalar::Bytes(vec![1, 2])), "Bytes([1, 2])");
    assert_eq!(format!("{:?}", CountedOrScalar::Word(7)), "Word(7)");
}

iap1_record! {
    strings = TestLingoString;
    struct DynamicChild {
        fields {
            required pub selector: u8 => { bit: 0, key: "selector", when: true, predicate_value: true },
            required pub length: u8 => { bit: 1, key: "length", when: true, predicate_value: true },
            required pub value: CountedOrScalar => { bit: 2, key: "value", when: true, predicate_value: false },
        }
        steps {
            1 => required selector: u8 => { bit: 0, key: "selector", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            2 => required length: u8 => { bit: 1, key: "length", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            3 => required value: CountedOrScalar => { bit: 2, key: "value", wire: [disjoint(decode_bytes, encode_bytes, Vec<u8>, [counted(length)])], project_wire: catplay_lingo::Iap1WireSpec::Disjoint { variant: const { TestLingoString::require_raw("decode_bytes") }, child: &catplay_lingo::Iap1WireSpec::CountedField(1) }, predicate_value: false, predicate: { expr: eq(&selector, &0u64), program: [catplay_lingo::FlatPredicateToken::Eq, catplay_lingo::FlatPredicateToken::Field(0), catplay_lingo::FlatPredicateToken::Integer(0u8)] } },
            4 => required value: CountedOrScalar => { bit: 2, key: "value", wire: [disjoint(decode_word, encode_word, u16, [scalar])], project_wire: catplay_lingo::Iap1WireSpec::Disjoint { variant: const { TestLingoString::require_raw("decode_word") }, child: &catplay_lingo::Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: ne(&selector, &0u64), program: [catplay_lingo::FlatPredicateToken::Ne, catplay_lingo::FlatPredicateToken::Field(0), catplay_lingo::FlatPredicateToken::Integer(0u8)] } },
        }
    }
}

#[test]
fn child_proxy_keeps_dynamic_count_and_scalar_wire_policies() {
    for payload in [&[0, 3, 0x11, 0x22, 0x33][..], &[1, 99, 0x12, 0x34][..]] {
        let mut reader = Reader::new(payload);
        let value = DynamicChild::decode(&mut reader).unwrap();
        reader.finish().unwrap();
        let mut encoded = Vec::new();
        value.encode(&mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, payload);
    }
}
