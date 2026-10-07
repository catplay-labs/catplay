// error-pattern: mismatched types
// error-pattern: Index16<First>
// error-pattern: Index16<Second>

use catplay_reflect::static_strings;

static_strings! {
    struct First {
        SHORT = "a",
    }
}

static_strings! {
    struct Second {
        PREFIX = "a considerably longer first record",
        LATER = "z",
    }
}

// Both tuple fields are accessible in this module, but their opaque carriers
// belong to different pools. A valid ID in Second need not exist in First.
fn transplant() -> First {
    First(Second::LATER.0)
}
