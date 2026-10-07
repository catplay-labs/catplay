/// A closed wire enum with explicitly listed integer values.
#[macro_export]
macro_rules! iap1_enum {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $storage:ident {
            $(#[$first_meta:meta])* $first:ident = $first_value:literal,
            $( $(#[$variant_meta:meta])* $variant:ident = $value:literal, )*
        }
    ) => {
        #[repr($storage)]
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(#[$first_meta])* $first = $first_value,
            $( $(#[$variant_meta])* $variant = $value, )*
        }

        impl ::core::default::Default for $name {
            fn default() -> Self { Self::$first }
        }

        impl $crate::Iap1Enum<$storage> for $name {
            type StringPool = $strings;
            const NAMES: &'static [$strings] = &[
                const { <$strings>::require(stringify!($first)) },
                $( const { <$strings>::require(stringify!($variant)) }, )*
            ];
            const VALUES: &'static [$storage] = &[
                $first_value as $storage,
                $( $value as $storage, )*
            ];

            fn raw_value(&self) -> $storage {
                self.raw()
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(
                    &$crate::Iap1EnumDebug::<_, $storage>(self, ::core::marker::PhantomData),
                    formatter,
                )
            }
        }

        impl $name {
            pub fn raw(self) -> $storage {
                self as $storage
            }
        }

        impl $crate::__NamedDebug for $name {
            fn fmt_named(&self, name: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(name).field(self).finish()
            }
        }

        #[allow(deprecated)]
        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                let raw = $crate::__iap1_decode_enum_raw(
                    reader,
                    <Self as $crate::Iap1Enum<$storage>>::VALUES,
                )?;
                // SAFETY: the shared decoder accepted only a value in VALUES,
                // generated from this repr($storage) enum's exact variants.
                Ok(unsafe { ::core::mem::transmute::<$storage, Self>(raw) })
            }
        }

        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                $crate::Iap1Encode::encode(&self.raw(), writer)
            }
        }

        impl $crate::PredicateValue for $name {
            fn predicate_integer(&self) -> Option<i128> {
                Some(self.raw() as i128)
            }
        }
    };
}
