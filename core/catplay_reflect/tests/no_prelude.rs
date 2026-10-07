#![no_implicit_prelude]

extern crate catplay_reflect;
extern crate core;

use ::catplay_reflect::static_strings;

static_strings! {
    struct Text {
        FIRST = "first",
        SECOND = "second",
    }
}

::catplay_reflect::static_objects! {
    struct Number: u32 {
        FIRST = 1,
        SECOND = 2,
    }
}

static_strings! {
    struct NarrowText(u8) {
        FIRST = "first",
        SECOND = "second",
    }
}

::catplay_reflect::static_objects! {
    struct NarrowNumber(u8): u32 {
        FIRST = 1,
        SECOND = 2,
    }
}

static_strings! {
    struct WideText(u16) {
        FIRST = "first",
        SECOND = "second",
    }
}

::catplay_reflect::static_objects! {
    struct WideNumber(u16): u32 {
        FIRST = 1,
        SECOND = 2,
    }
}

#[test]
fn macros_do_not_depend_on_callers_prelude() {
    ::core::assert_eq!(Text::FIRST.as_str(), "first");
    ::core::assert_eq!(Text::from_raw(1), ::core::option::Option::Some(Text::FIRST));
    ::core::assert_eq!(*Number::SECOND.get(), 2);

    ::core::assert_eq!(NarrowText::SECOND.as_str(), "second");
    ::core::assert_eq!(NarrowText::SECOND.to_raw(), 2_u8);
    ::core::assert_eq!(NarrowText::lookup("first"), ::core::option::Option::Some(NarrowText::FIRST),);
    ::core::assert_eq!(*NarrowNumber::SECOND.get(), 2);
    ::core::assert_eq!(NarrowNumber::from_raw(2_u8), ::core::option::Option::Some(NarrowNumber::SECOND),);

    ::core::assert_eq!(WideText::SECOND.as_str(), "second");
    ::core::assert_eq!(WideText::SECOND.to_raw(), 7_u16);
    ::core::assert_eq!(WideText::SECOND.index(), 1);
    ::core::assert_eq!(WideText::from_raw(2), ::core::option::Option::None);
    ::core::assert_eq!(Text::SECOND.to_raw(), 7_u16);
    ::core::assert_eq!(*WideNumber::SECOND.get(), 2);
    ::core::assert_eq!(::core::mem::size_of::<Text>(), 2);
    ::core::assert_eq!(::core::mem::size_of::<Number>(), 2);
    ::core::assert_eq!(::core::mem::size_of::<WideText>(), 2);
    ::core::assert_eq!(::core::mem::size_of::<WideNumber>(), 2);
    ::core::assert_eq!(::core::mem::size_of::<NarrowText>(), 1);
    ::core::assert_eq!(::core::mem::size_of::<NarrowNumber>(), 1);
}
