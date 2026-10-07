use catplay_reflect::static_strings;
use core::mem::{align_of, size_of};
use std::collections::HashSet;

const EXTERNAL: &str = "screen";

static_strings! {
    /// A test pool with Unicode, paths, empty values and aliases.
    pub struct Text {
        SCREEN = EXTERNAL,
        AUDIO = concat!("au", "dio"),
        UNICODE = "μs 🦀",
        PATH = "a/b",
        EMPTY = "",
        ALIAS = "audio",
        EMPTY_ALIAS = "",
    }
}

static_strings! { struct OtherText { AUDIO = "audio" } }
static_strings! { struct Empty {} }
static_strings! { struct Empty8(u8) {} }
static_strings! {
    struct Text8(u8) {
        SCREEN = EXTERNAL,
        AUDIO = "audio",
        UNICODE = "μs 🦀",
        PATH = "a/b",
        EMPTY = "",
        ALIAS = concat!("au", "dio"),
        EMPTY_ALIAS = "",
    }
}
static_strings! { struct Explicit16(u16) { SCREEN = EXTERNAL, AUDIO = "audio" } }
static_strings! {
    struct LeadingEmpty(u16) {
        EMPTY = "",
        UTF8 = "é🙂",
        WORD = "x",
        EMPTY_ALIAS = "",
    }
}
static_strings! { struct EmptyOnly { EMPTY = "", ALIAS = "" } }

const CONST_TEXT: &str = Text::UNICODE.as_str();
const CONST_LOOKUP: Option<Text> = Text::lookup("audio");
const CONST_DECODED: Option<Text> = Text::from_raw(Text::AUDIO.to_raw());
const CONST_TEXT8: &str = Text8::UNICODE.as_str();
const CONST_LOOKUP8: Option<Text8> = Text8::lookup("audio");
const CONST_DECODED8: Option<Text8> = Text8::from_raw(Text8::AUDIO.to_raw());
const CONST_EMPTY: &str = Text8::EMPTY.get();
const CONST_NUL_LOOKUP: Option<Text> = Text::lookup(concat!("audio", "\0"));
const CONST_NUL_LOOKUP8: Option<Text8> = Text8::lookup("\0");
const CONST_RESOLVED: Text = Text::resolve("audio").expect("string not in pool");
const CONST_RESOLVED8: Text8 = Text8::resolve("audio").expect("string not in pool");
const CONST_REQUIRED_RAW: u16 = Text::require_raw("audio");
const CONST_REQUIRED_RAW8: u8 = Text8::require_raw("audio");
const CONST_REQUIRED_STR: &str = Text::require_str("audio");
const CONST_REQUIRED_STR8: &str = Text8::require_str("audio");

#[test]
fn const_resolve_matches_lookup_for_both_widths_and_aliases() {
    assert_eq!(CONST_RESOLVED, Text::ALIAS);
    assert_eq!(CONST_RESOLVED8, Text8::ALIAS);
    assert_eq!(CONST_REQUIRED_RAW, Text::AUDIO.to_raw());
    assert_eq!(CONST_REQUIRED_RAW8, Text8::AUDIO.to_raw());
    assert_eq!(CONST_REQUIRED_STR, "audio");
    assert_eq!(CONST_REQUIRED_STR8, "audio");
    for query in ["screen", "audio", "μs 🦀", "a/b", "", "missing", "audio\0"] {
        assert_eq!(Text::resolve(query), Text::lookup(query));
        assert_eq!(Text8::resolve(query), Text8::lookup(query));
    }
    assert_eq!(Empty::resolve(""), None);
    assert_eq!(Empty8::resolve(""), None);
}

#[test]
fn utf8_paths_empty_and_const_access() {
    assert_eq!(CONST_TEXT, "μs 🦀");
    assert_eq!(Text::PATH.as_str().as_bytes(), b"a/b");
    assert_eq!(Text::EMPTY.get(), "");
    assert_eq!(CONST_EMPTY, "");
    assert_eq!(Text::SCREEN.as_ref(), "screen");
    let borrowed: &str = &Text::AUDIO;
    assert_eq!(borrowed, "audio");
    let static_text: &'static str = Text::UNICODE.into();
    assert_eq!(static_text, CONST_TEXT);
    assert_eq!(format!("{}", Text::PATH), "a/b");
    assert_eq!(format!("{:?}", Text::PATH), "\"a/b\"");
}

