use super::{CsmAccum, CsmEncode, CsmParam, CsmPayloadEncode, CsmReader, CsmWriter};
use crate::decoder::{CsmStruct, CsmStructName};
use catplay_reflect::StringPool;

pub use catplay_reflect::{FieldAccess, FieldOp, FieldProject, FieldProjector, ProjectorCapability, ProjectorOptionalCapability};

/// Marker for CSM structs whose fields are described by the CSM projector.
pub trait CsmStructProjected:
    CsmStructName + FieldProject<CsmEncodeProjector<Self::StringPool>, CsmFieldContext<Self::StringPool>>
{
    type StringPool: StringPool;
}

impl<T: CsmStructProjected> CsmStruct for T {
    fn feed_param(&mut self, param: &CsmParam) {
        feed_projected_fields(self, param);
    }

    fn prealloc(&mut self, reader: &CsmReader) {
        prealloc_projected_fields(self, reader);
    }
}

/// One projector interprets field metadata for CSM encoding, decoding, and Debug.
pub struct CsmEncodeProjector<S: StringPool>(core::marker::PhantomData<S>);

#[doc(hidden)]
pub struct CsmEncodeCapability<S: StringPool>(core::marker::PhantomData<S>);

#[doc(hidden)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct CsmFieldContext<S: StringPool> {
    id: u16,
    name: S,
}

impl<S: StringPool> CsmFieldContext<S> {
    pub const fn new(id: u16, name: S) -> Self {
        Self { id, name }
    }
}

