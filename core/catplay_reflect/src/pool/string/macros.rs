/// Define a UTF-8 pool and a distinct one- or two-byte handle type.
///
/// ```
/// use catplay_reflect::static_strings;
///
/// static_strings! {
///     /// A compact field name.
///     pub struct FieldName(u8) {
///         /// The screen field.
///         SCREEN = "screen",
///         AUDIO = concat!("au", "dio"),
///         ALIAS = "audio",
///     }
/// }
///
/// assert_eq!(FieldName::AUDIO, FieldName::ALIAS);
/// assert_eq!(FieldName::SCREEN.as_str(), "screen");
/// assert_eq!(FieldName::len(), 3);        // Declarations, including aliases.
/// assert_eq!(FieldName::unique_len(), 2); // Stored records.
/// assert_eq!(FieldName::lookup("audio"), Some(FieldName::AUDIO));
/// const AUDIO: FieldName = FieldName::resolve("audio").expect("string not in pool");
/// assert_eq!(AUDIO, FieldName::AUDIO);
/// const RAW_AUDIO: u8 = FieldName::require_raw("audio");
/// const AUDIO_TEXT: &str = FieldName::require_str("audio");
/// assert_eq!(RAW_AUDIO, FieldName::AUDIO.to_raw());
/// assert_eq!(AUDIO_TEXT, "audio");
/// assert_eq!(FieldName::lookup("audio\0"), None);
/// assert_eq!(FieldName::storage_bytes(), 13); // b"screen\0audio\0"
/// assert_eq!(FieldName::index_bytes(), 4);    // Two u16 offsets.
/// assert_eq!(core::mem::size_of::<FieldName>(), 1);
/// assert_eq!(core::mem::size_of::<Option<FieldName>>(), 1);
/// ```
///
/// ```compile_fail
/// use catplay_reflect::static_strings;
/// static_strings! { struct Text { PRESENT = "present" } }
/// const MISSING: Text = Text::resolve("missing").expect("string not in pool");
/// ```
///
/// Values may be any const-evaluable `&'static str` expression. Exact duplicates
/// share a record and a handle. Select `(u8)` for a dense ID and at most 255
/// unique strings, or `(u16)` (also the default) for a direct byte offset.
/// Zero is reserved so `Option<Handle>` has the same size as the handle.
///
/// The u8 variant uses a shared u16 offset table: two bytes per unique string.
/// The u16 variant has no offset table; byte zero of its pool is a reserved
/// NUL sentinel and actual records start at offset one. A declared empty
/// string has its own nonzero offset and is distinct from `None`.
/// Each record is UTF-8 followed by NUL; lengths are not stored. The byte pool,
/// including terminators and the u16 sentinel, must fit in 65,536 bytes.
/// A single string can contain at most 65,535 bytes for u8 or 65,534 for u16.
/// Empty strings and Unicode are valid; an embedded NUL is a compile-time error.
/// An empty declaration has no constructible handle; u16 retains its sentinel.
///
/// Access through `as_str()`, `get()`, `Deref`, and formatting scans for the
/// terminator in O(string byte length + 1) time. Const contexts perform that
/// scan during compilation. `offset()` and `from_raw()` are O(1). The dense
/// ordinal returned by `index()` is O(1) for u8, but scans preceding bytes in
/// O(offset) for u16. The u16 getter does not call `index()`.
///
/// The generated type implements `Copy`, `Clone`, `Eq`, `Hash`, `Debug`,
/// `Display`, `Deref<Target = str>`, `AsRef<str>`, and `From<Handle> for &str`.
/// Equality and hashing use pool identity. No lexical ordering or `Borrow<str>`
/// implementation is provided. Use `as_str()` for content-based hashing/order.
///
/// Both type and entry documentation comments are supported. Other attributes
/// belong on the macro invocation or a surrounding module. Entry names should
/// use the normal UPPER_CASE associated-constant convention.
#[macro_export]
macro_rules! static_strings {
    (
        $(#[doc = $type_doc:expr])*
        $vis:vis struct $name:ident (u8) { $($entries:tt)* }
    ) => {
        $crate::__static_pool_handle! {
            @name static_strings (u8, Index8, 0, "catplay_reflect: too many unique strings for u8");
            $(#[doc = $type_doc])*
            $vis struct $name { $($entries)* }
        }
    };
    (
        $(#[doc = $type_doc:expr])*
        $vis:vis struct $name:ident (u16) { $($entries:tt)* }
    ) => {
        $crate::__static_pool_handle! {
            @name static_strings (u16, Index16, 1, "catplay_reflect: too many unique strings for u16");
            $(#[doc = $type_doc])*
            $vis struct $name { $($entries)* }
        }
    };
    (
        $(#[doc = $type_doc:expr])*
        $vis:vis struct $name:ident ($($width:tt)*) { $($entries:tt)* }
    ) => {
        ::core::compile_error!("catplay_reflect: handle width must be u8 or u16");
    };
    (
        $(#[doc = $type_doc:expr])*
        $vis:vis struct $name:ident { $($entries:tt)* }
    ) => {
        $crate::static_strings! {
            $(#[doc = $type_doc])*
            $vis struct $name(u16) { $($entries)* }
        }
    };
    (
        @resolved $local:ident ($raw:ident, $carrier:ident, $prefix:literal, $limit_message:literal)
        $(#[doc = $type_doc:expr])*
        $vis:vis struct $name:ident {
            $(
                $(#[doc = $entry_doc:expr])*
                $entry:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $crate::__static_pool_handle! {
            @type $carrier;
            $(#[doc = $type_doc])*
            $vis struct $name
        }

        const _: () = {
            // The generated type already reserves this value-namespace name.
            // Evaluate caller expressions before introducing any helper names.
            #[allow(non_upper_case_globals)]
            const $name: [&str; <[&str]>::len(&[$(::core::stringify!($entry)),*])] =
                [$($value),*];

            {
                // Preserve the caller's type and input under a local name
                // before helper items can shadow the requested handle name.
                // Dispatch guarantees that this alias differs from $name.
                type $local = $name;
                #[allow(non_upper_case_globals)]
                const $local: [&str; $name.len()] = $name;

                {
                    const __COUNT: usize = $local.len();
                    const __INPUT: [&str; __COUNT] = $local;
                    const __HASH_SLOTS: usize = $crate::__private::string_hash_slots(__COUNT);
                    const __LAYOUT: $crate::__private::StringLayout<{ __COUNT }> =
                        $crate::__private::string_layout::<{ __COUNT }, { __HASH_SLOTS }>(&__INPUT, $prefix);
                    const __LOOKUP: [usize; __HASH_SLOTS] =
                        $crate::__private::string_lookup_table::<{ __COUNT }, { __HASH_SLOTS }>(&__INPUT);
                    ::core::assert!(__LAYOUT.unique_len <= ::core::primitive::$raw::MAX as usize, $limit_message);
                    const __BYTE_LEN: usize = __LAYOUT.byte_len;
                    static __BYTES: [u8; __BYTE_LEN] =
                        $crate::__private::pack_strings(&__INPUT, &__LAYOUT, $prefix);

                    #[allow(non_camel_case_types, dead_code)]
                    enum __Ordinal { $($entry),* }

                    // A const intermediate: it need not be emitted unless all() is used.
                    const __ALL: [$local; __COUNT] = [$($local::$entry),*];

                    $crate::static_strings! {
                        @storage $raw $local $carrier;
                        layout = __LAYOUT;
                        bytes = __BYTES;
                        declarations = __COUNT;
                    }

                    impl $local {
                        $(
                            $(#[doc = $entry_doc])*
                            #[allow(non_upper_case_globals)]
                            pub const $entry: Self = {
                                let raw = if $prefix == 0 {
                                    __LAYOUT.ids[__Ordinal::$entry as usize]
                                } else {
                                    __LAYOUT.offsets[__Ordinal::$entry as usize]
                                } as ::core::primitive::$raw;
                                // SAFETY: layout assigns either a checked dense ID
                                // or an in-bounds nonzero record start for this pool.
                                Self(unsafe { $crate::__private::$carrier::new_unchecked(raw) })
                            };
                        )*

                        /// Scan up to the NUL terminator and return the UTF-8
                        /// bytes before it. O(string byte length + 1) in runtime;
                        /// const contexts perform the scan during compilation.
                        #[inline]
                        pub const fn as_str(self) -> &'static str {
                            let bytes = $crate::__private::record_bytes(
                                &__BYTES, self.offset() as usize,
                            );
                            // SAFETY: the opaque carrier can only be safely obtained
                            // from a named constant, checked from_raw/lookup, or another
                            // valid handle for this very pool. Layout rejects every
                            // embedded NUL; packing copies the complete valid UTF-8
                            // input followed by one terminator. The scan therefore
                            // returns exactly the original string, without the NUL.
                            unsafe { ::core::str::from_utf8_unchecked(bytes) }
                        }

                        /// Uniform accessor shared with object-pool handles.
                        #[inline]
                        pub const fn get(self) -> &'static str { self.as_str() }

                        /// Look up a declared string by exact content, without insertion.
                        /// A linear scan of records and their terminators; const
                        /// calls are resolved during compilation. A query containing
                        /// NUL returns None; it is not truncated at that byte.
                        pub const fn lookup(value: &str) -> ::core::option::Option<Self> {
                            match $crate::__private::lookup_string::<{ $prefix != 0 }>(&__BYTES, value) {
                                // The backend returns an ID for u8 or offset for u16.
                                ::core::option::Option::Some(raw) => Self::from_raw(raw as ::core::primitive::$raw),
                                ::core::option::Option::None => ::core::option::Option::None,
                            }
                        }

                        /// Resolve a string declared in this pool. Use `expect` in a
                        /// const context to fail compilation for a missing key.
                        pub const fn resolve(value: &str) -> ::core::option::Option<Self> {
                            match $crate::__private::lookup_declared(&__INPUT, &__LOOKUP, value) {
                                ::core::option::Option::Some(index) => {
                                    let raw = if $prefix == 0 {
                                        __LAYOUT.ids[index]
                                    } else {
                                        __LAYOUT.offsets[index]
                                    } as ::core::primitive::$raw;
                                    Self::from_raw(raw)
                                }
                                ::core::option::Option::None => ::core::option::Option::None,
                            }
                        }

                        /// Resolve a declared string or fail in a const context.
                        pub const fn require(value: &str) -> Self {
                            Self::resolve(value).expect("string not in pool")
                        }

                        /// Raw encoding of a required string, checked at compile time
                        /// when called from a const expression.
                        pub const fn require_raw(value: &str) -> ::core::primitive::$raw {
                            Self::require(value).to_raw()
                        }

                        /// Text of a required string, checked at compile time when
                        /// called from a const expression.
                        pub const fn require_str(value: &str) -> &'static str {
                            Self::require(value).as_str()
                        }

                        /// Number of distinct stored strings.
                        pub const fn unique_len() -> usize { __LAYOUT.unique_len }
                        /// Byte-pool size including NUL terminators and the u16
                        /// sentinel, excluding the u8 variant's offset table.
                        pub const fn storage_bytes() -> usize { __BYTE_LEN }
                        /// Bytes in the pool and offset table, excluding linker padding
                        /// and the optional all() handle table.
                        pub const fn total_storage_bytes() -> usize {
                            Self::storage_bytes() + Self::index_bytes()
                        }
                        /// Handles in declaration order; aliases appear more than once.
                        /// Using this method materializes a table with the selected
                        /// handle width per declaration.
                        pub const fn all() -> &'static [Self] { &__ALL }
                    }

                    impl ::core::fmt::Display for $local {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            ::core::fmt::Display::fmt(self.as_str(), f)
                        }
                    }
                    impl ::core::fmt::Debug for $local {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            ::core::fmt::Debug::fmt(self.as_str(), f)
                        }
                    }
                }
            }
        };
    };
    (
        @storage u8 $handle:ident $carrier:ident;
        layout = $layout:ident;
        bytes = $bytes:ident;
        declarations = $count:ident;
    ) => {
        static __OFFSETS: [::core::primitive::u16; $layout.unique_len] =
            $crate::__private::string_offsets(&$layout);
        $crate::__static_pool_handle! {
            impl $handle: str, ::core::primitive::u8, $carrier;
            declarations = $count;
            values = $layout.unique_len;
        }
        impl $handle {
            /// Byte offset obtained from this pool's shared u16 table, in O(1).
            #[inline]
            pub const fn offset(self) -> ::core::primitive::u16 { __OFFSETS[self.index()] }

            /// Size of the shared offset table: two bytes per unique string.
            pub const fn index_bytes() -> usize { $layout.unique_len * 2 }
        }
    };
    (
        @storage u16 $handle:ident $carrier:ident;
        layout = $layout:ident;
        bytes = $bytes:ident;
        declarations = $count:ident;
    ) => {
        $crate::__static_pool_handle! {
            @common impl $handle: str, ::core::primitive::u16;
            declarations = $count;
        }
        impl $handle {
            /// Direct nonzero byte offset, in O(1), without a table lookup.
            #[inline]
            pub const fn offset(self) -> ::core::primitive::u16 { self.0.get() }

            /// Zero-based unique-string ordinal in first-occurrence order.
            /// Scans preceding record bytes in O(offset) without a stored index.
            /// Const calls perform this scan during compilation.
            pub const fn index(self) -> usize {
                $crate::__private::string_index(&$bytes, self.offset() as usize)
            }

            /// Validate a direct offset in O(1). Zero, out-of-bounds offsets,
            /// and offsets inside a record return `None`. Empty records are valid.
            #[inline]
            pub const fn from_raw(raw: ::core::primitive::u16) -> ::core::option::Option<Self> {
                if $crate::__private::is_string_offset(&$bytes, raw) {
                    // SAFETY: the check proves a nonzero record boundary in this
                    // exact immutable, NUL-terminated pool, including empty records.
                    ::core::option::Option::Some(Self(unsafe {
                        $crate::__private::$carrier::<Self>::new_unchecked(raw)
                    }))
                } else {
                    ::core::option::Option::None
                }
            }

            /// Direct-offset handles require no offset table.
            pub const fn index_bytes() -> usize { 0 }
        }
    };
    ($($invalid:tt)*) => {
        ::core::compile_error!(
            "expected static_strings! { [visibility] struct Handle(u8 or u16) { NAME = const_expression, ... } }; width may be omitted (u16); only documentation attributes are supported inside the declaration"
        );
    };
}
