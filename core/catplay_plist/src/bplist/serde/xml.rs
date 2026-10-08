//! Alloc-enabled XML plist adapter over borrowed wire events.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::{
    Date, Dictionary, Error, Value,
    bplist::wire::{
        scalar::{DecodedInteger, ScalarDecoder, ScalarEncoder},
        xml::{
            decode::{Decoder, Event, Scalar, resolve_reference},
            encode::{Encoder, Options},
        },
    },
};

const MAX_DEPTH: usize = 128;

pub fn from_slice(input: &[u8]) -> Result<Value, Error> {
    let mut decoder = Decoder::new(input).map_err(|_| Error::message("invalid XML plist"))?;
    let mut root = None;
    let mut in_plist = false;
    loop {
        match decoder
            .next()
            .map_err(|_| Error::message("invalid XML plist"))?
        {
            Event::Declaration | Event::Doctype | Event::Comment => {}
            Event::Text(text) if text.bytes().all(|byte| byte.is_ascii_whitespace()) => {}
            Event::Start("plist") if !in_plist && root.is_none() => in_plist = true,
            Event::Start(start) if root.is_none() => root = Some(read_value(&mut decoder, start, 0)?),
            Event::Empty(start) if root.is_none() => root = Some(read_empty(start)?),
            Event::End("plist") if in_plist => in_plist = false,
            Event::Eof if !in_plist => return root.ok_or_else(|| Error::message("empty XML plist")),
            _ => return Err(Error::message("invalid XML plist root")),
        }
    }
}

fn next_significant<'a>(decoder: &mut Decoder<'a>) -> Result<Event<'a>, Error> {
    loop {
        let event = decoder
            .next()
            .map_err(|_| Error::message("invalid XML plist"))?;
        match event {
            Event::Comment => continue,
            Event::Text(text) if text.bytes().all(|byte| byte.is_ascii_whitespace()) => continue,
            _ => return Ok(event),
        }
    }
}

fn read_text(decoder: &mut Decoder<'_>, tag: &str) -> Result<String, Error> {
    let mut result = String::new();
    loop {
        match decoder
            .next()
            .map_err(|_| Error::message("invalid XML plist"))?
        {
            Event::Text(text) | Event::CData(text) => result.push_str(text),
            Event::Reference(reference) => result.push(resolve_reference(reference).ok_or_else(|| Error::message("invalid XML entity"))?),
            Event::End(end) if end == tag => return Ok(result),
            Event::Comment => {}
            _ => return Err(Error::message("invalid XML scalar")),
        }
    }
}

fn read_empty(tag: &str) -> Result<Value, Error> {
    match tag {
        "array" => Ok(Value::Array(Vec::new())),
        "dict" => Ok(Value::Dictionary(Dictionary::new())),
        "string" => Ok(Value::String(String::new())),
        "data" => Ok(Value::Data(Vec::new())),
        "true" => Ok(Value::Boolean(true)),
        "false" => Ok(Value::Boolean(false)),
        _ => Err(Error::message("invalid empty XML plist element")),
    }
}

