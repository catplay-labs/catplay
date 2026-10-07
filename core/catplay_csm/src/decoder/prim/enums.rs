use crate::decoder::CsmWriter;

/// Shared wire behavior for fieldless enums represented by one byte.
pub trait CsmU8Enum: Copy + Default + Sized {
    const WIRE_SIZE: usize = 1;

    fn from_repr(value: u8) -> Option<Self>;

    fn to_repr(self) -> u8;

    #[inline]
    fn decode_enum(data: &[u8]) -> Self {
        data.first()
            .copied()
            .and_then(Self::from_repr)
            .unwrap_or_default()
    }

    #[inline]
    fn encode_enum(&self, out: &mut CsmWriter) {
        out.write_data_chunk(&[self.to_repr()]);
    }
}
