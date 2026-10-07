#[macro_export]
#[cfg(feature = "inventory")]
macro_rules! register_packet_type {
    ($id:expr, $ty:ty) => {
        inventory::submit! {
            $crate::decoder::CsmPacketRegistry::as_registration::<$ty>($id)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! register_packet_family {
    ($($id:literal => $ty:ty),+ $(,)?) => {
        #[cfg(feature = "inventory")]
        inventory::submit! {
            $crate::decoder::CsmPacketRegistry::as_batch_registration(|registry| {
                registry.register_types(
                    &[$(($id, core::any::TypeId::of::<$ty>())),+],
                    |id, data| match id {
                        $($id => match data {
                            Some(data) => <$ty as $crate::decoder::CsmDecode>::decode_from_bytes(data).into(),
                            None => <$ty as core::default::Default>::default().into(),
                        },)+
                        _ => unreachable!("unregistered CSM packet ID {id:#06x}"),
                    },
                );
            })
        }
    };
}

#[macro_export]
macro_rules! packet_type {
    (
        @no_register
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident : $id:tt {
            $($field:tt)*
        }
    ) => {
        $crate::csm_struct! {
            strings = $strings;
            $(#[$meta])*
            $vis struct $name { $($field)* }
        }

        impl $crate::decoder::CsmPacketId for $name {
            const PACKET_ID: u16 = $id;
        }
    };

    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident : $id:tt {
            $($field:tt)*
        }
    ) => {

        $crate::csm_struct! {
            strings = $strings;
            $(#[$meta])*
            $vis struct $name { $($field)* }
        }

        impl $crate::decoder::CsmPacketId for $name {
            const PACKET_ID: u16 = $id;
        }

        #[cfg(feature = "inventory")]
        $crate::register_packet_type!($id, $name);
    };
}

#[macro_export]
macro_rules! group_type {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:tt)*
        }
    ) => {
        $crate::csm_struct! {
            strings = $strings;
            $(#[$meta])*
            $vis struct $name { $($field)* }
        }
    };
}

#[macro_export]
macro_rules! enum_type {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(#[$first_meta:meta])*
            $first_key:ident = $first_val:expr,
            $($(#[$variant_meta:meta])* $key:ident = $val:expr),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(u8)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        $vis enum $name {
            $(#[$first_meta])*
            $first_key = $first_val,
            $($(#[$variant_meta])* $key = $val),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$first_key
            }
        }

        impl core::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                <Self as $crate::decoder::CsmU8Enum>::from_repr(value).ok_or(())
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                <$name as $crate::decoder::CsmU8Enum>::to_repr(value)
            }
        }

        impl $crate::decoder::CsmU8Enum for $name {
            #[inline]
            fn from_repr(value: u8) -> Option<Self> {
                match value {
                    $first_val => Some(Self::$first_key),
                    $($val => Some(Self::$key),)*
                    _ => None,
                }
            }

            #[inline]
            fn to_repr(self) -> u8 {
                self as u8
            }
        }

        impl $crate::decoder::CsmDecode for $name {
            #[inline]
            fn decode_from_bytes(data: &[u8]) -> Self {
                <Self as $crate::decoder::CsmU8Enum>::decode_enum(data)
            }
        }

        impl $crate::decoder::CsmPayloadEncode for $name {
            #[inline]
            fn encode_to_bytes(&self, out: &mut $crate::decoder::CsmWriter) {
                <Self as $crate::decoder::CsmU8Enum>::encode_enum(self, out);
            }

            #[inline]
            fn measure(&self) -> usize {
                <Self as $crate::decoder::CsmU8Enum>::WIRE_SIZE
            }
        }

        impl $crate::decoder::CsmPackedSize for $name {
            const PACKED_SIZE: usize = <Self as $crate::decoder::CsmU8Enum>::WIRE_SIZE;
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __csm_count_type {
    (one, $ty:ty) => { $ty };
    (optional, $ty:ty) => { Option<$ty> };
    (zero_or_more, $ty:ty) => { $crate::decoder::CsmVec<$ty> };
    (one_or_more, $ty:ty) => { $crate::decoder::CsmVec<$ty> };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __csm_packed_type {
    ($ty:ty; dynamic) => { $crate::decoder::CsmPacked<$ty> };
    ($ty:ty; $n:expr) => { $crate::decoder::CsmPackedTight<$ty, $n> };
    ($ty:ty; $head:expr, $($tail:expr),+) => {
        $crate::decoder::CsmPackedTight<
            $crate::__csm_packed_type!($ty; $($tail),+),
            $head
        >
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __csm_storage_type {
    ($count:ident, $ty:ty) => {
        $crate::__csm_count_type!($count, $ty)
    };
    ($count:ident, $ty:ty; $($packed:tt)+) => {
        $crate::__csm_count_type!(
            $count,
            $crate::__csm_packed_type!($ty; $($packed)+)
        )
    };
}

#[macro_export]
macro_rules! csm_struct {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($fields:tt)*
        }
    ) => {
        $crate::__csm_struct_parse! {
            [strings = $strings; $(#[$meta])* $vis struct $name]
            []
            $($fields)*,
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __csm_struct_parse {
    // All fields consumed: emit the shared implementation.
    (
        [strings = $strings:path; $(#[$meta:meta])* $vis:vis struct $name:ident]
        [$($out:tt)*]
        $(,)*
    ) => {
        $crate::__csm_struct_impl! {
            strings = $strings;
            $(#[$meta])*
            $vis struct $name {
                $($out)*
            }
        }
    };

    // A fixed byte sequence maps directly to an array. Match `u8` here,
    // before it is captured as a `ty` fragment by the generic packed arm.
    (
        [$($head:tt)*]
        [$($out:tt)*]
        #[csm_id($id:expr)]
        #[csm_count($count:ident)]
        #[csm_packed($size:literal)]
        $(#[$attrs:meta])*
        $field_vis:vis $field:ident : u8,
        $($rest:tt)*
    ) => {
        $crate::__csm_struct_parse! {
            [$($head)*]
            [
                $($out)*
                #[csm_id($id)]
                #[csm_count($count)]
                $(#[$attrs])*
                $field_vis $field: $crate::__csm_storage_type!(
                    $count, [u8; $size]
                ),
            ]
            $($rest)*
        }
    };

    // Generated field with a packed wire type.
    (
        [$($head:tt)*]
        [$($out:tt)*]
        #[csm_id($id:expr)]
        #[csm_count($count:ident)]
        #[csm_packed($($packed:tt)+)]
        $(#[$attrs:meta])*
        $field_vis:vis $field:ident : $base_ty:ty,
        $($rest:tt)*
    ) => {
        $crate::__csm_struct_parse! {
            [$($head)*]
            [
                $($out)*
                #[csm_id($id)]
                #[csm_count($count)]
                $(#[$attrs])*
                $field_vis $field: $crate::__csm_storage_type!(
                    $count, $base_ty; $($packed)+
                ),
            ]
            $($rest)*
        }
    };

    // Generated field without packing.
    (
        [$($head:tt)*]
        [$($out:tt)*]
        #[csm_id($id:expr)]
        #[csm_count($count:ident)]
        $(#[$attrs:meta])*
        $field_vis:vis $field:ident : $base_ty:ty,
        $($rest:tt)*
    ) => {
        $crate::__csm_struct_parse! {
            [$($head)*]
            [
                $($out)*
                #[csm_id($id)]
                #[csm_count($count)]
                $(#[$attrs])*
                $field_vis $field: $crate::__csm_storage_type!($count, $base_ty),
            ]
            $($rest)*
        }
    };

}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "project")]
macro_rules! __csm_struct_definition {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                #[csm_id($id:expr)]
                #[csm_count($count:ident)]
                $( #[$attrs:meta] )*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $crate::field_project! {
            #[derive(PartialEq, Default, Clone)]
            $(#[$meta])*
            $vis struct $name {
                $( $(#[$attrs])* $field_vis $field: $ty, )*
            }
            project $crate::decoder::CsmEncodeProjector<$strings>, context = $crate::decoder::CsmFieldContext<$strings>, pool = u8 {
                $( #[csm_count($count)] $field: $crate::decoder::CsmFieldContext::<$strings>::new($id, const { <$strings>::require(stringify!($field)) }), )*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "project"))]
macro_rules! __csm_struct_definition {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                #[csm_id($id:expr)]
                #[csm_count($count:ident)]
                $( #[$attrs:meta] )*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, PartialEq, Default, Clone)]
        $(#[$meta])*
        $vis struct $name {
            $( $(#[$attrs])* $field_vis $field : $ty, )*
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "project")]
macro_rules! __csm_struct_payload_encode {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "project"))]
macro_rules! __csm_struct_payload_encode {
    (
        $name:ident {
            $(#[csm_id($id:expr)] $field:ident),* $(,)?
        }
    ) => {
        impl $crate::decoder::CsmPayloadEncode for $name {
            #[allow(unused)]
            fn encode_to_bytes(&self, out: &mut $crate::decoder::CsmWriter) {
                #[allow(unused_imports)]
                use $crate::decoder::CsmEncode;
                $( self.$field.encode_param($id, out); )*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "project")]
macro_rules! __csm_struct_accum_impl {
    (strings = $strings:path; $name:ident { $($ignored:tt)* }) => {
        impl $crate::decoder::CsmStructProjected for $name {
            type StringPool = $strings;
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "project")]
macro_rules! __csm_struct_name {
    (strings = $strings:path; $name:ident) => {
        impl $crate::decoder::CsmStructName for $name {
            const STRUCT_NAME: &'static str = <$strings>::require_str(stringify!($name));
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "project"))]
macro_rules! __csm_struct_name {
    (strings = $strings:path; $name:ident) => {
        impl $crate::decoder::CsmStructName for $name {
            const STRUCT_NAME: &'static str = stringify!($name);
        }
    };
}

// Rust's orphan rules prohibit a blanket `Debug` implementation for all
// projected structs, so each generated type needs this tiny bridge. The
// actual formatting interpreter lives in wire/project.rs.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "project")]
macro_rules! __csm_struct_debug_bridge {
    ($name:ident) => {
        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                $crate::decoder::CsmStructName::fmt_projected_debug(self, formatter)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "project"))]
macro_rules! __csm_struct_accum_impl {
    (
        strings = $strings:path;
        $name:ident {
            $(#[csm_id($id:expr)] $field:ident),* $(,)?
        }
    ) => {
        impl $crate::decoder::CsmStruct for $name {
            #[allow(unused)]
            fn feed_param(&mut self, param: &$crate::decoder::CsmParam) {
                #[allow(unused_imports)]
                use $crate::decoder::CsmAccum;
                match param.id {
                    $( $id => self.$field.add_param(param), )*
                    _ => {}
                }
            }

            #[allow(unused)]
            fn prealloc(&mut self, reader: &$crate::decoder::CsmReader) {
                $(
                    let size = reader.count_repeating($id);
                    $crate::decoder::CsmAccum::prealloc(&mut self.$field, size);
                )*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __csm_struct_impl {
    (
        strings = $strings:path;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                #[csm_id($id:expr)]
                #[csm_count($count:ident)]
                $( #[$attrs:meta] )*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $crate::__csm_struct_definition! {
            strings = $strings;
            $(#[$meta])*
            $vis struct $name {
                $(
                    #[csm_id($id)]
                    #[csm_count($count)]
                    $( #[$attrs] )*
                    $field_vis $field : $ty
                ),*
            }
        }

        $crate::__csm_struct_accum_impl! {
            strings = $strings;
            $name {
                $( #[csm_id($id)] $field, )*
            }
        }

        $crate::__csm_struct_name!(strings = $strings; $name);

        #[cfg(feature = "project")]
        $crate::__csm_struct_debug_bridge!($name);

        $crate::__csm_struct_payload_encode! {
            $name {
                $( #[csm_id($id)] $field, )*
            }
        }

        impl AsRef<dyn $crate::decoder::CsmPacket> for $name {
            fn as_ref(&self) -> &dyn $crate::decoder::CsmPacket {
                self
            }
        }
    };
}
