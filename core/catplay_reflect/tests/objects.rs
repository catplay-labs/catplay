use catplay_reflect::{StaticHandle, static_objects};
use core::hash::Hash;
use core::mem::{align_of, size_of};
use core::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashSet;

// Deliberately has no Copy, Clone, Debug, Eq or Hash implementations.
struct Config {
    code: u16,
    payload: [u8; 3],
}

static_objects! {
    struct ConfigRef: Config {
        FIRST = Config { code: 7, payload: [1, 2, 3] },
        SAME_VALUE = Config { code: 7, payload: [1, 2, 3] },
        OTHER = Config { code: 9, payload: [4, 5, 6] },
    }
}

static_objects! {
    /// Small pool whose references and optional references occupy one byte.
    struct CompactConfigRef(u8): Config {
        /// The first object still needs no payload traits.
        FIRST = Config { code: 17, payload: [1, 3, 5] },
        SAME_VALUE = Config { code: 17, payload: [1, 3, 5] },
        OTHER = Config { code: 19, payload: [2, 4, 6] },
    }
}

static_objects! {
    struct ExplicitWideConfigRef(u16): Config {
        FIRST = Config { code: 23, payload: [7, 8, 9] },
    }
}

static_objects! { struct Counter: AtomicU32 { VALUE = AtomicU32::new(11) } }
static_objects! { struct ZeroSize: () { FIRST = (), SECOND = () } }
static_objects! { struct Empty: Config {} }
static_objects! { struct CompactEmpty(u8): Config {} }

const STATIC_CONFIG: &Config = ConfigRef::FIRST.get();
// Keep both CTFE paths: the minimum compiler supports a const reference to
// this shared, interior-mutable static object as well as a static initializer.
const CONST_COUNTER: &'static AtomicU32 = Counter::VALUE.get();
static STATIC_COUNTER: &AtomicU32 = Counter::VALUE.get();
const DECODED: Option<ConfigRef> = ConfigRef::from_raw(2);
const COMPACT_CONFIG: &Config = CompactConfigRef::FIRST.get();
const COMPACT_RAW: u8 = CompactConfigRef::OTHER.to_raw();
const COMPACT_DECODED: Option<CompactConfigRef> = CompactConfigRef::from_raw(COMPACT_RAW);

#[test]
fn non_copy_objects_and_const_borrows() {
    assert_eq!(STATIC_CONFIG.code, 7);
    assert_eq!(ConfigRef::OTHER.code, 9);
    assert_eq!(ConfigRef::OTHER.as_ref().payload, [4, 5, 6]);
    let value: &'static Config = ConfigRef::FIRST.into();
    assert!(core::ptr::eq(value, STATIC_CONFIG));
    assert_eq!(DECODED, Some(ConfigRef::SAME_VALUE));
    assert_eq!(format!("{:?}", ConfigRef::OTHER), "ConfigRef(2)");
}

#[test]
fn identity_and_raw_range() {
    assert_ne!(ConfigRef::FIRST, ConfigRef::SAME_VALUE);
    assert!(!core::ptr::eq(ConfigRef::FIRST.get(), ConfigRef::SAME_VALUE.get()));
    assert_eq!(ConfigRef::FIRST.to_raw(), 1);
    assert_eq!(ConfigRef::OTHER.index(), 2);
    assert_eq!(ConfigRef::all(), &[ConfigRef::FIRST, ConfigRef::SAME_VALUE, ConfigRef::OTHER,]);
    let unique: HashSet<_> = ConfigRef::all().iter().copied().collect();
    assert_eq!(unique.len(), 3);
    for raw in 0..=u16::MAX {
        let expected = ConfigRef::all()
            .iter()
            .copied()
            .find(|id| id.to_raw() == raw);
        assert_eq!(ConfigRef::from_raw(raw), expected);
    }
}

#[test]
fn interior_mutability_is_shared() {
    assert!(core::ptr::eq(CONST_COUNTER, STATIC_COUNTER));
    assert_eq!(CONST_COUNTER.fetch_add(1, Ordering::Relaxed), 11);
    assert_eq!(STATIC_COUNTER.load(Ordering::Relaxed), 12);
    assert_eq!(Counter::VALUE.load(Ordering::Relaxed), 12);
    assert_eq!(<Counter as StaticHandle>::get(Counter::VALUE).load(Ordering::Relaxed), 12,);
    assert!(core::ptr::eq(CONST_COUNTER, Counter::VALUE.get()));
}

