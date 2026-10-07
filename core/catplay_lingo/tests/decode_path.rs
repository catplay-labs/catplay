use catplay_lingo::{DecodeContext, DecodeError, Iap1Decode, Iap1Message, Reader, iap1, iap1_enum_tokens, iap1_record, lingo_0x0};
use catplay_reflect::static_strings;

static_strings! {
    pub struct TestLingoString(u16) {
        S1 = "count",
        S2 = "displayName",
        S3 = "display_name",
        S4 = "enabledFlag",
        S5 = "enabled_flag",
        S6 = "entries",
        S7 = "entryCount",
        S8 = "fidTokens",
        S9 = "recentEntries",
        S10 = "tokens",
        S11 = "Entry",
        S12 = "TokenList",
        S13 = "Update",
        S14 = "Tokens",
    }
}

iap1_record! {
    strings = TestLingoString;
    pub struct Entry {
        fields {
            required pub enabled_flag: bool => { bit: 0, key: "enabledFlag", when: true, predicate_value: false },
            required pub display_name: String => { bit: 1, key: "displayName", when: true, predicate_value: false },
        }
        steps {
            0 => required enabled_flag: bool => { bit: 0, key: "enabledFlag", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            1 => required display_name: String => { bit: 1, key: "displayName", wire: [counted(2)], project_wire: catplay_lingo::Iap1WireSpec::CountedLiteral(2), predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum_tokens! {
    strings = TestLingoString;
    pub enum Tokens {
        #[iap1_token(fid_type = 1, fid_subtype = 2)]
        Entry(Entry),
    }
}

iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0, command = 1, source = device,
        response = false, ack = false, deprecated = false, transaction_id = prohibited)]
    pub struct Update {
        fields {
            required pub count: u8 => { bit: 0, key: "entryCount", when: true, predicate_value: true },
            required pub entries: Vec<Entry> => { bit: 1, key: "recentEntries", when: true, predicate_value: false },
        }
        steps {
            0 => required count: u8 => { bit: 0, key: "entryCount", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
            1 => required entries: Vec<Entry> => { bit: 1, key: "recentEntries", wire: [counted(count)], project_wire: catplay_lingo::Iap1WireSpec::CountedField(0), predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = TestLingoString;
    pub struct TokenList {
        fields {
            required pub tokens: Vec<Tokens> => { bit: 0, key: "fidTokens", when: true, predicate_value: false },
        }
        steps {
            0 => required tokens: Vec<Tokens> => { bit: 0, key: "fidTokens", wire: [counted_remaining], project_wire: catplay_lingo::Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: true, program: [catplay_lingo::FlatPredicateToken::Bool(true)] } },
        }
    }
}

fn check(error: DecodeError, path: &str, cause: DecodeError) {
    assert_eq!(error.path().unwrap().as_str(), path);
    assert!(!error.path().unwrap().truncated());
    assert_eq!(error.cause(), &cause);
    assert!(error.to_string().starts_with(path));
}

#[test]
fn counted_list_reports_the_exact_element_and_schema_key() {
    for (bytes, path, cause) in [
        (
            &[2, 1, b'a', 0, 7][..],
            "Update.recentEntries[1].enabledFlag",
            DecodeError::InvalidBool(7),
        ),
        (
            &[2, 1, b'a', 0, 1, 0xff, 0][..],
            "Update.recentEntries[1].displayName",
            DecodeError::InvalidUtf8,
        ),
        (
            &[2, 1, b'a', 0][..],
            "Update.recentEntries[1].enabledFlag",
            DecodeError::UnexpectedEof { needed: 1, remaining: 0 },
        ),
        (
            &[1, 1, b'a', b'b'][..],
            "Update.recentEntries[0].displayName",
            DecodeError::MissingNullTerminator,
        ),
        (&[][..], "Update.entryCount", DecodeError::UnexpectedEof { needed: 1, remaining: 0 }),
    ] {
        check(Update::decode(&DecodeContext::default(), bytes).unwrap_err(), path, cause);
    }
}

#[test]
fn trailing_message_bytes_belong_to_the_root() {
    check(
        Update::decode(&DecodeContext::default(), &[0, 42]).unwrap_err(),
        "Update",
        DecodeError::TrailingData { remaining: 1 },
    );
}

#[test]
fn direct_record_starts_with_its_struct_name() {
    check(
        Entry::decode(&mut Reader::new(&[2])).unwrap_err(),
        "Entry.enabledFlag",
        DecodeError::InvalidBool(2),
    );
}

#[test]
fn enum_error_uses_the_schema_path() {
    check(
        lingo_0x0::SetFIDTokenValuesTokensAccessoryInfoToken::decode(&mut Reader::new(&[0])).unwrap_err(),
        "SetFIDTokenValuesTokensAccessoryInfoToken.accInfoType",
        DecodeError::InvalidEnumValue { value: 0 },
    );
}

#[test]
fn token_payload_keeps_the_parent_path() {
    // First token is valid; second has an invalid boolean inside its record.
    check(
        TokenList::decode(&mut Reader::new(&[5, 1, 2, 1, b'a', 0, 5, 1, 2, 9, b'a', 0])).unwrap_err(),
        "TokenList.fidTokens[1].enabledFlag",
        DecodeError::InvalidBool(9),
    );
}

#[test]
fn token_headers_and_trailing_bytes_identify_the_element() {
    for (bytes, cause) in [
        (&[1][..], DecodeError::InvalidFieldValue { field: "token length" }),
        (&[4, 1, 2][..], DecodeError::UnexpectedEof { needed: 4, remaining: 2 }),
        (
            &[2, 9, 9][..],
            DecodeError::UnknownToken {
                token: "Tokens",
                fid_type: 9,
                fid_subtype: 9,
            },
        ),
        (&[6, 1, 2, 1, b'a', 0, 42][..], DecodeError::TrailingData { remaining: 1 }),
    ] {
        check(
            TokenList::decode(&mut Reader::new(bytes)).unwrap_err(),
            "TokenList.fidTokens[0]",
            cause,
        );
    }
}
