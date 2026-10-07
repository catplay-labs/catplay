use catplay_reflect::{static_objects, static_strings};
use core::mem::{align_of, size_of};

// A byte-sized string handle indexes records, so even this entire 64 KiB
// payload is addressable. The empty record is the second distinct string.
const LONG_BYTES: [u8; 65_534] = [b'x'; 65_534];
const LONG: &str = match core::str::from_utf8(&LONG_BYTES) {
    Ok(text) => text,
    Err(_) => panic!("ASCII fixture is valid UTF-8"),
};

static_strings! {
    struct Full(u8) {
        LONG = LONG,
        END = "",
        ALIAS = LONG,
    }
}

#[test]
fn byte_handle_reaches_the_end_of_a_full_64_kib_payload() {
    assert_eq!(size_of::<Full>(), 1);
    assert_eq!(size_of::<Option<Full>>(), 1);
    assert_eq!(Full::storage_bytes(), 65_536);
    assert_eq!(Full::index_bytes(), 4);
    assert_eq!(Full::total_storage_bytes(), 65_540);
    assert_eq!(Full::unique_len(), 2);
    assert_eq!(Full::LONG, Full::ALIAS);
    assert_eq!(Full::LONG.as_str(), LONG);
    assert_eq!(Full::END.offset(), u16::MAX);
    assert_eq!(Full::END.to_raw(), 2_u8);
    assert_eq!(Full::from_raw(2), Some(Full::END));
    assert_eq!(Full::END.as_str(), "");
    assert_eq!(Full::from_raw(0), None);
    assert_eq!(Full::from_raw(3), None);
    assert_eq!(Full::from_raw(u8::MAX), None);
    assert_eq!(Full::lookup(""), Some(Full::END));
    assert_eq!(Full::lookup(LONG), Some(Full::LONG));
    assert_eq!(Full::lookup("x"), None);
    assert_eq!(Full::lookup("missing"), None);
    assert_eq!(Full::lookup("\0"), None);
}

// One maximum-length string puts its terminating NUL at byte 65,535.
// There is no room left for another distinct record, including an empty one.
const MAXIMUM_BYTES: [u8; 65_535] = [b'm'; 65_535];
const MAXIMUM: &str = match core::str::from_utf8(&MAXIMUM_BYTES) {
    Ok(text) => text,
    Err(_) => panic!("ASCII fixture is valid UTF-8"),
};

static_strings! {
    struct MaximumSingle(u8) {
        VALUE = MAXIMUM,
        ALIAS = MAXIMUM,
    }
}

const MAXIMUM_CONST_TEXT: &str = MaximumSingle::VALUE.as_str();

#[test]
fn u8_maximum_single_string_uses_the_final_byte_for_its_terminator() {
    assert_eq!(MaximumSingle::VALUE.as_str().len(), 65_535);
    assert_eq!(MaximumSingle::VALUE.offset(), 0);
    assert_eq!(MaximumSingle::VALUE.to_raw(), 1_u8);
    assert_eq!(MaximumSingle::VALUE, MaximumSingle::ALIAS);
    assert_eq!(MaximumSingle::unique_len(), 1);
    assert_eq!(MaximumSingle::storage_bytes(), 65_536);
    assert_eq!(MaximumSingle::index_bytes(), 2);
    assert_eq!(MaximumSingle::total_storage_bytes(), 65_538);
    assert_eq!(MAXIMUM_CONST_TEXT, MAXIMUM);
    assert_eq!(MaximumSingle::lookup(MAXIMUM), Some(MaximumSingle::VALUE));
    assert_eq!(MaximumSingle::lookup(""), None);
    assert_eq!(MaximumSingle::lookup("m"), None);
    assert_eq!(MaximumSingle::lookup("missing"), None);
    assert_eq!(MaximumSingle::lookup("\0"), None);
    assert_eq!(MaximumSingle::from_raw(2), None);
}

// A direct-offset pool reserves byte zero. Its longest individual string
// is therefore one byte shorter than the maximum u8-pool string above.
static_strings! {
    struct MaximumWide {
        VALUE = LONG,
        ALIAS = LONG,
    }
}

