use core::borrow::Borrow;

use super::OffsetWidth;
use crate::PoolHandle;

/// Identifies an operation and its interpreter request type.
pub trait FieldProjector<C: 'static> {
    type Request;
    type Capability;
    /// Pool handle stored in each operation; it must match the generated pool width.
    type ContextPoolWidth: PoolHandle;
    /// Object stored by `static_objects!`: `C` for direct storage or
    /// `&'static C` to share identical contexts between pools.
    type ContextPoolObject: Borrow<C> + 'static;
    /// Width of a field's byte offset within the projected struct. `u32`
    /// supports large structs; `u16` saves two bytes per packed table entry
    /// but rejects projected fields beyond byte offset 65,535.
    type FieldOffsetWidth: OffsetWidth;
}
/// Access mode used when an erased field operation is invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldAccess {
    ReadOnly,
    Mutable,
}

/// The selected field must satisfy a projector-specific capability.
///
/// Implement this for a local capability type, with a bound on `F` such as
/// `F: CsmEncode`. Different projectors can require different traits.
pub trait ProjectorCapability<F, C> {
    type Request;

    /// # Safety
    /// `field` must point to a live, correctly aligned value of `F`. When
    /// `access` is `ReadOnly`, the implementation must only create shared
    /// references and must not mutate the field.
    unsafe fn project(field: *mut F, access: FieldAccess, context: &C, request: &mut Self::Request);
}
