/// Defines a static pool of sized objects and a handle with a chosen width.
///
/// A handle stores the object's declaration index plus one. Zero is reserved,
/// so `Option<Handle>` has the same size as the handle. Use `struct Handle(u8)`
/// for one byte and at most 255 objects, or `struct Handle(u16)` for two bytes
/// and at most 65,535 objects. Omitting the width selects `u16` for compatibility.
/// Empty pools are supported. These limits apply to the number of objects,
/// rather than the pool's total byte size.
///
/// Object expressions must be valid static initializers. The object type must
/// be `Sized` and `Sync`, but need not implement `Copy`, `Clone`, `Debug`, `Eq`,
/// or `Hash`. Values are stored directly in an immutable static array, with
/// their natural alignment. Interior mutability is allowed when the type is
/// `Sync`.
///
/// The macro accepts `///` comments and `#[doc = "..."]` attributes on the
/// handle type and each named handle. Other attributes on these declarations
/// are not supported. Apply `#[cfg(...)]` to the complete macro invocation to
/// conditionally include a pool.
///
/// Equality and hashing compare handle identity within the pool, rather than
/// the stored objects' contents. Distinct declarations have distinct handles,
/// even when their values compare equal or are zero-sized. Raw IDs follow
/// declaration order and can change when entries are inserted or reordered.
/// The generated type implements [`StaticHandle`](crate::StaticHandle), so
/// generic code can resolve handles without storing a base pointer in them.
///
/// ```
/// use catplay_reflect::static_objects;
///
/// struct Setting {
///     baud: u32,
///     timeout_ms: u32,
/// }
///
/// static_objects! {
///     /// One of the statically declared serial configurations.
///     struct SettingId(u8): Setting {
///         /// The normal operating configuration.
///         NORMAL = Setting { baud: 115_200, timeout_ms: 100 },
///         RECOVERY = Setting { baud: 9_600, timeout_ms: 1_000 },
///     }
/// }
///
/// const NORMAL: &Setting = SettingId::NORMAL.get();
/// const RECOVERED: Option<SettingId> = SettingId::from_raw(2);
///
/// assert_eq!(NORMAL.baud, 115_200);
/// assert_eq!(SettingId::RECOVERY.timeout_ms, 1_000);
/// assert_eq!(RECOVERED, Some(SettingId::RECOVERY));
/// assert_eq!(SettingId::all(), &[SettingId::NORMAL, SettingId::RECOVERY]);
/// let raw: u8 = SettingId::NORMAL.to_raw();
/// assert_eq!(raw, 1);
/// assert_eq!(core::mem::size_of::<SettingId>(), 1);
/// assert_eq!(core::mem::size_of::<Option<SettingId>>(), 1);
/// ```
#[macro_export]
macro_rules! static_objects {
    (
        $(#[doc = $handle_doc:expr])*
        $vis:vis struct $handle:ident (u8) : $object:ty {
            $($entries:tt)*
        }
    ) => {
        $crate::__static_pool_handle! {
            @name static_objects [u8 Index8];
            $(#[doc = $handle_doc])*
            $vis struct $handle: $object { $($entries)* }
        }
    };
    (
        $(#[doc = $handle_doc:expr])*
        $vis:vis struct $handle:ident (u16) : $object:ty {
            $($entries:tt)*
        }
    ) => {
        $crate::__static_pool_handle! {
            @name static_objects [u16 Index16];
            $(#[doc = $handle_doc])*
            $vis struct $handle: $object { $($entries)* }
        }
    };
    (
        $(#[doc = $handle_doc:expr])*
        $vis:vis struct $handle:ident : $object:ty {
            $($entries:tt)*
        }
    ) => {
        $crate::__static_pool_handle! {
            @name static_objects [u16 Index16];
            $(#[doc = $handle_doc])*
            $vis struct $handle: $object { $($entries)* }
        }
    };
    (
        @resolved $local:ident [$raw:ident $carrier:ident]
        $(#[doc = $handle_doc:expr])*
        $vis:vis struct $handle:ident : $object:ty {
            $(
                $(#[doc = $entry_doc:expr])*
                $name:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $crate::__static_pool_handle! {
            @type $carrier;
            $(#[doc = $handle_doc])*
            $vis struct $handle
        }

        // A separate anonymous scope permits multiple pools in one module
        // without leaking helper items into the caller's namespace.
        const _: () = {
            // Reuse the requested handle's name in the value namespace. Its
            // tuple-struct constructor already reserves that caller name;
            // this introduces no unrelated helper name into the scope in
            // which the caller's type and initializer expressions resolve.
            #[allow(non_upper_case_globals)]
            static $handle: [
                $object;
                <[&str]>::len(&[$(::core::stringify!($name)),*])
            ] = [$($value),*];

            ::core::assert!(
                $handle.len() <= ::core::primitive::$raw::MAX as usize,
                ::core::concat!(
                    "catplay_reflect: too many objects for ",
                    ::core::stringify!($raw),
                ),
            );

            $crate::__static_pool_handle! {
                impl $handle: $object, ::core::primitive::$raw, $carrier;
                declarations = $handle.len();
                values = $handle.len();
            }

            // Keep the enum out of the scope of every caller-supplied type
            // and initializer expression.
            {
                // Preserve the real handle type even if its requested name
                // happens to match the ordinal enum. Dispatch chooses an
                // alias distinct from the requested name.
                type $local = $handle;

                {
                    #[allow(non_camel_case_types, dead_code)]
                    enum __StaticObjectsIndex {
                        $($name),*
                    }

                    impl $local {
                        $(
                            $(#[doc = $entry_doc])*
                            #[allow(non_upper_case_globals)]
                            pub const $name: Self = {
                                let raw = (__StaticObjectsIndex::$name as usize + 1)
                                    as ::core::primitive::$raw;
                                // SAFETY: the checked count fits the selected width,
                                // the discriminant identifies this entry in this
                                // exact pool.
                                Self(unsafe {
                                    $crate::__private::$carrier::<Self>::new_unchecked(raw)
                                })
                            };
                        )*
                    }
                }
            }

            impl $handle {
                /// Returns the statically stored object.
                ///
                /// This only borrows the object; it does not copy or allocate it.
                #[inline]
                pub const fn get(self) -> &'static $object {
                    &$handle[self.index()]
                }

                /// Returns every handle in declaration order.
                ///
                /// Using this method may materialize an additional
                /// `size_of::<Self>()` bytes per handle for the returned table.
                /// Object storage is shared.
                #[inline]
                pub const fn all() -> &'static [Self] {
                    // A constant, rather than another mandatory runtime
                    // static: its backing array is needed when all() is used.
                    #[allow(non_upper_case_globals)]
                    const $local: [$handle; $handle.len()] =
                        [$($handle::$name),*];
                    &$local
                }
            }

            // SAFETY: the static array is checked to fit `$raw`, and LENGTH
            // is derived from that same array before the cast.
            unsafe impl $crate::StaticObjectPool<$object, ::core::primitive::$raw> for $handle {
                const OBJECTS: &'static [$object] = &$handle;
                const LENGTH: ::core::primitive::$raw = $handle.len() as ::core::primitive::$raw;
            }

            impl ::core::fmt::Debug for $handle {
                /// Formats the handle's type name and zero-based index.
                /// The object itself need not implement `Debug`.
                fn fmt(
                    &self,
                    formatter: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    formatter
                        .debug_tuple(::core::stringify!($handle))
                        .field(&self.index())
                        .finish()
                }
            }
        };
    };
    (
        $(#[doc = $handle_doc:expr])*
        $vis:vis struct $handle:ident ($width:ty) : $object:ty {
            $($entries:tt)*
        }
    ) => {
        ::core::compile_error!(
            "catplay_reflect: handle width must be u8 or u16",
        );
    };
    ($($invalid:tt)*) => {
        ::core::compile_error!(
            "expected static_objects! { [visibility] struct Handle(u8 or u16): Type { NAME = const_expression, ... } }; the width is optional and only documentation attributes are supported inside the declaration",
        );
    };
}
