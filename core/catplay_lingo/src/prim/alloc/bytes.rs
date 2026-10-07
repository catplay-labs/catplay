use crate::{DecodeError, EncodeError, Iap1Decode, Iap1DecodeRemaining, Iap1Encode, Reader, Writer};
use alloc::vec::Vec;

impl Iap1Decode for Vec<u8> {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Self::decode_remaining(reader, "bytes")
    }
}

impl Iap1Encode for Vec<u8> {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        self.as_slice().encode(writer)
    }
}
