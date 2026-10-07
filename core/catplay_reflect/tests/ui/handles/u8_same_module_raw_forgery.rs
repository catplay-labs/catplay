// error-pattern: mismatched types
// error-pattern: Index8<Label>
// error-pattern: NonZero<u8>

use core::num::NonZeroU8;
use catplay_reflect::static_strings;

static_strings! {
    struct Label(u8) {
        VALID = "valid",
    }
}

// The caller owns this module, but a nonzero byte is not a pool carrier.
fn forge() -> Label {
    Label(NonZeroU8::new(2).unwrap())
}
