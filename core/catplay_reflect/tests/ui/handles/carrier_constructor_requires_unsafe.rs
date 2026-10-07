// error-pattern: new_unchecked` is unsafe and requires unsafe

use catplay_reflect::{__private::Index, static_strings};

static_strings! {
    struct Label {
        VALID = "valid",
    }
}

// __private is public for downstream macro expansion, so its constructor
// must require unsafe even though the generated handles have safe APIs.
const FORGED: Label = Label(Index::<Label>::new_unchecked(2));
