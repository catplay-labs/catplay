//! Type-erased projection of homogeneous containers such as `Vec<T>`.

/// Typed operation for one item in a homogeneous container.
///
/// `apply_container_erased` owns the shared iteration loop; this capability
/// supplies only the type-specific operation for each item.
pub trait ProjectorContainerCapability<T, C = ()> {
    type Request;
    type Error;

    /// # Safety
    /// `item` must point to a live, correctly aligned `T` for this call.
    unsafe fn project_item(item: *const T, context: &C, request: &mut Self::Request) -> Result<(), Self::Error>;
}

/// The erased, type-specific callback accepted by `apply_container_erased`.
pub type ErasedContainerItem<C, R, E> = unsafe fn(*const (), usize, &C, &mut R) -> Result<(), E>;

/// Shared container iteration loop. The item callback remains checked and
/// typed when it is constructed by `project_container_item`.
///
/// # Safety
/// `items` must point to `len` consecutive live items of the type paired with
/// `project_item`; `context` and `request` must satisfy that callback's contract.
pub unsafe fn apply_container_erased<C, R, E>(
    items: *const (),
    len: usize,
    context: &C,
    request: &mut R,
    project_item: ErasedContainerItem<C, R, E>,
) -> Result<(), E> {
    for index in 0..len {
        // SAFETY: the caller pairs `items` with this callback and supplies the
        // matching length; the callback validates and projects each item.
        unsafe { project_item(items, index, context, request)? };
    }
    Ok(())
}

/// Adapter from a typed container item to the erased shared iteration loop.
pub unsafe fn project_container_item<T, C, P>(
    items: *const (),
    index: usize,
    context: &C,
    request: &mut <P as ProjectorContainerCapability<T, C>>::Request,
) -> Result<(), <P as ProjectorContainerCapability<T, C>>::Error>
where
    P: ProjectorContainerCapability<T, C>,
{
    // SAFETY: `apply_container_erased` pairs this adapter with a `T` slice and
    // iterates only indices smaller than its length.
    let item = unsafe { &*items.cast::<T>().add(index) };
    // SAFETY: the adapter derived `item` from the paired typed container.
    unsafe { P::project_item(item, context, request) }
}
