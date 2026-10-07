/// Untagged alternatives selected by the enclosing codec's active step.
#[macro_export]
macro_rules! iap1_disjoint {
    (
        strings = $strings:path;
        #[iap1_disjoint(predicate = $predicate:ident)]
        $(#[$meta:meta])* $vis:vis enum $name:ident {
            #[iap1_disjoint(decode = $first_decode:ident, encode = $first_encode:ident)]
            $(#[$first_meta:meta])* $first:ident($first_payload:ty),
            $(
                #[iap1_disjoint(decode = $decode:ident, encode = $encode:ident)]
                $(#[$variant_meta:meta])* $variant:ident($payload:ty),
            )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq)]
        $vis enum $name {
            $(#[$first_meta])* $first($first_payload),
            $( $(#[$variant_meta])* $variant($payload), )+
        }

        impl ::core::default::Default for $name {
            fn default() -> Self { Self::$first(::core::default::Default::default()) }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #[allow(non_upper_case_globals)]
                const $first: &str = <$strings>::require_str(stringify!($first));
                $(
                    #[allow(non_upper_case_globals)]
                    const $variant: &str = <$strings>::require_str(stringify!($variant));
                )+
                match self {
                    Self::$first(value) => $crate::__NamedDebug::fmt_named(value, $first, f),
                    $(Self::$variant(value) => $crate::__NamedDebug::fmt_named(value, $variant, f),)+
                }
            }
        }

        impl $name {
            pub fn $first_decode(
                reader: &mut $crate::Reader<'_>,
                decode: impl FnOnce(&mut $crate::Reader<'_>) -> Result<$first_payload, $crate::DecodeError>,
            ) -> Result<Self, $crate::DecodeError> { decode(reader).map(Self::$first) }
            pub fn $first_encode(
                &self,
                writer: &mut $crate::Writer<'_>,
                field: &'static str,
                encode: impl FnOnce(&$first_payload, &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError>,
            ) -> Result<(), $crate::EncodeError> {
                match self { Self::$first(value) => encode(value, writer), _ => Err($crate::EncodeError::InvalidFieldValue { field }) }
            }
            $(
                pub fn $decode(
                    reader: &mut $crate::Reader<'_>,
                    decode: impl FnOnce(&mut $crate::Reader<'_>) -> Result<$payload, $crate::DecodeError>,
                ) -> Result<Self, $crate::DecodeError> { decode(reader).map(Self::$variant) }
                pub fn $encode(
                    &self,
                    writer: &mut $crate::Writer<'_>,
                    field: &'static str,
                    encode: impl FnOnce(&$payload, &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError>,
                ) -> Result<(), $crate::EncodeError> {
                    match self { Self::$variant(value) => encode(value, writer), _ => Err($crate::EncodeError::InvalidFieldValue { field }) }
                }
            )+

            fn __iap1_decode_projected(
                reader: &mut $crate::Reader<'_>,
                variant: u16,
                wire: $crate::Iap1WireSpec,
                values: &[Option<i128>; 128],
                key: &'static str,
            ) -> Result<Self, $crate::DecodeError> {
                if variant == const { <$strings>::require_raw(stringify!($first_decode)) as u16 } {
                    return Self::$first_decode(reader, |reader| <$first_payload as $crate::Iap1WireValue>::decode_wire(reader, wire, values, key));
                }
                $(if variant == const { <$strings>::require_raw(stringify!($decode)) as u16 } {
                    return Self::$decode(reader, |reader| <$payload as $crate::Iap1WireValue>::decode_wire(reader, wire, values, key));
                })+
                Err($crate::DecodeError::InvalidFieldValue { field: key })
            }

            fn __iap1_encode_projected(
                &self,
                writer: &mut $crate::Writer<'_>,
                variant: u16,
                wire: $crate::Iap1WireSpec,
                values: &[Option<i128>; 128],
                key: &'static str,
            ) -> Result<bool, $crate::EncodeError> {
                match self {
                    Self::$first(value) if variant == const { <$strings>::require_raw(stringify!($first_decode)) as u16 } => <$first_payload as $crate::Iap1WireValue>::encode_wire(value, writer, wire, values, key),
                    $(Self::$variant(value) if variant == const { <$strings>::require_raw(stringify!($decode)) as u16 } => <$payload as $crate::Iap1WireValue>::encode_wire(value, writer, wire, values, key),)+
                    _ => Ok(false),
                }
            }
        }

        impl $crate::Iap1WireValue for $name {
            fn decode_wire(
                reader: &mut $crate::Reader<'_>,
                wire: $crate::Iap1WireSpec,
                values: &[Option<i128>; 128],
                key: &'static str,
            ) -> Result<Self, $crate::DecodeError> {
                let $crate::Iap1WireSpec::Disjoint { variant, child } = wire else {
                    return Err($crate::DecodeError::InvalidFieldValue { field: key });
                };
                Self::__iap1_decode_projected(reader, variant, *child, values, key)
            }

            fn encode_wire(
                &self,
                writer: &mut $crate::Writer<'_>,
                wire: $crate::Iap1WireSpec,
                values: &[Option<i128>; 128],
                key: &'static str,
            ) -> Result<bool, $crate::EncodeError> {
                let $crate::Iap1WireSpec::Disjoint { variant, child } = wire else {
                    return Err($crate::EncodeError::InvalidFieldValue { field: key });
                };
                self.__iap1_encode_projected(writer, variant, *child, values, key)
            }

            fn predicate_integer(&self) -> Option<i128> {
                $crate::__iap1_disjoint_predicate_value!($predicate, self)
            }
        }

        $crate::__iap1_disjoint_predicate!($predicate, $name, [$first, $($variant),+]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_disjoint_predicate {
    (integer, $name:ident, [$($variant:ident),+]) => {
        impl $crate::PredicateValue for $name {
            fn predicate_integer(&self) -> Option<i128> {
                match self { $(Self::$variant(value) => $crate::PredicateValue::predicate_integer(value),)+ }
            }
        }
    };
    (opaque, $name:ident, [$($variant:ident),+]) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_disjoint_predicate_value {
    (integer, $value:ident) => {
        $crate::PredicateValue::predicate_integer($value)
    };
    (opaque, $value:ident) => {
        None
    };
}
