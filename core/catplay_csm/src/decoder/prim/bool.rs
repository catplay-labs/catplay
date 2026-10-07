use crate::decoder::CsmWriter;

use crate::decoder::wire::{CsmDecode, CsmPayloadEncode};

impl CsmDecode for bool {
    fn decode_from_bytes(data: &[u8]) -> Self {
        data.first().copied().unwrap_or(0) == 1
    }
}

impl CsmPayloadEncode for bool {
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        out.write_data_chunk(&[u8::from(*self)]);
    }

    fn measure(&self) -> usize {
        1
    }
}
