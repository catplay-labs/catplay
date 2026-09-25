use bytes::BytesMut;

use crate::video::{AnnexBBuilder, AnnexBCacheVerifyKind, AnnexBIter, NalChunk, NalError};

/// Generic AnnexB <-> length-prefixed converter (AVCC/HVCC payload-compatible).
#[derive(Debug, Clone, Default)]
pub struct AnnexBConverter {
    pub nal_size_len: usize,
    pub offsets_cache: Vec<NalChunk>,
}

impl AnnexBConverter {
    pub fn new(nal_size_len: usize) -> Self {
        Self {
            nal_size_len,
            offsets_cache: Vec::new(),
        }
    }

    pub fn with_cache(nal_size_len: usize, cache: Vec<NalChunk>) -> Self {
        Self {
            nal_size_len,
            offsets_cache: cache,
        }
    }

    pub fn convert(&mut self, src: &mut BytesMut, to_length_prefixed: bool) -> Result<BytesMut, NalError> {
        if !matches!(self.nal_size_len, 1..5) {
            return Err(NalError::Param);
        }

        if to_length_prefixed {
            self.convert_annexb_to_length_prefixed(src)
        } else {
            self.convert_length_prefixed_to_annexb(src)
        }
    }

    pub fn scan_only(src: &[u8], nal_size_len: usize, from_length_prefixed: bool) -> Result<Vec<NalChunk>, NalError> {
        if !matches!(nal_size_len, 1..5) {
            return Err(NalError::Param);
        }

        if from_length_prefixed {
            AnnexBIter::length_prefixed(src, nal_size_len).collect()
        } else {
            AnnexBIter::annexb(src).collect()
        }
    }

    fn convert_annexb_to_length_prefixed(&mut self, src: &mut BytesMut) -> Result<BytesMut, NalError> {
        let chunks = self
            .annexb_chunks_iter(src)
            .collect::<Result<Vec<_>, _>>()?;
        let (out, out_chunks) = AnnexBBuilder::rebuild_from_chunks(src, chunks, true, self.nal_size_len)?;
        self.offsets_cache = out_chunks;
        Ok(out)
    }

    fn convert_length_prefixed_to_annexb(&mut self, src: &mut BytesMut) -> Result<BytesMut, NalError> {
        let chunks = self
            .length_prefixed_chunks_iter(src)
            .collect::<Result<Vec<_>, _>>()?;
        let (out, out_chunks) = AnnexBBuilder::rebuild_from_chunks(src, chunks, false, self.nal_size_len)?;
        self.offsets_cache = out_chunks;
        Ok(out)
    }

    fn annexb_chunks_iter<'a>(&'a self, src: &'a [u8]) -> AnnexBIter<'a> {
        if self.offsets_cache.is_empty() {
            AnnexBIter::annexb(src)
        } else {
            AnnexBIter::cached(src, self.offsets_cache.clone().into_iter(), AnnexBCacheVerifyKind::AnnexB)
        }
    }

    fn length_prefixed_chunks_iter<'a>(&'a self, src: &'a [u8]) -> AnnexBIter<'a> {
        if self.offsets_cache.is_empty() {
            AnnexBIter::length_prefixed(src, self.nal_size_len)
        } else {
            AnnexBIter::cached(
                src,
                self.offsets_cache.clone().into_iter(),
                AnnexBCacheVerifyKind::LengthPrefixed {
                    nal_size_len: self.nal_size_len,
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::video::{AnnexBConverter, NalError};

    fn sample_annexb() -> BytesMut {
        BytesMut::from(
            &[
                0x00, 0x00, 0x00, 0x01, // start
                0x67, 0x42, 0xE0, 0x1E, 0x89, 0x8B, // nal
                0x00, 0x00, 0x00, 0x01, // start
                0x68, 0xCE, // nal
            ][..],
        )
    }

    fn assert_roundtrip_with_cache_reuse(nal_size_len: usize) {
        let mut to_length = AnnexBConverter::new(nal_size_len);
        let annexb = sample_annexb();
        let avcc = to_length
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc failed");

        let mut to_annexb = AnnexBConverter::with_cache(nal_size_len, to_length.offsets_cache.clone());

        let annexb2 = to_annexb
            .convert(&mut avcc.clone(), false)
            .expect("avcc->annexb failed");
        assert_eq!(annexb, annexb2);
    }

    #[test]
    fn roundtrip_len4_with_cache_reuse() {
        assert_roundtrip_with_cache_reuse(4);
    }

    #[test]
    fn roundtrip_len2_with_cache_reuse() {
        assert_roundtrip_with_cache_reuse(2);
    }

    #[test]
    fn roundtrip_len3_with_cache_reuse() {
        assert_roundtrip_with_cache_reuse(3);
    }

    #[test]
    fn conversion_without_cache_both_directions() {
        let annexb = sample_annexb();
        let mut to_avcc_no_cache = AnnexBConverter::new(4);
        let avcc = to_avcc_no_cache
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc without cache failed");
        assert!(!to_avcc_no_cache.offsets_cache.is_empty());

        let mut to_annexb_no_cache = AnnexBConverter::new(4);
        let annexb2 = to_annexb_no_cache
            .convert(&mut avcc.clone(), false)
            .expect("avcc->annexb without cache failed");
        assert_eq!(annexb, annexb2);
        assert!(!to_annexb_no_cache.offsets_cache.is_empty());
    }

    #[test]
    fn avcc_to_annexb_without_cache_explicit() {
        let annexb = sample_annexb();
        let mut to_avcc = AnnexBConverter::new(4);
        let avcc = to_avcc
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc failed");

        let mut fresh_decoder = AnnexBConverter::new(4);
        let annexb2 = fresh_decoder
            .convert(&mut avcc.clone(), false)
            .expect("avcc->annexb without cache failed");
        assert_eq!(annexb, annexb2);
    }

    #[test]
    fn annexb_to_avcc_without_cache_explicit() {
        let annexb = sample_annexb();
        let mut fresh_encoder = AnnexBConverter::new(4);
        let avcc = fresh_encoder
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc without cache failed");
        assert!(!avcc.is_empty());
    }

    #[test]
    fn duplicate_conversion_with_cache_fails_verification_annexb() {
        let mut conv = AnnexBConverter::new(4);
        let annexb = sample_annexb();
        let avcc = conv
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc failed");

        let mut annexb2 = conv
            .convert(&mut avcc.clone(), false)
            .expect("avcc->annexb failed");
        assert_eq!(annexb, annexb2);
        let err = conv.convert(&mut annexb2, false).unwrap_err();
        assert!(matches!(err, NalError::Param | NalError::Underrun));
    }

    #[test]
    fn duplicate_conversion_with_cache_fails_verification_avcc() {
        let mut conv = AnnexBConverter::new(4);
        let annexb = sample_annexb();
        let avcc = conv
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc failed");
        let mut avcc2 = conv
            .convert(&mut annexb.clone(), true)
            .expect("annexb->avcc (cached) failed");
        assert_eq!(avcc, avcc2);
        let err = conv.convert(&mut avcc2, true).unwrap_err();
        assert!(matches!(err, NalError::Param | NalError::Underrun));
    }

    #[test]
    fn rejects_invalid_input() {
        let mut converter = AnnexBConverter::new(4);
        let mut invalid = BytesMut::from(&b"\x12\x34\x56\x78"[..]);
        let err = converter.convert(&mut invalid, true).unwrap_err();
        assert!(matches!(err, NalError::Param));
    }
}