#[test]
fn exact_deduplication_and_declaration_iteration() {
    assert_eq!(Text::AUDIO, Text::ALIAS);
    assert_eq!(Text::EMPTY, Text::EMPTY_ALIAS);
    assert_eq!(Text::AUDIO.as_str().as_ptr(), Text::ALIAS.as_str().as_ptr());
    assert_eq!(Text::len(), 7);
    assert_eq!(Text::unique_len(), 5);
    assert_eq!(Text::storage_bytes(), 1 + 5 + 6 + 5 + "μs 🦀".len() + 3);
    assert_eq!(
        Text::all(),
        &[
            Text::SCREEN,
            Text::AUDIO,
            Text::UNICODE,
            Text::PATH,
            Text::EMPTY,
            Text::ALIAS,
            Text::EMPTY_ALIAS,
        ]
    );
    let unique: HashSet<_> = Text::all().iter().copied().collect();
    assert_eq!(unique.len(), 5);
    assert_eq!(CONST_LOOKUP, Some(Text::AUDIO));
    assert_eq!(CONST_DECODED, Some(Text::AUDIO));
    assert_eq!(Text::lookup("missing"), None);
    assert_eq!(Text::lookup("a"), None); // A prefix isn't a declared string.
}

#[test]
fn lookup_requires_exact_contents_and_real_record_boundaries() {
    assert_eq!(Text::lookup(""), Some(Text::EMPTY));
    assert_eq!(Text8::lookup(""), Some(Text8::EMPTY));
    assert_eq!(Text::lookup("a/b"), Some(Text::PATH));
    assert_eq!(Text8::lookup("a/b"), Some(Text8::PATH));
    for query in ["a", "b", "audio/", "screenaudio", "missing"] {
        assert_eq!(Text::lookup(query), None, "query={query:?}");
        assert_eq!(Text8::lookup(query), None, "query={query:?}");
    }

    // The input query remains an ordinary Rust str. A NUL inside the query
    // must never turn it into a match for an existing prefix or empty record.
    for query in ["\0", "audio\0", "audio\0extra", "\0audio", "a\0b", "μs 🦀\0"] {
        assert_eq!(Text::lookup(query), None, "query={query:?}");
        assert_eq!(Text8::lookup(query), None, "query={query:?}");
    }
    assert_eq!(CONST_NUL_LOOKUP, None);
    assert_eq!(CONST_NUL_LOOKUP8, None);
    assert_eq!(Empty::lookup("\0"), None);
    assert_eq!(Empty8::lookup("\0"), None);
}

#[test]
fn every_raw_u16_is_checked_against_real_record_offsets() {
    for raw in 0..=u16::MAX {
        let expected = Text::all().iter().copied().find(|id| id.to_raw() == raw);
        let decoded = Text::from_raw(raw);
        assert_eq!(decoded, expected, "raw={raw}");
        if let Some(id) = decoded {
            assert_eq!(Text::lookup(id.as_str()), Some(id));
        }
    }
}

#[test]
fn compact_layout_and_empty_pool() {
    assert_eq!(size_of::<Text>(), 2);
    assert_eq!(size_of::<Option<Text>>(), 2);
    assert_eq!(align_of::<Text>(), align_of::<u16>());
    assert_eq!(size_of::<[Text; 100]>(), 200);
    assert!(Empty::is_empty());
    assert_eq!(Empty::len(), 0);
    assert_eq!(Empty::unique_len(), 0);
    assert_eq!(Empty::storage_bytes(), 1);
    assert_eq!(Empty::index_bytes(), 0);
    assert_eq!(Empty::total_storage_bytes(), 1);
    assert_eq!(Empty::all(), &[]);
    assert_eq!(Empty::lookup(""), None);
    assert_eq!(Empty::from_raw(1), None);
    assert_eq!(OtherText::AUDIO.as_str(), "audio");
}

#[test]
fn u16_raw_values_are_offsets_and_index_recovers_the_dense_ordinal() {
    let ids = [Text::SCREEN, Text::AUDIO, Text::UNICODE, Text::PATH, Text::EMPTY];
    let offsets = [1u16, 8, 14, 23, 27];
    for (index, id) in ids.into_iter().enumerate() {
        assert_eq!(id.index(), index);
        assert_eq!(id.to_raw(), offsets[index]);
        assert_eq!(id.offset(), offsets[index]);
    }
    assert_eq!(Text::index_bytes(), 0);
    assert_eq!(Text::total_storage_bytes(), Text::storage_bytes());
    assert_eq!(Text::from_raw(Text::AUDIO.offset()), Some(Text::AUDIO));
    assert_eq!(Text::from_raw(2), None); // Dense ID 2 is inside "screen".
    assert_eq!(Text::ALIAS.index(), Text::AUDIO.index());
    let _: u16 = Explicit16::AUDIO.to_raw();
    assert_eq!(Explicit16::AUDIO.to_raw(), Text::AUDIO.to_raw());
    assert_eq!(Explicit16::AUDIO.offset(), Text::AUDIO.offset());
}

