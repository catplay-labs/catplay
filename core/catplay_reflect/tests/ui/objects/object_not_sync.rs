// error-pattern: cannot be shared between threads safely
use core::cell::Cell;
use catplay_reflect::static_objects;
static_objects! { struct Invalid: Cell<u8> { VALUE = Cell::new(0) } }
