/// Bitflags with iAP1 scalar codec and predicate support.
#[macro_export]
macro_rules! iap1_bitflags {
    (
        strings = $strings:path;
        $(#[$($meta:tt)*])*
        $vis:vis struct $name:ident: $storage:ident $(, $endian:ident)? {
            $( $(#[$($flag_meta:tt)*])* const $flag:ident = $value:expr; )*
        }
    ) => {
        $crate::__bitflags::bitflags! {
            $(#[$($meta)*])*
            #[derive(Clone, Copy, Default, PartialEq, Eq)]
            $vis struct $name: $storage {
                $( $(#[$($flag_meta)*])* const $flag = $value; )*
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                $crate::fmt_iap1_bitflags::<$storage>(
                    self.bits() as u64,
                    Self::all().bits() as u64,
                    $storage::BITS,
                    &[$(const { <$strings>::require_str(stringify!($flag)) }),*],
                    &[$(($value) as $storage),*],
                    f,
                )
            }
        }

        impl $crate::__NamedDebug for $name {
            fn fmt_named(&self, name: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(name).field(self).finish()
            }
        }

        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                Ok(Self::from_bits_retain($crate::__iap1_bit_storage_decode!(reader, $storage $(, $endian)?)?))
            }
        }

        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                $crate::__iap1_bit_storage_encode!(self.bits(), writer, $storage $(, $endian)?)
            }
        }

        impl $crate::PredicateValue for $name {
            fn predicate_integer(&self) -> Option<i128> {
                Some(self.bits() as i128)
            }
        }
    };
}

/// Storage policy for numeric fields versus byte-indexed Data masks.
#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_bit_storage_decode {
    ($reader:expr, $storage:ident) => {
        <$storage as $crate::Iap1Decode>::decode($reader)
    };
    ($reader:expr, $storage:ident, little_endian) => {
        $reader
            .read_bytes(::core::mem::size_of::<$storage>())
            .map(|bytes| $storage::from_le_bytes(bytes.try_into().expect("read_bytes checked storage width")))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_bit_storage_encode {
    ($value:expr, $writer:expr, $storage:ident) => {
        $crate::Iap1Encode::encode(&$value, $writer)
    };
    ($value:expr, $writer:expr, $storage:ident, little_endian) => {{
        $writer.write_bytes(&$value.to_le_bytes());
        Ok::<(), $crate::EncodeError>(())
    }};
}
