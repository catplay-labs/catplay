use core::borrow::Borrow;

use super::{FieldAccess, FieldEntry, FieldProjector, OffsetWidth};
use crate::{HandleShared, PoolHandle, StaticObjectPool};

/// One static context pool shared by all entries in a field table.
///
/// Fields are read by value so the packed layout never creates an unaligned
/// reference to either field.
#[repr(C, packed)]
pub struct FieldTable<C: 'static, P>
where
    P: FieldProjector<C> + 'static,
{
    entries: &'static [FieldEntry<C, P>],
    shared: HandleShared<P::ContextPoolObject, P::ContextPoolWidth>,
}

impl<C, P> FieldTable<C, P>
where
    P: FieldProjector<C> + 'static,
    P::ContextPoolWidth: PoolHandle,
{
    pub const fn new<Pool: StaticObjectPool<P::ContextPoolObject, P::ContextPoolWidth>>(entries: &'static [FieldEntry<C, P>]) -> Self {
        Self {
            entries,
            shared: HandleShared::new::<Pool>(),
        }
    }

    pub const fn len(&self) -> usize {
        let entries = self.entries;
        entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        let entries = self.entries;
        entries.is_empty()
    }

    pub fn iter(&self) -> FieldOps<'_, C, P> {
        let entries = self.entries;
        FieldOps {
            entries: entries.iter(),
            table: self,
        }
    }
}

/// Operation bound to the context pool of its field table.
pub struct FieldOp<'a, C: 'static, P>
where
    P: FieldProjector<C> + 'static,
{
    entry: &'a FieldEntry<C, P>,
    table: &'a FieldTable<C, P>,
}

impl<C, P> FieldOp<'_, C, P>
where
    P: FieldProjector<C> + 'static,
    P::ContextPoolWidth: PoolHandle,
{
    pub fn context(&self) -> &'static C {
        let shared = self.table.shared;
        shared.resolve(self.entry.context_id).borrow()
    }

    /// # Safety
    /// S must be the exact struct used to create this table.
    pub unsafe fn apply<S>(&self, value: &mut S, request: &mut P::Request) {
        let base = (value as *mut S).cast::<u8>();
        // SAFETY: required from the caller.
        unsafe { self.apply_raw(base, request) }
    }

    /// # Safety
    /// S must match this table and the projector must only read the field.
    pub unsafe fn apply_ref<S>(&self, value: &S, request: &mut P::Request) {
        let base = (value as *const S).cast::<u8>();
        // SAFETY: required from the caller.
        unsafe { self.apply_ref_raw(base, request) }
    }

    /// # Safety
    /// base must point to the live struct used to create this table.
    pub unsafe fn apply_raw(&self, base: *mut u8, request: &mut P::Request) {
        // SAFETY: required from the caller; the typed callback matches the offset.
        let field = unsafe { base.add(self.entry.offset.as_usize()) };
        unsafe { (self.entry.project)(field, FieldAccess::Mutable, self.context(), request) }
    }

    /// # Safety
    /// base must point to the matching live struct, and the callback must only read.
    pub unsafe fn apply_ref_raw(&self, base: *const u8, request: &mut P::Request) {
        // SAFETY: required from the caller; the callback honors ReadOnly.
        let field = unsafe { base.add(self.entry.offset.as_usize()).cast_mut() };
        unsafe { (self.entry.project)(field, FieldAccess::ReadOnly, self.context(), request) }
    }
}

pub struct FieldOps<'a, C: 'static, P>
where
    P: FieldProjector<C> + 'static,
{
    entries: core::slice::Iter<'a, FieldEntry<C, P>>,
    table: &'a FieldTable<C, P>,
}

impl<'a, C, P> Iterator for FieldOps<'a, C, P>
where
    P: FieldProjector<C> + 'static,
    P::ContextPoolWidth: PoolHandle,
{
    type Item = FieldOp<'a, C, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .next()
            .map(|entry| FieldOp { entry, table: self.table })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl<'a, C, P> ExactSizeIterator for FieldOps<'a, C, P>
where
    P: FieldProjector<C> + 'static,
    P::ContextPoolWidth: PoolHandle,
{
}

impl<'a, C, P> IntoIterator for &'a FieldTable<C, P>
where
    P: FieldProjector<C> + 'static,
    P::ContextPoolWidth: PoolHandle,
{
    type Item = FieldOp<'a, C, P>;
    type IntoIter = FieldOps<'a, C, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Implemented once per struct and projector/context combination.
///
/// # Safety
/// Every operation in `FIELDS` must have been created for `Self`, and its
/// offset, field type, and projector callback must describe that same field.
pub unsafe trait FieldProject<P, C>
where
    P: FieldProjector<C> + 'static,
    C: 'static,
{
    const FIELDS: &'static FieldTable<C, P>;

    fn fields() -> &'static FieldTable<C, P> {
        Self::FIELDS
    }
}