#[test]
fn u16_validation_rejects_interior_bytes_terminators_and_unused_space() {
    // The Unicode record begins at 14. These offsets include both a
    // continuation byte of μ and the interior of the four-byte emoji.
    for raw in [0, 2, 7, 9, 13, 15, 18, 19, 20, 21, 22, 24, 26, 28, u16::MAX] {
        assert_eq!(Text::from_raw(raw), None, "raw={raw}");
    }
    for (raw, expected) in [
        (1, Text::SCREEN),
        (8, Text::AUDIO),
        (14, Text::UNICODE),
        (23, Text::PATH),
        (27, Text::EMPTY),
    ] {
        assert_eq!(Text::from_raw(raw), Some(expected));
    }
    assert_eq!(Text::lookup(""), Some(Text::EMPTY));
    assert_eq!(Text::EMPTY.to_raw(), 27);
    assert_eq!(Text::EMPTY.index(), 4);
}

#[test]
fn empty_record_is_distinct_from_the_reserved_prefix() {
    assert_eq!(EmptyOnly::storage_bytes(), 2);
    assert_eq!(EmptyOnly::index_bytes(), 0);
    assert_eq!(EmptyOnly::unique_len(), 1);
    assert_eq!(EmptyOnly::EMPTY.to_raw(), 1);
    assert_eq!(EmptyOnly::EMPTY.offset(), 1);
    assert_eq!(EmptyOnly::EMPTY.index(), 0);
    assert_eq!(EmptyOnly::EMPTY, EmptyOnly::ALIAS);
    assert_eq!(EmptyOnly::lookup(""), Some(EmptyOnly::EMPTY));
    assert_eq!(EmptyOnly::from_raw(0), None);
    assert_eq!(EmptyOnly::from_raw(1), Some(EmptyOnly::EMPTY));
    assert_eq!(EmptyOnly::from_raw(2), None);

    // Prefix NUL, empty-record NUL, then the first UTF-8 byte: both offsets
    // 1 and 2 are record starts, while the terminators after text are not.
    assert_eq!(LeadingEmpty::storage_bytes(), 11);
    assert_eq!(LeadingEmpty::EMPTY.to_raw(), 1);
    assert_eq!(LeadingEmpty::UTF8.to_raw(), 2);
    assert_eq!(LeadingEmpty::WORD.to_raw(), 9);
    assert_eq!(LeadingEmpty::EMPTY, LeadingEmpty::EMPTY_ALIAS);
    assert_eq!(LeadingEmpty::UTF8.as_str(), "é🙂");
    assert_eq!(LeadingEmpty::EMPTY.index(), 0);
    assert_eq!(LeadingEmpty::UTF8.index(), 1);
    assert_eq!(LeadingEmpty::WORD.index(), 2);
    for raw in 0..=12 {
        let expected = match raw {
            1 => Some(LeadingEmpty::EMPTY),
            2 => Some(LeadingEmpty::UTF8),
            9 => Some(LeadingEmpty::WORD),
            _ => None,
        };
        assert_eq!(LeadingEmpty::from_raw(raw), expected, "raw={raw}");
    }
    assert_eq!(LeadingEmpty::lookup(""), Some(LeadingEmpty::EMPTY));
    assert_eq!(LeadingEmpty::lookup("é🙂"), Some(LeadingEmpty::UTF8));
    assert_eq!(LeadingEmpty::lookup("x"), Some(LeadingEmpty::WORD));
    assert_eq!(LeadingEmpty::lookup("missing"), None);
}

