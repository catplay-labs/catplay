// error-pattern: catplay_reflect: handle width must be u8 or u16

use catplay_reflect::static_strings;

static_strings! {
    struct Unsupported(u32) {
        VALUE = "value",
    }
}
