use crate::{DecodeError, EncodeError, Iap1Decode, Iap1Encode, Reader, Writer};

impl Iap1Decode for bool {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.read_bool()
    }
}

impl Iap1Encode for bool {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        writer.write_bool(*self);
        Ok(())
    }
}
