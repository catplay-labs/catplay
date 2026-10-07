// error-pattern: catplay_reflect: handle width must be u8 or u16

use catplay_reflect::static_objects;

static_objects! {
    struct Unsupported(u32): u32 {
        VALUE = 42,
    }
}
