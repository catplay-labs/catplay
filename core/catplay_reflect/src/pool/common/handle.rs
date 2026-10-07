//! Shared expansion for both kinds of pool. No runtime wrapper is introduced.

#[doc(hidden)]
#[macro_export]
macro_rules! __static_pool_handle {
    // Both macros need a local type alias that cannot equal the handle name.
    // Keep caller expressions outside the generated helper scopes.
    (
        @name $pool:ident $settings:tt;
        $(#[doc = $doc:expr])*
        $vis:vis struct __StaticPoolsLocal $($body:tt)*
    ) => {
        $crate::$pool! {
            @resolved __StaticPoolsAlternate $settings
            $(#[doc = $doc])*
            $vis struct __StaticPoolsLocal $($body)*
        }
    };
    (
        @name $pool:ident $settings:tt;
        $(#[doc = $doc:expr])*
        $vis:vis struct $handle:ident $($body:tt)*
    ) => {
        $crate::$pool! {
            @resolved __StaticPoolsLocal $settings
            $(#[doc = $doc])*
            $vis struct $handle $($body)*
        }
    };
    (
        @type $carrier:ident;
        $(#[doc = $doc:expr])*
        $vis:vis struct $handle:ident
    ) => {
        $(#[doc = $doc])*
        #[repr(transparent)]
        #[derive(
            ::core::marker::Copy, ::core::clone::Clone,
            ::core::cmp::PartialEq, ::core::cmp::Eq, ::core::hash::Hash
        )]
        $vis struct $handle($crate::__private::$carrier<Self>);
    };
    (
        impl $handle:ident: $target:ty, $raw:ty, $carrier:ident;
        declarations = $declarations:expr;
        values = $values:expr;
    ) => {
        $crate::__static_pool_handle! {
            @common impl $handle: $target, $raw;
            declarations = $declarations;
        }

        impl $handle {
            /// Zero-based index of the stored value within this pool.
            #[inline]
            pub const fn index(self) -> usize { self.0.get() as usize - 1 }

            /// Validate a dense ID in O(1). Zero and IDs beyond the stored
            /// values return `None`. This accepts IDs, not byte offsets.
            #[inline]
            pub const fn from_raw(raw: $raw) -> ::core::option::Option<Self> {
                if raw != 0 && raw as usize <= $values {
                    // SAFETY: each pool supplies its checked count of stored
                    // values, so this is exactly its valid nonzero ID range.
                    ::core::option::Option::Some(Self(unsafe {
                        $crate::__private::$carrier::<Self>::new_unchecked(raw)
                    }))
                } else {
                    ::core::option::Option::None
                }
            }
        }
    };
    (
        @common impl $handle:ident: $target:ty, $raw:ty;
        declarations = $declarations:expr;
    ) => {
        ::core::assert!(::core::mem::size_of::<$handle>() == ::core::mem::size_of::<$raw>());
        ::core::assert!(::core::mem::size_of::<::core::option::Option<$handle>>() == ::core::mem::size_of::<$raw>());

        impl $handle {
            /// Nonzero pool-local encoding: a dense ID or a direct byte offset.
            /// The encoding depends on this pool's type and declaration layout;
            /// it is not a stable wire ID.
            #[inline]
            pub const fn to_raw(self) -> $raw { self.0.get() }

            /// Number of declarations, including aliases in string pools.
            #[inline]
            pub const fn len() -> usize { $declarations }

            /// Whether this pool has no declarations.
            #[inline]
            pub const fn is_empty() -> bool { Self::len() == 0 }
        }

        impl $crate::StaticHandle for $handle {
            type Target = $target;
            type Raw = $raw;
            #[inline]
            fn get(self) -> &'static Self::Target { Self::get(self) }
            #[inline]
            fn to_raw(self) -> Self::Raw { Self::to_raw(self) }
            #[inline]
            fn from_raw(raw: Self::Raw) -> ::core::option::Option<Self> {
                Self::from_raw(raw)
            }
        }

        impl ::core::ops::Deref for $handle {
            type Target = $target;
            #[inline]
            fn deref(&self) -> &Self::Target { (*self).get() }
        }

        impl ::core::convert::AsRef<$target> for $handle {
            #[inline]
            fn as_ref(&self) -> &$target { (*self).get() }
        }

        impl ::core::convert::From<$handle> for &'static $target {
            #[inline]
            fn from(handle: $handle) -> Self { handle.get() }
        }
    };
}
