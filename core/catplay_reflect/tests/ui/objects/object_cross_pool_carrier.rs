// error-pattern: mismatched types
// error-pattern: Index16<First>
// error-pattern: Index16<Second>

use catplay_reflect::static_objects;

static_objects! {
    struct First: u32 {
        ONE = 1,
    }
}

static_objects! {
    struct Second: u32 {
        ONE = 1,
        TWO = 2,
    }
}

// Identical object types do not make the two pools interchangeable.
fn transplant() -> First {
    First(Second::TWO.0)
}
