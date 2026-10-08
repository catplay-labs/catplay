//! Borrowed XML tokenizer for the plist vocabulary. No allocation or Serde.

use crate::bplist::wire::scalar::{DecodedInteger, ScalarDecoder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidXml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<'a> {
    Start(&'a str),
    End(&'a str),
    Empty(&'a str),
    Text(&'a str),
    CData(&'a str),
    Reference(&'a str),
    Declaration,
    Doctype,
    Comment,
    Eof,
}

/// A scalar element after its text has been collected by the XML adapter.
pub struct Scalar<'a> {
    pub tag: &'a str,
    pub text: &'a str,
}

impl ScalarDecoder for Scalar<'_> {
    type Error = Error;

    fn boolean(&self) -> Result<Option<bool>, Self::Error> {
        match self.tag {
            "true" if self.text.is_empty() => Ok(Some(true)),
            "false" if self.text.is_empty() => Ok(Some(false)),
            "true" | "false" => Err(Error::InvalidXml),
            _ => Ok(None),
        }
    }

    fn integer(&self) -> Result<Option<DecodedInteger>, Self::Error> {
        if self.tag != "integer" {
            return Ok(None);
        }
        let text = self.text.trim();
        if let Some(hex) = text.strip_prefix("0x") {
            return u64::from_str_radix(hex, 16)
                .map(DecodedInteger::Unsigned)
                .map(Some)
                .map_err(|_| Error::InvalidXml);
        }
        if let Ok(value) = text.parse::<i64>() {
            return Ok(Some(DecodedInteger::Signed(value)));
        }
        text.parse::<u64>()
            .map(DecodedInteger::Unsigned)
            .map(Some)
            .map_err(|_| Error::InvalidXml)
    }

    fn real(&self) -> Result<Option<f64>, Self::Error> {
        if self.tag != "real" {
            return Ok(None);
        }
        self.text
            .trim()
            .parse()
            .map(Some)
            .map_err(|_| Error::InvalidXml)
    }
}

pub struct Decoder<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self, Error> {
        let input = core::str::from_utf8(input).map_err(|_| Error::InvalidXml)?;
        Ok(Self {
            input: input.strip_prefix('\u{feff}').unwrap_or(input),
            position: 0,
        })
    }

    pub fn next(&mut self) -> Result<Event<'a>, Error> {
        let rest = &self.input[self.position..];
        if rest.is_empty() {
            return Ok(Event::Eof);
        }
        if let Some(tail) = rest.strip_prefix("<!--") {
            let end = tail.find("-->").ok_or(Error::InvalidXml)?;
            self.position += 4 + end + 3;
            return Ok(Event::Comment);
        }
        if let Some(tail) = rest.strip_prefix("<![CDATA[") {
            let end = tail.find("]]>").ok_or(Error::InvalidXml)?;
            self.position += 9 + end + 3;
            return Ok(Event::CData(&tail[..end]));
        }
        if let Some(tail) = rest.strip_prefix("<?") {
            let end = tail.find("?>").ok_or(Error::InvalidXml)?;
            self.position += 2 + end + 2;
            return Ok(Event::Declaration);
        }
        if let Some(tail) = rest.strip_prefix("<!DOCTYPE") {
            let end = tag_end(tail)?;
            self.position += 9 + end + 1;
            return Ok(Event::Doctype);
        }
        if let Some(tail) = rest.strip_prefix("</") {
            let end = tail.find('>').ok_or(Error::InvalidXml)?;
            let name = tail[..end].trim();
            if !valid_name(name) {
                return Err(Error::InvalidXml);
            }
            self.position += 2 + end + 1;
            return Ok(Event::End(name));
        }
        if let Some(tail) = rest.strip_prefix('<') {
            let end = tag_end(tail)?;
            let content = tail[..end].trim_end();
            let (content, empty) = match content.strip_suffix('/') {
                Some(value) => (value.trim_end(), true),
                None => (content, false),
            };
            let name_end = content.find(char::is_whitespace).unwrap_or(content.len());
            let name = &content[..name_end];
            if !valid_name(name) {
                return Err(Error::InvalidXml);
            }
            self.position += 1 + end + 1;
            return Ok(if empty { Event::Empty(name) } else { Event::Start(name) });
        }
        if let Some(tail) = rest.strip_prefix('&') {
            let end = tail.find(';').ok_or(Error::InvalidXml)?;
            self.position += 1 + end + 1;
            return Ok(Event::Reference(&tail[..end]));
        }
        let end = rest.find(['<', '&']).unwrap_or(rest.len());
        self.position += end;
        Ok(Event::Text(&rest[..end]))
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b':' | b'.'))
}

fn tag_end(input: &str) -> Result<usize, Error> {
    let mut quote = None;
    for (index, byte) in input.bytes().enumerate() {
        match byte {
            b'\'' | b'"' if quote.is_none() => quote = Some(byte),
            b'\'' | b'"' if quote == Some(byte) => quote = None,
            b'>' if quote.is_none() => return Ok(index),
            _ => {}
        }
    }
    Err(Error::InvalidXml)
}

pub fn resolve_reference(reference: &str) -> Option<char> {
    match reference {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let code = if let Some(hex) = reference.strip_prefix("#x") {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                reference.strip_prefix('#')?.parse().ok()?
            };
            char::from_u32(code)
        }
    }
}
