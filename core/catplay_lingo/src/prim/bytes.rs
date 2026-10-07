use crate::{DecodeError, EncodeError, Iap1Decode, Iap1Encode, Reader, Writer};

impl Iap1Encode for &[u8] {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        writer.write_bytes(self);
        Ok(())
    }
}

impl<const N: usize> Iap1Decode for [u8; N] {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(reader
            .read_bytes(N)?
            .try_into()
            .expect("slice has the requested length"))
    }
}
impl<const N: usize> Iap1Encode for [u8; N] {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        writer.write_bytes(self);
        Ok(())
    }
}
