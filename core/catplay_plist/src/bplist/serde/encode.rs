//! Serde frontend for the caller-scratch binary writer.
use super::super::wire::encode::{self as core, Encoder, Scratch};
use super::builder::{Builder, ValueBuilder};
use ::core::fmt;
use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use serde::{Serialize, ser};

pub(super) type EmitError = Box<dyn ::core::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum Error {
    Core(core::Error<EmitError>),
    Message(String),
    MissingValue,
    InvalidMap,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(e) => write!(f, "bplist writer: {e:?}"),
            Self::Message(s) => f.write_str(s),
            Self::MissingValue => f.write_str("plist value is absent"),
            Self::InvalidMap => f.write_str("invalid plist dictionary"),
        }
    }
}
impl ::core::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
impl From<core::Error<EmitError>> for Error {
    fn from(e: core::Error<EmitError>) -> Self {
        Self::Core(e)
    }
}

pub fn encode<T, F, E>(value: &T, scratch: Scratch<'_>, mut emit: F) -> Result<(), Error>
where
    T: Serialize + ?Sized,
    F: FnMut(&[u8]) -> Result<(), E>,
    E: ::core::error::Error + Send + Sync + 'static,
{
    let mut encoder = Encoder::new(scratch, |chunk: &[u8]| -> Result<(), EmitError> {
        emit(chunk).map_err(|err| Box::new(err) as EmitError)
    })?;
    let root = value
        .serialize(Serializer {
            encoder: &mut encoder,
            mode: Mode::Root,
        })?
        .ok_or(Error::MissingValue)?;
    encoder.finish(root)?;
    Ok(())
}

pub(crate) fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<crate::Value, Error> {
    value
        .serialize(Serializer {
            encoder: &mut ValueBuilder,
            mode: Mode::Root,
        })?
        .ok_or(Error::MissingValue)
}

#[derive(Clone, Copy)]
enum Mode {
    Root,
    Explicit,
    Field,
}

struct Serializer<'a, B: Builder> {
    encoder: &'a mut B,
    mode: Mode,
}

impl<B: Builder> Serializer<'_, B> {
    fn nested<T: Serialize + ?Sized>(&mut self, value: &T, mode: Mode) -> Result<Option<B::Item>, Error> {
        value.serialize(Serializer {
            encoder: self.encoder,
            mode,
        })
    }
    fn named<T: Serialize + ?Sized>(&mut self, name: &str, value: &T, mode: Mode) -> Result<Option<B::Item>, Error> {
        let Some(id) = self.nested(value, mode)? else { return Ok(None) };
        let mut state = self.encoder.begin_dict()?;
        let key = self.encoder.string(name)?;
        self.encoder.child(&mut state, key)?;
        self.encoder.child(&mut state, id)?;
        Ok(Some(self.encoder.end(state)?))
    }
}

struct Compound<'a, B: Builder> {
    encoder: &'a mut B,
    state: B::Collection,
    pending_key: Option<B::Item>,
    outer: Option<B::Collection>,
}

impl<B: Builder> Compound<'_, B> {
    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        let id = value
            .serialize(Serializer {
                encoder: self.encoder,
                mode: Mode::Explicit,
            })?
            .ok_or(Error::MissingValue)?;
        self.encoder.child(&mut self.state, id)?;
        Ok(())
    }
    fn field<T: Serialize + ?Sized>(&mut self, name: &str, value: &T) -> Result<(), Error> {
        let Some(id) = value.serialize(Serializer {
            encoder: self.encoder,
            mode: Mode::Field,
        })?
        else {
            return Ok(());
        };
        let key = self.encoder.string(name)?;
        self.encoder.child(&mut self.state, key)?;
        self.encoder.child(&mut self.state, id)?;
        Ok(())
    }
    fn finish(self) -> Result<Option<B::Item>, Error> {
        if self.pending_key.is_some() {
            return Err(Error::InvalidMap);
        }
        let id = self.encoder.end(self.state)?;
        if let Some(mut outer) = self.outer {
            self.encoder.child(&mut outer, id)?;
            Ok(Some(self.encoder.end(outer)?))
        } else {
            Ok(Some(id))
        }
    }
}

