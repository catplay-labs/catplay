use super::{CsmParam, CsmWriter};

pub trait CsmEncode {
    fn encode_param(&self, id: u16, out: &mut CsmWriter);
}

#[cfg(feature = "alloc")]
extern crate alloc;

pub trait CsmPayloadEncode {
    fn encode_to_bytes(&self, writer: &mut CsmWriter);

    fn measure(&self) -> usize {
        // Measurement pass
        CsmWriter::measure(|w| self.encode_to_bytes(w))
    }

    #[cfg(feature = "alloc")]
    fn serialize(&self) -> alloc::vec::Vec<u8> {
        CsmWriter::serialize(|w| self.encode_to_bytes(w))
    }
}

impl<T: CsmPayloadEncode> CsmEncode for T {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        let len = self.measure();

        out.write_tlv_header(id, len);
        if out.overflow {
            return;
        }
        let start = out.written;
        self.encode_to_bytes(out);
        debug_assert!(
            out.written - start == len,
            "Measurement violation in CsmEncode::encode_param(): {} vs {}",
            out.written - start,
            len
        );
    }
}

pub trait CsmDecode: Sized {
    fn decode_from_bytes(data: &[u8]) -> Self;
}

pub trait CsmAccum {
    fn add_param(&mut self, param: &CsmParam);

    #[allow(unused)]
    fn prealloc(&mut self, size: usize) {}
}

impl<T: CsmDecode> CsmAccum for T {
    fn add_param(&mut self, param: &CsmParam) {
        // Last one wins (in case of schema violations and duplicate IDs)
        *self = T::decode_from_bytes(param.value);
    }
}
