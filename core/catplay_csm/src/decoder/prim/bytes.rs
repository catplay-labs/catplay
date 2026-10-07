use crate::decoder::{CsmPackedSize, CsmWriter};

use crate::decoder::wire::CsmPayloadEncode;

impl CsmPackedSize for u8 {
    const PACKED_SIZE: usize = 1;

    #[inline]
    fn decode_packed_array<const N: usize>(bytes: &[u8]) -> [Self; N] {
        let mut out = [0u8; N];
        let len = bytes.len().min(N);
        out[..len].copy_from_slice(&bytes[..len]);
        out
    }
}

impl<const N: usize> CsmPayloadEncode for [u8; N] {
    #[inline]
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        out.write_data_chunk(self);
    }

    #[inline]
    fn measure(&self) -> usize {
        N
    }
}

impl CsmPayloadEncode for &[u8] {
    #[inline]
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        out.write_data_chunk(self);
    }

    #[inline]
    fn measure(&self) -> usize {
        self.len()
    }
}