const MAXIMUM_WIDE_TEXT: &str = MaximumWide::VALUE.as_str();

#[test]
fn u16_maximum_single_string_includes_the_reserved_prefix() {
    assert_eq!(MaximumWide::VALUE.as_str().len(), 65_534);
    assert_eq!(MaximumWide::VALUE.to_raw(), 1_u16);
    assert_eq!(MaximumWide::VALUE.offset(), 1);
    assert_eq!(MaximumWide::VALUE.index(), 0);
    assert_eq!(MaximumWide::VALUE, MaximumWide::ALIAS);
    assert_eq!(MaximumWide::unique_len(), 1);
    assert_eq!(MaximumWide::storage_bytes(), 65_536);
    assert_eq!(MaximumWide::index_bytes(), 0);
    assert_eq!(MaximumWide::total_storage_bytes(), 65_536);
    assert_eq!(MAXIMUM_WIDE_TEXT, LONG);
    assert_eq!(MaximumWide::lookup(LONG), Some(MaximumWide::VALUE));
    assert_eq!(MaximumWide::lookup(""), None);
    assert_eq!(MaximumWide::lookup("missing"), None);
    assert_eq!(MaximumWide::from_raw(0), None);
    assert_eq!(MaximumWide::from_raw(1), Some(MaximumWide::VALUE));
    assert_eq!(MaximumWide::from_raw(2), None);
    assert_eq!(MaximumWide::from_raw(u16::MAX), None); // Final terminator.
}

const WIDE_LONG: &str = match core::str::from_utf8(LONG_BYTES.split_at(65_533).0) {
    Ok(text) => text,
    Err(_) => panic!("ASCII fixture is valid UTF-8"),
};

static_strings! {
    struct FullWide(u16) {
        LONG = WIDE_LONG,
        END = "",
        ALIAS = WIDE_LONG,
    }
}

const WIDE_LAST_RAW: u16 = FullWide::END.to_raw();
const WIDE_LAST_INDEX: usize = FullWide::END.index();

#[test]
fn direct_u16_offset_reaches_the_last_empty_record_at_65535() {
    assert_eq!(FullWide::LONG.to_raw(), 1);
    assert_eq!(FullWide::LONG, FullWide::ALIAS);
    assert_eq!(FullWide::LONG.as_str(), WIDE_LONG);
    assert_eq!(FullWide::storage_bytes(), 65_536);
    assert_eq!(FullWide::index_bytes(), 0);
    assert_eq!(FullWide::total_storage_bytes(), 65_536);
    assert_eq!(FullWide::unique_len(), 2);
    assert_eq!(WIDE_LAST_RAW, u16::MAX);
    assert_eq!(FullWide::END.offset(), u16::MAX);
    assert_eq!(WIDE_LAST_INDEX, 1);
    assert_eq!(FullWide::END.as_str(), "");
    assert_eq!(FullWide::from_raw(u16::MAX), Some(FullWide::END));
    assert_eq!(FullWide::lookup(""), Some(FullWide::END));
    assert_eq!(FullWide::lookup(WIDE_LONG), Some(FullWide::LONG));
    assert_eq!(FullWide::lookup("missing"), None);
    assert_eq!(FullWide::lookup("\0"), None);
    for raw in [0, 2, 65_533, 65_534] {
        assert_eq!(FullWide::from_raw(raw), None, "raw={raw}");
    }
}

// Keep one readable entry list for three related boundary cases. Expansions
// below contain 255 unique strings for u8 and 256 strings/objects for u16.
macro_rules! boundary_pools {
    ($($entry:ident = $number:expr),* $(,)?) => {
        static_strings! {
            struct NarrowStrings(u8) {
                $($entry = stringify!($number),)*
            }
        }
        static_strings! {
            struct WideStrings(u16) {
                $($entry = stringify!($number),)*
                E255 = "255",
            }
        }
        static_objects! {
            struct WideObjects(u16): u16 {
                $($entry = $number,)*
                E255 = 255,
            }
        }
    };
}

