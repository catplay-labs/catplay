use catplay_reflect::{FieldAccess, FieldProject, FieldProjector, ProjectorCapability, field_project};

struct CsmEncodeProjector;
struct CsmAccumProjector;
struct CsmEncodeCapability;
struct CsmAccumCapability;

#[derive(Default, Debug, PartialEq)]
struct MockCsmWriter(Vec<(u8, String)>);

trait MockCsmEncode {
    fn encode_field(&self, id: u8, writer: &mut MockCsmWriter);
}

trait MockCsmAccum {}

impl MockCsmEncode for u16 {
    fn encode_field(&self, id: u8, writer: &mut MockCsmWriter) {
        writer.0.push((id, self.to_string()));
    }
}
impl MockCsmEncode for Vec<u8> {
    fn encode_field(&self, id: u8, writer: &mut MockCsmWriter) {
        writer.0.push((id, format!("{:?}", self)));
    }
}
impl MockCsmAccum for Vec<u8> {}

impl FieldProjector<u8> for CsmEncodeProjector {
    type Request = MockCsmWriter;
    type Capability = CsmEncodeCapability;
    type ContextPoolWidth = u16;
    type ContextPoolObject = u8;
    type FieldOffsetWidth = u32;
}
impl FieldProjector<u8> for CsmAccumProjector {
    type Request = ();
    type Capability = CsmAccumCapability;
    type ContextPoolWidth = u16;
    type ContextPoolObject = u8;
    type FieldOffsetWidth = u32;
}

impl<F: MockCsmEncode> ProjectorCapability<F, u8> for CsmEncodeCapability
where
    F: MockCsmEncode,
{
    type Request = MockCsmWriter;

    unsafe fn project(field: *mut F, _: FieldAccess, id: &u8, writer: &mut MockCsmWriter) {
        // SAFETY: FieldOp supplies a pointer to the selected, live F field.
        unsafe { (&*field).encode_field(*id, writer) };
    }
}
impl<F: MockCsmAccum> ProjectorCapability<F, u8> for CsmAccumCapability {
    type Request = ();

    unsafe fn project(_: *mut F, _: FieldAccess, _: &u8, _: &mut ()) {}
}

trait CsmEncode {
    fn encode(&mut self, writer: &mut MockCsmWriter);
}

impl<T> CsmEncode for T
where
    T: FieldProject<CsmEncodeProjector, u8>,
{
    fn encode(&mut self, writer: &mut MockCsmWriter) {
        for op in T::FIELDS {
            // SAFETY: the FieldProject table is generated specifically for T.
            unsafe { op.apply(self, writer) };
        }
    }
}

field_project! {
    pub struct Packet {
        command: u16,
        payload: Vec<u8>,
    }

    project CsmEncodeProjector, context = u8 {
        command: 1,
        payload: 2,
    }

    project CsmAccumProjector, context = u8 {
        payload: 2,
    }
}

fn main() {
    let mut packet = Packet {
        command: 7,
        payload: vec![1, 2, 3],
    };
    let mut writer = MockCsmWriter::default();
    packet.encode(&mut writer);

    assert_eq!(writer.0, [(1, "7".into()), (2, "[1, 2, 3]".into())]);

    let _: &catplay_reflect::FieldTable<u8, CsmAccumProjector> = <Packet as FieldProject<CsmAccumProjector, u8>>::FIELDS;
}
