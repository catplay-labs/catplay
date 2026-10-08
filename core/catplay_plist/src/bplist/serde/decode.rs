//! Serde adapter over the allocation-free slice reader.
use alloc::string::{String, ToString};
use core::fmt;

use serde::de::{self, Deserialize, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};

use super::super::wire::decode::{self, Document, Integer, Object, Refs};

const MAX_DEPTH: usize = 128;

#[derive(Debug)]
pub enum Error {
    Format(decode::Error),
    DepthLimit,
    TypeMismatch,
    TrailingElements,
    InvalidDate,
    InvalidUtf16,
    InvalidOption,
    Message(String),
}

impl From<decode::Error> for Error {
    fn from(value: decode::Error) -> Self {
        Self::Format(value)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl core::error::Error for Error {}
impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

pub fn from_slice<'de, T: Deserialize<'de>>(data: &'de [u8]) -> Result<T, Error> {
    let doc = Document::parse(data)?;
    from_object(&doc, doc.root())
}

/// Deserializes one object selected from a parsed document.
pub fn from_object<'de, T: Deserialize<'de>>(doc: &Document<'de>, id: u64) -> Result<T, Error> {
    T::deserialize(ValueDecoder {
        doc,
        id,
        depth: 0,
        mode: OptionMode::Root,
        in_value: false,
    })
}

#[derive(Clone, Copy)]
enum OptionMode {
    Root,
    Field,
    Explicit,
}

#[derive(Clone, Copy)]
struct ValueDecoder<'a, 'de> {
    doc: &'a Document<'de>,
    id: u64,
    depth: usize,
    mode: OptionMode,
    in_value: bool,
}

impl<'a, 'de> ValueDecoder<'a, 'de> {
    fn visit_array<V: Visitor<'de>>(self, refs: Refs<'de>, visitor: V) -> Result<V::Value, Error> {
        let mut access = ArrayAccess {
            parent: self,
            refs,
            index: 0,
        };
        let value = visitor.visit_seq(&mut access)?;
        if access.index != refs.len() {
            return Err(Error::TrailingElements);
        }
        Ok(value)
    }

    fn visit_dictionary<V: Visitor<'de>>(
        self,
        keys: Refs<'de>,
        values: Refs<'de>,
        struct_fields: bool,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let mut access = DictAccess {
            parent: self,
            keys,
            values,
            index: 0,
            pending: None,
            struct_fields,
        };
        let value = visitor.visit_map(&mut access)?;
        if access.pending.is_some() || access.index != keys.len() {
            return Err(Error::TrailingElements);
        }
        Ok(value)
    }

    fn object(&self) -> Result<Object<'de>, Error> {
        if self.depth >= MAX_DEPTH {
            return Err(Error::DepthLimit);
        }
        Ok(self.doc.object(self.id)?)
    }
    fn child(&self, id: u64, mode: OptionMode) -> Result<Self, Error> {
        let depth = self.depth.checked_add(1).ok_or(Error::DepthLimit)?;
        if depth >= MAX_DEPTH {
            return Err(Error::DepthLimit);
        }
        Ok(Self {
            doc: self.doc,
            id,
            depth,
            mode,
            in_value: self.in_value,
        })
    }
}

fn utf16_string(bytes: &[u8]) -> Result<String, Error> {
    let chars = || {
        char::decode_utf16(
            bytes
                .chunks_exact(2)
                .map(|v| u16::from_be_bytes([v[0], v[1]])),
        )
    };
    // Validate and size first: collecting Result<char, _> loses the size hint
    // and repeatedly grows the String, even with a warm CachingSerializer.
    let len = chars().try_fold(0usize, |len, ch| {
        len.checked_add(ch.map_err(|_| Error::InvalidUtf16)?.len_utf8())
            .ok_or(Error::Format(decode::Error::OutOfBounds))
    })?;
    let mut text = String::with_capacity(len);
    for ch in chars() {
        text.push(ch.map_err(|_| Error::InvalidUtf16)?);
    }
    Ok(text)
}

fn string_matches(object: Object<'_>, expected: &str) -> bool {
    match object {
        Object::String(value) => value == expected,
        Object::Utf16(bytes) => {
            bytes.len() == expected.len() * 2
                && bytes
                    .chunks_exact(2)
                    .zip(expected.bytes())
                    .all(|(unit, ascii)| unit == [0, ascii])
        }
        _ => false,
    }
}

