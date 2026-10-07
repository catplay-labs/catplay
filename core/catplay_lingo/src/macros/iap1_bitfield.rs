/// A raw bitfield with optional independent flags and one or more typed bitgroups.
#[macro_export]
macro_rules! iap1_bitfield {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident: $storage:ident $(, $endian:ident)? {
            $(
                #[flags(mask = $flags_mask:literal)]
                $flags_vis:vis $flags_field:ident: $flags_ty:ident,
            )?
            $(
                #[group(mask = $group_mask:literal, shift = $shift:literal)]
                $(#[$group_meta:meta])*
                $group_vis:vis $group_field:ident: $group_ty:ident {
                    $( $(#[$variant_meta:meta])* $variant:ident = $value:literal, )*
                },
            )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $name {
            $( $flags_vis $flags_field: $flags_ty, )?
            $( $(#[$group_meta])* $group_vis $group_field: $group_ty, )+
            /// Bits outside the flags and bitgroup masks.
            pub reserved_bits: $storage,
        }

        #[allow(deprecated)]
        impl $name {
            pub const KNOWN_MASK: $storage = 0 $(| $flags_mask)? $(| $group_mask)+;

            pub fn new($($flags_field: $flags_ty,)? $($group_field: $group_ty),+) -> Self {
                Self {
                    $($flags_field,)?
                    $($group_field,)+
                    reserved_bits: 0,
                }
            }

            pub fn from_raw(raw: $storage) -> Self {
                Self {
                    $($flags_field: $flags_ty::from_bits_retain(raw & $flags_mask),)?
                    $(
                        $group_field: match (raw & $group_mask) >> $shift {
                            $( $value => $group_ty::$variant, )*
                            other => $group_ty::Other(other),
                        },
                    )+
                    reserved_bits: raw & !Self::KNOWN_MASK,
                }
            }

            pub fn raw(self) -> $storage {
                let mut raw = self.reserved_bits & !Self::KNOWN_MASK;
                $(raw |= self.$flags_field.bits() & $flags_mask;)?
                $(raw |= (self.$group_field.raw() << $shift) & $group_mask;)+
                raw
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::from_raw(0)
            }
        }

        impl $crate::__NamedDebug for $name {
            fn fmt_named(&self, name: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(name).field(self).finish()
            }
        }

        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                Ok(Self::from_raw($crate::__iap1_bit_storage_decode!(reader, $storage $(, $endian)?)?))
            }
        }

        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                $crate::__iap1_bit_storage_encode!(self.raw(), writer, $storage $(, $endian)?)
            }
        }

        impl $crate::PredicateValue for $name {
            fn predicate_integer(&self) -> Option<i128> {
                Some(self.raw() as i128)
            }
        }

        $(
            $(#[$group_meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            $vis enum $group_ty {
                $( $(#[$variant_meta])* $variant, )*
                Other($storage),
            }

            #[allow(deprecated)]
            impl $group_ty {
                pub fn raw(self) -> $storage {
                    match self {
                        $( Self::$variant => $value, )*
                        Self::Other(raw) => raw,
                    }
                }
            }
        )+
    };
}
