use catplay_reflect::{FieldAccess, FieldProject, FieldProjector, FieldTable, ProjectorCapability, field_project, static_objects};

struct Projector;
struct NarrowProjector;
struct IndirectProjector;
struct Capability;

struct Context {
    id: u16,
    label: &'static str,
}

impl FieldProjector<Context> for Projector {
    type Request = Vec<(u16, u8)>;
    type Capability = Capability;
    type ContextPoolWidth = u8;
    type ContextPoolObject = Context;
    type FieldOffsetWidth = u32;
}

impl FieldProjector<Context> for NarrowProjector {
    type Request = Vec<(u16, u8)>;
    type Capability = Capability;
    type ContextPoolWidth = u8;
    type ContextPoolObject = Context;
    type FieldOffsetWidth = u16;
}

impl FieldProjector<Context> for IndirectProjector {
    type Request = Vec<(u16, u8)>;
    type Capability = Capability;
    type ContextPoolWidth = u8;
    type ContextPoolObject = &'static Context;
    type FieldOffsetWidth = u32;
}

impl ProjectorCapability<u8, Context> for Capability {
    type Request = Vec<(u16, u8)>;

    unsafe fn project(field: *mut u8, _: FieldAccess, context: &Context, request: &mut Self::Request) {
        // SAFETY: FieldOp passes a pointer to the selected live u8 field.
        request.push((context.id, unsafe { *field }));
    }
}

field_project! {
    struct Repeated {
        value: u8,
    }
    project Projector, context = Context, pool = u8 {
        value: Context { id: 10, label: "first" },
        value: Context { id: 20, label: "second" },
    }
}

field_project! {
    struct Other {
        value: u8,
    }
    project Projector, context = Context, pool = u8 {
        value: Context { id: 30, label: "other pool" },
    }
}

static_objects! {
    struct EmptyPool(u8): Context {}
}

field_project! {
    struct Narrow {
        value: u8,
    }
    project NarrowProjector, context = Context, pool = u8 {
        value: Context { id: 40, label: "narrow" },
    }
}

field_project! {
    struct Indirect {
        value: u8,
    }
    project IndirectProjector, context = Context, pool = u8 {
        value: &Context { id: 50, label: "indirect" },
        value: &Context { id: 60, label: "second indirect" },
    }
}

#[test]
fn repeated_fields_keep_distinct_contexts_and_tables_keep_distinct_pools() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(core::mem::size_of::<catplay_reflect::FieldEntry<Context, Projector>>(), 13);
        assert_eq!(core::mem::size_of::<FieldTable<Context, Projector>>(), 25);
        assert_eq!(core::mem::size_of::<catplay_reflect::FieldOp<'static, Context, Projector>>(), 16);
    }

    let fields = <Repeated as FieldProject<Projector, Context>>::FIELDS;
    let mut operations = fields.iter();
    let first = operations.next().unwrap();
    let second = operations.next().unwrap();
    assert_eq!((first.context().id, first.context().label), (10, "first"));
    assert_eq!((second.context().id, second.context().label), (20, "second"));

    let other = <Other as FieldProject<Projector, Context>>::FIELDS;
    assert_eq!(other.iter().next().unwrap().context().id, 30);

    let mut value = Repeated { value: 7 };
    let mut seen = Vec::new();
    for field in fields {
        // SAFETY: both operations were generated for Repeated.
        unsafe { field.apply(&mut value, &mut seen) };
    }
    assert_eq!(seen, [(10, 7), (20, 7)]);
}

#[test]
fn empty_table_has_no_contexts() {
    let table = FieldTable::<Context, Projector>::new::<EmptyPool>(&[]);
    assert!(table.is_empty());
    assert_eq!(table.iter().count(), 0);
}

#[test]
fn projector_can_choose_u16_field_offsets() {
    #[cfg(target_pointer_width = "64")]
    assert_eq!(core::mem::size_of::<catplay_reflect::FieldEntry<Context, NarrowProjector>>(), 11);

    let mut value = Narrow { value: 9 };
    let mut seen = Vec::new();
    for field in <Narrow as FieldProject<NarrowProjector, Context>>::FIELDS {
        // SAFETY: the operation was generated for Narrow.
        unsafe { field.apply(&mut value, &mut seen) };
    }
    assert_eq!(seen, [(40, 9)]);
}

#[test]
fn indirect_pool_resolves_contexts_and_preserves_projection() {
    let fields = <Indirect as FieldProject<IndirectProjector, Context>>::FIELDS;
    let contexts: Vec<_> = fields
        .iter()
        .map(|field| (field.context().id, field.context().label))
        .collect();
    assert_eq!(contexts, [(50, "indirect"), (60, "second indirect")]);

    let mut value = Indirect { value: 7 };
    let mut seen = Vec::new();
    for field in fields {
        // SAFETY: the operations were generated for Indirect.
        unsafe { field.apply(&mut value, &mut seen) };
    }
    assert_eq!(seen, [(50, 7), (60, 7)]);
}
