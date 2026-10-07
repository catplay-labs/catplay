extern crate alloc;

use alloc::vec::Vec;

use core::ops::{Deref, DerefMut};

use crate::decoder::{CsmDecode, CsmPackedSize, CsmPayloadEncode, CsmWriter};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CsmPacked<T>(pub Vec<T>);

impl<T> CsmPacked<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for CsmPacked<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T: Clone> From<&[T]> for CsmPacked<T> {
    fn from(value: &[T]) -> Self {
        Self(value.to_vec())
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for CsmPacked<T> {
    fn from(value: &[T; N]) -> Self {
        Self(value.to_vec())
    }
}

impl<T> From<CsmPacked<T>> for Vec<T> {
    fn from(value: CsmPacked<T>) -> Self {
        value.0
    }
}

impl<T> Deref for CsmPacked<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CsmPacked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: CsmDecode + CsmPackedSize> CsmDecode for CsmPacked<T> {
    fn decode_from_bytes(bytes: &[u8]) -> Self {
        assert!(T::PACKED_SIZE > 0, "zero-width packed element");

        Self(
            bytes
                .chunks(T::PACKED_SIZE)
                .map(T::decode_from_bytes)
                .collect(),
        )
    }
}

impl<T: CsmPayloadEncode> CsmPayloadEncode for CsmPacked<T> {
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        for item in &self.0 {
            item.encode_to_bytes(out);
        }
    }
}
