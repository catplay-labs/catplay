use crate::decoder::{CsmDecode, CsmPayloadEncode, CsmWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsmRational<T> {
    pub numerator: T,
    pub denominator: T,
}

pub type CsmRat16 = CsmRational<i16>;
pub type CsmRat32 = CsmRational<i32>;
pub type CsmURat16 = CsmRational<u16>;
pub type CsmURat32 = CsmRational<u32>;

macro_rules! impl_rational_codec {
    ($($ty:ty),* $(,)?) => {$(
        impl CsmDecode for CsmRational<$ty> {
            #[inline]
            fn decode_from_bytes(data: &[u8]) -> Self {
                const SIZE: usize = core::mem::size_of::<$ty>();

                if data.len() < SIZE * 2 {
                    return Self {
                        numerator: 0,
                        denominator: 0,
                    };
                }

                Self {
                    numerator: <$ty>::from_be_bytes(data[..SIZE].try_into().unwrap()),
                    denominator: <$ty>::from_be_bytes(data[SIZE..SIZE * 2].try_into().unwrap()),
                }
            }
        }

        impl CsmPayloadEncode for CsmRational<$ty> {
            #[inline]
            fn encode_to_bytes(&self, writer: &mut CsmWriter) {
                writer.write_data_chunk(&self.numerator.to_be_bytes());
                writer.write_data_chunk(&self.denominator.to_be_bytes());
            }

            #[inline]
            fn measure(&self) -> usize {
                core::mem::size_of::<$ty>() * 2
            }
        }
    )*};
}

impl_rational_codec!(i16, i32, u16, u32);
