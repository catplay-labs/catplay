//! Binary plist writer. This module uses only `core` and caller-owned storage.
//! Object bodies may occur in any physical order; the offset table maps IDs to bodies.

use super::format::{self, Trailer, marker};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectId(pub u64);

#[derive(Clone, Copy, Debug, Default)]
pub struct RefEdge {
    pub object: ObjectId,
    pub next: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Collection {
    pub object: ObjectId,
    pub first: Option<usize>,
    pub count: usize,
    pub dictionary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchKind {
    Offsets,
    Collections,
    Edges,
}

#[derive(Debug)]
pub enum Error<E> {
    Emit(E),
    ScratchTooSmall {
        kind: ScratchKind,
        needed: usize,
        available: usize,
    },
    InvalidCollection,
    Overflow,
}

pub trait Emit {
    type Error;
    fn emit(&mut self, chunk: &[u8]) -> Result<(), Self::Error>;
}

impl<F, E> Emit for F
where
    F: FnMut(&[u8]) -> Result<(), E>,
{
    type Error = E;
    fn emit(&mut self, chunk: &[u8]) -> Result<(), E> {
        self(chunk)
    }
}

pub struct Scratch<'a> {
    pub offsets: &'a mut [u64],
    pub collections: &'a mut [Collection],
    pub edges: &'a mut [RefEdge],
}

#[derive(Debug)]
pub struct CollectionState {
    object: ObjectId,
    first: Option<usize>,
    last: Option<usize>,
    count: usize,
    dictionary: bool,
}

pub trait ObjectIndex {
    fn unique(&mut self) -> Option<ObjectId>;
    fn scalar(&mut self, scalar: ScalarRef<'_>) -> Option<InternResult>;
    fn len(&self) -> u64;
}

/// Borrowed scalar identity supplied to a possible future interning policy.
#[derive(Clone, Copy, Debug)]
pub enum ScalarRef<'a> {
    Boolean(bool),
    Integer { value: u64, signed_negative: bool },
    Real(u64),
    Date(u64),
    Uid(u64),
    String(&'a str),
    Data(&'a [u8]),
}

#[derive(Clone, Copy, Debug)]
pub struct InternResult {
    pub id: ObjectId,
    pub is_new: bool,
}

#[derive(Default)]
pub struct SequentialObjectIndex {
    next: u64,
}

impl ObjectIndex for SequentialObjectIndex {
    fn unique(&mut self) -> Option<ObjectId> {
        let id = ObjectId(self.next);
        self.next = self.next.checked_add(1)?;
        Some(id)
    }
    fn scalar(&mut self, _scalar: ScalarRef<'_>) -> Option<InternResult> {
        self.unique().map(|id| InternResult { id, is_new: true })
    }
    fn len(&self) -> u64 {
        self.next
    }
}

pub struct Encoder<'a, E: Emit, I: ObjectIndex = SequentialObjectIndex> {
    emit: E,
    scratch: Scratch<'a>,
    index: I,
    position: u64,
    collections: usize,
    open_collections: usize,
    edges: usize,
}

impl<'a, E: Emit> Encoder<'a, E> {
    pub fn new(scratch: Scratch<'a>, emit: E) -> Result<Self, Error<E::Error>> {
        Self::with_index(scratch, emit, SequentialObjectIndex::default())
    }
}

impl<'a, E: Emit, I: ObjectIndex> Encoder<'a, E, I> {
    pub fn with_index(scratch: Scratch<'a>, emit: E, index: I) -> Result<Self, Error<E::Error>> {
        let mut writer = Self {
            emit,
            scratch,
            index,
            position: 0,
            collections: 0,
            open_collections: 0,
            edges: 0,
        };
        writer.write(format::HEADER)?;
        Ok(writer)
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Error<E::Error>> {
        let next = self
            .position
            .checked_add(bytes.len() as u64)
            .ok_or(Error::Overflow)?;
        self.emit.emit(bytes).map_err(Error::Emit)?;
        self.position = next;
        Ok(())
    }

    fn slot(&mut self, scalar: Option<ScalarRef<'_>>) -> Result<(ObjectId, bool), Error<E::Error>> {
        let result = match scalar {
            Some(scalar) => self.index.scalar(scalar).ok_or(Error::Overflow)?,
            None => InternResult {
                id: self.index.unique().ok_or(Error::Overflow)?,
                is_new: true,
            },
        };
        let n = usize::try_from(result.id.0).map_err(|_| Error::Overflow)?;
        if n >= self.scratch.offsets.len() {
            return Err(Error::ScratchTooSmall {
                kind: ScratchKind::Offsets,
                needed: n + 1,
                available: self.scratch.offsets.len(),
            });
        }
        if result.is_new {
            self.scratch.offsets[n] = self.position;
        }
        Ok((result.id, result.is_new))
    }

    fn scalar_bytes(&mut self, scalar: ScalarRef<'_>, bytes: &[u8]) -> Result<ObjectId, Error<E::Error>> {
        let (id, new) = self.slot(Some(scalar))?;
        if new {
            self.write(bytes)?;
        }
        Ok(id)
    }

    pub fn boolean(&mut self, value: bool) -> Result<ObjectId, Error<E::Error>> {
        self.scalar_bytes(ScalarRef::Boolean(value), &[if value { marker::TRUE } else { marker::FALSE }])
    }

    pub fn integer(&mut self, value: i64) -> Result<ObjectId, Error<E::Error>> {
        self.unsigned_integer(value as u64, value < 0)
    }

    pub fn unsigned_integer(&mut self, value: u64, signed_negative: bool) -> Result<ObjectId, Error<E::Error>> {
        let mut buf = [0u8; 17];
        let (marker, width) = if signed_negative {
            (marker::INTEGER | 3, 8)
        } else if value <= u8::MAX as u64 {
            (marker::INTEGER, 1)
        } else if value <= u16::MAX as u64 {
            (marker::INTEGER | 1, 2)
        } else if value <= u32::MAX as u64 {
            (marker::INTEGER | 2, 4)
        } else if value <= i64::MAX as u64 {
            (marker::INTEGER | 3, 8)
        } else {
            (marker::INTEGER | 4, 16)
        };
        buf[0] = marker;
        if width <= 8 {
            buf[1..1 + width].copy_from_slice(&value.to_be_bytes()[8 - width..]);
        }
        // Unsigned values above i64::MAX are positive signed 128-bit values.
        if width == 16 {
            buf[1..9].fill(0);
            buf[9..17].copy_from_slice(&value.to_be_bytes());
        }
        self.scalar_bytes(ScalarRef::Integer { value, signed_negative }, &buf[..1 + width])
    }

    pub fn real(&mut self, value: f64) -> Result<ObjectId, Error<E::Error>> {
        let mut bytes = [0; 9];
        bytes[0] = marker::REAL | 3;
        bytes[1..].copy_from_slice(&value.to_be_bytes());
        self.scalar_bytes(ScalarRef::Real(value.to_bits()), &bytes)
    }

    pub fn date(&mut self, seconds: f64) -> Result<ObjectId, Error<E::Error>> {
        let mut bytes = [0; 9];
        bytes[0] = marker::DATE | 3;
        bytes[1..].copy_from_slice(&seconds.to_be_bytes());
        self.scalar_bytes(ScalarRef::Date(seconds.to_bits()), &bytes)
    }

    pub fn uid(&mut self, value: u64) -> Result<ObjectId, Error<E::Error>> {
        let width = format::integer_width(value);
        let mut bytes = [0; 9];
        bytes[0] = marker::UID | (width - 1) as u8;
        bytes[1..1 + width].copy_from_slice(&value.to_be_bytes()[8 - width..]);
        self.scalar_bytes(ScalarRef::Uid(value), &bytes[..1 + width])
    }

    fn sized(&mut self, kind: u8, size: usize) -> Result<(), Error<E::Error>> {
        if size < 15 {
            return self.write(&[kind | size as u8]);
        }
        self.write(&[kind | marker::EXTENDED_LENGTH])?;
        let size = u64::try_from(size).map_err(|_| Error::Overflow)?;
        let width = format::integer_width(size);
        self.write(&[marker::INTEGER | width.trailing_zeros() as u8])?;
        self.write(&size.to_be_bytes()[8 - width..])
    }

    pub fn data(&mut self, value: &[u8]) -> Result<ObjectId, Error<E::Error>> {
        let (id, new) = self.slot(Some(ScalarRef::Data(value)))?;
        if new {
            self.sized(marker::DATA, value.len())?;
            self.write(value)?;
        }
        Ok(id)
    }

    pub fn string(&mut self, value: &str) -> Result<ObjectId, Error<E::Error>> {
        let (id, new) = self.slot(Some(ScalarRef::String(value)))?;
        if new {
            if value.is_ascii() {
                self.sized(marker::ASCII, value.len())?;
                self.write(value.as_bytes())?;
            } else {
                self.sized(marker::UTF16, value.encode_utf16().count())?;
                let mut buf = [0u8; 128];
                let mut used = 0;
                for code in value.encode_utf16() {
                    buf[used..used + 2].copy_from_slice(&code.to_be_bytes());
                    used += 2;
                    if used == buf.len() {
                        self.write(&buf)?;
                        used = 0;
                    }
                }
                if used != 0 {
                    self.write(&buf[..used])?;
                }
            }
        }
        Ok(id)
    }

    pub fn begin_array(&mut self) -> Result<CollectionState, Error<E::Error>> {
        self.begin(false)
    }
    pub fn begin_dict(&mut self) -> Result<CollectionState, Error<E::Error>> {
        self.begin(true)
    }
    fn begin(&mut self, dictionary: bool) -> Result<CollectionState, Error<E::Error>> {
        let (object, _) = self.slot(None)?;
        self.open_collections = self
            .open_collections
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        Ok(CollectionState {
            object,
            first: None,
            last: None,
            count: 0,
            dictionary,
        })
    }

    pub fn child(&mut self, state: &mut CollectionState, object: ObjectId) -> Result<(), Error<E::Error>> {
        if object.0 >= self.index.len() {
            return Err(Error::InvalidCollection);
        }
        if self.edges >= self.scratch.edges.len() {
            return Err(Error::ScratchTooSmall {
                kind: ScratchKind::Edges,
                needed: self.edges + 1,
                available: self.scratch.edges.len(),
            });
        }
        let edge = self.edges;
        self.edges += 1;
        self.scratch.edges[edge] = RefEdge { object, next: None };
        if let Some(last) = state.last {
            self.scratch.edges[last].next = Some(edge);
        } else {
            state.first = Some(edge);
        }
        state.last = Some(edge);
        state.count = state.count.checked_add(1).ok_or(Error::Overflow)?;
        Ok(())
    }

    pub fn end(&mut self, state: CollectionState) -> Result<ObjectId, Error<E::Error>> {
        if self.open_collections == 0 || state.object.0 >= self.index.len() {
            return Err(Error::InvalidCollection);
        }
        if state.dictionary && state.count % 2 != 0 {
            return Err(Error::InvalidCollection);
        }
        if self.collections >= self.scratch.collections.len() {
            return Err(Error::ScratchTooSmall {
                kind: ScratchKind::Collections,
                needed: self.collections + 1,
                available: self.scratch.collections.len(),
            });
        }
        self.scratch.collections[self.collections] = Collection {
            object: state.object,
            first: state.first,
            count: state.count,
            dictionary: state.dictionary,
        };
        self.collections += 1;
        self.open_collections -= 1;
        Ok(state.object)
    }

    fn write_ref(&mut self, object: ObjectId, width: usize) -> Result<(), Error<E::Error>> {
        self.write(&object.0.to_be_bytes()[8 - width..])
    }

    pub fn finish(mut self, root: ObjectId) -> Result<(), Error<E::Error>> {
        let num = self.index.len();
        if num == 0 || root.0 >= num || self.open_collections != 0 {
            return Err(Error::InvalidCollection);
        }
        let ref_width = format::integer_width(num - 1);
        for i in 0..self.collections {
            let c = self.scratch.collections[i];
            self.scratch.offsets[c.object.0 as usize] = self.position;
            self.sized(
                if c.dictionary { marker::DICTIONARY } else { marker::ARRAY },
                if c.dictionary { c.count / 2 } else { c.count },
            )?;
            if c.dictionary {
                for parity in 0..2 {
                    let mut edge = c.first;
                    for n in 0..c.count {
                        let e = self.scratch.edges[edge.ok_or(Error::InvalidCollection)?];
                        if n % 2 == parity {
                            self.write_ref(e.object, ref_width)?;
                        }
                        edge = e.next;
                    }
                }
            } else {
                let mut edge = c.first;
                for _ in 0..c.count {
                    let e = self.scratch.edges[edge.ok_or(Error::InvalidCollection)?];
                    self.write_ref(e.object, ref_width)?;
                    edge = e.next;
                }
            }
        }
        let table_offset = self.position;
        let offset_width = format::integer_width(table_offset);
        for i in 0..num as usize {
            self.write(&self.scratch.offsets[i].to_be_bytes()[8 - offset_width..])?;
        }
        self.write(
            &Trailer {
                offset_width: offset_width as u8,
                ref_width: ref_width as u8,
                object_count: num,
                root: root.0,
                table_offset,
            }
            .to_bytes(),
        )
    }
}

impl<E: Emit, I: ObjectIndex> super::scalar::ScalarEncoder for Encoder<'_, E, I> {
    type Error = Error<E::Error>;
    type Item = ObjectId;

    fn boolean(&mut self, value: bool) -> Result<ObjectId, Self::Error> {
        Encoder::boolean(self, value)
    }
    fn integer(&mut self, value: i64) -> Result<ObjectId, Self::Error> {
        Encoder::integer(self, value)
    }
    fn unsigned_integer(&mut self, value: u64) -> Result<ObjectId, Self::Error> {
        Encoder::unsigned_integer(self, value, false)
    }
    fn real(&mut self, value: f64) -> Result<ObjectId, Self::Error> {
        Encoder::real(self, value)
    }
    fn string(&mut self, value: &str) -> Result<ObjectId, Self::Error> {
        Encoder::string(self, value)
    }
    fn data(&mut self, value: &[u8]) -> Result<ObjectId, Self::Error> {
        Encoder::data(self, value)
    }
    fn uid(&mut self, value: u64) -> Result<ObjectId, Self::Error> {
        Encoder::uid(self, value)
    }
}