#[test]
fn layouts_empty_pools_and_zero_sized_identity() {
    assert_eq!(size_of::<ConfigRef>(), 2);
    assert_eq!(size_of::<Option<ConfigRef>>(), 2);
    assert_eq!(align_of::<ConfigRef>(), align_of::<u16>());
    assert_eq!(size_of::<[ConfigRef; 100]>(), 200);
    assert_ne!(ZeroSize::FIRST, ZeroSize::SECOND);
    assert_eq!(ZeroSize::SECOND.get(), &());
    assert!(Empty::is_empty());
    assert_eq!(Empty::len(), 0);
    assert_eq!(Empty::all(), &[]);
    assert_eq!(Empty::from_raw(1), None);
    assert_eq!(size_of::<CompactEmpty>(), 1);
    assert_eq!(size_of::<Option<CompactEmpty>>(), 1);
    assert!(CompactEmpty::is_empty());
    assert!(CompactEmpty::all().is_empty());
    assert_eq!(CompactEmpty::from_raw(0), None);
    assert_eq!(CompactEmpty::from_raw(1), None);
    assert_eq!(CompactEmpty::from_raw(u8::MAX), None);
}

#[test]
fn carrier_traits_do_not_require_payload_traits() {
    #[allow(dead_code)]
    struct NoTraits(core::cell::Cell<u8>);
    fn required<T: Copy + Clone + Eq + Hash + Send + Sync>() {}
    required::<catplay_reflect::__private::Index<NoTraits>>();
    required::<catplay_reflect::__private::Index8<NoTraits>>();
    required::<catplay_reflect::__private::Index16<NoTraits>>();
}

#[test]
fn u8_layout_const_api_and_raw_membership() {
    assert_eq!(COMPACT_CONFIG.code, 17);
    assert_eq!(COMPACT_DECODED, Some(CompactConfigRef::OTHER));
    assert_eq!(size_of::<CompactConfigRef>(), 1);
    assert_eq!(size_of::<Option<CompactConfigRef>>(), 1);
    assert_eq!(align_of::<CompactConfigRef>(), align_of::<u8>());
    assert_eq!(size_of::<[CompactConfigRef; 100]>(), 100);
    assert_ne!(CompactConfigRef::FIRST, CompactConfigRef::SAME_VALUE);
    assert_eq!(CompactConfigRef::OTHER.payload, [2, 4, 6]);
    assert_eq!(format!("{:?}", CompactConfigRef::OTHER), "CompactConfigRef(2)");
    let unique: HashSet<_> = CompactConfigRef::all().iter().copied().collect();
    assert_eq!(unique.len(), 3);

    for raw in 0..=u8::MAX {
        let expected = CompactConfigRef::all()
            .iter()
            .copied()
            .find(|id| id.to_raw() == raw);
        assert_eq!(CompactConfigRef::from_raw(raw), expected);
        assert_eq!(<CompactConfigRef as StaticHandle>::from_raw(raw), expected);
    }
}

#[test]
fn generic_resolver_preserves_pool_and_raw_types() {
    fn resolve<H: StaticHandle>(handle: H) -> &'static H::Target {
        handle.get()
    }

    fn round_trip<H: StaticHandle>(handle: H) -> Option<H> {
        H::from_raw(handle.to_raw())
    }

    struct Linked<H: StaticHandle> {
        current: H,
        next: Option<H>,
    }

    let current = CompactConfigRef::FIRST;
    assert!(core::ptr::eq(resolve(current), current.get()));
    assert_eq!(round_trip(current), Some(current));
    let raw: u8 = <CompactConfigRef as StaticHandle>::to_raw(current);
    assert_eq!(raw, 1);

    let linked = Linked {
        current,
        next: Some(CompactConfigRef::OTHER),
    };
    assert_eq!(size_of::<Linked<CompactConfigRef>>(), 2);
    assert_eq!(resolve(linked.current).code, 17);
    assert_eq!(resolve(linked.next.unwrap()).code, 19);

    let default_raw: u16 = <ConfigRef as StaticHandle>::to_raw(ConfigRef::FIRST);
    let explicit_raw: u16 = <ExplicitWideConfigRef as StaticHandle>::to_raw(ExplicitWideConfigRef::FIRST);
    assert_eq!((default_raw, explicit_raw), (1, 1));
    assert_eq!(ExplicitWideConfigRef::from_raw(u16::MAX), None);
    assert_eq!(size_of::<ExplicitWideConfigRef>(), 2);
    assert_eq!(size_of::<Option<ExplicitWideConfigRef>>(), 2);
    assert_eq!(resolve(ExplicitWideConfigRef::FIRST).code, 23);
    assert_eq!(round_trip(ConfigRef::OTHER), Some(ConfigRef::OTHER));
}

