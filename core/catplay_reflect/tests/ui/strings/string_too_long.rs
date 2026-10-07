// error-pattern: catplay_reflect: a string plus its NUL terminator and pool prefix exceeds 64 KiB

use catplay_reflect::static_strings;

const BYTES: [u8; 65_535] = [b'x'; 65_535];
const TEXT: &str = match core::str::from_utf8(&BYTES) {
    Ok(text) => text,
    Err(_) => panic!("valid ASCII"),
};

static_strings! { struct TooLong { VALUE = TEXT } }
