use crate::decoder::CsmWriter;

use crate::decoder::wire::CsmPayloadEncode;

impl CsmPayloadEncode for &str {
    #[inline]
    fn encode_to_bytes(&self, out: &mut CsmWriter) {
        out.write_data_chunk(self.as_bytes());
        out.write_data_chunk(&[0]);
    }

    #[inline]
    fn measure(&self) -> usize {
        self.len() + 1
    }
}
