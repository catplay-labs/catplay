use core::ops::{Deref, DerefMut};

use crate::decoder::{CsmDecode, CsmPayloadEncode, CsmWriter};

/// Fixed encoded payload size, excluding the TLV header.
///
/// A type implementing this trait and CsmEncode must emit exactly one TLV
/// with this payload size. Do not implement it for Option, CsmFlag, or CsmVec.
///
/// The size must be positive when the type is used as a packed element.
pub trait CsmPackedSize: Sized {
    const PACKED_SIZE: usize;

    /// Decode packed elements, allowing types to provide a bulk-copy fast path.
    fn decode_packed_array<const N: usize>(bytes: &[u8]) -> [Self; N]
    where
        Self: CsmDecode + Default,
    {
        const { assert!(Self::PACKED_SIZE > 0, "zero-width packed element") };
        let mut chunks = bytes.chunks(Self::PACKED_SIZE);

        core::array::from_fn(|_| match chunks.next() {
            // A partial last element follows Self's existing decode policy.
            Some(chunk) => Self::decode_from_bytes(chunk),
            None => Self::default(),
        })
    }
}

macro_rules! impl_packed_size {
    ($($ty:ty),* $(,)?) => {$(
        impl CsmPackedSize for $ty {
            const PACKED_SIZE: usize = core::mem::size_of::<Self>();
        }
    )*};
}

impl_packed_size!(bool, u16, u32, u64, i8, i16, i32, i64);

/// Exactly N values inside ONE TLV. No alloc or unsafe code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CsmPackedTight<T, const N: usize> {
    pub data: [T; N],
}

impl<T, const N: usize> CsmPackedTight<T, N> {
    pub const fn new(data: [T; N]) -> Self {
        Self { data }
    }

    pub fn into_inner(self) -> [T; N] {
        self.data
    }
}

impl<T: Default, const N: usize> Default for CsmPackedTight<T, N> {
    fn default() -> Self {
        Self::new(core::array::from_fn(|_| T::default()))
    }
}

impl<T, const N: usize> From<[T; N]> for CsmPackedTight<T, N> {
    fn from(data: [T; N]) -> Self {
        Self::new(data)
    }
}

impl<T, const N: usize> Deref for CsmPackedTight<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, const N: usize> DerefMut for CsmPackedTight<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T, const N: usize> AsRef<[T]> for CsmPackedTight<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T, const N: usize> AsMut<[T]> for CsmPackedTight<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: CsmPackedSize, const N: usize> CsmPackedSize for CsmPackedTight<T, N> {
    const PACKED_SIZE: usize = {
        assert!(N > 0 && T::PACKED_SIZE > 0, "zero-width packed element");

        match T::PACKED_SIZE.checked_mul(N) {
            Some(size) => size,
            None => panic!("packed size overflow"),
        }
    };
}

impl<T: CsmDecode + CsmPackedSize + Default, const N: usize> CsmDecode for CsmPackedTight<T, N> {
    fn decode_from_bytes(bytes: &[u8]) -> Self {
        let _ = Self::PACKED_SIZE;
        Self::new(<[T; N]>::decode_from_bytes(bytes))
    }
}

impl<T: CsmPayloadEncode, const N: usize> CsmPayloadEncode for CsmPackedTight<T, N> {
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        self.data.encode_to_bytes(out);
    }

    fn measure(&self) -> usize {
        self.data.measure()
    }
}

impl<T: CsmPayloadEncode> CsmPayloadEncode for [T] {
    fn encode_to_bytes(&self, writer: &mut CsmWriter) {
        for item in self {
            item.encode_to_bytes(writer);
        }
    }

    fn measure(&self) -> usize {
        self.iter()
            .fold(0usize, |size, item| size.saturating_add(item.measure()))
    }
}

impl<T: CsmDecode + CsmPackedSize + Default, const N: usize> CsmDecode for [T; N] {
    #[inline]
    fn decode_from_bytes(bytes: &[u8]) -> Self {
        T::decode_packed_array(bytes)
    }
}
