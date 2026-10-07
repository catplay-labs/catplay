use crate::decoder::CsmWriter;

use crate::decoder::wire::{CsmDecode, CsmEncode};

/// A "flag" type with no payload, which could either exist in the message or not, but will never have any payload.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum CsmFlag {
    #[default]
    No,
    Yes,
}

impl CsmDecode for CsmFlag {
    fn decode_from_bytes(_data: &[u8]) -> Self {
        CsmFlag::Yes
    }
}

impl CsmEncode for CsmFlag {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        if *self == CsmFlag::Yes {
            out.write_tlv(id, &[]);
        }
    }
}
