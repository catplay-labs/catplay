use catplay_lingo::{Iap1Decode, Reader, iap1_bitflags, lingo_0x0, lingos};
use catplay_reflect::static_strings;

static_strings! {
    struct DebugString(u8) {
        FOO = "FOO",
        BAR = "BAR",
    }
}

#[test]
fn global_lingo_debug_uses_dotted_command_without_wrapper() {
    let message = lingos::LingoMessage::General(lingo_0x0::LingoMessage::StartIDPS(lingo_0x0::StartIDPS {}));
    assert_eq!(format!("{message:?}"), "General.StartIDPS");
    assert_eq!(format!("{message:#?}"), "General.StartIDPS");
}

#[test]
fn token_debug_uses_variant_name_and_record_fields() {
    let token = lingo_0x0::SetFIDTokenValuesTokens::decode(&mut Reader::new(&[7, 0, 2, 12, 0, 0, 0, 0])).unwrap();
    for debug in [format!("{token:?}"), format!("{token:#?}")] {
        assert!(debug.starts_with("AccessoryInfoToken {"), "{debug}");
        assert!(debug.contains("accInfoType: RFCertifications"));
        assert!(!debug.contains("AccessoryInfoToken("));
        assert!(!debug.contains("AccessoryInfoToken {\n    SetFID"));
    }
}

iap1_bitflags! {
    strings = DebugString;
    struct DebugFlags: u32 {
        const FOO = 1 << 0;
        const BAR = 1 << 3;
    }
}

#[test]
fn bitflags_debug_lists_names_and_each_unknown_bit() {
    for (bits, expected) in [
        (0, "(empty)"),
        (9, "FOO | BAR"),
        (1 << 2, "BIT_2"),
        (9 | (1 << 2) | (1 << 31), "FOO | BAR | BIT_2 | BIT_31"),
    ] {
        let flags = DebugFlags::from_bits_retain(bits);
        assert_eq!(format!("{flags:?}"), expected);
        assert_eq!(format!("{flags:#?}"), expected);
    }
}
