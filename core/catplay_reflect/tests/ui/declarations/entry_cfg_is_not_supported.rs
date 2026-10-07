// error-pattern: only documentation attributes are supported
use catplay_reflect::static_strings;
static_strings! {
    struct Invalid {
        #[cfg(any())]
        FIRST = "first",
        SECOND = "second",
    }
}
