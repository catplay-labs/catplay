#[cfg(feature = "serde")]
use crate::bplist::wire::encode::Emit;
#[cfg(feature = "serde")]
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};
#[cfg(feature = "serde")]
use core::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[cfg(feature = "serde")]
use super::Error;
use super::{Date, Dictionary, Integer, Uid};

/// Owned property-list value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    Array(Vec<Value>),
    Dictionary(Dictionary),
    Boolean(bool),
    Data(Vec<u8>),
    Date(Date),
    Real(f64),
    Integer(Integer),
    String(String),
    Uid(Uid),
}

impl Value {
    #[cfg(feature = "serde")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        crate::from_bytes(bytes).map_err(|err| Error::message(err.to_string()))
    }
    #[cfg(feature = "serde")]
    pub fn from_xml_bytes(bytes: &[u8]) -> Result<Self, Error> {
        crate::from_xml_bytes(bytes)
    }
    #[cfg(feature = "serde")]
    pub fn to_writer_binary<W: Emit>(&self, writer: &mut W) -> Result<(), Error>
    where
        W::Error: core::error::Error + Send + Sync + 'static,
    {
        crate::to_writer_binary(writer, self).map_err(|err| Error::message(err.to_string()))
    }
    #[cfg(feature = "serde")]
    pub fn to_writer_xml<W: Emit>(&self, writer: &mut W) -> Result<(), Error>
    where
        W::Error: fmt::Display,
    {
        crate::to_writer_xml(writer, self)
    }
    #[cfg(feature = "serde")]
    pub fn to_writer_xml_with_options<W: Emit>(&self, writer: &mut W, options: &crate::XmlWriteOptions) -> Result<(), Error>
    where
        W::Error: fmt::Display,
    {
        crate::to_writer_xml_with_options(writer, self, options)
    }
    pub fn into_array(self) -> Option<Vec<Value>> {
        if let Self::Array(v) = self { Some(v) } else { None }
    }
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Self::Array(v) = self { Some(v) } else { None }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Self::Array(v) = self { Some(v) } else { None }
    }
    pub fn into_dictionary(self) -> Option<Dictionary> {
        if let Self::Dictionary(v) = self { Some(v) } else { None }
    }
    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        if let Self::Dictionary(v) = self { Some(v) } else { None }
    }
    pub fn as_dictionary_mut(&mut self) -> Option<&mut Dictionary> {
        if let Self::Dictionary(v) = self { Some(v) } else { None }
    }
    pub fn as_boolean(&self) -> Option<bool> {
        if let Self::Boolean(v) = self { Some(*v) } else { None }
    }
    pub fn into_data(self) -> Option<Vec<u8>> {
        if let Self::Data(v) = self { Some(v) } else { None }
    }
    pub fn as_data(&self) -> Option<&[u8]> {
        if let Self::Data(v) = self { Some(v) } else { None }
    }
    pub fn as_date(&self) -> Option<Date> {
        if let Self::Date(v) = self { Some(*v) } else { None }
    }
    pub fn as_real(&self) -> Option<f64> {
        if let Self::Real(v) = self { Some(*v) } else { None }
    }
    pub fn as_signed_integer(&self) -> Option<i64> {
        if let Self::Integer(v) = self { v.as_signed() } else { None }
    }
    pub fn as_unsigned_integer(&self) -> Option<u64> {
        if let Self::Integer(v) = self { v.as_unsigned() } else { None }
    }
    pub fn into_string(self) -> Option<String> {
        if let Self::String(v) = self { Some(v) } else { None }
    }
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(v) = self { Some(v) } else { None }
    }
    pub fn into_uid(self) -> Option<Uid> {
        if let Self::Uid(v) = self { Some(v) } else { None }
    }
    pub fn as_uid(&self) -> Option<&Uid> {
        if let Self::Uid(v) = self { Some(v) } else { None }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Array(v) => v.serialize(serializer),
            Self::Dictionary(v) => v.serialize(serializer),
            Self::Boolean(v) => v.serialize(serializer),
            Self::Data(v) => serializer.serialize_bytes(v),
            Self::Date(v) => v.serialize(serializer),
            Self::Real(v) => serializer.serialize_f64(*v),
            Self::Integer(v) => v.serialize(serializer),
            Self::String(v) => serializer.serialize_str(v),
            Self::Uid(v) => v.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a plist value")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Boolean(v))
            }
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Value, E> {
                Ok(Value::Data(v.to_vec()))
            }
            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Value, E> {
                Ok(Value::Data(v))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Integer(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
                Ok(Value::Integer(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
                Ok(Value::Real(v))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }
            fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Value, M::Error> {
                let mut result = Dictionary::new();
                while let Some((key, value)) = map.next_entry()? {
                    result.insert(key, value);
                }
                Ok(Value::Dictionary(result))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                let mut result = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(value) = seq.next_element()? {
                    result.push(value);
                }
                Ok(Value::Array(result))
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
                deserializer.deserialize_any(self)
            }
            fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<Value, A::Error> {
                use de::VariantAccess;
                let (name, variant) = data.variant::<String>()?;
                match name.as_str() {
                    crate::PLIST_DATE => Ok(Value::Date(variant.newtype_variant()?)),
                    crate::PLIST_UID => Ok(Value::Uid(variant.newtype_variant()?)),
                    _ => Err(de::Error::unknown_variant(&name, &[crate::PLIST_DATE, crate::PLIST_UID])),
                }
            }
        }
        deserializer.deserialize_newtype_struct("PLIST-VALUE", Visitor)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::Array(v)
    }
}
impl From<Dictionary> for Value {
    fn from(v: Dictionary) -> Self {
        Self::Dictionary(v)
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}
impl From<Date> for Value {
    fn from(v: Date) -> Self {
        Self::Date(v)
    }
}
impl From<Uid> for Value {
    fn from(v: Uid) -> Self {
        Self::Uid(v)
    }
}
impl From<Integer> for Value {
    fn from(v: Integer) -> Self {
        Self::Integer(v)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Real(v)
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Real(v.into())
    }
}
macro_rules! from_integer {
    ($($ty:ty),*) => {$ (
        impl From<$ty> for Value { fn from(v: $ty) -> Self { Self::Integer(v.into()) } }
        impl From<&$ty> for Value { fn from(v: &$ty) -> Self { Self::Integer((*v).into()) } }
    )*};
}
from_integer!(i8, i16, i32, i64, u8, u16, u32, u64);
impl From<&bool> for Value {
    fn from(v: &bool) -> Self {
        Self::Boolean(*v)
    }
}
impl From<&Date> for Value {
    fn from(v: &Date) -> Self {
        Self::Date(*v)
    }
}
impl From<&f32> for Value {
    fn from(v: &f32) -> Self {
        Self::Real((*v).into())
    }
}
impl From<&f64> for Value {
    fn from(v: &f64) -> Self {
        Self::Real(*v)
    }
}