// The storage type and input expressions retain caller scope even for helper
// names formerly used by the macro expansion.
mod hygiene {
    #![allow(non_camel_case_types, non_upper_case_globals)]
    pub struct __StaticObjectsIndex(pub u32);
    const __STATIC_OBJECTS_COUNT: __StaticObjectsIndex = __StaticObjectsIndex(99);
    const __STATIC_OBJECTS_VALUES: __StaticObjectsIndex = __StaticObjectsIndex(42);

    catplay_reflect::static_objects! {
        pub struct Values: __StaticObjectsIndex {
            COUNT = __STATIC_OBJECTS_COUNT,
            VALUES = __STATIC_OBJECTS_VALUES,
        }
    }

    catplay_reflect::static_objects! {
        pub struct CompactValues(u8): __StaticObjectsIndex {
            COUNT = __STATIC_OBJECTS_COUNT,
            VALUES = __STATIC_OBJECTS_VALUES,
        }
    }
}

#[test]
fn caller_type_and_value_names_are_not_captured() {
    assert_eq!(hygiene::Values::COUNT.get().0, 99);
    assert_eq!(hygiene::Values::VALUES.get().0, 42);
    assert_eq!(hygiene::CompactValues::COUNT.get().0, 99);
    assert_eq!(hygiene::CompactValues::VALUES.get().0, 42);
}

struct Node {
    value: u32,
    next: Option<NodeRef>,
}

static_objects! {
    struct NodeRef: Node {
        FIRST = Node { value: 1, next: Some(NodeRef::SECOND) },
        SECOND = Node { value: 2, next: Some(NodeRef::FIRST) },
    }
}

#[test]
fn recursive_metadata_uses_compact_handles() {
    let first = NodeRef::FIRST;
    let second = first.next.unwrap();
    assert_eq!(second.value, 2);
    assert_eq!(second.next, Some(first));
    assert_eq!(size_of::<Option<NodeRef>>(), 2);
}

struct CompactNode {
    value: u32,
    next: Option<CompactNodeRef>,
}

static_objects! {
    struct CompactNodeRef(u8): CompactNode {
        FIRST = CompactNode { value: 31, next: Some(CompactNodeRef::SECOND) },
        SECOND = CompactNode { value: 32, next: Some(CompactNodeRef::FIRST) },
    }
}

#[test]
fn recursive_u8_metadata_needs_no_payload_traits() {
    let first = CompactNodeRef::FIRST;
    let second = first.next.unwrap();
    assert_eq!(second.value, 32);
    assert_eq!(second.next, Some(first));
    assert_eq!(size_of::<CompactNodeRef>(), 1);
    assert_eq!(size_of::<Option<CompactNodeRef>>(), 1);
}

// The maximum u8 pool exercises the nonzero encoding at raw 255. Unit values
// keep the test focused on handle identity and the declaration-count limit.
macro_rules! full_byte_pool {
    ($($name:ident),* $(,)?) => {
        static_objects! { struct FullBytePool(u8): () { $($name = ()),* } }
    };
}

