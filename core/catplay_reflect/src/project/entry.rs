use core::marker::PhantomData;

use super::{ConstOffset, FieldAccess, FieldOffset, FieldProjector, ProjectorCapability};

/// Compact operation stored in a field table.
#[repr(C, packed)]
pub struct FieldEntry<C: 'static, P>
where
    P: FieldProjector<C>,
{
    pub(super) offset: P::FieldOffsetWidth,
    pub(super) context_id: P::ContextPoolWidth,
    pub(super) project: unsafe fn(*mut u8, FieldAccess, &C, &mut P::Request),
    pub(super) _projector: PhantomData<fn() -> P>,
}

unsafe fn project_erased<F, C, P>(field: *mut u8, access: FieldAccess, context: &C, request: &mut P::Request)
where
    P: FieldProjector<C>,
    P::Capability: ProjectorCapability<F, C, Request = P::Request>,
{
    // SAFETY: the typed offset and callback are created together.
    unsafe { P::Capability::project(field.cast::<F>(), access, context, request) }
}
impl<C, P> FieldEntry<C, P>
where
    P: FieldProjector<C>,
{
    /// Construct a typed operation for a field of S.
    pub const fn from_offset<S, F, const OFFSET: usize>(offset: FieldOffset<S, F>, context_id: P::ContextPoolWidth) -> Self
    where
        P::Capability: ProjectorCapability<F, C, Request = P::Request>,
        P::FieldOffsetWidth: ConstOffset<OFFSET>,
    {
        assert!(offset.byte_offset() == OFFSET, "field_project!: field offset mismatch");
        Self {
            offset: <P::FieldOffsetWidth as ConstOffset<OFFSET>>::VALUE,
            context_id,
            project: project_erased::<F, C, P>,
            _projector: PhantomData,
        }
    }
}
