use catplay_reflect::{HandleShared, static_objects};

static_objects! {
    struct FirstPool(u8): u32 {
        ONE = 11,
        TWO = 22,
    }
}

static_objects! {
    struct SecondPool(u8): u32 {
        ONE = 33,
        TWO = 44,
    }
}

static_objects! {
    struct EmptyPool(u8): u32 {}
}

static_objects! {
    struct WidePool(u16): u32 {
        ONE = 55,
    }
}

const FIRST: HandleShared<u32, u8> = HandleShared::new::<FirstPool>();
const SECOND: HandleShared<u32, u8> = HandleShared::new::<SecondPool>();
const WIDE: HandleShared<u32, u16> = HandleShared::new::<WidePool>();

#[test]
fn shared_type_resolves_distinct_typed_pools() {
    #[cfg(target_pointer_width = "64")]
    assert_eq!(core::mem::size_of::<HandleShared<u32, u8>>(), 9);

    assert_eq!(FIRST.len(), 2);
    assert_eq!(*FIRST.resolve(1), 11);
    assert_eq!(*FIRST.resolve(2), 22);
    assert_eq!(*SECOND.resolve(1), 33);
    assert_eq!(*SECOND.resolve(2), 44);
}

#[test]
fn empty_pool_has_no_entries() {
    let shared = HandleShared::<u32, u8>::new::<EmptyPool>();
    assert!(shared.is_empty());
    assert_eq!(shared.len(), 0);
}

#[test]
fn wide_pool_keeps_its_u16_length() {
    #[cfg(target_pointer_width = "64")]
    assert_eq!(core::mem::size_of::<HandleShared<u32, u16>>(), 10);
    assert_eq!(WIDE.len(), 1);
    assert_eq!(*WIDE.resolve(1), 55);
}

#[test]
#[should_panic]
fn rejects_an_out_of_range_handle() {
    let _ = FIRST.resolve(3);
}
