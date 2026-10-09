#[cfg(feature = "serde")]
pub mod serde;
pub mod wire;

#[cfg(feature = "alloc")]
use self::wire::encode::{self, Collection, RefEdge, Scratch, ScratchKind};
#[cfg(feature = "serde")]
use crate::{PlistError, PlistResult};
#[cfg(feature = "serde")]
use alloc::string::ToString;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use bytes::BytesMut;
#[cfg(feature = "alloc")]
use core::convert::Infallible;

#[cfg(feature = "alloc")]
impl wire::encode::Emit for Vec<u8> {
    type Error = Infallible;
    fn emit(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl wire::encode::Emit for BytesMut {
    type Error = Infallible;
    fn emit(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}

/// Reusable storage for object offsets and deferred collection references.
#[derive(Default)]
#[cfg(feature = "alloc")]
pub struct ScratchBuffers {
    offsets: Vec<u64>,
    collections: Vec<Collection>,
    edges: Vec<RefEdge>,
}

#[cfg(feature = "alloc")]
impl ScratchBuffers {
    /// Ensures space for the requested wire scratch, retaining previous capacity.
    pub fn grow(&mut self, kind: ScratchKind, needed: usize) -> Result<(), encode::Error<Infallible>> {
        let buffer_len = needed
            .checked_next_power_of_two()
            .ok_or(encode::Error::Overflow)?;
        match kind {
            ScratchKind::Offsets => self.offsets.resize(buffer_len.max(self.offsets.len()), 0),
            ScratchKind::Collections => self
                .collections
                .resize(buffer_len.max(self.collections.len()), Collection::default()),
            ScratchKind::Edges => self
                .edges
                .resize(buffer_len.max(self.edges.len()), RefEdge::default()),
        }
        Ok(())
    }

    /// Borrows the reusable buffers for a wire encoder without requiring Serde.
    pub fn as_scratch(&mut self) -> Scratch<'_> {
        Scratch {
            offsets: &mut self.offsets,
            collections: &mut self.collections,
            edges: &mut self.edges,
        }
    }
}

#[cfg(feature = "serde")]
impl ScratchBuffers {
    fn measure<T: ::serde::Serialize + ?Sized>(&mut self, value: &T) -> PlistResult<usize> {
        // The first traversal measures the exact serialized length and warms scratch.
        // Collection edges remain necessary for nested forward-only output.
        loop {
            let mut size = 0usize;
            let mut overflow = false;
            let scratch = self.as_scratch();
            match serde::encode::encode(value, scratch, |chunk| {
                if let Some(next) = size.checked_add(chunk.len()) {
                    size = next;
                } else {
                    overflow = true;
                }
                Ok::<(), Infallible>(())
            }) {
                Ok(()) if !overflow => return Ok(size),
                Ok(()) => return Err(PlistError::UnexpectedState),
                Err(serde::encode::Error::Core(encode::Error::ScratchTooSmall { kind, needed, .. })) => {
                    self.grow(kind, needed)
                        .map_err(|_| PlistError::UnexpectedState)?;
                }
                Err(err) => return Err(PlistError::Encode(err.to_string())),
            }
        }
    }

    pub fn encode_into<T: ::serde::Serialize + ?Sized>(&mut self, value: &T, dst: &mut BytesMut) -> PlistResult<()> {
        let encoded_size = self.measure(value)?;
        dst.reserve(encoded_size);
        let start = dst.len();
        let scratch = self.as_scratch();
        match serde::encode::encode(value, scratch, |chunk| {
            dst.extend_from_slice(chunk);
            Ok::<(), Infallible>(())
        }) {
            Ok(()) if dst.len() - start == encoded_size => make_root_first(&mut dst[start..]),
            Ok(()) => {
                dst.truncate(start);
                Err(PlistError::UnexpectedState)
            }
            Err(err) => {
                dst.truncate(start);
                Err(PlistError::Encode(err.to_string()))
            }
        }
    }

    /// Encodes in one traversal when scratch has enough capacity. A capacity
    /// miss truncates the partial output, grows scratch, and retries.
    pub fn encode_into_reusing_capacity<T: ::serde::Serialize + ?Sized>(&mut self, value: &T, dst: &mut BytesMut) -> PlistResult<()> {
        let start = dst.len();
        loop {
            let scratch = self.as_scratch();
            match serde::encode::encode(value, scratch, |chunk| {
                dst.extend_from_slice(chunk);
                Ok::<(), Infallible>(())
            }) {
                Ok(()) => return make_root_first(&mut dst[start..]),
                Err(serde::encode::Error::Core(encode::Error::ScratchTooSmall { kind, needed, .. })) => {
                    dst.truncate(start);
                    self.grow(kind, needed)
                        .map_err(|_| PlistError::UnexpectedState)?;
                }
                Err(err) => {
                    dst.truncate(start);
                    return Err(PlistError::Encode(err.to_string()));
                }
            }
        }
    }

    /// Emits into a caller-owned sink after measuring the required scratch.
    pub fn encode_to_writer<T: ::serde::Serialize + ?Sized, W: wire::encode::Emit>(&mut self, value: &T, dst: &mut W) -> PlistResult<()>
    where
        W::Error: core::error::Error + Send + Sync + 'static,
    {
        let mut encoded = BytesMut::new();
        self.encode_into(value, &mut encoded)?;
        dst.emit(&encoded)
            .map_err(|err| PlistError::Encode(err.to_string()))
    }
}

/// CarPlay 210 reads the root directly at byte 8 instead of consulting the
/// trailer's top-object index. Move its complete body there and repair offsets.
#[cfg(feature = "serde")]
fn make_root_first(data: &mut [u8]) -> PlistResult<()> {
    if data.len() < 41 || &data[..8] != b"bplist00" {
        return Err(PlistError::UnexpectedState);
    }
    let trailer = data.len() - 32;
    let width = data[trailer + 6] as usize;
    if !matches!(width, 1 | 2 | 4 | 8) {
        return Err(PlistError::UnexpectedState);
    }
    let read_u64 = |bytes: &[u8]| -> u64 { bytes.iter().fold(0, |n, &byte| (n << 8) | u64::from(byte)) };
    let count = usize::try_from(read_u64(&data[trailer + 8..trailer + 16])).map_err(|_| PlistError::UnexpectedState)?;
    let root = usize::try_from(read_u64(&data[trailer + 16..trailer + 24])).map_err(|_| PlistError::UnexpectedState)?;
    let table = usize::try_from(read_u64(&data[trailer + 24..trailer + 32])).map_err(|_| PlistError::UnexpectedState)?;
    if root >= count
        || table < 9
        || count
            .checked_mul(width)
            .and_then(|n| table.checked_add(n))
            .is_none_or(|end| end > trailer)
    {
        return Err(PlistError::UnexpectedState);
    }
    let entry = |i: usize| table + i * width;
    let root_start = usize::try_from(read_u64(&data[entry(root)..entry(root) + width])).map_err(|_| PlistError::UnexpectedState)?;
    if !(8..table).contains(&root_start) {
        return Err(PlistError::UnexpectedState);
    }
    if root_start == 8 {
        return Ok(());
    }
    let mut root_end = table;
    for i in 0..count {
        let offset = usize::try_from(read_u64(&data[entry(i)..entry(i) + width])).map_err(|_| PlistError::UnexpectedState)?;
        if !(8..table).contains(&offset) {
            return Err(PlistError::UnexpectedState);
        }
        if offset > root_start {
            root_end = root_end.min(offset);
        }
    }
    let root_len = root_end - root_start;
    data[8..root_end].rotate_right(root_len);
    for i in 0..count {
        let at = entry(i);
        let offset = usize::try_from(read_u64(&data[at..at + width])).map_err(|_| PlistError::UnexpectedState)?;
        let adjusted = if i == root {
            8
        } else if offset < root_start {
            offset
                .checked_add(root_len)
                .ok_or(PlistError::UnexpectedState)?
        } else {
            offset
        };
        let bytes = (adjusted as u64).to_be_bytes();
        data[at..at + width].copy_from_slice(&bytes[8 - width..]);
    }
    Ok(())
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use ::serde::Serialize;
    use alloc::collections::BTreeMap;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Sample<'a> {
        label: &'a str,
        data: &'a crate::PlistByteArray,
        nested: Vec<Vec<u64>>,
        optional: Option<u64>,
        omitted: Option<u64>,
    }

    #[test]
    fn carplay_210_dictionary_root_starts_at_byte_eight() {
        // More than 255 objects also exercises a multi-byte offset table.
        let values: BTreeMap<String, u64> = (0..150)
            .map(|i| (alloc::format!("field{i:03}"), i))
            .collect();
        let mut scratch = ScratchBuffers::default();
        let mut encoded = BytesMut::from(&b"prefix"[..]);
        scratch.encode_into(&values, &mut encoded).unwrap();
        let data = &encoded[6..];
        assert_eq!(data[8] & 0xf0, 0xd0, "CarPlay 210 reads the root at byte 8");

        let trailer = data.len() - 32;
        let width = data[trailer + 6] as usize;
        assert!(width > 1);
        let root = u64::from_be_bytes(data[trailer + 16..trailer + 24].try_into().unwrap()) as usize;
        let table = u64::from_be_bytes(data[trailer + 24..trailer + 32].try_into().unwrap()) as usize;
        let root_offset = data[table + root * width..table + (root + 1) * width]
            .iter()
            .fold(0usize, |n, byte| (n << 8) | *byte as usize);
        assert_eq!(root_offset, 8);
        assert_eq!(crate::from_bytes::<BTreeMap<String, u64>>(data).unwrap(), values);

        let mut streamed = Vec::new();
        crate::to_writer_binary(&mut streamed, &values).unwrap();
        assert_eq!(streamed, data);

        let mut caching = crate::CachingSerializer::default();
        let cached = caching.serialize(&values).unwrap();
        assert_eq!(&cached[..], data);
    }

    fn check<T: Serialize>(value: &T) {
        let mut new = BytesMut::new();
        ScratchBuffers::default()
            .encode_into(value, &mut new)
            .unwrap();
        let _: crate::Value = crate::from_bytes(&new).unwrap();
        let mut streamed = Vec::new();
        crate::to_writer_binary(&mut streamed, value).unwrap();
        assert_eq!(streamed, new);
        let trailer = &new[new.len() - 32..];
        let count = u64::from_be_bytes(trailer[8..16].try_into().unwrap());
        let root = u64::from_be_bytes(trailer[16..24].try_into().unwrap());
        let table = u64::from_be_bytes(trailer[24..32].try_into().unwrap());
        assert!(count > root);
        assert!(table >= 8 && table < (new.len() - 32) as u64);
        assert_eq!(table as usize + count as usize * trailer[6] as usize + 32, new.len());
    }

    #[test]
    fn nested_and_large_payload() {
        let data = crate::PlistByteArray::from(vec![0x52; 4096]);
        let sample = Sample {
            label: "CarPlay zażółć 🚗",
            data: &data,
            nested: vec![(0..300).collect(), vec![u64::MAX, i64::MAX as u64, 0]],
            optional: Some(44),
            omitted: None,
        };
        check(&sample);
    }

    #[test]
    fn plist_special_values() {
        let uid = crate::Value::Uid(crate::Uid::new(65_000));
        check(&uid);
        let date = crate::Date::from_xml_format("2024-01-01T00:00:00Z").unwrap();
        check(&crate::Value::Date(date));
    }

    #[test]
    fn map_keys_and_options() {
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert("alpha", Some(1u64));
        map.insert("beta", None);
        check(&map);
        let mut invalid = BTreeMap::new();
        invalid.insert(1u64, "value");
        let mut bytes = BytesMut::new();
        assert!(
            ScratchBuffers::default()
                .encode_into(&invalid, &mut bytes)
                .is_err()
        );
        assert!(bytes.is_empty());
    }

    #[test]
    fn core_fixed_scratch() {
        let mut offsets = [0; 8];
        let mut collections = [Collection::default(); 2];
        let mut edges = [RefEdge::default(); 3];
        let mut bytes = Vec::new();
        let scratch = Scratch {
            offsets: &mut offsets,
            collections: &mut collections,
            edges: &mut edges,
        };
        let mut encoder = encode::Encoder::new(scratch, |chunk: &[u8]| -> Result<(), ()> {
            bytes.extend_from_slice(chunk);
            Ok(())
        })
        .unwrap();
        let mut array = encoder.begin_array().unwrap();
        let first = encoder.string("hello").unwrap();
        encoder.child(&mut array, first).unwrap();
        let second = encoder.integer(-1).unwrap();
        encoder.child(&mut array, second).unwrap();
        let root = encoder.end(array).unwrap();
        encoder.finish(root).unwrap();
        let decoded: crate::Value = crate::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, crate::Value::Array(vec!["hello".into(), (-1i64).into()]));
    }

    #[test]
    fn writer_error_is_returned() {
        struct FailingWriter;
        impl wire::encode::Emit for FailingWriter {
            type Error = std::io::Error;
            fn emit(&mut self, _: &[u8]) -> Result<(), Self::Error> {
                Err(std::io::Error::other("sink unavailable"))
            }
        }
        let err = ScratchBuffers::default()
            .encode_to_writer(&true, &mut FailingWriter)
            .unwrap_err();
        assert!(err.to_string().contains("sink unavailable"));
    }
}