impl<'a, B: Builder> ser::Serializer for Serializer<'a, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    type SerializeSeq = Compound<'a, B>;
    type SerializeTuple = Compound<'a, B>;
    type SerializeTupleStruct = Compound<'a, B>;
    type SerializeTupleVariant = Compound<'a, B>;
    type SerializeMap = Compound<'a, B>;
    type SerializeStruct = Compound<'a, B>;
    type SerializeStructVariant = Compound<'a, B>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.boolean(v)?))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.integer(v)?))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Error> {
        self.serialize_u64(v as u64)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.unsigned_integer(v)?))
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Error> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.real(v)?))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Error> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.string(v)?))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Error> {
        Ok(Some(self.encoder.data(v)?))
    }
    fn serialize_none(mut self) -> Result<Self::Ok, Error> {
        match self.mode {
            Mode::Explicit => self.named("None", &(), Mode::Explicit),
            Mode::Root | Mode::Field => Ok(None),
        }
    }
    fn serialize_some<T: Serialize + ?Sized>(mut self, value: &T) -> Result<Self::Ok, Error> {
        match self.mode {
            Mode::Explicit => self.named("Some", value, Mode::Explicit),
            Mode::Root | Mode::Field => self.nested(value, Mode::Explicit),
        }
    }
    fn serialize_unit(self) -> Result<Self::Ok, Error> {
        self.serialize_str("")
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<Self::Ok, Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(mut self, name: &'static str, value: &T) -> Result<Self::Ok, Error> {
        match name {
            crate::PLIST_UID => {
                let uid = value.serialize(UidValue)?;
                Ok(Some(self.encoder.uid(uid)?))
            }
            crate::PLIST_DATE => {
                let date = value.serialize(DateValue)?;
                Ok(Some(self.encoder.date(date)?))
            }
            _ => self.nested(value, self.mode),
        }
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        mut self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Error> {
        self.named(variant, value, self.mode)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        let state = self.encoder.begin_array()?;
        Ok(Compound {
            encoder: self.encoder,
            state,
            pending_key: None,
            outer: None,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_tuple(len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        let mut outer = self.encoder.begin_dict()?;
        let key = self.encoder.string(variant)?;
        self.encoder.child(&mut outer, key)?;
        let state = self.encoder.begin_array()?;
        let _ = len;
        Ok(Compound {
            encoder: self.encoder,
            state,
            pending_key: None,
            outer: Some(outer),
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
        let state = self.encoder.begin_dict()?;
        Ok(Compound {
            encoder: self.encoder,
            state,
            pending_key: None,
            outer: None,
        })
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        let mut outer = self.encoder.begin_dict()?;
        let key = self.encoder.string(variant)?;
        self.encoder.child(&mut outer, key)?;
        let state = self.encoder.begin_dict()?;
        let _ = len;
        Ok(Compound {
            encoder: self.encoder,
            state,
            pending_key: None,
            outer: Some(outer),
        })
    }
}

impl<B: Builder> ser::SerializeSeq for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeTuple for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeTupleStruct for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeTupleVariant for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeMap for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        if self.pending_key.is_some() {
            return Err(Error::InvalidMap);
        }
        self.pending_key = Some(key.serialize(StringKey { encoder: self.encoder })?);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        let key = self.pending_key.take().ok_or(Error::InvalidMap)?;
        let id = value
            .serialize(Serializer {
                encoder: self.encoder,
                mode: Mode::Explicit,
            })?
            .ok_or(Error::MissingValue)?;
        self.encoder.child(&mut self.state, key)?;
        self.encoder.child(&mut self.state, id)?;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeStruct for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error> {
        self.field(key, value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}
impl<B: Builder> ser::SerializeStructVariant for Compound<'_, B> {
    type Ok = Option<B::Item>;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error> {
        self.field(key, value)
    }
    fn end(self) -> Result<Self::Ok, Error> {
        self.finish()
    }
}

struct UidValue;
struct DateValue;
struct StringKey<'a, B: Builder> {
    encoder: &'a mut B,
}

// The plist crate presents Uid and Date as private Serde newtypes. These small
// serializers capture only their documented inner scalar representation.
macro_rules! capture_unsupported {
    ($ok:ty) => {
        type Ok = $ok;
        type Error = Error;
        type SerializeSeq = ser::Impossible<Self::Ok, Error>;
        type SerializeTuple = ser::Impossible<Self::Ok, Error>;
        type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
        type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
        type SerializeMap = ser::Impossible<Self::Ok, Error>;
        type SerializeStruct = ser::Impossible<Self::Ok, Error>;
        type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;
        fn serialize_bool(self, _: bool) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_i8(self, _: i8) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_i16(self, _: i16) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_i32(self, _: i32) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_i64(self, _: i64) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_u8(self, _: u8) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_u16(self, _: u16) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_u32(self, _: u32) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_f32(self, _: f32) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_f64(self, _: f64) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_none(self) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_some<T: Serialize + ?Sized>(self, _: &T) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_unit(self) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_newtype_variant<T: Serialize + ?Sized>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Error> {
            Err(Error::InvalidMap)
        }
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Error> {
            Err(Error::InvalidMap)
        }
    };
}

impl ser::Serializer for UidValue {
    capture_unsupported!(u64);
    fn serialize_char(self, _: char) -> Result<u64, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<u64, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, _: &T) -> Result<u64, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_u64(self, v: u64) -> Result<u64, Error> {
        Ok(v)
    }
    fn serialize_str(self, _: &str) -> Result<u64, Error> {
        Err(Error::InvalidMap)
    }
}
impl ser::Serializer for DateValue {
    capture_unsupported!(crate::Date);
    fn serialize_char(self, _: char) -> Result<crate::Date, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<crate::Date, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, _: &T) -> Result<crate::Date, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_u64(self, _: u64) -> Result<crate::Date, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_str(self, value: &str) -> Result<crate::Date, Error> {
        crate::Date::from_xml_format(value).map_err(|_| Error::Message("invalid plist date".into()))
    }
    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<crate::Date, Error> {
        self.serialize_str(&value.to_string())
    }
}

impl<B: Builder> ser::Serializer for StringKey<'_, B> {
    capture_unsupported!(B::Item);
    fn serialize_char(self, value: char) -> Result<B::Item, Error> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<B::Item, Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, value: &T) -> Result<B::Item, Error> {
        value.serialize(self)
    }
    fn serialize_u64(self, _: u64) -> Result<B::Item, Error> {
        Err(Error::InvalidMap)
    }
    fn serialize_str(self, value: &str) -> Result<B::Item, Error> {
        Ok(self.encoder.string(value)?)
    }
}
