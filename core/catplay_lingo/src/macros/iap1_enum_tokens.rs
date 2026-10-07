/// Tagged FID tokens with a wire header on each known variant.
#[macro_export]
macro_rules! iap1_enum_tokens {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            #[iap1_token(fid_type = $first_fid_type:literal, fid_subtype = $first_fid_subtype:literal)]
            $(#[$first_variant_meta:meta])* $first_variant:ident($first_payload:ty),
            $(
                #[iap1_token(fid_type = $fid_type:literal, fid_subtype = $fid_subtype:literal)]
                $(#[$variant_meta:meta])*
                $variant:ident($payload:ty),
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq)]
        $vis enum $name {
            $(#[$first_variant_meta])* $first_variant($first_payload),
            $( $(#[$variant_meta])* $variant($payload), )*
        }

        impl ::core::default::Default for $name {
            fn default() -> Self { Self::$first_variant(::core::default::Default::default()) }
        }

        impl $crate::PredicateValue for $name {
            fn predicate_integer(&self) -> Option<i128> { None }
        }

        #[allow(deprecated)]
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #[allow(non_upper_case_globals)]
                const $first_variant: &str = <$strings>::require_str(stringify!($first_variant));
                $(
                    #[allow(non_upper_case_globals)]
                    const $variant: &str = <$strings>::require_str(stringify!($variant));
                )*
                match self {
                    Self::$first_variant(value) => $crate::__NamedDebug::fmt_named(value, $first_variant, f),
                    $(Self::$variant(value) => $crate::__NamedDebug::fmt_named(value, $variant, f),)*
                }
            }
        }

        #[allow(deprecated)]
        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                $crate::__iap1_decode_tagged_token(
                    reader,
                    const { <$strings>::require(stringify!($name)) },
                    &[
                        $crate::Iap1TokenDecodeOp {
                            fid_type: $first_fid_type,
                            fid_subtype: $first_fid_subtype,
                            decode: |reader| <$first_payload as $crate::Iap1Decode>::decode(reader).map(Self::$first_variant),
                        },
                        $(
                            $crate::Iap1TokenDecodeOp {
                                fid_type: $fid_type,
                                fid_subtype: $fid_subtype,
                                decode: |reader| <$payload as $crate::Iap1Decode>::decode(reader).map(Self::$variant),
                            },
                        )*
                    ],
                )
            }
        }

        #[allow(deprecated)]
        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                match self {
                    Self::$first_variant(value) => writer.write_tagged_token(
                        $first_fid_type,
                        $first_fid_subtype,
                        |writer| $crate::Iap1Encode::encode(value, writer),
                    ),
                    $(
                        Self::$variant(value) => writer.write_tagged_token(
                            $fid_type,
                            $fid_subtype,
                            |writer| $crate::Iap1Encode::encode(value, writer),
                        ),
                    )*
                }
            }
        }
    };
}
