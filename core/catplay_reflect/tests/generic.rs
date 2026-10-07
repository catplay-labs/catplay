use catplay_reflect::{StaticHandle, static_objects, static_strings};
use core::mem::size_of;

static_strings! {
    struct ShortText(u8) {
        FIRST = "from the small pool",
        SECOND = "second",
        ALIAS = "from the small pool",
    }
}

static_strings! {
    struct WideText(u16) {
        FIRST = "from the wide pool",
        SECOND = "another second",
    }
}

// Sized targets deliberately lack Copy, Clone, Eq, Hash and Debug.
struct Value {
    code: u32,
}

static_objects! {
    struct ShortValue(u8): Value {
        FIRST = Value { code: 10 },
        SECOND = Value { code: 20 },
    }
}

static_objects! {
    struct WideValue(u16): Value {
        FIRST = Value { code: 110 },
        SECOND = Value { code: 120 },
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Row<H: StaticHandle> {
    handle: H,
}

fn resolve<H: StaticHandle>(handle: H) -> &'static H::Target {
    handle.get()
}

fn restore<H: StaticHandle>(raw: H::Raw) -> Option<&'static H::Target> {
    H::from_raw(raw).map(resolve::<H>)
}

fn round_trip<H: StaticHandle>(handle: H) -> Option<H> {
    H::from_raw(StaticHandle::to_raw(handle))
}

fn row_code<H: StaticHandle<Target = Value>>(row: Row<H>) -> u32 {
    resolve(row.handle).code
}

#[test]
fn one_generic_resolver_accepts_sized_and_unsized_targets() {
    assert_eq!(resolve(ShortText::FIRST), "from the small pool");
    assert_eq!(resolve(WideText::FIRST), "from the wide pool");
    assert_eq!(resolve(ShortValue::FIRST).code, 10);
    assert_eq!(resolve(WideValue::FIRST).code, 110);
}

#[test]
fn pool_type_selects_the_target_even_with_equal_raw_ids() {
    assert_eq!(ShortText::FIRST.to_raw() as u16, WideText::FIRST.to_raw());
    assert_eq!(ShortValue::FIRST.to_raw() as u16, WideValue::FIRST.to_raw());
    assert_eq!(restore::<ShortText>(1), Some("from the small pool"));
    assert_eq!(restore::<WideText>(1), Some("from the wide pool"));
    assert_eq!(row_code(Row { handle: ShortValue::FIRST }), 10);
    assert_eq!(row_code(Row { handle: WideValue::FIRST }), 110);
    assert!(core::ptr::eq(resolve(ShortValue::FIRST), ShortValue::FIRST.get()));
    assert!(core::ptr::eq(resolve(WideValue::FIRST), WideValue::FIRST.get()));
}

#[test]
fn generic_resolver_handles_both_dense_and_offset_string_ids() {
    assert_eq!(ShortText::SECOND.to_raw(), 2_u8);
    assert_eq!(WideText::SECOND.to_raw(), 20_u16);
    assert_eq!(WideText::SECOND.offset(), 20);
    assert_eq!(WideText::SECOND.index(), 1);
    assert_eq!(restore::<ShortText>(2), Some("second"));
    assert_eq!(restore::<WideText>(20), Some("another second"));
    assert_eq!(restore::<WideText>(2), None); // Inside the first string.
    assert_eq!(round_trip(WideText::SECOND), Some(WideText::SECOND));
    assert_eq!(ShortText::index_bytes(), 4);
    assert_eq!(WideText::index_bytes(), 0);
}

#[test]
fn generic_rows_store_only_their_handle_width() {
    assert_eq!(size_of::<Row<ShortText>>(), 1);
    assert_eq!(size_of::<Row<WideText>>(), 2);
    assert_eq!(size_of::<Row<ShortValue>>(), 1);
    assert_eq!(size_of::<Row<WideValue>>(), 2);
    assert_eq!(size_of::<[Row<ShortText>; 9]>(), 9);
    assert_eq!(size_of::<[Row<WideText>; 9]>(), 18);
    assert_eq!(size_of::<Option<ShortText>>(), 1);
    assert_eq!(size_of::<Option<WideText>>(), 2);
    assert_eq!(size_of::<Option<ShortValue>>(), 1);
    assert_eq!(size_of::<Option<WideValue>>(), 2);
}

#[test]
fn generic_raw_round_trips_preserve_width_and_validate_membership() {
    let _: <ShortText as StaticHandle>::Raw = 1u8;
    let _: <WideText as StaticHandle>::Raw = 1u16;
    let _: <ShortValue as StaticHandle>::Raw = 1u8;
    let _: <WideValue as StaticHandle>::Raw = 1u16;
    assert_eq!(round_trip(ShortText::ALIAS), Some(ShortText::FIRST));
    assert_eq!(round_trip(WideText::SECOND), Some(WideText::SECOND));
    assert_eq!(round_trip(ShortValue::SECOND), Some(ShortValue::SECOND));
    assert_eq!(round_trip(WideValue::SECOND), Some(WideValue::SECOND));
    assert!(restore::<ShortText>(0).is_none());
    assert!(restore::<WideText>(0).is_none());
    assert!(restore::<ShortValue>(0).is_none());
    assert!(restore::<WideValue>(0).is_none());
    assert!(restore::<ShortText>(3).is_none()); // ALIAS is not another unique ID.
    assert!(restore::<WideText>(u16::MAX).is_none());
    assert!(restore::<ShortValue>(u8::MAX).is_none());
    assert!(restore::<WideValue>(u16::MAX).is_none());
}

#[test]
fn inherent_const_access_and_generic_runtime_access_agree() {
    const TEXT8: &str = ShortText::FIRST.as_str();
    const TEXT16: &str = WideText::FIRST.get();
    const VALUE8: &Value = ShortValue::SECOND.get();
    const VALUE16: &Value = WideValue::SECOND.get();
    const LOOKUP: Option<ShortText> = ShortText::lookup("from the small pool");
    const LOOKUP16: Option<WideText> = WideText::lookup("another second");
    const INDEX16: usize = WideText::SECOND.index();
    assert_eq!(resolve(LOOKUP.unwrap()), TEXT8);
    assert_eq!(resolve(WideText::FIRST), TEXT16);
    assert_eq!(resolve(LOOKUP16.unwrap()), "another second");
    assert_eq!(INDEX16, 1);
    assert!(core::ptr::eq(resolve(ShortValue::SECOND), VALUE8));
    assert!(core::ptr::eq(resolve(WideValue::SECOND), VALUE16));
}
