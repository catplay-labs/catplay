use crate::{DecodeError, EncodeError, Iap1DecodeCounted, Iap1Encode, Iap1EncodeCounted, Reader, Writer};

impl<'a> Iap1DecodeCounted<'a> for &'a str {
    fn decode_counted(reader: &mut Reader<'a>, count: usize) -> Result<Self, DecodeError> {
        let bytes = reader.read_bytes(count)?;
        let text = bytes
            .strip_suffix(&[0])
            .ok_or(DecodeError::MissingNullTerminator)?;
        core::str::from_utf8(text).map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl Iap1Encode for str {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        writer.write_bytes(self.as_bytes());
        writer.write_bytes(&[0]);
        Ok(())
    }
}

impl Iap1EncodeCounted for str {
    fn encode_counted(&self, writer: &mut Writer<'_>, count: usize, field: &'static str) -> Result<(), EncodeError> {
        if self.len().checked_add(1) != Some(count) {
            return Err(EncodeError::InvalidFieldValue { field });
        }
        self.encode(writer)
    }
}
