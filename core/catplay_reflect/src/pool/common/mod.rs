//! Typed pool handles shared by string and object pools.

mod handle;
pub(super) mod index;

/// A compact handle whose type selects a static pool.
///
/// Implemented by both macros. Generic code can resolve different pools
/// without storing a resolver or a base pointer alongside each handle.
/// Rust specializes generic calls for the concrete handle type.
///
/// This is an ordinary trait, not a const trait. Use the generated inherent
/// `get()`, `as_str()`, `to_raw()` and `from_raw()` methods in const contexts.
/// It is intended for static dispatch (`H: StaticHandle`), not `dyn` objects.
///
/// ```
/// use catplay_reflect::{static_objects, static_strings, StaticHandle};
///
/// static_strings! { struct Field(u8) { NAME = "name" } }
/// static_strings! { struct Label(u16) { NAME = "display name" } }
/// static_objects! { struct Number(u8): u32 { ONE = 1 } }
///
/// fn resolve<H: StaticHandle>(handle: H) -> &'static H::Target {
///     handle.get()
/// }
///
/// fn decode<H: StaticHandle>(raw: H::Raw) -> Option<&'static H::Target> {
///     H::from_raw(raw).map(H::get)
/// }
///
/// assert_eq!(resolve(Field::NAME), "name");
/// assert_eq!(resolve(Label::NAME), "display name");
/// assert_eq!(resolve(Number::ONE), &1);
/// assert_eq!(decode::<Field>(1u8), Some("name"));
/// assert_eq!(decode::<Label>(1u16), Some("display name"));
/// ```
pub trait StaticHandle: Copy + 'static {
    /// Statically stored referent: `str` or a sized object type.
    type Target: ?Sized + 'static;
    /// Encoded integer representation, `u8` or `u16` for generated handles.
    /// String u16 handles use direct offsets; other handles use dense IDs.
    type Raw: Copy;

    /// Resolve this handle in the pool selected by its concrete type.
    fn get(self) -> &'static Self::Target;
    /// Read the encoded pool-local ID or offset.
    fn to_raw(self) -> Self::Raw;
    /// Validate the raw encoding against this type's pool.
    fn from_raw(raw: Self::Raw) -> Option<Self>;
}