#[doc(hidden)]
pub enum CsmFieldRequest {
    Encode(*mut CsmWriter<'static>),
    Feed { param: *const CsmParam<'static>, handled: bool },
    Prealloc(*const CsmReader<'static>),
    Debug(*mut core::fmt::DebugStruct<'static, 'static>),
}

impl<S: StringPool> FieldProjector<CsmFieldContext<S>> for CsmEncodeProjector<S> {
    type Request = CsmFieldRequest;
    type Capability = CsmEncodeCapability<S>;
    type ContextPoolWidth = u8;
    type ContextPoolObject = CsmFieldContext<S>;
    type FieldOffsetWidth = u16;
}

trait CsmProjectedField {
    fn encode_field(&self, id: u16, writer: &mut CsmWriter<'static>);
    fn feed_field(&mut self, param: &CsmParam<'static>);
    fn prealloc_field(&mut self, size: usize);
    fn debug_field(&self, name: &'static str, debug: &mut core::fmt::DebugStruct<'static, 'static>);
}

impl<T> CsmProjectedField for T
where
    T: CsmEncode + CsmAccum + core::fmt::Debug,
{
    fn encode_field(&self, id: u16, writer: &mut CsmWriter<'static>) {
        self.encode_param(id, writer);
    }

    fn feed_field(&mut self, param: &CsmParam<'static>) {
        self.add_param(param);
    }

    fn prealloc_field(&mut self, size: usize) {
        self.prealloc(size);
    }

    fn debug_field(&self, name: &'static str, debug: &mut core::fmt::DebugStruct<'static, 'static>) {
        debug.field(name, self);
    }
}

fn project_shared_field<S: StringPool>(field: &dyn CsmProjectedField, context: &CsmFieldContext<S>, request: &mut CsmFieldRequest) {
    match request {
        CsmFieldRequest::Encode(writer) => {
            // SAFETY: the writer remains live for this synchronous projection pass.
            let writer = unsafe { &mut **writer };
            field.encode_field(context.id, writer);
        }
        CsmFieldRequest::Debug(debug) => {
            // SAFETY: the builder remains live for this synchronous projection pass.
            let debug = unsafe { &mut **debug };
            field.debug_field(context.name.get(), debug);
        }
        CsmFieldRequest::Feed { .. } | CsmFieldRequest::Prealloc(_) => {}
    }
}

fn project_mut_field<S: StringPool>(field: &mut dyn CsmProjectedField, context: &CsmFieldContext<S>, request: &mut CsmFieldRequest) {
    match request {
        CsmFieldRequest::Feed { param, handled } => {
            // SAFETY: the caller keeps the parameter alive for this projection pass.
            let param = unsafe { &**param };
            if !*handled && param.id == context.id {
                field.feed_field(param);
                *handled = true;
            }
        }
        CsmFieldRequest::Prealloc(reader) => {
            // SAFETY: the caller keeps the reader alive for this projection pass.
            let size = unsafe { (&**reader).count_repeating(context.id) };
            field.prealloc_field(size);
        }
        CsmFieldRequest::Encode(_) | CsmFieldRequest::Debug(_) => {
            project_shared_field(field, context, request);
        }
    }
}

impl<F, S: StringPool> ProjectorCapability<F, CsmFieldContext<S>> for CsmEncodeCapability<S>
where
    F: CsmEncode + CsmAccum + core::fmt::Debug,
{
    type Request = CsmFieldRequest;

    unsafe fn project(field: *mut F, access: FieldAccess, context: &CsmFieldContext<S>, request: &mut Self::Request) {
        match (request, access) {
            (CsmFieldRequest::Encode(writer), _) => {
                // SAFETY: The caller supplies a live writer for this synchronous operation.
                let writer = unsafe { &mut **writer };
                unsafe { (&*field).encode_param(context.id, writer) };
            }
            (CsmFieldRequest::Feed { param, handled }, FieldAccess::Mutable) => {
                // SAFETY: The caller keeps the CsmParam alive while interpreting the table.
                let param = unsafe { &**param };
                if !*handled && param.id == context.id {
                    // SAFETY: The projection table points at the live mutable field.
                    unsafe { (&mut *field).add_param(param) };
                    *handled = true;
                }
            }
            (CsmFieldRequest::Prealloc(reader), FieldAccess::Mutable) => {
                // SAFETY: The caller keeps the reader alive for this operation.
                let reader = unsafe { &**reader };
                let size = reader.count_repeating(context.id);
                // SAFETY: The projection table points at the live mutable field.
                unsafe { (&mut *field).prealloc(size) };
            }
            (CsmFieldRequest::Debug(debug), _) => {
                // SAFETY: The builder outlives this synchronous projection pass.
                let debug = unsafe { &mut **debug };
                // SAFETY: The projection table points at a live field and Debug only reads it.
                unsafe { debug.field(context.name.get(), &*field) };
            }
            (CsmFieldRequest::Feed { .. } | CsmFieldRequest::Prealloc(_), FieldAccess::ReadOnly) => {}
        }
    }
}

impl<T, S: StringPool> ProjectorOptionalCapability<T, CsmFieldContext<S>> for CsmEncodeCapability<S>
where
    Option<T>: CsmEncode + CsmAccum + core::fmt::Debug,
{
    type Request = CsmFieldRequest;

    unsafe fn project_option(field: *mut Option<T>, access: FieldAccess, context: &CsmFieldContext<S>, request: &mut Self::Request) {
        match access {
            FieldAccess::Mutable => {
                // SAFETY: FieldOp supplies a live mutable Option<T> of the checked type.
                project_mut_field(unsafe { &mut *field }, context, request);
            }
            FieldAccess::ReadOnly => {
                // SAFETY: FieldOp supplies a live shared Option<T>; this path only reads it.
                project_shared_field(unsafe { &*field }, context, request);
            }
        }
    }
}

#[doc(hidden)]
pub type CsmFields<S> = catplay_reflect::FieldTable<CsmFieldContext<S>, CsmEncodeProjector<S>>;

#[inline(never)]
fn apply_projected_ref<T, S: StringPool>(value: &T, request: &mut CsmFieldRequest, fields: &CsmFields<S>) {
    for op in fields {
        // SAFETY: Callers use this only with the read-only Encode and Debug requests.
        unsafe { op.apply_ref(value, request) };
    }
}

#[inline(never)]
fn apply_projected_mut<T, S: StringPool>(value: &mut T, request: &mut CsmFieldRequest, fields: &CsmFields<S>) {
    for op in fields {
        // SAFETY: The table matches T and the caller supplies a live mutable value.
        unsafe { op.apply(value, request) };
    }
}

#[doc(hidden)]
pub fn encode_projected_fields<T>(value: &T, writer: &mut CsmWriter<'_>)
where
    T: CsmStructProjected,
{
    let writer = (writer as *mut CsmWriter<'_>).cast::<CsmWriter<'static>>();
    apply_projected_ref(
        value,
        &mut CsmFieldRequest::Encode(writer),
        <T as FieldProject<CsmEncodeProjector<T::StringPool>, CsmFieldContext<T::StringPool>>>::fields(),
    );
}

#[doc(hidden)]
pub fn feed_projected_fields<T>(value: &mut T, param: &CsmParam<'_>)
where
    T: CsmStructProjected,
{
    let param = (param as *const CsmParam<'_>).cast::<CsmParam<'static>>();
    apply_projected_mut(
        value,
        &mut CsmFieldRequest::Feed { param, handled: false },
        <T as FieldProject<CsmEncodeProjector<T::StringPool>, CsmFieldContext<T::StringPool>>>::fields(),
    );
}

#[doc(hidden)]
pub fn prealloc_projected_fields<T>(value: &mut T, reader: &CsmReader<'_>)
where
    T: CsmStructProjected,
{
    let reader = (reader as *const CsmReader<'_>).cast::<CsmReader<'static>>();
    apply_projected_mut(
        value,
        &mut CsmFieldRequest::Prealloc(reader),
        <T as FieldProject<CsmEncodeProjector<T::StringPool>, CsmFieldContext<T::StringPool>>>::fields(),
    );
}

#[doc(hidden)]
pub fn debug_projected_fields<T>(value: &T, type_name: &'static str, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
where
    T: CsmStructProjected,
{
    let mut debug = formatter.debug_struct(type_name);
    let request = (&mut debug as *mut core::fmt::DebugStruct<'_, '_>).cast::<core::fmt::DebugStruct<'static, 'static>>();
    apply_projected_ref(
        value,
        &mut CsmFieldRequest::Debug(request),
        <T as FieldProject<CsmEncodeProjector<T::StringPool>, CsmFieldContext<T::StringPool>>>::fields(),
    );
    debug.finish()
}

impl<T> CsmPayloadEncode for T
where
    T: CsmStructProjected,
{
    fn encode_to_bytes(&self, writer: &mut CsmWriter) {
        encode_projected_fields(self, writer);
    }
}

#[cfg(test)]
mod tests {
    use super::{CsmEncode, CsmFieldContext, CsmStruct, CsmWriter};
    use crate::field_project;
    use catplay_reflect::static_strings;

    static_strings! {
        struct TestStrings(u8) {
            COMMAND = "command",
            PAYLOAD = "payload",
            TOKEN = "token",
            PACKET = "ProjectedPacket",
        }
    }

    field_project! {
        struct ProjectedPacket {
            command: u16,
            payload: Vec<u8>,
            token: Option<u8>,
        }

        project super::CsmEncodeProjector<TestStrings>, context = CsmFieldContext<TestStrings>, pool = u8 {
            command: CsmFieldContext::new(1, TestStrings::COMMAND),
            payload: CsmFieldContext::new(2, TestStrings::PAYLOAD),
            #[csm_count(optional)]
            token: CsmFieldContext::new(3, TestStrings::TOKEN),
        }
    }

    impl super::CsmStructName for ProjectedPacket {
        const STRUCT_NAME: &'static str = "ProjectedPacket";
    }

    impl super::CsmStructProjected for ProjectedPacket {
        type StringPool = TestStrings;
    }

    impl core::fmt::Debug for ProjectedPacket {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            super::debug_projected_fields(self, "ProjectedPacket", formatter)
        }
    }

    #[test]
    fn projected_encode_writes_expected_tlvs() {
        let packet = ProjectedPacket {
            command: 0x1234,
            payload: vec![0xaa, 0xbb],
            token: Some(0xcc),
        };
        let mut encoded = Vec::new();
        CsmWriter::with_vec(&mut encoded, |writer| packet.encode_param(0x4444, writer));

        assert_eq!(
            encoded,
            [
                0x00, 0x19, 0x44, 0x44, // outer packet TLV
                0x00, 0x06, 0x00, 0x01, 0x12, 0x34, // command TLV
                0x00, 0x05, 0x00, 0x02, 0xaa, // first repeated payload TLV
                0x00, 0x05, 0x00, 0x02, 0xbb, // second repeated payload TLV
                0x00, 0x05, 0x00, 0x03, 0xcc, // optional token TLV
            ]
        );
    }

    #[test]
    fn projected_optional_field_preserves_accumulation_and_debug() {
        let mut packet = ProjectedPacket {
            command: 0,
            payload: Vec::new(),
            token: None,
        };
        let param = super::CsmParam::new(3, &[0x42]);
        packet.feed_param(&param);

        assert_eq!(packet.token, Some(0x42));
        assert_eq!(
            format!("{packet:?}"),
            "ProjectedPacket { command: 0, payload: [], token: Some(66) }"
        );
    }
}
