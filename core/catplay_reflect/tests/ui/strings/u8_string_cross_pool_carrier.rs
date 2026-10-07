// error-pattern: mismatched types
// error-pattern: Index8<First>
// error-pattern: Index8<Second>

use catplay_reflect::static_strings;

static_strings! {
    struct First(u8) {
        FIRST = "first",
    }
}

static_strings! {
    struct Second(u8) {
        FIRST = "first",
        SECOND = "second",
    }
}

// Same width and payload type do not permit exchanging the pool brand.
fn transplant() -> First {
    First(Second::SECOND.0)
}