boundary_pools! {
    E000 = 0,
    E001 = 1,
    E002 = 2,
    E003 = 3,
    E004 = 4,
    E005 = 5,
    E006 = 6,
    E007 = 7,
    E008 = 8,
    E009 = 9,
    E010 = 10,
    E011 = 11,
    E012 = 12,
    E013 = 13,
    E014 = 14,
    E015 = 15,
    E016 = 16,
    E017 = 17,
    E018 = 18,
    E019 = 19,
    E020 = 20,
    E021 = 21,
    E022 = 22,
    E023 = 23,
    E024 = 24,
    E025 = 25,
    E026 = 26,
    E027 = 27,
    E028 = 28,
    E029 = 29,
    E030 = 30,
    E031 = 31,
    E032 = 32,
    E033 = 33,
    E034 = 34,
    E035 = 35,
    E036 = 36,
    E037 = 37,
    E038 = 38,
    E039 = 39,
    E040 = 40,
    E041 = 41,
    E042 = 42,
    E043 = 43,
    E044 = 44,
    E045 = 45,
    E046 = 46,
    E047 = 47,
    E048 = 48,
    E049 = 49,
    E050 = 50,
    E051 = 51,
    E052 = 52,
    E053 = 53,
    E054 = 54,
    E055 = 55,
    E056 = 56,
    E057 = 57,
    E058 = 58,
    E059 = 59,
    E060 = 60,
    E061 = 61,
    E062 = 62,
    E063 = 63,
    E064 = 64,
    E065 = 65,
    E066 = 66,
    E067 = 67,
    E068 = 68,
    E069 = 69,
    E070 = 70,
    E071 = 71,
    E072 = 72,
    E073 = 73,
    E074 = 74,
    E075 = 75,
    E076 = 76,
    E077 = 77,
    E078 = 78,
    E079 = 79,
    E080 = 80,
    E081 = 81,
    E082 = 82,
    E083 = 83,
    E084 = 84,
    E085 = 85,
    E086 = 86,
    E087 = 87,
    E088 = 88,
    E089 = 89,
    E090 = 90,
    E091 = 91,
    E092 = 92,
    E093 = 93,
    E094 = 94,
    E095 = 95,
    E096 = 96,
    E097 = 97,
    E098 = 98,
    E099 = 99,
    E100 = 100,
    E101 = 101,
    E102 = 102,
    E103 = 103,
    E104 = 104,
    E105 = 105,
    E106 = 106,
    E107 = 107,
    E108 = 108,
    E109 = 109,
    E110 = 110,
    E111 = 111,
    E112 = 112,
    E113 = 113,
    E114 = 114,
    E115 = 115,
    E116 = 116,
    E117 = 117,
    E118 = 118,
    E119 = 119,
    E120 = 120,
    E121 = 121,
    E122 = 122,
    E123 = 123,
    E124 = 124,
    E125 = 125,
    E126 = 126,
    E127 = 127,
    E128 = 128,
    E129 = 129,
    E130 = 130,
    E131 = 131,
    E132 = 132,
    E133 = 133,
    E134 = 134,
    E135 = 135,
    E136 = 136,
    E137 = 137,
    E138 = 138,
    E139 = 139,
    E140 = 140,
    E141 = 141,
    E142 = 142,
    E143 = 143,
    E144 = 144,
    E145 = 145,
    E146 = 146,
    E147 = 147,
    E148 = 148,
    E149 = 149,
    E150 = 150,
    E151 = 151,
    E152 = 152,
    E153 = 153,
    E154 = 154,
    E155 = 155,
    E156 = 156,
    E157 = 157,
    E158 = 158,
    E159 = 159,
    E160 = 160,
    E161 = 161,
    E162 = 162,
    E163 = 163,
    E164 = 164,
    E165 = 165,
    E166 = 166,
    E167 = 167,
    E168 = 168,
    E169 = 169,
    E170 = 170,
    E171 = 171,
    E172 = 172,
    E173 = 173,
    E174 = 174,
    E175 = 175,
    E176 = 176,
    E177 = 177,
    E178 = 178,
    E179 = 179,
    E180 = 180,
    E181 = 181,
    E182 = 182,
    E183 = 183,
    E184 = 184,
    E185 = 185,
    E186 = 186,
    E187 = 187,
    E188 = 188,
    E189 = 189,
    E190 = 190,
    E191 = 191,
    E192 = 192,
    E193 = 193,
    E194 = 194,
    E195 = 195,
    E196 = 196,
    E197 = 197,
    E198 = 198,
    E199 = 199,
    E200 = 200,
    E201 = 201,
    E202 = 202,
    E203 = 203,
    E204 = 204,
    E205 = 205,
    E206 = 206,
    E207 = 207,
    E208 = 208,
    E209 = 209,
    E210 = 210,
    E211 = 211,
    E212 = 212,
    E213 = 213,
    E214 = 214,
    E215 = 215,
    E216 = 216,
    E217 = 217,
    E218 = 218,
    E219 = 219,
    E220 = 220,
    E221 = 221,
    E222 = 222,
    E223 = 223,
    E224 = 224,
    E225 = 225,
    E226 = 226,
    E227 = 227,
    E228 = 228,
    E229 = 229,
    E230 = 230,
    E231 = 231,
    E232 = 232,
    E233 = 233,
    E234 = 234,
    E235 = 235,
    E236 = 236,
    E237 = 237,
    E238 = 238,
    E239 = 239,
    E240 = 240,
    E241 = 241,
    E242 = 242,
    E243 = 243,
    E244 = 244,
    E245 = 245,
    E246 = 246,
    E247 = 247,
    E248 = 248,
    E249 = 249,
    E250 = 250,
    E251 = 251,
    E252 = 252,
    E253 = 253,
    E254 = 254,
}

