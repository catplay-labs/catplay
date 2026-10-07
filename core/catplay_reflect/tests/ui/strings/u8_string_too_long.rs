// error-pattern: catplay_reflect: a string plus its NUL terminator and pool prefix exceeds 64 KiB

use catplay_reflect::static_strings;

const BYTES: [u8; 65_536] = [b'x'; 65_536];
const TEXT: &str = match core::str::from_utf8(&BYTES) {
    Ok(text) => text,
    Err(_) => panic!("valid ASCII"),
};

static_strings! { struct TooLong(u8) { VALUE = TEXT } }
