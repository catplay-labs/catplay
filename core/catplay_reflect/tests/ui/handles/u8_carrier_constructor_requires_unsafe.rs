// error-pattern: new_unchecked` is unsafe and requires unsafe

use catplay_reflect::{__private::Index8, static_strings};

static_strings! {
    struct Label(u8) {
        VALID = "valid",
    }
}

// A one-byte carrier needs the same validated-pool invariant as Index16.
const FORGED: Label = Label(Index8::<Label>::new_unchecked(2));
