use alloc::string::ToString;
use bytes::BytesMut;
use serde::{Serialize, de::DeserializeOwned};

use crate::bplist::{ScratchBuffers, serde::decode};
use crate::{PlistError, PlistResult};

/// A caching binary-plist encoder with reusable output and scratch.
///
/// - returned [BytesMut] will automatically re-use backing allocations if it's dropped before next call to `serialize`
///   _and_ the size of serialized payload didn't exceed pre-configured cache limit (4KB by default, enters CoW-style realloc when exceeded)
pub struct CachingSerializer {
    buf: BytesMut,
    cache_size: usize,
    scratch: ScratchBuffers,
}

impl Default for CachingSerializer {
    fn default() -> Self {
        Self::new(Self::CACHE_DEFAULT)
    }
}

impl CachingSerializer {
    pub const CACHE_DEFAULT: usize = 4096;

    pub fn new(cache_size: usize) -> Self {
        Self {
            buf: BytesMut::new(),
            cache_size,
            scratch: ScratchBuffers::default(),
        }
    }

    pub fn serialize<T: Serialize>(&mut self, value: &T) -> PlistResult<BytesMut> {
        self.buf.reserve(self.cache_size);
        let mut out = self.buf.split_off(0);
        if let Err(err) = self.scratch.encode_into_reusing_capacity(value, &mut out) {
            self.reclaim(out);
            return Err(err);
        }
        Ok(out)
    }

    pub fn deserialize<T: DeserializeOwned>(&mut self, data: &[u8]) -> PlistResult<T> {
        self.deserializer_inner(data)
    }

    fn deserializer_inner<T: DeserializeOwned>(&mut self, data: &[u8]) -> PlistResult<T> {
        if data.is_empty() {
            return Err(PlistError::Empty);
        }

        decode::from_slice(data).map_err(|err| PlistError::Decode(err.to_string()))
    }

    pub fn reclaim(&mut self, mut buf: BytesMut) {
        buf.clear();
        self.buf = buf;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{CachingSerializer, PlistByteArray, plist_decode, plist_encode, plist_struct};
    use serde::Serialize;

    struct Counted<'a>(&'a Cell<usize>);

    impl Serialize for Counted<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.set(self.0.get() + 1);
            serializer.serialize_str("CatPlay")
        }
    }

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

    #[test]
    fn warm_serialize_traverses_once() {
        let calls = Cell::new(0);
        let mut caching = CachingSerializer::default();
        let first = caching.serialize(&Counted(&calls)).unwrap();
        caching.reclaim(first);
        calls.set(0);

        let second = caching.serialize(&Counted(&calls)).unwrap();
        assert_eq!(calls.get(), 1);
        let decoded: String = caching.deserialize(&second).unwrap();
        assert_eq!(decoded, "CatPlay");
    }
}