fn date(seconds: f64) -> Result<crate::Date, Error> {
    crate::Date::from_seconds_since_plist_epoch(seconds).ok_or(Error::InvalidDate)
}

impl<'de> de::Deserializer<'de> for ValueDecoder<'_, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.object()? {
            Object::Boolean(v) => visitor.visit_bool(v),
            Object::Integer(Integer::Signed(v)) => visitor.visit_i64(v),
            Object::Integer(Integer::Unsigned(v)) => visitor.visit_u64(v),
            Object::Real(v) => visitor.visit_f64(v),
            Object::String(v) => visitor.visit_borrowed_str(v),
            Object::Utf16(v) => visitor.visit_string(utf16_string(v)?),
            Object::Data(v) => visitor.visit_borrowed_bytes(v),
            Object::Uid(v) if self.in_value => visitor.visit_enum(SpecialEnum::uid(v)),
            Object::Uid(v) => visitor.visit_u64(v),
            Object::Date(v) if self.in_value => visitor.visit_enum(SpecialEnum::date(date(v)?)),
            Object::Date(v) => visitor.visit_str(date(v)?.xml_text().as_str()),
            Object::Array(refs) => self.visit_array(refs, visitor),
            Object::Dictionary { keys, values } => self.visit_dictionary(keys, values, false, visitor),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.mode {
            OptionMode::Root | OptionMode::Field => visitor.visit_some(Self {
                mode: OptionMode::Explicit,
                ..self
            }),
            OptionMode::Explicit => {
                let Object::Dictionary { keys, values } = self.object()? else {
                    return Err(Error::InvalidOption);
                };
                if keys.len() != 1 {
                    return Err(Error::InvalidOption);
                }
                let key = self.doc.object(keys.get(0).ok_or(Error::InvalidOption)?)?;
                let value_id = values.get(0).ok_or(Error::InvalidOption)?;
                if string_matches(key, "None") {
                    match self.doc.object(value_id)? {
                        Object::String(_) | Object::Utf16(_) => visitor.visit_none(),
                        _ => Err(Error::InvalidOption),
                    }
                } else if string_matches(key, "Some") {
                    visitor.visit_some(self.child(value_id, OptionMode::Explicit)?)
                } else {
                    Err(Error::InvalidOption)
                }
            }
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.object()? {
            Object::String(_) | Object::Utf16(_) => visitor.visit_unit(),
            _ => Err(Error::TypeMismatch),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Error> {
        let in_value = self.in_value || name == "PLIST-VALUE";
        visitor.visit_newtype_struct(Self { in_value, ..self })
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        let Object::Dictionary { keys, values } = self.object()? else {
            return Err(Error::TypeMismatch);
        };
        self.visit_dictionary(keys, values, true, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        match self.object()? {
            Object::String(v) => visitor.visit_enum(v.into_deserializer()),
            Object::Utf16(v) => visitor.visit_enum(utf16_string(v)?.into_deserializer()),
            Object::Dictionary { keys, values } if keys.len() == 1 => {
                let key = self.child(keys.get(0).ok_or(Error::TypeMismatch)?, OptionMode::Explicit)?;
                let value = self.child(values.get(0).ok_or(Error::TypeMismatch)?, self.mode)?;
                visitor.visit_enum(Variant { key, value })
            }
            _ => Err(Error::TypeMismatch),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.object()? {
            Object::Data(v) => visitor.visit_borrowed_bytes(v),
            _ => Err(Error::TypeMismatch),
        }
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        seq tuple tuple_struct map unit_struct identifier
    }
}

struct ArrayAccess<'a, 'de> {
    parent: ValueDecoder<'a, 'de>,
    refs: Refs<'de>,
    index: usize,
}
impl<'de> SeqAccess<'de> for ArrayAccess<'_, 'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        let Some(id) = self.refs.get(self.index) else {
            return Ok(None);
        };
        self.index += 1;
        seed.deserialize(self.parent.child(id, OptionMode::Explicit)?)
            .map(Some)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.refs.len() - self.index)
    }
}

struct DictAccess<'a, 'de> {
    parent: ValueDecoder<'a, 'de>,
    keys: Refs<'de>,
    values: Refs<'de>,
    index: usize,
    pending: Option<u64>,
    struct_fields: bool,
}
impl<'de> MapAccess<'de> for DictAccess<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Error> {
        if self.pending.is_some() {
            return Err(Error::TypeMismatch);
        }
        let Some(key) = self.keys.get(self.index) else {
            return Ok(None);
        };
        let value = self.values.get(self.index).ok_or(Error::TypeMismatch)?;
        self.pending = Some(value);
        seed.deserialize(self.parent.child(key, OptionMode::Explicit)?)
            .map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let id = self.pending.take().ok_or(Error::TypeMismatch)?;
        self.index += 1;
        seed.deserialize(self.parent.child(
            id,
            if self.struct_fields {
                OptionMode::Field
            } else {
                OptionMode::Explicit
            },
        )?)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.keys.len() - self.index)
    }
}

