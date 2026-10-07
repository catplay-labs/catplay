// error-pattern: mismatched types
// error-pattern: Index8<First>
// error-pattern: Index8<Second>

use catplay_reflect::static_objects;

static_objects! {
    struct First(u8): u16 {
        FIRST = 1,
    }
}

static_objects! {
    struct Second(u8): u16 {
        FIRST = 1,
        SECOND = 2,
    }
}

// Same width and payload type do not permit exchanging the pool brand.
fn transplant() -> First {
    First(Second::SECOND.0)
}