const NARROW_LAST: u8 = NarrowStrings::E254.to_raw();
const WIDE_STRING_LAST: u16 = WideStrings::E255.to_raw();
const WIDE_OBJECT_LAST: u16 = WideObjects::E255.to_raw();

#[test]
fn byte_handle_holds_exactly_255_unique_strings() {
    assert_eq!(size_of::<NarrowStrings>(), 1);
    assert_eq!(size_of::<Option<NarrowStrings>>(), 1);
    assert_eq!(align_of::<NarrowStrings>(), align_of::<u8>());
    assert_eq!(NarrowStrings::len(), 255);
    assert_eq!(NarrowStrings::unique_len(), 255);
    assert_eq!(NARROW_LAST, u8::MAX);
    assert_eq!(NarrowStrings::from_raw(0), None);
    assert_eq!(NarrowStrings::from_raw(u8::MAX), Some(NarrowStrings::E254));
    assert_eq!(NarrowStrings::E254.as_str(), "254");
    assert_eq!(NarrowStrings::index_bytes(), 510);
    assert!(NarrowStrings::storage_bytes() > 255);
    for (index, &handle) in NarrowStrings::all().iter().enumerate() {
        let raw = (index + 1) as u8;
        assert_eq!(handle.to_raw(), raw);
        assert_eq!(NarrowStrings::from_raw(raw), Some(handle));
        assert_eq!(NarrowStrings::lookup(handle.as_str()), Some(handle));
    }
}

#[test]
fn u16_pools_support_more_than_255_entries_with_their_respective_encodings() {
    assert_eq!(size_of::<WideStrings>(), 2);
    assert_eq!(size_of::<Option<WideStrings>>(), 2);
    assert_eq!(WideStrings::len(), 256);
    assert_eq!(WideStrings::unique_len(), 256);
    // String raw values are byte offsets, while index() recovers ordinal.
    assert_eq!(WideStrings::E254.index(), 254);
    assert_eq!(WideStrings::E255.index(), 255);
    assert_eq!(WideStrings::E254.to_raw(), 907);
    assert_eq!(WIDE_STRING_LAST, 911);
    assert_eq!(WideStrings::from_raw(911), Some(WideStrings::E255));
    assert_eq!(WideStrings::from_raw(912), None);
    assert_eq!(WideStrings::from_raw(915), None);
    assert_eq!(WideStrings::lookup("255"), Some(WideStrings::E255));
    assert_eq!(WideStrings::E255.as_str(), "255");
    assert_eq!(WideStrings::index_bytes(), 0);
    assert_eq!(WideStrings::storage_bytes(), 915);

    assert_eq!(size_of::<WideObjects>(), 2);
    assert_eq!(size_of::<Option<WideObjects>>(), 2);
    assert_eq!(WideObjects::len(), 256);
    assert_eq!(WideObjects::E254.to_raw(), 255);
    assert_eq!(WIDE_OBJECT_LAST, 256);
    assert_eq!(WideObjects::from_raw(256), Some(WideObjects::E255));
    assert_eq!(WideObjects::from_raw(257), None);
    assert_eq!(*WideObjects::E255.get(), 255);
}