full_byte_pool! {
    N001, N002, N003, N004, N005, N006, N007, N008, N009, N010, N011, N012,
    N013, N014, N015, N016, N017, N018, N019, N020, N021, N022, N023, N024,
    N025, N026, N027, N028, N029, N030, N031, N032, N033, N034, N035, N036,
    N037, N038, N039, N040, N041, N042, N043, N044, N045, N046, N047, N048,
    N049, N050, N051, N052, N053, N054, N055, N056, N057, N058, N059, N060,
    N061, N062, N063, N064, N065, N066, N067, N068, N069, N070, N071, N072,
    N073, N074, N075, N076, N077, N078, N079, N080, N081, N082, N083, N084,
    N085, N086, N087, N088, N089, N090, N091, N092, N093, N094, N095, N096,
    N097, N098, N099, N100, N101, N102, N103, N104, N105, N106, N107, N108,
    N109, N110, N111, N112, N113, N114, N115, N116, N117, N118, N119, N120,
    N121, N122, N123, N124, N125, N126, N127, N128, N129, N130, N131, N132,
    N133, N134, N135, N136, N137, N138, N139, N140, N141, N142, N143, N144,
    N145, N146, N147, N148, N149, N150, N151, N152, N153, N154, N155, N156,
    N157, N158, N159, N160, N161, N162, N163, N164, N165, N166, N167, N168,
    N169, N170, N171, N172, N173, N174, N175, N176, N177, N178, N179, N180,
    N181, N182, N183, N184, N185, N186, N187, N188, N189, N190, N191, N192,
    N193, N194, N195, N196, N197, N198, N199, N200, N201, N202, N203, N204,
    N205, N206, N207, N208, N209, N210, N211, N212, N213, N214, N215, N216,
    N217, N218, N219, N220, N221, N222, N223, N224, N225, N226, N227, N228,
    N229, N230, N231, N232, N233, N234, N235, N236, N237, N238, N239, N240,
    N241, N242, N243, N244, N245, N246, N247, N248, N249, N250, N251, N252,
    N253, N254, N255,
}

#[test]
fn u8_supports_all_255_nonzero_ids() {
    assert_eq!(FullBytePool::len(), u8::MAX as usize);
    assert_eq!(FullBytePool::N001.to_raw(), 1);
    assert_eq!(FullBytePool::N255.to_raw(), u8::MAX);
    assert_eq!(FullBytePool::N255.index(), 254);
    assert_eq!(FullBytePool::from_raw(0), None);
    assert_eq!(FullBytePool::from_raw(u8::MAX), Some(FullBytePool::N255));
    assert_eq!(size_of::<Option<FullBytePool>>(), 1);
    for (index, handle) in FullBytePool::all().iter().copied().enumerate() {
        assert_eq!(handle.index(), index);
        assert_eq!(FullBytePool::from_raw(handle.to_raw()), Some(handle));
        assert_eq!(handle.get(), &());
    }
}

#[test]
#[allow(non_camel_case_types)]
fn handle_names_may_match_helpers_and_local_aliases() {
    struct Payload {
        value: u32,
    }

    macro_rules! check {
        ($name:ident, $width:ident) => {{
            static_objects! {
                struct $name($width): Payload {
                    FIRST = Payload { value: 41 },
                    SECOND = Payload { value: 42 },
                }
            }

            const FIRST: &Payload = $name::FIRST.get();
            let raw: $width = $name::SECOND.to_raw();
            assert_eq!(FIRST.value, 41);
            assert_eq!($name::SECOND.value, 42);
            assert_eq!($name::from_raw(raw), Some($name::SECOND));
            assert_eq!($name::all(), &[$name::FIRST, $name::SECOND]);
            assert!(core::ptr::eq(<$name as StaticHandle>::get($name::FIRST), FIRST,));
            assert_eq!(size_of::<Option<$name>>(), size_of::<$width>());
        }};
    }

    check!(__StaticObjectsIndex, u8);
    check!(__StaticObjectsIndex, u16);
    check!(__STATIC_OBJECTS_HANDLES, u8);
    check!(__STATIC_OBJECTS_HANDLES, u16);
    check!(__StaticPoolsLocal, u8);
    check!(__StaticPoolsLocal, u16);
    check!(__StaticPoolsAlternate, u8);
    check!(__StaticPoolsAlternate, u16);

    // The implementation's type alias must not capture the payload type.
    struct __StaticPoolsLocal {
        value: u32,
    }
    static_objects! {
        struct PayloadAliasPool(u8): __StaticPoolsLocal {
            FIRST = __StaticPoolsLocal { value: 77 },
        }
    }
    assert_eq!(PayloadAliasPool::FIRST.value, 77);
}
