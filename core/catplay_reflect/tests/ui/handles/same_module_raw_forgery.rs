// error-pattern: mismatched types
// error-pattern: Index16<Label>
// error-pattern: NonZero<u16>

use core::num::NonZeroU16;
use catplay_reflect::static_strings;

static_strings! {
    struct Label {
        VALID = "valid",
    }
}

// A macro-created tuple field belongs to this caller module. Its privacy
// alone must never be the protection against forging an unchecked record ID.
fn forge() -> Label {
    Label(NonZeroU16::new(2).unwrap())
}
