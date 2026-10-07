use crate::{DecodeError, EncodeError, Iap1Decode, Iap1Encode, Reader, Writer};

macro_rules! wire_number {
    ($ty:ty, $read:ident, $write:ident) => {
        impl Iap1Decode for $ty {
            fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
                reader.$read()
            }
        }
        impl Iap1Encode for $ty {
            fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
                writer.$write(*self);
                Ok(())
            }
        }
    };
}
wire_number!(u8, read_u8, write_u8);
wire_number!(i8, read_i8, write_i8);
wire_number!(u16, read_u16_be, write_u16_be);
wire_number!(i16, read_i16_be, write_i16_be);
wire_number!(u32, read_u32_be, write_u32_be);
wire_number!(i32, read_i32_be, write_i32_be);
wire_number!(u64, read_u64_be, write_u64_be);
wire_number!(i64, read_i64_be, write_i64_be);
