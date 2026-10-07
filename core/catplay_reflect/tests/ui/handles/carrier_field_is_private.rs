// error-pattern: field `raw` of struct `catplay_reflect::__private::Index16` is private

use core::num::NonZeroU16;
use catplay_reflect::static_strings;

static_strings! {
    struct Label {
        VALID = "valid",
    }
}

// The caller may reach the carrier through the generated tuple field, but
// cannot overwrite that carrier's crate-private raw ID.
fn overwrite() {
    let mut label = Label::VALID;
    label.0.raw = NonZeroU16::new(2).unwrap();
}
