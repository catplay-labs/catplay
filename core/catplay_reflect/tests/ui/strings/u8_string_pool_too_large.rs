// error-pattern: catplay_reflect: deduplicated string pool exceeds 64 KiB

use catplay_reflect::static_strings;

const BYTES: [u8; 65_535] = [b'x'; 65_535];
const TEXT: &str = match core::str::from_utf8(&BYTES) {
    Ok(text) => text,
    Err(_) => panic!("valid ASCII"),
};

// The valid first record fills the available byte pool. Adding an empty
// record needs one more byte, for 65,537 bytes in total.
static_strings! { struct TooLarge(u8) { FIRST = TEXT, SECOND = "" } }
