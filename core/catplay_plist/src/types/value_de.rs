//! Direct deserialization from an owned plist value, without a binary round trip.

use alloc::string::String;
use serde::de::{self, DeserializeOwned, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};

use super::{Error, Value};

pub fn from_value<T: DeserializeOwned>(value: &Value) -> Result<T, Error> {
    T::deserialize(ValueDecoder {
        value,
        mode: Mode::Root,
        in_value: false,
    })
}

#[derive(Clone, Copy)]
enum Mode {
    Root,
    Field,
    Explicit,
}

#[derive(Clone, Copy)]
struct ValueDecoder<'a> {
    value: &'a Value,
    mode: Mode,
    in_value: bool,
}

impl<'a> ValueDecoder<'a> {
    fn visit_array<'de, V: Visitor<'de>>(self, values: &'a [Value], visitor: V) -> Result<V::Value, Error> {
        let mut access = ArrayAccess {
            parent: self,
            values,
            index: 0,
        };
        let value = visitor.visit_seq(&mut access)?;
        if access.index != values.len() {
            return Err(Error::message("trailing plist array elements"));
        }
        Ok(value)
    }

    fn visit_dictionary<'de, V: Visitor<'de>>(self, values: &'a super::Dictionary, fields: bool, visitor: V) -> Result<V::Value, Error> {
        let mut access = DictAccess {
            parent: self,
            iter: values.iter(),
            pending: None,
            fields,
        };
        let value = visitor.visit_map(&mut access)?;
        if access.pending.is_some() || access.iter.len() != 0 {
            return Err(Error::message("trailing plist dictionary entries"));
        }
        Ok(value)
    }

    fn child(self, value: &'a Value, mode: Mode) -> Self {
        Self {
            value,
            mode,
            in_value: self.in_value,
        }
    }
}

impl<'de> de::Deserializer<'de> for ValueDecoder<'_> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            Value::Array(values) => self.visit_array(values, visitor),
            Value::Dictionary(values) => self.visit_dictionary(values, false, visitor),
            Value::Boolean(value) => visitor.visit_bool(*value),
            Value::Data(value) => visitor.visit_bytes(value),
            Value::Date(value) if self.in_value => visitor.visit_enum(SpecialEnum::Date(*value)),
            Value::Date(value) => visitor.visit_str(value.xml_text().as_str()),
            Value::Real(value) => visitor.visit_f64(*value),
            Value::Integer(value) => match value.as_unsigned() {
                Some(value) => visitor.visit_u64(value),
                None => visitor.visit_i64(
                    value
                        .as_signed()
                        .ok_or_else(|| Error::message("integer out of range"))?,
                ),
            },
            Value::String(value) => visitor.visit_str(value),
            Value::Uid(value) if self.in_value => visitor.visit_enum(SpecialEnum::Uid(value.get())),
            Value::Uid(value) => visitor.visit_u64(value.get()),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.mode {
            Mode::Root | Mode::Field => visitor.visit_some(self.child(self.value, Mode::Explicit)),
            Mode::Explicit => {
                let Value::Dictionary(map) = self.value else {
                    return Err(Error::message("invalid plist option"));
                };
                if map.len() != 1 {
                    return Err(Error::message("invalid plist option"));
                }
                let (key, value) = map
                    .iter()
                    .next()
                    .ok_or_else(|| Error::message("invalid plist option"))?;
                match key.as_str() {
                    "Some" => visitor.visit_some(self.child(value, Mode::Explicit)),
                    "None" if matches!(value, Value::String(_)) => visitor.visit_none(),
                    _ => Err(Error::message("invalid plist option")),
                }
            }
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if matches!(self.value, Value::String(_)) {
            visitor.visit_unit()
        } else {
            Err(Error::message("expected plist unit"))
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(Self {
            in_value: self.in_value || name == "PLIST-VALUE",
            ..self
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        let Value::Dictionary(map) = self.value else {
            return Err(Error::message("expected plist dictionary"));
        };
        self.visit_dictionary(map, true, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        match self.value {
            Value::String(value) => visitor.visit_enum(value.as_str().into_deserializer()),
            Value::Dictionary(map) if map.len() == 1 => {
                let (key, value) = map
                    .iter()
                    .next()
                    .ok_or_else(|| Error::message("invalid plist enum"))?;
                visitor.visit_enum(EnumValue {
                    key,
                    value: self.child(value, self.mode),
                })
            }
            _ => Err(Error::message("invalid plist enum")),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let Value::Array(values) = self.value else {
            return Err(Error::message("expected plist array"));
        };
        self.visit_array(values, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let Value::Dictionary(map) = self.value else {
            return Err(Error::message("expected plist dictionary"));
        };
        self.visit_dictionary(map, false, visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        unit_struct tuple tuple_struct identifier
    }
}

struct ArrayAccess<'a> {
    parent: ValueDecoder<'a>,
    values: &'a [Value],
    index: usize,
}
impl<'de> SeqAccess<'de> for ArrayAccess<'_> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        let Some(value) = self.values.get(self.index) else {
            return Ok(None);
        };
        self.index += 1;
        seed.deserialize(self.parent.child(value, Mode::Explicit))
            .map(Some)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len() - self.index)
    }
}

struct DictAccess<'a> {
    parent: ValueDecoder<'a>,
    iter: indexmap::map::Iter<'a, String, Value>,
    pending: Option<&'a Value>,
    fields: bool,
}
impl<'de> MapAccess<'de> for DictAccess<'_> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Error> {
        if self.pending.is_some() {
            return Err(Error::message("plist map value pending"));
        }
        let Some((key, value)) = self.iter.next() else { return Ok(None) };
        self.pending = Some(value);
        seed.deserialize(de::value::StrDeserializer::<Error>::new(key))
            .map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .pending
            .take()
            .ok_or_else(|| Error::message("plist map key missing"))?;
        seed.deserialize(
            self.parent
                .child(value, if self.fields { Mode::Field } else { Mode::Explicit }),
        )
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct EnumValue<'a> {
    key: &'a str,
    value: ValueDecoder<'a>,
}
impl<'de, 'a> de::EnumAccess<'de> for EnumValue<'a> {
    type Error = Error;
    type Variant = ValueDecoder<'a>;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Error> {
        let key = seed.deserialize(de::value::StrDeserializer::<Error>::new(self.key))?;
        // The value decoder only borrows data from the input Value.
        Ok((key, self.value))
    }
}
impl<'de> VariantAccess<'de> for ValueDecoder<'_> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        de::Deserialize::deserialize(self)
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        self.deserialize_struct("", fields, visitor)
    }
}

enum SpecialEnum {
    Uid(u64),
    Date(crate::Date),
}
impl<'de> de::EnumAccess<'de> for SpecialEnum {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self), Error> {
        let name = match self {
            Self::Uid(_) => crate::PLIST_UID,
            Self::Date(_) => crate::PLIST_DATE,
        };
        let key = seed.deserialize(de::value::StrDeserializer::<Error>::new(name))?;
        Ok((key, self))
    }
}
impl<'de> VariantAccess<'de> for SpecialEnum {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        Err(Error::message("expected plist newtype"))
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        match self {
            Self::Uid(value) => seed.deserialize(value.into_deserializer()),
            Self::Date(value) => seed.deserialize(de::value::StrDeserializer::<Error>::new(value.xml_text().as_str())),
        }
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, _: V) -> Result<V::Value, Error> {
        Err(Error::message("expected plist newtype"))
    }
    fn struct_variant<V: Visitor<'de>>(self, _: &'static [&'static str], _: V) -> Result<V::Value, Error> {
        Err(Error::message("expected plist newtype"))
    }
}
