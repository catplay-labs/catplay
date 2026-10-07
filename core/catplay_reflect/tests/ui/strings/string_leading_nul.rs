// error-pattern: catplay_reflect: strings must not contain NUL bytes

use catplay_reflect::static_strings;

static_strings! {
    struct LeadingNul(u8) {
        INVALID = "\0audio",
    }
}
