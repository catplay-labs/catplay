// error-pattern: catplay_reflect: strings must not contain NUL bytes

use catplay_reflect::static_strings;

const EXTERNAL: &str = "audio\0";

static_strings! {
    struct TrailingNul(u8) {
        VALID = "audio",
        INVALID = EXTERNAL,
    }
}
