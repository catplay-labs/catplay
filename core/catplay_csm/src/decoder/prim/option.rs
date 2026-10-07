use crate::decoder::CsmWriter;

use crate::decoder::wire::{CsmDecode, CsmEncode};

impl<T: CsmDecode> CsmDecode for Option<T> {
    #[inline]
    fn decode_from_bytes(data: &[u8]) -> Self {
        Some(T::decode_from_bytes(data))
    }
}

impl<T: CsmEncode> CsmEncode for Option<T> {
    #[inline]
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        if let Some(value) = self {
            value.encode_param(id, out);
        }
    }
}
