//! Output backends shared by the Serde adapter. The wire writer keeps its
//! caller-owned scratch; Value construction never passes through binary floats.
use alloc::{string::String, vec::Vec};

use super::encode::{EmitError, Error};
use crate::{
    Date, Dictionary, Uid, Value,
    bplist::wire::encode::{CollectionState, Emit, Encoder, ObjectId},
};

pub(super) trait Builder {
    type Item;
    type Collection;
    fn boolean(&mut self, value: bool) -> Result<Self::Item, Error>;
    fn integer(&mut self, value: i64) -> Result<Self::Item, Error>;
    fn unsigned_integer(&mut self, value: u64) -> Result<Self::Item, Error>;
    fn real(&mut self, value: f64) -> Result<Self::Item, Error>;
    fn string(&mut self, value: &str) -> Result<Self::Item, Error>;
    fn data(&mut self, value: &[u8]) -> Result<Self::Item, Error>;
    fn date(&mut self, value: Date) -> Result<Self::Item, Error>;
    fn uid(&mut self, value: u64) -> Result<Self::Item, Error>;
    fn begin_array(&mut self) -> Result<Self::Collection, Error>;
    fn begin_dict(&mut self) -> Result<Self::Collection, Error>;
    fn child(&mut self, state: &mut Self::Collection, value: Self::Item) -> Result<(), Error>;
    fn end(&mut self, state: Self::Collection) -> Result<Self::Item, Error>;
}

impl<E: Emit<Error = EmitError>> Builder for Encoder<'_, E> {
    type Item = ObjectId;
    type Collection = CollectionState;

    fn boolean(&mut self, value: bool) -> Result<ObjectId, Error> {
        Ok(Encoder::boolean(self, value)?)
    }
    fn integer(&mut self, value: i64) -> Result<ObjectId, Error> {
        Ok(Encoder::integer(self, value)?)
    }
    fn unsigned_integer(&mut self, value: u64) -> Result<ObjectId, Error> {
        Ok(Encoder::unsigned_integer(self, value, false)?)
    }
    fn real(&mut self, value: f64) -> Result<ObjectId, Error> {
        Ok(Encoder::real(self, value)?)
    }
    fn string(&mut self, value: &str) -> Result<ObjectId, Error> {
        Ok(Encoder::string(self, value)?)
    }
    fn data(&mut self, value: &[u8]) -> Result<ObjectId, Error> {
        Ok(Encoder::data(self, value)?)
    }
    fn date(&mut self, value: Date) -> Result<ObjectId, Error> {
        Ok(Encoder::date(self, value.as_seconds_since_plist_epoch())?)
    }
    fn uid(&mut self, value: u64) -> Result<ObjectId, Error> {
        Ok(Encoder::uid(self, value)?)
    }
    fn begin_array(&mut self) -> Result<CollectionState, Error> {
        Ok(Encoder::begin_array(self)?)
    }
    fn begin_dict(&mut self) -> Result<CollectionState, Error> {
        Ok(Encoder::begin_dict(self)?)
    }
    fn child(&mut self, state: &mut CollectionState, value: ObjectId) -> Result<(), Error> {
        Ok(Encoder::child(self, state, value)?)
    }
    fn end(&mut self, state: CollectionState) -> Result<ObjectId, Error> {
        Ok(Encoder::end(self, state)?)
    }
}

pub(super) struct ValueBuilder;

pub(super) enum ValueCollection {
    Array(Vec<Value>),
    Dictionary(Dictionary, Option<String>),
}

impl Builder for ValueBuilder {
    type Item = Value;
    type Collection = ValueCollection;

    fn boolean(&mut self, value: bool) -> Result<Value, Error> {
        Ok(Value::Boolean(value))
    }
    fn integer(&mut self, value: i64) -> Result<Value, Error> {
        Ok(Value::Integer(value.into()))
    }
    fn unsigned_integer(&mut self, value: u64) -> Result<Value, Error> {
        Ok(Value::Integer(value.into()))
    }
    fn real(&mut self, value: f64) -> Result<Value, Error> {
        Ok(Value::Real(value))
    }
    fn string(&mut self, value: &str) -> Result<Value, Error> {
        Ok(Value::String(value.into()))
    }
    fn data(&mut self, value: &[u8]) -> Result<Value, Error> {
        Ok(Value::Data(value.into()))
    }
    fn date(&mut self, value: Date) -> Result<Value, Error> {
        Ok(Value::Date(value))
    }
    fn uid(&mut self, value: u64) -> Result<Value, Error> {
        Ok(Value::Uid(Uid::new(value)))
    }
    fn begin_array(&mut self) -> Result<ValueCollection, Error> {
        Ok(ValueCollection::Array(Vec::new()))
    }
    fn begin_dict(&mut self) -> Result<ValueCollection, Error> {
        Ok(ValueCollection::Dictionary(Dictionary::new(), None))
    }
    fn child(&mut self, state: &mut ValueCollection, value: Value) -> Result<(), Error> {
        match state {
            ValueCollection::Array(values) => values.push(value),
            ValueCollection::Dictionary(values, key) => {
                if let Some(key) = key.take() {
                    values.insert(key, value);
                } else {
                    *key = Some(value.into_string().ok_or(Error::InvalidMap)?);
                }
            }
        }
        Ok(())
    }
    fn end(&mut self, state: ValueCollection) -> Result<Value, Error> {
        match state {
            ValueCollection::Array(values) => Ok(Value::Array(values)),
            ValueCollection::Dictionary(values, None) => Ok(Value::Dictionary(values)),
            ValueCollection::Dictionary(_, Some(_)) => Err(Error::InvalidMap),
        }
    }
}