#[test]
fn u8_width_preserves_contents_dedup_and_const_access() {
    assert_eq!(CONST_TEXT8, CONST_TEXT);
    assert_eq!(CONST_LOOKUP8, Some(Text8::AUDIO));
    assert_eq!(CONST_DECODED8, Some(Text8::AUDIO));
    assert_eq!(Text8::AUDIO, Text8::ALIAS);
    assert_eq!(Text8::EMPTY, Text8::EMPTY_ALIAS);
    assert_eq!(Text8::PATH.as_str().as_bytes(), b"a/b");
    assert_eq!(Text8::all().len(), 7);
    assert_eq!(Text8::unique_len(), 5);
    assert_eq!(Text8::storage_bytes() + 1, Text::storage_bytes());
    assert_eq!(Text8::index_bytes(), 10);
    assert_eq!(Text::index_bytes(), 0);
    for (&small, &wide) in Text8::all().iter().zip(Text::all()) {
        assert_eq!(small.to_raw() as usize, small.index() + 1);
        assert_eq!(small.index(), wide.index());
        assert_eq!(small.offset() + 1, wide.offset());
        assert_eq!(wide.to_raw(), wide.offset());
        assert_eq!(small.get(), wide.get());
    }
    let _: u8 = Text8::AUDIO.to_raw();
    let _: u16 = Text8::AUDIO.offset();
    assert_eq!(size_of::<Text8>(), 1);
    assert_eq!(size_of::<Option<Text8>>(), 1);
    assert_eq!(align_of::<Text8>(), 1);
    assert_eq!(size_of::<[Text8; 100]>(), 100);
    assert_eq!(size_of::<Option<Empty8>>(), 1);
    assert!(Empty8::is_empty());
    assert_eq!(Empty8::all(), &[]);
    assert_eq!(Empty8::total_storage_bytes(), 0);
}

#[test]
fn every_raw_u8_is_checked_and_roundtrips() {
    for raw in 0..=u8::MAX {
        let expected = Text8::all().iter().copied().find(|id| id.to_raw() == raw);
        let decoded = Text8::from_raw(raw);
        assert_eq!(decoded, expected, "raw={raw}");
        if let Some(id) = decoded {
            assert_eq!(Text8::lookup(id.as_str()), Some(id));
        }
        assert_eq!(Empty8::from_raw(raw), None);
    }
    assert_eq!(Text8::lookup("missing"), None);
    assert_eq!(Text8::lookup("a"), None);
    assert_eq!(Empty8::lookup(""), None);
}

// Input expressions must retain the caller's lexical bindings even when
// they have the same names as the implementation's helper items.
mod hygiene {
    #![allow(non_upper_case_globals)]
    use catplay_reflect::static_strings;

    const __COUNT: &str = "count";
    const __INPUT: &str = "input";
    const __LAYOUT: &str = "layout";
    const __BYTES: &str = "bytes";
    const __OFFSETS: &str = "offsets";
    const __Ordinal: &str = "ordinal";

    static_strings! {
        pub struct Names(u8) {
            COUNT = __COUNT,
            INPUT = __INPUT,
            LAYOUT = __LAYOUT,
            BYTES = __BYTES,
            OFFSETS = __OFFSETS,
            ORDINAL = __Ordinal,
            ALL = "all",
            SIZE = "size",
        }
    }
}

#[test]
fn caller_names_are_not_captured() {
    assert_eq!(hygiene::Names::COUNT.as_str(), "count");
    assert_eq!(hygiene::Names::INPUT.as_str(), "input");
    assert_eq!(hygiene::Names::LAYOUT.as_str(), "layout");
    assert_eq!(hygiene::Names::BYTES.as_str(), "bytes");
    assert_eq!(hygiene::Names::OFFSETS.as_str(), "offsets");
    assert_eq!(hygiene::Names::ORDINAL.as_str(), "ordinal");
    assert_eq!(hygiene::Names::ALL.as_str(), "all");
}

#[test]
#[allow(non_camel_case_types)]
fn handle_names_may_match_helpers_and_local_aliases() {
    const TEXT: &str = "μs/";

    macro_rules! check {
        ($name:ident, $width:ident) => {{
            static_strings! {
                struct $name($width) {
                    FIRST = TEXT,
                    ALIAS = TEXT,
                    OTHER = "other",
                }
            }

            const RESOLVED: &str = $name::ALIAS.get();
            let raw: $width = $name::FIRST.to_raw();
            assert_eq!(RESOLVED, TEXT);
            assert_eq!($name::FIRST, $name::ALIAS);
            assert_eq!($name::lookup(TEXT), Some($name::FIRST));
            assert_eq!($name::from_raw(raw), Some($name::FIRST));
            assert_eq!($name::all(), &[$name::FIRST, $name::ALIAS, $name::OTHER]);
            assert_eq!(<$name as catplay_reflect::StaticHandle>::get($name::FIRST), TEXT,);
            assert_eq!(size_of::<Option<$name>>(), size_of::<$width>());
        }};
    }

    check!(__COUNT, u8);
    check!(__INPUT, u16);
    check!(__HASH_SLOTS, u8);
    check!(__LAYOUT, u16);
    check!(__BYTE_LEN, u8);
    check!(__BYTES, u16);
    check!(__OFFSETS, u8);
    check!(__Ordinal, u16);
    check!(__ALL, u8);
    check!(__StaticPoolsLocal, u8);
    check!(__StaticPoolsAlternate, u16);
}
