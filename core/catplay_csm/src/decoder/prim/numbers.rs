use crate::decoder::CsmWriter;

use crate::decoder::wire::{CsmDecode, CsmPayloadEncode};

macro_rules! impl_payload_be_bytes {
    ($($ty:ty),* $(,)?) => {$(
        impl CsmDecode for $ty {
            #[inline]
            fn decode_from_bytes(data: &[u8]) -> Self {
                const SIZE: usize = core::mem::size_of::<$ty>();

                if data.len() < SIZE {
                    return 0;
                }

                <$ty>::from_be_bytes(data[..SIZE].try_into().unwrap())
            }
        }

        impl CsmPayloadEncode for $ty {
            #[inline]
            fn encode_to_bytes(&self, writer: &mut CsmWriter) {
                writer.write_data_chunk(&self.to_be_bytes());
            }

            #[inline]
            fn measure(&self) -> usize {
                core::mem::size_of::<$ty>()
            }
        }
    )*};
}

impl_payload_be_bytes!(u8, u16, u32, u64, i8, i16, i32, i64);
