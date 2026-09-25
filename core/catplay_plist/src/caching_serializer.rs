use std::{
    io::{self, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex, MutexGuard},
};

use bytes::BytesMut;
use serde::{Serialize, de::DeserializeOwned};

use crate::{PlistError, PlistResult};

#[derive(Default)]
struct SharedBytesMutIoState {
    write_buf: Option<BytesMut>,
}

#[derive(Clone, Default)]
struct SharedBytesMutIo {
    state: Arc<Mutex<SharedBytesMutIoState>>,
}

impl SharedBytesMutIo {
    #[inline]
    fn lock_state(&self) -> PlistResult<MutexGuard<'_, SharedBytesMutIoState>> {
        self.state.lock().map_err(|_| PlistError::UnexpectedState)
    }

    #[inline]
    fn lock_state_io(&self) -> io::Result<MutexGuard<'_, SharedBytesMutIoState>> {
        self.state
            .lock()
            .map_err(|_| io::Error::other("internal state mutex poisoned"))
    }

    fn bind_write_buffer(&self, buf: BytesMut) -> PlistResult<()> {
        let mut state = self.lock_state()?;
        if state.write_buf.is_some() {
            return Err(PlistError::UnexpectedState);
        }
        state.write_buf = Some(buf);
        Ok(())
    }

    fn take_write_buffer(&self) -> PlistResult<BytesMut> {
        let mut state = self.lock_state()?;
        state.write_buf.take().ok_or(PlistError::UnexpectedState)
    }
}

impl Write for SharedBytesMutIo {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut state = self.lock_state_io()?;
        let Some(buf) = state.write_buf.as_mut() else {
            return Err(io::Error::other("no active BytesMut bound"));
        };

        buf.extend_from_slice(data);
        Ok(data.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct SliceIo<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SliceIo<'a> {
    #[inline]
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl Read for SliceIo<'_> {
    #[inline]
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let remaining = self.data.len().saturating_sub(self.pos);
        if remaining == 0 || out.is_empty() {
            return Ok(0);
        }

        let n = remaining.min(out.len());
        let start = self.pos;
        let end = start + n;
        out[..n].copy_from_slice(&self.data[start..end]);
        self.pos = end;
        Ok(n)
    }
}

impl Seek for SliceIo<'_> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.data.len() as i64;
        let cur = self.pos as i64;

        let next = match pos {
            SeekFrom::Start(v) => i64::try_from(v).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?,
            SeekFrom::Current(v) => cur.saturating_add(v),
            SeekFrom::End(v) => len.saturating_add(v),
        };

        if next < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before start"));
        }

        self.pos = next as usize;
        Ok(self.pos as u64)
    }
}

/// A caching layer for `plist`, reducing allocations.
///
/// - returned [BytesMut] will automatically re-use backing allocations if it's dropped before next call to `serialize`
///   _and_ the size of serialized payload didn't exceed pre-configured cache limit (4KB by default, enters CoW-style realloc when exceeded)
/// - `BinaryWriter` and `BinaryReader` are re-used, allowing internal collections to stay warmed-up
pub struct CachingSerializer {
    buf: BytesMut,
    cache_size: usize,
    sink: SharedBytesMutIo,
    writer: Option<plist::stream::BinaryWriter<SharedBytesMutIo>>,
}

impl Default for CachingSerializer {
    fn default() -> Self {
        Self::new(Self::CACHE_DEFAULT)
    }
}

impl CachingSerializer {
    pub const CACHE_DEFAULT: usize = 4096;

    pub fn new(cache_size: usize) -> Self {
        let sink = SharedBytesMutIo::default();
        let writer = plist::stream::BinaryWriter::new(sink.clone());
        Self {
            buf: BytesMut::new(),
            cache_size,
            sink,
            writer: Some(writer),
        }
    }

    pub fn serialize<T: Serialize>(&mut self, value: &T) -> PlistResult<BytesMut> {
        let mut serialize_fn = |ser: &mut plist::Serializer<plist::stream::BinaryWriter<SharedBytesMutIo>>| value.serialize(ser);
        self.serialize_inner(&mut serialize_fn)
    }

    #[allow(clippy::type_complexity)]
    fn serialize_inner(
        &mut self,
        serialize_fn: &mut dyn FnMut(&mut plist::Serializer<plist::stream::BinaryWriter<SharedBytesMutIo>>) -> Result<(), plist::Error>,
    ) -> PlistResult<BytesMut> {
        self.buf.reserve(self.cache_size);
        let out = self.buf.split_off(0);
        self.sink.bind_write_buffer(out)?;

        let writer = self.writer.take().ok_or(PlistError::UnexpectedState)?;
        let mut ser = plist::Serializer::new(writer);
        let result = serialize_fn(&mut ser);
        self.writer = Some(ser.into_inner());

        match result {
            Ok(()) => self.sink.take_write_buffer(),
            Err(err) => {
                if let Ok(buf) = self.sink.take_write_buffer() {
                    self.reclaim(buf);
                }
                Err(err.into())
            }
        }
    }

