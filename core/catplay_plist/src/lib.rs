use std::io::{self, Write};

use bytes::BytesMut;
use catplay_util::ArcBox;
use log::trace;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod caching_serializer;
mod flex_bool;
mod plist_byte_array;
mod pretty_plist;
pub use caching_serializer::CachingSerializer;
pub use flex_bool::FlexBool;
pub use plist_byte_array::PlistByteArray;
pub use pretty_plist::{PLIST_PRETTY_DATA_LIMIT, PrettyPlistValue, pretty_plist_value, pretty_plist_value_with_limit};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PlistError {
    #[error("attempted to decode bplist from empty buffer")]
    Empty,
    #[error("bplist error: {0}")]
    Bplist(ArcBox<plist::Error>),
    #[error("unexpected state")]
    UnexpectedState,
}

impl From<plist::Error> for PlistError {
    fn from(value: plist::Error) -> Self {
        Self::Bplist(value.into())
    }
}
pub type PlistResult<T> = Result<T, PlistError>;

fn plist_decode<S: DeserializeOwned + std::fmt::Debug>(data: &[u8]) -> PlistResult<S> {
    if data.is_empty() {
        return Err(PlistError::Empty);
    }

    #[cfg(debug_assertions)]
    if log::log_enabled!(log::Level::Trace) {
        match plist::from_bytes::<plist::Value>(data) {
            Ok(raw) => trace!("raw plist data (any undocumented extra keys?):\n{}", pretty_plist_value(&raw)),
            Err(err) => trace!("raw plist parse failed: {err:?}"),
        }
    }
    let decoded = plist::from_bytes::<S>(data)?;

    #[cfg(debug_assertions)]
    trace!("decoded plist data: \n{decoded:?}");

    Ok(decoded)
}

struct BytesMutWriter<'a> {
    buf: &'a mut BytesMut,
}

impl<'a> Write for BytesMutWriter<'a> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn plist_encode<S: Serialize>(s: &S) -> PlistResult<BytesMut> {
    let mut buf = BytesMut::new();
    plist_encode_into(s, &mut buf)?;
    Ok(buf)
}

fn plist_encode_into<S: Serialize>(s: &S, buf: &mut BytesMut) -> PlistResult<()> {
    let mut w = BytesMutWriter { buf };
    plist::to_writer_binary(&mut w, s)?;
    Ok(())
}

pub trait PlistSerializable: Sized {
    fn pdecode(data: &[u8]) -> PlistResult<Self>;
    fn pencode(&self) -> PlistResult<BytesMut>;
    fn pencode_into(&self, buf: &mut BytesMut) -> PlistResult<()>;
}

impl<S: Serialize + DeserializeOwned + std::fmt::Debug> PlistSerializable for S {
    fn pdecode(data: &[u8]) -> PlistResult<Self> {
        plist_decode(data)
    }

    fn pencode(&self) -> PlistResult<BytesMut> {
        plist_encode(&self)
    }

    fn pencode_into(&self, buf: &mut BytesMut) -> PlistResult<()> {
        plist_encode_into(&self, buf)
    }
}

pub mod libs {
    pub mod serde_with {
        pub use serde_with::*;
    }
    pub mod serde {
        pub use serde::*;
    }

    pub mod serde_repr {
        pub use serde_repr::*;
    }
}

pub use plist::*;

#[macro_export]
macro_rules! plist_struct {
    // ===== Braced struct =====
    (
        $vis:vis struct $name:ident $(<$($gen:tt),*>)? { $($body:tt)* }
    ) => {

        #[derive(Debug, Clone, $crate::libs::serde::Serialize, $crate::libs::serde::Deserialize, Default, PartialEq)]
        #[$crate::libs::serde_with::skip_serializing_none]
        #[serde(rename_all = "camelCase")]
        $vis struct $name $(<$($gen),*>)? { $($body)* }
    };

    // ===== Tuple struct =====
    (
        $vis:vis struct $name:ident $(<$($gen:tt),*>)? ( $($body:tt)* );
    ) => {
        #[derive(Debug, Clone, $crate::libs::serde::Serialize, $crate::libs::serde::Deserialize, Default, PartialEq)]
        #[$crate::libs::serde_with::skip_serializing_none]
        #[serde(rename_all = "camelCase")]
        $vis struct $name $(<$($gen),*>)? ( $($body)* );
    };
}

#[macro_export]
macro_rules! plist_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident $(<$($gen:tt),*>)? $body:tt
    ) => {
        #[derive(Debug, Clone, Copy, $crate::libs::serde::Serialize, $crate::libs::serde::Deserialize, PartialEq, Eq, Default)]
        #[serde(rename_all = "camelCase")]
        $(#[$meta])*
        $vis enum $name $(<$($gen),*>)? $body
    };
}

#[macro_export]
macro_rules! plist_enum_untagged {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident $(<$($gen:tt),*>)? $body:tt
    ) => {
        #[derive(Debug, Clone, $crate::libs::serde::Serialize, $crate::libs::serde::Deserialize, PartialEq, Default)]
        #[serde(untagged)]
        $(#[$meta])*
        $vis enum $name $(<$($gen),*>)? $body
    };
}

#[macro_export]
macro_rules! plist_enum_repr {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident $(<$($gen:tt),*>)? $body:tt
    ) => {
        #[derive(Debug, Clone, Copy, $crate::libs::serde_repr::Serialize_repr, $crate::libs::serde_repr::Deserialize_repr, PartialEq, Eq, Default, Hash)]
        $(#[$meta])*
        $vis enum $name $(<$($gen),*>)? $body
    };
}

#[macro_export]
macro_rules! plist_bitflags {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident : $ty:ty {
            $($body:tt)*
        }
    ) => {
        bitflags::bitflags! {
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
            $vis struct $name : $ty {
                $($body)*
            }
        }

        impl $crate::libs::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                <$ty as serde::Serialize>::serialize(&self.bits(), serializer)
            }
        }

        impl<'de> $crate::libs::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: $crate::libs::serde::Deserializer<'de>,
            {
                let bits = <$ty>::deserialize(deserializer)?;
                Ok(Self::from_bits_truncate(bits))
            }
        }
    };
}

pub mod u64_as_i64 {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: i64 = Deserialize::deserialize(deserializer)?;
        Ok(v as u64)
    }

    pub fn serialize<S>(x: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(*x as i64)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PlistByteArray, plist_decode, plist_encode};

    plist_struct! {
         struct InfoMessageResponse {
            pub oem_icon: Option<PlistByteArray>,
        }
    }

    #[test]
    pub fn test() {
        let mut b = InfoMessageResponse::default();
        b.oem_icon
            .replace(PlistByteArray::from(vec![1, 2, 3, 4, 5]));

        let a = plist_encode(&b).unwrap();
        let x: InfoMessageResponse = plist_decode(&a).unwrap();
        assert_eq!(x, b);
    }
}
