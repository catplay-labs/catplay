//! Slice-native binary plist reader. No allocation and no Serde dependency.

use super::format::{self, Trailer, marker};
use super::scalar::{DecodedInteger, ScalarDecoder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooShort,
    InvalidMagic,
    InvalidTrailer,
    OutOfBounds,
    InvalidReference,
    InvalidObject,
    InvalidUtf8,
    IntegerOutOfRange,
}

#[derive(Clone, Copy, Debug)]
pub enum Integer {
    Signed(i64),
    Unsigned(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum Object<'a> {
    Boolean(bool),
    Integer(Integer),
    Real(f64),
    Date(f64),
    Uid(u64),
    Data(&'a [u8]),
    String(&'a str),
    Utf16(&'a [u8]),
    Array(Refs<'a>),
    Dictionary { keys: Refs<'a>, values: Refs<'a> },
}

impl ScalarDecoder for Object<'_> {
    type Error = core::convert::Infallible;

    fn boolean(&self) -> Result<Option<bool>, Self::Error> {
        Ok(if let Self::Boolean(value) = self { Some(*value) } else { None })
    }

    fn integer(&self) -> Result<Option<DecodedInteger>, Self::Error> {
        Ok(match self {
            Self::Integer(Integer::Signed(value)) => Some(DecodedInteger::Signed(*value)),
            Self::Integer(Integer::Unsigned(value)) => Some(DecodedInteger::Unsigned(*value)),
            _ => None,
        })
    }

    fn real(&self) -> Result<Option<f64>, Self::Error> {
        Ok(if let Self::Real(value) = self { Some(*value) } else { None })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Refs<'a> {
    pub(crate) bytes: &'a [u8],
    width: usize,
}

impl Refs<'_> {
    pub fn len(&self) -> usize {
        self.bytes.len() / self.width
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<u64> {
        let start = index.checked_mul(self.width)?;
        let bytes = self.bytes.get(start..start.checked_add(self.width)?)?;
        Some(format::read_integer(bytes))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Document<'a> {
    input: &'a [u8],
    table_offset: usize,
    offset_width: usize,
    ref_width: usize,
    object_count: usize,
    root: u64,
}

impl<'a> Document<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self, Error> {
        if input.len() < format::HEADER.len() + format::TRAILER_SIZE {
            return Err(Error::TooShort);
        }
        if input.get(..format::HEADER.len()) != Some(format::HEADER) {
            return Err(Error::InvalidMagic);
        }
        let trailer_offset = input.len() - format::TRAILER_SIZE;
        let trailer = Trailer::from_bytes(&input[trailer_offset..]);
        let offset_width = trailer.offset_width as usize;
        let ref_width = trailer.ref_width as usize;
        if !format::valid_read_width(offset_width) || !format::valid_read_width(ref_width) {
            return Err(Error::InvalidTrailer);
        }
        let object_count = usize::try_from(trailer.object_count).map_err(|_| Error::InvalidTrailer)?;
        let root = trailer.root;
        let table_offset = usize::try_from(trailer.table_offset).map_err(|_| Error::InvalidTrailer)?;
        if object_count == 0 || root >= object_count as u64 || table_offset < format::HEADER.len() {
            return Err(Error::InvalidTrailer);
        }
        let table_end = table_offset
            .checked_add(
                object_count
                    .checked_mul(offset_width)
                    .ok_or(Error::InvalidTrailer)?,
            )
            .ok_or(Error::InvalidTrailer)?;
        if table_end > trailer_offset {
            return Err(Error::InvalidTrailer);
        }
        let doc = Self {
            input,
            table_offset,
            offset_width,
            ref_width,
            object_count,
            root,
        };
        // Validate every offset once, without copying the table.
        for id in 0..object_count {
            doc.offset(id as u64)?;
        }
        Ok(doc)
    }

    pub fn root(&self) -> u64 {
        self.root
    }
    pub fn object_count(&self) -> usize {
        self.object_count
    }
    pub fn ref_width(&self) -> usize {
        self.ref_width
    }

    /// Looks up a dictionary value without allocating or decoding unrelated values.
    pub fn dictionary_get(&self, dictionary: u64, wanted: &str) -> Result<Option<u64>, Error> {
        let Object::Dictionary { keys, values } = self.object(dictionary)? else {
            return Err(Error::InvalidObject);
        };
        for index in 0..keys.len() {
            let key_id = keys.get(index).ok_or(Error::InvalidReference)?;
            let matches = match self.object(key_id)? {
                Object::String(value) => value == wanted,
                Object::Utf16(bytes) => {
                    bytes.len() / 2 == wanted.encode_utf16().count()
                        && bytes
                            .chunks_exact(2)
                            .zip(wanted.encode_utf16())
                            .all(|(pair, unit)| u16::from_be_bytes([pair[0], pair[1]]) == unit)
                }
                _ => return Err(Error::InvalidObject),
            };
            if matches {
                return values.get(index).ok_or(Error::InvalidReference).map(Some);
            }
        }
        Ok(None)
    }

    fn offset(&self, id: u64) -> Result<usize, Error> {
        let id = usize::try_from(id).map_err(|_| Error::InvalidReference)?;
        if id >= self.object_count {
            return Err(Error::InvalidReference);
        }
        let start = self.table_offset + id * self.offset_width;
        let offset =
            usize::try_from(format::read_integer(&self.input[start..start + self.offset_width])).map_err(|_| Error::OutOfBounds)?;
        if offset < format::HEADER.len() || offset >= self.table_offset {
            return Err(Error::OutOfBounds);
        }
        Ok(offset)
    }

    fn body(&self, start: usize, len: usize) -> Result<&'a [u8], Error> {
        let end = start.checked_add(len).ok_or(Error::OutOfBounds)?;
        if end > self.table_offset {
            return Err(Error::OutOfBounds);
        }
        self.input.get(start..end).ok_or(Error::OutOfBounds)
    }

    fn length(&self, nibble: u8, mut pos: usize) -> Result<(usize, usize), Error> {
        if nibble != marker::EXTENDED_LENGTH {
            return Ok((nibble as usize, pos));
        }
        let code = *self.body(pos, 1)?.first().ok_or(Error::OutOfBounds)?;
        pos += 1;
        if code >> 4 != marker::INTEGER >> 4 || code & 0x0f > 3 {
            return Err(Error::InvalidObject);
        }
        let width = 1usize << (code & 0x0f);
        let len = usize::try_from(format::read_integer(self.body(pos, width)?)).map_err(|_| Error::OutOfBounds)?;
        Ok((len, pos + width))
    }

    pub fn object(&self, id: u64) -> Result<Object<'a>, Error> {
        let start = self.offset(id)?;
        let code = self.input[start];
        let tag = code & 0xf0;
        let nibble = code & 0x0f;
        let pos = start + 1;
        match (tag, nibble) {
            (0, 8) if code == marker::FALSE => Ok(Object::Boolean(false)),
            (0, 9) if code == marker::TRUE => Ok(Object::Boolean(true)),
            (marker::INTEGER, 0..=2) => {
                let width = 1usize << nibble;
                Ok(Object::Integer(Integer::Unsigned(format::read_integer(self.body(pos, width)?))))
            }
            (marker::INTEGER, 3) => {
                let bytes: [u8; 8] = self
                    .body(pos, 8)?
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)?;
                Ok(Object::Integer(Integer::Signed(i64::from_be_bytes(bytes))))
            }
            (marker::INTEGER, 4) => {
                let bytes: [u8; 16] = self
                    .body(pos, 16)?
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)?;
                let value = u64::try_from(i128::from_be_bytes(bytes)).map_err(|_| Error::IntegerOutOfRange)?;
                Ok(Object::Integer(Integer::Unsigned(value)))
            }
            (marker::REAL, 2) => {
                let bytes: [u8; 4] = self
                    .body(pos, 4)?
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)?;
                Ok(Object::Real(f32::from_be_bytes(bytes) as f64))
            }
            (marker::REAL, 3) => {
                let bytes: [u8; 8] = self
                    .body(pos, 8)?
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)?;
                Ok(Object::Real(f64::from_be_bytes(bytes)))
            }
            (marker::DATE, 3) => {
                let bytes: [u8; 8] = self
                    .body(pos, 8)?
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)?;
                Ok(Object::Date(f64::from_be_bytes(bytes)))
            }
            (marker::DATA, _) => {
                let (len, pos) = self.length(nibble, pos)?;
                Ok(Object::Data(self.body(pos, len)?))
            }
            (marker::ASCII, _) => {
                let (len, pos) = self.length(nibble, pos)?;
                let text = core::str::from_utf8(self.body(pos, len)?).map_err(|_| Error::InvalidUtf8)?;
                Ok(Object::String(text))
            }
            (marker::UTF16, _) => {
                let (len, pos) = self.length(nibble, pos)?;
                let bytes = self.body(pos, len.checked_mul(2).ok_or(Error::OutOfBounds)?)?;
                Ok(Object::Utf16(bytes))
            }
            (marker::UID, 0..=7) => Ok(Object::Uid(format::read_integer(self.body(pos, nibble as usize + 1)?))),
            (marker::ARRAY, _) => {
                let (len, pos) = self.length(nibble, pos)?;
                let bytes = self.body(pos, len.checked_mul(self.ref_width).ok_or(Error::OutOfBounds)?)?;
                Ok(Object::Array(Refs {
                    bytes,
                    width: self.ref_width,
                }))
            }
            (marker::DICTIONARY, _) => {
                let (len, pos) = self.length(nibble, pos)?;
                let byte_len = len.checked_mul(self.ref_width).ok_or(Error::OutOfBounds)?;
                let bytes = self.body(pos, byte_len.checked_mul(2).ok_or(Error::OutOfBounds)?)?;
                let (keys, values) = bytes.split_at(byte_len);
                Ok(Object::Dictionary {
                    keys: Refs {
                        bytes: keys,
                        width: self.ref_width,
                    },
                    values: Refs {
                        bytes: values,
                        width: self.ref_width,
                    },
                })
            }
            _ => Err(Error::InvalidObject),
        }
    }
}