struct Variant<'a, 'de> {
    key: ValueDecoder<'a, 'de>,
    value: ValueDecoder<'a, 'de>,
}
impl<'a, 'de> de::EnumAccess<'de> for Variant<'a, 'de> {
    type Error = Error;
    type Variant = VariantValue<'a, 'de>;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Error> {
        let key = seed.deserialize(self.key)?;
        Ok((key, VariantValue(self.value)))
    }
}
struct VariantValue<'a, 'de>(ValueDecoder<'a, 'de>);
impl<'de> VariantAccess<'de> for VariantValue<'_, 'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        Deserialize::deserialize(self.0)
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self.0)
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_seq(self.0, visitor)
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_struct(self.0, "", fields, visitor)
    }
}

enum SpecialEnum {
    Uid(u64),
    Date(crate::Date),
}
impl SpecialEnum {
    fn uid(value: u64) -> Self {
        Self::Uid(value)
    }
    fn date(value: crate::Date) -> Self {
        Self::Date(value)
    }
}
impl<'de> de::EnumAccess<'de> for SpecialEnum {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Error> {
        let name = match self {
            Self::Uid(_) => crate::PLIST_UID,
            Self::Date(_) => crate::PLIST_DATE,
        };
        let variant = seed.deserialize(de::value::StrDeserializer::<Error>::new(name))?;
        Ok((variant, self))
    }
}
impl<'de> VariantAccess<'de> for SpecialEnum {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        Err(Error::TypeMismatch)
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        match self {
            Self::Uid(value) => seed.deserialize(value.into_deserializer()),
            Self::Date(value) => seed.deserialize(de::value::StrDeserializer::<Error>::new(value.xml_text().as_str())),
        }
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, _: V) -> Result<V::Value, Error> {
        Err(Error::TypeMismatch)
    }
    fn struct_variant<V: Visitor<'de>>(self, _: &'static [&'static str], _: V) -> Result<V::Value, Error> {
        Err(Error::TypeMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlistByteArray, bplist::ScratchBuffers};
    use bytes::BytesMut;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        name: String,
        data: PlistByteArray,
        nested: Vec<Vec<i64>>,
        absent: Option<String>,
        present: Option<String>,
    }

    #[test]
    fn decodes_minimal_integer_fixture() {
        // One integer object (42), one offset-table entry, then the 32-byte trailer.
        let mut bytes = [0u8; 43];
        bytes[..8].copy_from_slice(b"bplist00");
        bytes[8..11].copy_from_slice(&[0x10, 42, 8]);
        bytes[17] = 1; // offsetIntSize
        bytes[18] = 1; // objectRefSize
        bytes[26] = 1; // numObjects
        bytes[42] = 10; // offsetTableOffset
        assert_eq!(from_slice::<u64>(&bytes).unwrap(), 42);
    }

    #[test]
    fn decodes_owned_writer_output() {
        let sample = Sample {
            name: "zażółć 🚗".into(),
            data: PlistByteArray::from(vec![7; 512]),
            nested: vec![vec![1, -1], vec![3, 4]],
            absent: None,
            present: Some("yes".into()),
        };
        let mut encoded = BytesMut::new();
        ScratchBuffers::default()
            .encode_into(&sample, &mut encoded)
            .unwrap();
        assert_eq!(from_slice::<Sample>(&encoded).unwrap(), sample);
        let mut streamed = Vec::new();
        crate::to_writer_binary(&mut streamed, &sample).unwrap();
        assert_eq!(streamed, encoded);
    }

    #[test]
    fn plist_value_date_uid_and_options() {
        let date = crate::Date::from_xml_format("2024-01-01T00:00:00Z").unwrap();
        let values = crate::Value::Array(vec![
            crate::Value::Date(date),
            crate::Value::Uid(crate::Uid::new(123)),
            crate::Value::String("hello".into()),
        ]);
        let mut bytes = Vec::new();
        crate::to_writer_binary(&mut bytes, &values).unwrap();
        assert_eq!(from_slice::<crate::Value>(&bytes).unwrap(), values);

        let options = vec![Some(1u64), None, Some(3)];
        bytes.clear();
        crate::to_writer_binary(&mut bytes, &options).unwrap();
        assert_eq!(from_slice::<Vec<Option<u64>>>(&bytes).unwrap(), options);
    }

    #[test]
    fn selective_dictionary_lookup() {
        let mut dictionary = crate::Dictionary::new();
        dictionary.insert("type".into(), "duckAudio".into());
        dictionary.insert("params".into(), crate::Value::Array(vec![1u64.into(), 2u64.into()]));
        let mut bytes = Vec::new();
        crate::to_writer_binary(&mut bytes, &dictionary).unwrap();
        let doc = Document::parse(&bytes).unwrap();
        let type_id = doc.dictionary_get(doc.root(), "type").unwrap().unwrap();
        let params_id = doc.dictionary_get(doc.root(), "params").unwrap().unwrap();
        assert_eq!(from_object::<&str>(&doc, type_id).unwrap(), "duckAudio");
        assert_eq!(from_object::<Vec<u64>>(&doc, params_id).unwrap(), vec![1, 2]);
        assert_eq!(doc.dictionary_get(doc.root(), "missing").unwrap(), None);
    }

    #[test]
    fn core_borrows_ascii_and_data() {
        let mut bytes = Vec::new();
        crate::to_writer_binary(&mut bytes, &"hello").unwrap();
        let doc = Document::parse(&bytes).unwrap();
        let Object::String(text) = doc.object(doc.root()).unwrap() else {
            panic!("expected string")
        };
        assert_eq!(text, "hello");
        assert!(text.as_ptr() >= bytes.as_ptr());
        assert!((text.as_ptr() as usize) < bytes.as_ptr() as usize + bytes.len());
    }

    #[test]
    fn malformed_input_and_cycle_are_bounded() {
        assert!(Document::parse(b"bad").is_err());
        let mut bytes = Vec::new();
        crate::to_writer_binary(&mut bytes, &vec![1u64]).unwrap();
        let doc = Document::parse(&bytes).unwrap();
        let root = doc.root();
        let Object::Array(refs) = doc.object(root).unwrap() else {
            panic!("expected array")
        };
        let ref_ptr = refs.bytes.as_ptr() as usize - bytes.as_ptr() as usize;
        bytes[ref_ptr] = root as u8;
        assert!(matches!(from_slice::<crate::Value>(&bytes), Err(Error::DepthLimit)));
    }

    #[test]
    fn out_of_range_date_returns_error() {
        let mut offsets = [0u64; 1];
        let mut collections = [];
        let mut edges = [];
        let mut bytes = Vec::new();
        let scratch = crate::bplist::wire::encode::Scratch {
            offsets: &mut offsets,
            collections: &mut collections,
            edges: &mut edges,
        };
        let mut writer = crate::bplist::wire::encode::Encoder::new(scratch, |chunk: &[u8]| -> Result<(), core::convert::Infallible> {
            bytes.extend_from_slice(chunk);
            Ok(())
        })
        .unwrap();
        let root = writer.date(1e100).unwrap();
        writer.finish(root).unwrap();
        assert!(matches!(from_slice::<crate::Date>(&bytes), Err(Error::InvalidDate)));
    }

    #[test]
    fn mutated_binary_input_does_not_panic() {
        let mut valid = Vec::new();
        crate::to_writer_binary(&mut valid, &vec!["hello", "zażółć", "world"]).unwrap();
        for index in 0..valid.len() {
            for replacement in [0, 0xff] {
                let mut mutated = valid.clone();
                mutated[index] = replacement;
                assert!(
                    std::panic::catch_unwind(|| from_slice::<crate::Value>(&mutated)).is_ok(),
                    "panic at byte {index}"
                );
            }
        }
    }

    #[test]
    fn accepts_three_byte_offset_table_entries() {
        let mut bytes = Vec::new();
        crate::to_writer_binary(&mut bytes, &"hello").unwrap();
        let trailer = bytes.len() - 32;
        let table = u64::from_be_bytes(bytes[trailer + 24..trailer + 32].try_into().unwrap()) as usize;
        assert_eq!(bytes[trailer + 6], 1);
        bytes.splice(table..table, [0, 0]);
        let trailer = bytes.len() - 32;
        bytes[trailer + 6] = 3;
        assert_eq!(from_slice::<String>(&bytes).unwrap(), "hello");
    }
}
