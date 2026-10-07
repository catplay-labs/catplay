use core::marker::PhantomData;

use super::{ConstOffset, FieldAccess, FieldEntry, FieldOffset, FieldProjector};

/// Capability for projecting an optional field while keeping the option
/// container erased from the shared projector path.
///
/// The typed adapter is selected when the field table is built; the runtime
/// executor receives only an erased pointer and the projector request.
pub trait ProjectorOptionalCapability<T, C> {
    type Request;

    /// # Safety
    /// `field` must point to a live `Option<T>` selected by the field table.
    /// In `ReadOnly` mode the implementation must not mutate it or create
    /// mutable references.
    unsafe fn project_option(field: *mut Option<T>, access: FieldAccess, context: &C, request: &mut Self::Request);
}

pub(super) unsafe fn project_option_erased<T, C, P>(field: *mut u8, access: FieldAccess, context: &C, request: &mut P::Request)
where
    P: FieldProjector<C>,
    P::Capability: ProjectorOptionalCapability<T, C, Request = P::Request>,
{
    // SAFETY: the typed offset and callback are created together.
    unsafe { P::Capability::project_option(field.cast::<Option<T>>(), access, context, request) }
}

impl<C, P> FieldEntry<C, P>
where
    P: FieldProjector<C>,
{
    /// Construct a typed operation for an `Option<T>` field of S.
    pub const fn from_optional_offset<S, T, const OFFSET: usize>(offset: FieldOffset<S, Option<T>>, context_id: P::ContextPoolWidth) -> Self
    where
        P::Capability: ProjectorOptionalCapability<T, C, Request = P::Request>,
        P::FieldOffsetWidth: ConstOffset<OFFSET>,
    {
        assert!(offset.byte_offset() == OFFSET, "field_project!: field offset mismatch");
        Self {
            offset: <P::FieldOffsetWidth as ConstOffset<OFFSET>>::VALUE,
            context_id,
            project: project_option_erased::<T, C, P>,
            _projector: PhantomData,
        }
    }
}
