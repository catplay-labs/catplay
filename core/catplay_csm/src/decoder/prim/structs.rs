use crate::decoder::wire::{CsmDecode, CsmParam, CsmReader};

pub trait CsmStructName {
    const STRUCT_NAME: &'static str;

    #[cfg(feature = "project")]
    #[doc(hidden)]
    fn fmt_projected_debug(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        Self: crate::decoder::CsmStructProjected + Sized,
    {
        crate::decoder::debug_projected_fields(self, Self::STRUCT_NAME, formatter)
    }
}

/// A struct needs to implement this.
pub trait CsmStruct {
    fn feed_param(&mut self, param: &CsmParam);

    fn prealloc(&mut self, reader: &CsmReader);
}

/// Basic bytes->params implementation fitting the same shared CsmDecode interface.
/// So for a struct it decodes bytes into struct, for primitive it decodes TLV values into a primitive.
impl<T: CsmStruct + Default> CsmDecode for T {
    fn decode_from_bytes(data: &[u8]) -> Self {
        let mut ret = T::default();
        let mut reader = CsmReader::new(data);
        ret.prealloc(&reader);
        let mut feed_param = |b: CsmParam| ret.feed_param(&b);
        reader.stream_all_erased(&mut feed_param);
        ret
    }
}