static_strings! {
    struct ManyAliases(u8) {
        ALIAS000 = "shared",
        ALIAS001 = "shared",
        ALIAS002 = "shared",
        ALIAS003 = "shared",
        ALIAS004 = "shared",
        ALIAS005 = "shared",
        ALIAS006 = "shared",
        ALIAS007 = "shared",
        ALIAS008 = "shared",
        ALIAS009 = "shared",
        ALIAS010 = "shared",
        ALIAS011 = "shared",
        ALIAS012 = "shared",
        ALIAS013 = "shared",
        ALIAS014 = "shared",
        ALIAS015 = "shared",
        ALIAS016 = "shared",
        ALIAS017 = "shared",
        ALIAS018 = "shared",
        ALIAS019 = "shared",
        ALIAS020 = "shared",
        ALIAS021 = "shared",
        ALIAS022 = "shared",
        ALIAS023 = "shared",
        ALIAS024 = "shared",
        ALIAS025 = "shared",
        ALIAS026 = "shared",
        ALIAS027 = "shared",
        ALIAS028 = "shared",
        ALIAS029 = "shared",
        ALIAS030 = "shared",
        ALIAS031 = "shared",
        ALIAS032 = "shared",
        ALIAS033 = "shared",
        ALIAS034 = "shared",
        ALIAS035 = "shared",
        ALIAS036 = "shared",
        ALIAS037 = "shared",
        ALIAS038 = "shared",
        ALIAS039 = "shared",
        ALIAS040 = "shared",
        ALIAS041 = "shared",
        ALIAS042 = "shared",
        ALIAS043 = "shared",
        ALIAS044 = "shared",
        ALIAS045 = "shared",
        ALIAS046 = "shared",
        ALIAS047 = "shared",
        ALIAS048 = "shared",
        ALIAS049 = "shared",
        ALIAS050 = "shared",
        ALIAS051 = "shared",
        ALIAS052 = "shared",
        ALIAS053 = "shared",
        ALIAS054 = "shared",
        ALIAS055 = "shared",
        ALIAS056 = "shared",
        ALIAS057 = "shared",
        ALIAS058 = "shared",
        ALIAS059 = "shared",
        ALIAS060 = "shared",
        ALIAS061 = "shared",
        ALIAS062 = "shared",
        ALIAS063 = "shared",
        ALIAS064 = "shared",
        ALIAS065 = "shared",
        ALIAS066 = "shared",
        ALIAS067 = "shared",
        ALIAS068 = "shared",
        ALIAS069 = "shared",
        ALIAS070 = "shared",
        ALIAS071 = "shared",
        ALIAS072 = "shared",
        ALIAS073 = "shared",
        ALIAS074 = "shared",
        ALIAS075 = "shared",
        ALIAS076 = "shared",
        ALIAS077 = "shared",
        ALIAS078 = "shared",
        ALIAS079 = "shared",
        ALIAS080 = "shared",
        ALIAS081 = "shared",
        ALIAS082 = "shared",
        ALIAS083 = "shared",
        ALIAS084 = "shared",
        ALIAS085 = "shared",
        ALIAS086 = "shared",
        ALIAS087 = "shared",
        ALIAS088 = "shared",
        ALIAS089 = "shared",
        ALIAS090 = "shared",
        ALIAS091 = "shared",
        ALIAS092 = "shared",
        ALIAS093 = "shared",
        ALIAS094 = "shared",
        ALIAS095 = "shared",
        ALIAS096 = "shared",
        ALIAS097 = "shared",
        ALIAS098 = "shared",
        ALIAS099 = "shared",
        ALIAS100 = "shared",
        ALIAS101 = "shared",
        ALIAS102 = "shared",
        ALIAS103 = "shared",
        ALIAS104 = "shared",
        ALIAS105 = "shared",
        ALIAS106 = "shared",
        ALIAS107 = "shared",
        ALIAS108 = "shared",
        ALIAS109 = "shared",
        ALIAS110 = "shared",
        ALIAS111 = "shared",
        ALIAS112 = "shared",
        ALIAS113 = "shared",
        ALIAS114 = "shared",
        ALIAS115 = "shared",
        ALIAS116 = "shared",
        ALIAS117 = "shared",
        ALIAS118 = "shared",
        ALIAS119 = "shared",
        ALIAS120 = "shared",
        ALIAS121 = "shared",
        ALIAS122 = "shared",
        ALIAS123 = "shared",
        ALIAS124 = "shared",
        ALIAS125 = "shared",
        ALIAS126 = "shared",
        ALIAS127 = "shared",
        ALIAS128 = "shared",
        ALIAS129 = "shared",
        ALIAS130 = "shared",
        ALIAS131 = "shared",
        ALIAS132 = "shared",
        ALIAS133 = "shared",
        ALIAS134 = "shared",
        ALIAS135 = "shared",
        ALIAS136 = "shared",
        ALIAS137 = "shared",
        ALIAS138 = "shared",
        ALIAS139 = "shared",
        ALIAS140 = "shared",
        ALIAS141 = "shared",
        ALIAS142 = "shared",
        ALIAS143 = "shared",
        ALIAS144 = "shared",
        ALIAS145 = "shared",
        ALIAS146 = "shared",
        ALIAS147 = "shared",
        ALIAS148 = "shared",
        ALIAS149 = "shared",
        ALIAS150 = "shared",
        ALIAS151 = "shared",
        ALIAS152 = "shared",
        ALIAS153 = "shared",
        ALIAS154 = "shared",
        ALIAS155 = "shared",
        ALIAS156 = "shared",
        ALIAS157 = "shared",
        ALIAS158 = "shared",
        ALIAS159 = "shared",
        ALIAS160 = "shared",
        ALIAS161 = "shared",
        ALIAS162 = "shared",
        ALIAS163 = "shared",
        ALIAS164 = "shared",
        ALIAS165 = "shared",
        ALIAS166 = "shared",
        ALIAS167 = "shared",
        ALIAS168 = "shared",
        ALIAS169 = "shared",
        ALIAS170 = "shared",
        ALIAS171 = "shared",
        ALIAS172 = "shared",
        ALIAS173 = "shared",
        ALIAS174 = "shared",
        ALIAS175 = "shared",
        ALIAS176 = "shared",
        ALIAS177 = "shared",
        ALIAS178 = "shared",
        ALIAS179 = "shared",
        ALIAS180 = "shared",
        ALIAS181 = "shared",
        ALIAS182 = "shared",
        ALIAS183 = "shared",
        ALIAS184 = "shared",
        ALIAS185 = "shared",
        ALIAS186 = "shared",
        ALIAS187 = "shared",
        ALIAS188 = "shared",
        ALIAS189 = "shared",
        ALIAS190 = "shared",
        ALIAS191 = "shared",
        ALIAS192 = "shared",
        ALIAS193 = "shared",
        ALIAS194 = "shared",
        ALIAS195 = "shared",
        ALIAS196 = "shared",
        ALIAS197 = "shared",
        ALIAS198 = "shared",
        ALIAS199 = "shared",
        ALIAS200 = "shared",
        ALIAS201 = "shared",
        ALIAS202 = "shared",
        ALIAS203 = "shared",
        ALIAS204 = "shared",
        ALIAS205 = "shared",
        ALIAS206 = "shared",
        ALIAS207 = "shared",
        ALIAS208 = "shared",
        ALIAS209 = "shared",
        ALIAS210 = "shared",
        ALIAS211 = "shared",
        ALIAS212 = "shared",
        ALIAS213 = "shared",
        ALIAS214 = "shared",
        ALIAS215 = "shared",
        ALIAS216 = "shared",
        ALIAS217 = "shared",
        ALIAS218 = "shared",
        ALIAS219 = "shared",
        ALIAS220 = "shared",
        ALIAS221 = "shared",
        ALIAS222 = "shared",
        ALIAS223 = "shared",
        ALIAS224 = "shared",
        ALIAS225 = "shared",
        ALIAS226 = "shared",
        ALIAS227 = "shared",
        ALIAS228 = "shared",
        ALIAS229 = "shared",
        ALIAS230 = "shared",
        ALIAS231 = "shared",
        ALIAS232 = "shared",
        ALIAS233 = "shared",
        ALIAS234 = "shared",
        ALIAS235 = "shared",
        ALIAS236 = "shared",
        ALIAS237 = "shared",
        ALIAS238 = "shared",
        ALIAS239 = "shared",
        ALIAS240 = "shared",
        ALIAS241 = "shared",
        ALIAS242 = "shared",
        ALIAS243 = "shared",
        ALIAS244 = "shared",
        ALIAS245 = "shared",
        ALIAS246 = "shared",
        ALIAS247 = "shared",
        ALIAS248 = "shared",
        ALIAS249 = "shared",
        ALIAS250 = "shared",
        ALIAS251 = "shared",
        ALIAS252 = "shared",
        ALIAS253 = "shared",
        ALIAS254 = "shared",
        ALIAS255 = "shared",
        ALIAS256 = "shared",
        ALIAS257 = "shared",
        ALIAS258 = "shared",
        ALIAS259 = "shared",
        ALIAS260 = "shared",
        ALIAS261 = "shared",
        ALIAS262 = "shared",
        ALIAS263 = "shared",
        ALIAS264 = "shared",
        ALIAS265 = "shared",
        ALIAS266 = "shared",
        ALIAS267 = "shared",
        ALIAS268 = "shared",
        ALIAS269 = "shared",
        ALIAS270 = "shared",
        ALIAS271 = "shared",
        ALIAS272 = "shared",
        ALIAS273 = "shared",
        ALIAS274 = "shared",
        ALIAS275 = "shared",
        ALIAS276 = "shared",
        ALIAS277 = "shared",
        ALIAS278 = "shared",
        ALIAS279 = "shared",
        ALIAS280 = "shared",
        ALIAS281 = "shared",
        ALIAS282 = "shared",
        ALIAS283 = "shared",
        ALIAS284 = "shared",
        ALIAS285 = "shared",
        ALIAS286 = "shared",
        ALIAS287 = "shared",
        ALIAS288 = "shared",
        ALIAS289 = "shared",
        ALIAS290 = "shared",
        ALIAS291 = "shared",
        ALIAS292 = "shared",
        ALIAS293 = "shared",
        ALIAS294 = "shared",
        ALIAS295 = "shared",
        ALIAS296 = "shared",
        ALIAS297 = "shared",
        ALIAS298 = "shared",
        ALIAS299 = "shared",
        DISTINCT = "different",
    }
}

#[test]
fn alias_declarations_do_not_consume_the_u8_unique_id_budget() {
    assert_eq!(ManyAliases::len(), 301);
    assert_eq!(ManyAliases::unique_len(), 2);
    assert_eq!(ManyAliases::all().len(), 301);
    assert_eq!(ManyAliases::ALIAS000, ManyAliases::ALIAS299);
    assert_eq!(ManyAliases::ALIAS299.to_raw(), 1_u8);
    assert_eq!(ManyAliases::DISTINCT.to_raw(), 2_u8);
    assert_eq!(ManyAliases::from_raw(3), None);
    assert_eq!(ManyAliases::lookup("shared"), Some(ManyAliases::ALIAS299));
    assert_eq!(ManyAliases::lookup("different"), Some(ManyAliases::DISTINCT));
    assert_eq!(ManyAliases::index_bytes(), 4);
    assert_eq!(ManyAliases::storage_bytes(), 1 + 6 + 1 + 9);
    assert_eq!(ManyAliases::total_storage_bytes(), 21);
    for &alias in &ManyAliases::all()[..300] {
        assert_eq!(alias, ManyAliases::ALIAS000);
    }
}