fn read_value(decoder: &mut Decoder<'_>, start: &str, depth: usize) -> Result<Value, Error> {
    if depth >= MAX_DEPTH {
        return Err(Error::message("XML plist depth limit"));
    }
    match start {
        "array" => {
            let mut values = Vec::new();
            loop {
                match next_significant(decoder)? {
                    Event::Start(child) => values.push(read_value(decoder, child, depth + 1)?),
                    Event::Empty(child) => values.push(read_empty(child)?),
                    Event::End("array") => return Ok(Value::Array(values)),
                    _ => return Err(Error::message("invalid XML array")),
                }
            }
        }
        "dict" => {
            let mut values = Dictionary::new();
            loop {
                match next_significant(decoder)? {
                    key_event @ (Event::Start("key") | Event::Empty("key")) => {
                        let key = if matches!(key_event, Event::Empty(_)) {
                            String::new()
                        } else {
                            read_text(decoder, "key")?
                        };
                        let value = match next_significant(decoder)? {
                            Event::Start(child) => read_value(decoder, child, depth + 1)?,
                            Event::Empty(child) => read_empty(child)?,
                            _ => return Err(Error::message("missing XML dictionary value")),
                        };
                        values.insert(key, value);
                    }
                    Event::End("dict") => return Ok(Value::Dictionary(values)),
                    _ => return Err(Error::message("invalid XML dictionary")),
                }
            }
        }
        "true" | "false" => {
            let tag = start;
            let text = read_text(decoder, tag)?;
            let value = Scalar { tag, text: &text }
                .boolean()
                .map_err(|_| Error::message("invalid XML boolean"))?;
            Ok(Value::Boolean(value.ok_or_else(|| Error::message("invalid XML boolean"))?))
        }
        "string" => Ok(Value::String(read_text(decoder, "string")?)),
        "integer" => {
            let text = read_text(decoder, "integer")?;
            let number = match (Scalar {
                tag: "integer",
                text: &text,
            })
            .integer()
            .map_err(|_| Error::message("invalid XML integer"))?
            {
                Some(DecodedInteger::Signed(value)) => value.into(),
                Some(DecodedInteger::Unsigned(value)) => value.into(),
                None => return Err(Error::message("invalid XML integer")),
            };
            Ok(Value::Integer(number))
        }
        "real" => {
            let text = read_text(decoder, "real")?;
            Ok(Value::Real(
                (Scalar { tag: "real", text: &text })
                    .real()
                    .map_err(|_| Error::message("invalid XML real"))?
                    .ok_or_else(|| Error::message("invalid XML real"))?,
            ))
        }
        "date" => Ok(Value::Date(
            Date::from_xml_format(&read_text(decoder, "date")?).map_err(|_| Error::message("invalid XML date"))?,
        )),
        "data" => {
            let text = read_text(decoder, "data")?;
            let data: Vec<u8> = text
                .bytes()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect();
            Ok(Value::Data(
                STANDARD
                    .decode(data)
                    .map_err(|_| Error::message("invalid XML data"))?,
            ))
        }
        _ => Err(Error::message("unknown XML plist element")),
    }
}

pub fn to_emitter<F, E>(value: &Value, options: Options, emit: F) -> Result<(), Error>
where
    F: FnMut(&[u8]) -> Result<(), E>,
    E: core::fmt::Display,
{
    let mut writer = Encoder::new(emit, options).map_err(|err| Error::message(err.to_string()))?;
    write_value(&mut writer, value, 0).map_err(|err| Error::message(err.to_string()))?;
    writer
        .finish()
        .map_err(|err| Error::message(err.to_string()))
}

fn write_value<E: crate::bplist::wire::encode::Emit>(
    writer: &mut Encoder<E>,
    value: &Value,
    depth: usize,
) -> Result<(), crate::bplist::wire::xml::encode::Error<E::Error>> {
    if depth >= MAX_DEPTH {
        return Err(crate::bplist::wire::xml::encode::Error::InvalidState);
    }
    match value {
        Value::Array(values) => {
            writer.begin_array()?;
            for value in values {
                write_value(writer, value, depth + 1)?;
            }
            writer.end_array()
        }
        Value::Dictionary(values) => {
            writer.begin_dict()?;
            for (key, value) in values {
                writer.key(key)?;
                write_value(writer, value, depth + 1)?;
            }
            writer.end_dict()
        }
        Value::Boolean(value) => writer.boolean(*value),
        Value::Data(value) => writer.data(value),
        Value::Date(value) => writer.date_text(value.xml_text().as_str()),
        Value::Real(value) => writer.real(*value),
        Value::Integer(value) => match value.as_signed() {
            Some(value) => writer.integer(value),
            None => writer.unsigned_integer(
                value
                    .as_unsigned()
                    .ok_or(crate::bplist::wire::xml::encode::Error::InvalidState)?,
            ),
        },
        Value::String(value) => writer.string(value),
        Value::Uid(value) => writer.uid(value.get()),
    }
}
