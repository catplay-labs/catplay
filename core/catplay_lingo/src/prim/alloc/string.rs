use crate::{DecodeError, EncodeError, Iap1Decode, Iap1DecodeCounted, Iap1DecodeRemaining, Iap1Encode, Iap1EncodeCounted, Reader, Writer};
use alloc::{borrow::ToOwned, string::String};

impl Iap1Decode for String {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Self::decode_remaining(reader, "string")
    }
}

impl Iap1Encode for String {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        self.as_str().encode(writer)
    }
}

impl<'a> Iap1DecodeCounted<'a> for String {
    fn decode_counted(reader: &mut Reader<'a>, count: usize) -> Result<Self, DecodeError> {
        <&str>::decode_counted(reader, count).map(str::to_owned)
    }
}

impl Iap1EncodeCounted for String {
    fn encode_counted(&self, writer: &mut Writer<'_>, count: usize, field: &'static str) -> Result<(), EncodeError> {
        self.as_str().encode_counted(writer, count, field)
    }
}

impl Iap1DecodeRemaining for String {
    fn decode_remaining(reader: &mut Reader<'_>, _field: &'static str) -> Result<Self, DecodeError> {
        Self::decode_counted(reader, reader.remaining())
    }
}
