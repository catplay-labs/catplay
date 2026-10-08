#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "serde")]
use alloc::string::{String, ToString};
#[cfg(feature = "serde")]
use bytes::BytesMut;
#[cfg(feature = "serde")]
use core::fmt;
#[cfg(feature = "serde")]
use log::trace;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub mod bplist;
#[cfg(feature = "serde")]
mod caching_serializer;
#[cfg(feature = "serde")]
mod flex_bool;
#[cfg(feature = "alloc")]
mod plist_byte_array;
#[cfg(feature = "alloc")]
mod pretty_plist;
#[cfg(feature = "alloc")]
mod types;
pub use bplist::wire::xml::encode::Options as XmlWriteOptions;
#[cfg(feature = "serde")]
pub use caching_serializer::CachingSerializer;
#[cfg(feature = "serde")]
pub use flex_bool::FlexBool;
#[cfg(feature = "alloc")]
pub use plist_byte_array::PlistByteArray;
#[cfg(feature = "alloc")]
pub use pretty_plist::{PLIST_PRETTY_DATA_LIMIT, PrettyPlistValue, pretty_plist_value, pretty_plist_value_with_limit};
#[cfg(feature = "serde")]
pub use types::from_value;
#[cfg(feature = "alloc")]
pub use types::{Date, Dictionary, Error, Integer, InvalidXmlDate, Uid, Value};

#[cfg(feature = "serde")]
pub(crate) const PLIST_DATE: &str = "PLIST-DATE";
#[cfg(feature = "serde")]
pub(crate) const PLIST_UID: &str = "PLIST-UID";

#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PlistError {
    #[error("attempted to decode bplist from empty buffer")]
    Empty,
    #[error("unexpected state")]
    UnexpectedState,
    #[error("bplist encode error: {0}")]
    Encode(String),
    #[error("bplist decode error: {0}")]
    Decode(String),
}

#[cfg(feature = "serde")]
pub type PlistResult<T> = Result<T, PlistError>;

#[cfg(feature = "serde")]
fn plist_decode<S: DeserializeOwned + fmt::Debug>(data: &[u8]) -> PlistResult<S> {
    if data.is_empty() {
        return Err(PlistError::Empty);
    }

    #[cfg(debug_assertions)]
    if log::log_enabled!(log::Level::Trace) {
        match from_bytes::<Value>(data) {
            Ok(raw) => trace!("raw plist data (any undocumented extra keys?):\n{}", pretty_plist_value(&raw)),
            Err(err) => trace!("raw plist parse failed: {err:?}"),
        }
    }
    let decoded = from_bytes::<S>(data)?;

    #[cfg(debug_assertions)]
    trace!("decoded plist data: \n{decoded:?}");

    Ok(decoded)
}

#[cfg(feature = "serde")]
fn plist_encode<S: Serialize>(s: &S) -> PlistResult<BytesMut> {
    let mut buf = BytesMut::new();
    plist_encode_into(s, &mut buf)?;
    Ok(buf)
}

#[cfg(feature = "serde")]
fn plist_encode_into<S: Serialize>(s: &S, buf: &mut BytesMut) -> PlistResult<()> {
    bplist::ScratchBuffers::default().encode_into(s, buf)
}

#[cfg(feature = "serde")]
pub trait PlistSerializable: Sized {
    fn pdecode(data: &[u8]) -> PlistResult<Self>;
    fn pencode(&self) -> PlistResult<BytesMut>;
    fn pencode_into(&self, buf: &mut BytesMut) -> PlistResult<()>;
}

#[cfg(feature = "serde")]
impl<S: Serialize + DeserializeOwned + fmt::Debug> PlistSerializable for S {
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
/// Decodes binary and XML plists with the local wire readers.
pub fn from_bytes<T: DeserializeOwned>(data: &[u8]) -> PlistResult<T> {
    if data.is_empty() {
        return Err(PlistError::Empty);
    }
    if data.starts_with(b"bplist00") {
        bplist::serde::decode::from_slice(data).map_err(|err| PlistError::Decode(err.to_string()))
    } else if data
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(b'<')
        || data.strip_prefix(b"\xef\xbb\xbf").is_some_and(|tail| {
            tail.iter()
                .copied()
                .find(|byte| !byte.is_ascii_whitespace())
                == Some(b'<')
        })
    {
        from_xml_bytes(data).map_err(|err| PlistError::Decode(err.to_string()))
    } else {
        Err(PlistError::Decode("unsupported plist format".into()))
    }
}

#[cfg(feature = "serde")]
/// Encodes binary plist into a caller-owned sink.
pub fn to_writer_binary<W: bplist::wire::encode::Emit, T: Serialize + ?Sized>(writer: &mut W, value: &T) -> PlistResult<()>
where
    W::Error: core::error::Error + Send + Sync + 'static,
{
    bplist::ScratchBuffers::default().encode_to_writer(value, writer)
}

#[cfg(feature = "serde")]
/// Converts a serializable value to our owned plist representation.
pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, Error> {
    bplist::serde::encode::to_value(value).map_err(|err| Error::message(err.to_string()))
}

#[cfg(feature = "serde")]
pub fn from_xml_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    let value = bplist::serde::xml::from_slice(bytes)?;
    from_value(&value)
}

#[cfg(feature = "serde")]
pub fn to_writer_xml<W: bplist::wire::encode::Emit, T: Serialize>(writer: &mut W, value: &T) -> Result<(), Error>
where
    W::Error: fmt::Display,
{
    to_writer_xml_with_options(writer, value, &XmlWriteOptions::default())
}

#[cfg(feature = "serde")]
pub fn to_writer_xml_with_options<W: bplist::wire::encode::Emit, T: Serialize>(
    writer: &mut W,
    value: &T,
    options: &XmlWriteOptions,
) -> Result<(), Error>
where
    W::Error: fmt::Display,
{
    let value = to_value(value)?;
    bplist::serde::xml::to_emitter(&value, *options, |chunk| writer.emit(chunk))
}

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
pub mod downcast_enum_to_legacy_u64 {
    use bitflags::Flags;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Flags<Bits = u128>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(T::from_bits_truncate(u128::from(bits)))
    }

    pub fn serialize<S, T>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Flags<Bits = u128>,
    {
        serializer.serialize_u64(x.bits() as u64)
    }

    pub mod option {
        use super::*;
        use serde::Serialize;

        pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
        where
            D: Deserializer<'de>,
            T: Flags<Bits = u128>,
        {
            Ok(Option::<u64>::deserialize(deserializer)?.map(|bits| T::from_bits_truncate(u128::from(bits))))
        }

        pub fn serialize<S, T>(x: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: Flags<Bits = u128>,
        {
            x.as_ref()
                .map(|flags| flags.bits() as u64)
                .serialize(serializer)
        }
    }
}
#[cfg(all(test, feature = "serde"))]
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
