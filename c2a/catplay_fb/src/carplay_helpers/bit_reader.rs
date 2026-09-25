use exp_golomb::ExpGolombDecoder;

use super::RewriteResult;
use crate::H264FrameBufferError;

pub(super) struct BitReader<'a> {
    data: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> BitReader<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub(super) fn bit(&mut self) -> RewriteResult<u8> {
        let byte = *self
            .data
            .get(self.pos / 8)
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("truncated RBSP"))?;
        let bit = (byte >> (7 - self.pos % 8)) & 1;
        self.pos += 1;
        Ok(bit)
    }

    pub(super) fn bits(&mut self, count: usize) -> RewriteResult<u32> {
        if count > 32 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("bit field is too wide"));
        }
        let mut value = 0;
        for _ in 0..count {
            value = (value << 1) | self.bit()? as u32;
        }
        Ok(value)
    }

    pub(super) fn ue(&mut self) -> RewriteResult<u32> {
        let mut decoder = ExpGolombDecoder::new(&self.data[self.pos / 8..], (self.pos % 8) as u32)
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("truncated Exp-Golomb value"))?;
        let value = decoder
            .next_unsigned()
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("invalid Exp-Golomb value"))?;
        self.pos += exp_golomb_bit_len(value)?;
        value
            .try_into()
            .map_err(|_| H264FrameBufferError::X264BitstreamRewrite("Exp-Golomb value exceeds u32"))
    }

    pub(super) fn se(&mut self) -> RewriteResult<i32> {
        let mut decoder = ExpGolombDecoder::new(&self.data[self.pos / 8..], (self.pos % 8) as u32)
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("truncated signed Exp-Golomb value"))?;
        let value = decoder
            .next_signed()
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("invalid signed Exp-Golomb value"))?;
        let code_num = if value <= 0 {
            value.unsigned_abs() * 2
        } else {
            value as u64 * 2 - 1
        };
        self.pos += exp_golomb_bit_len(code_num)?;
        value
            .try_into()
            .map_err(|_| H264FrameBufferError::X264BitstreamRewrite("signed Exp-Golomb value exceeds i32"))
    }
}

fn exp_golomb_bit_len(value: u64) -> RewriteResult<usize> {
    let code_num = value
        .checked_add(1)
        .ok_or(H264FrameBufferError::X264BitstreamRewrite("Exp-Golomb value is too large"))?;
    let significant_bits = (u64::BITS - code_num.leading_zeros()) as usize;
    Ok(significant_bits * 2 - 1)
}
