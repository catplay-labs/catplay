// error-pattern: field `raw` of struct `catplay_reflect::__private::Index8` is private

use core::num::NonZeroU8;
use catplay_reflect::static_strings;

static_strings! {
    struct Label(u8) {
        VALID = "valid",
    }
}

fn overwrite() {
    let mut label = Label::VALID;
    label.0.raw = NonZeroU8::new(2).unwrap();
}
