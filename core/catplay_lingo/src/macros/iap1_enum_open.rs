/// A wire enum with named values and one or more permitted ranges for `Other(raw)`.
#[macro_export]
macro_rules! iap1_enum_open {
    (
        strings = $strings:path;
        #[iap1_enum_open(storage = $storage:ident, ranges = [$($min:literal ..= $max:literal),+ $(,)?])]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(#[$first_meta:meta])* $first:ident = $first_value:literal,
            $( $(#[$variant_meta:meta])* $variant:ident = $value:literal, )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(#[$first_meta])* $first,
            $( $(#[$variant_meta])* $variant, )*
            Other($storage),
        }

        impl ::core::default::Default for $name {
            fn default() -> Self { Self::$first }
        }

        impl $crate::Iap1Enum<$storage> for $name {
            type StringPool = $strings;
            const NAMES: &'static [$strings] = &[
                const { <$strings>::require(stringify!($first)) },
                $(const { <$strings>::require(stringify!($variant)) },)*
            ];
            const VALUES: &'static [$storage] = &[
                $first_value as $storage,
                $($value as $storage,)*
            ];

            fn raw_value(&self) -> $storage { self.raw() }

            fn is_other(&self) -> bool { matches!(self, Self::Other(_)) }

            fn fmt_unknown(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    Self::Other(raw) => formatter.debug_tuple("Other").field(raw).finish(),
                    _ => formatter.write_str("<unknown>"),
                }
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

        impl $crate::__NamedDebug for $name {
            fn fmt_named(&self, name: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(name).field(self).finish()
            }
        }

        #[allow(deprecated)]
        impl $name {
            pub fn raw(self) -> $storage {
                match self {
                    Self::$first => $first_value,
                    $( Self::$variant => $value, )*
                    Self::Other(raw) => raw,
                }
            }
        }

        #[allow(deprecated)]
        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                let raw = <$storage as $crate::Iap1Decode>::decode(reader)?;
                match raw {
                    $first_value => Ok(Self::$first),
                    $( $value => Ok(Self::$variant), )*
                    _ if $(($min as i128..=$max as i128).contains(&(raw as i128)))||+ => Ok(Self::Other(raw)),
                    _ => Err($crate::DecodeError::InvalidEnumValue { value: raw as u64 }),
                }
            }
        }

        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                if let Self::Other(raw) = self
                    && !($(($min as i128..=$max as i128).contains(&(*raw as i128)))||+)
                {
                    return Err($crate::EncodeError::InvalidEnumValue { value: *raw as u64 });
                }
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