    pub fn deserialize<T: DeserializeOwned>(&mut self, data: &[u8]) -> PlistResult<T> {
        self.deserializer_inner(data)
    }

    fn deserializer_inner<T: DeserializeOwned>(&mut self, data: &[u8]) -> PlistResult<T> {
        if data.is_empty() {
            return Err(PlistError::Empty);
        }

        let reader = plist::stream::BinaryReader::new(SliceIo::new(data));
        let mut de = plist::Deserializer::new(reader);
        <T as serde::Deserialize>::deserialize(&mut de).map_err(PlistError::from)
    }

    pub fn reclaim(&mut self, mut buf: BytesMut) {
        buf.clear();
        self.buf = buf;
    }
}

#[cfg(test)]
mod tests {
    use crate::{CachingSerializer, PlistByteArray, plist_decode, plist_encode, plist_struct};

    plist_struct! {
         struct InfoMessageResponse {
            pub oem_icon: Option<PlistByteArray>,
        }
    }

    plist_struct! {
         struct TextMessage {
            pub text: String,
        }
    }

    #[test]
    fn test_caching_serializer_roundtrip() {
        let mut caching = CachingSerializer::new(CachingSerializer::CACHE_DEFAULT);
        let mut b = InfoMessageResponse::default();
        b.oem_icon
            .replace(PlistByteArray::from(vec![1, 2, 3, 4, 5]));

        let encoded = caching.serialize(&b).unwrap();
        let decoded: InfoMessageResponse = caching.deserialize(&encoded).unwrap();

        assert_eq!(decoded, b);
    }

    #[test]
    fn test_caching_serializer_reclaim() {
        let mut caching = CachingSerializer::new(CachingSerializer::CACHE_DEFAULT);
        let mut b = InfoMessageResponse::default();
        b.oem_icon
            .replace(PlistByteArray::from(vec![1, 2, 3, 4, 5]));

        let encoded1 = caching.serialize(&b).unwrap();
        let decoded1: InfoMessageResponse = plist_decode(&encoded1).unwrap();
        assert_eq!(decoded1, b);

        caching.reclaim(encoded1);

        let mut c = InfoMessageResponse::default();
        c.oem_icon.replace(PlistByteArray::from(vec![9, 8, 7, 6]));
        let encoded2 = caching.serialize(&c).unwrap();
        let decoded2: InfoMessageResponse = plist_decode(&encoded2).unwrap();
        assert_eq!(decoded2, c);
    }

    #[test]
    fn test_caching_serializer_double_serialize_different_objects() {
        let mut caching = CachingSerializer::new(CachingSerializer::CACHE_DEFAULT);

        let mut a = InfoMessageResponse::default();
        a.oem_icon.replace(PlistByteArray::from(vec![1, 2, 3]));
        let encoded_a = caching.serialize(&a).unwrap();

        let b = TextMessage { text: "hello".into() };
        let encoded_b = caching.serialize(&b).unwrap();

        let decoded_a: InfoMessageResponse = plist_decode(&encoded_a).unwrap();
        let decoded_b: TextMessage = plist_decode(&encoded_b).unwrap();
        assert_eq!(decoded_a, a);
        assert_eq!(decoded_b, b);
        assert_ne!(encoded_a, encoded_b);
    }

    #[test]
    fn test_caching_serializer_double_deserialize_different_objects() {
        let mut caching = CachingSerializer::new(CachingSerializer::CACHE_DEFAULT);

        let mut a = InfoMessageResponse::default();
        a.oem_icon.replace(PlistByteArray::from(vec![2, 4, 6]));
        let encoded_a = plist_encode(&a).unwrap();

        let mut b = InfoMessageResponse::default();
        b.oem_icon.replace(PlistByteArray::from(vec![1, 3, 5, 7]));
        let encoded_b = plist_encode(&b).unwrap();

        let decoded_a: InfoMessageResponse = caching.deserialize(&encoded_a).unwrap();
        let decoded_b: InfoMessageResponse = caching.deserialize(&encoded_b).unwrap();
        assert_eq!(decoded_a, a);
        assert_eq!(decoded_b, b);
    }

    #[test]
    fn test_caching_serializer_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CachingSerializer>();
    }
}
