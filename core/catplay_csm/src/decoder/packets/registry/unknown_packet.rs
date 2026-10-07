extern crate alloc;

use alloc::vec::Vec;
use core::fmt::{self, Debug};

use crate::decoder::{CsmPayloadEncode, CsmWriter};

/// Represents a decoded CSM packet with unknown id and raw payload data.
///
/// It can be freely written to another socket in a "proxy" mode if desired.
#[derive(Clone, PartialEq, Eq)]
pub struct CsmUnknownPacket(pub u16, pub Vec<u8>);

impl CsmUnknownPacket {
    pub fn id(&self) -> u16 {
        self.0
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.1
    }
}

impl CsmPayloadEncode for CsmUnknownPacket {
    fn encode_to_bytes(&self, writer: &mut CsmWriter) {
        writer.write_data_chunk(self.payload());
    }
}

impl Debug for CsmUnknownPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CsmUnknownPacket({:#06X}) {{ {:?} }}", self.0, &self.1)
    }
}
