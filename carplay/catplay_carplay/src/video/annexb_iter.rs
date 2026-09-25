use std::vec;

use crate::video::{NalChunk, NalError};

const START_CODE: [u8; 4] = [0, 0, 0, 1];

/// Iterates over AnnexB or AVCC/HVCC-compatible buffers to split NALs.
pub enum AnnexBIter<'a> {
    /// Cached offsets which can speed up conversion from AnnexB to AVCC/HVCC
    Cached {
        src: &'a [u8],
        chunks: vec::IntoIter<NalChunk>,
        prev_end: usize,
        cached_kind: AnnexBCacheVerifyKind,
    },
    /// Assumed to always have nal_size_len == 4
    AnnexB { src: &'a [u8], pos: usize, done: bool },
    /// AVCC/HVCC compatible length prefixes
    LengthPrefixed { src: &'a [u8], nal_size_len: usize, offset: usize },
}

pub enum AnnexBCacheVerifyKind {
    None,
    AnnexB,
    LengthPrefixed { nal_size_len: usize },
}

impl<'a> AnnexBIter<'a> {
    pub fn cached(src: &'a [u8], chunks: vec::IntoIter<NalChunk>, cached_kind: AnnexBCacheVerifyKind) -> Self {
        Self::Cached {
            src,
            chunks,
            prev_end: 0,
            cached_kind,
        }
    }

    pub fn annexb(src: &'a [u8]) -> Self {
        Self::AnnexB { src, pos: 0, done: false }
    }

    pub fn length_prefixed(src: &'a [u8], nal_size_len: usize) -> Self {
        Self::LengthPrefixed {
            src,
            nal_size_len,
            offset: 0,
        }
    }
}

impl Iterator for AnnexBIter<'_> {
    type Item = Result<NalChunk, NalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Cached {
                src,
                chunks,
                prev_end,
                cached_kind,
            } => {
                let chunk = chunks.next()?;
                let Ok((_payload_start, payload_end)) = chunk_payload_bounds(src, &chunk) else {
                    return Some(Err(NalError::Param));
                };

                if chunk.prefix_start < *prev_end {
                    return Some(Err(NalError::Param));
                }

                match cached_kind {
                    AnnexBCacheVerifyKind::None => {}
                    AnnexBCacheVerifyKind::AnnexB => {
                        if chunk.prefix_len != 4 || !src[chunk.prefix_start..].starts_with(&START_CODE) {
                            return Some(Err(NalError::Param));
                        }
                    }
                    AnnexBCacheVerifyKind::LengthPrefixed { nal_size_len } => {
                        if chunk.prefix_len != *nal_size_len {
                            return Some(Err(NalError::Param));
                        }

                        let Ok((nal_len, _)) = read_length_prefixed_nal_len(src, chunk.prefix_start, *nal_size_len) else {
                            return Some(Err(NalError::Param));
                        };
                        if nal_len != chunk.data_size {
                            return Some(Err(NalError::Param));
                        }
                    }
                }

                *prev_end = payload_end;
                Some(Ok(chunk))
            }
            Self::AnnexB { src, pos, done } => {
                if *done {
                    return None;
                }

                if *pos + 4 > src.len() {
                    *done = true;
                    if *pos == src.len() {
                        return None;
                    }
                    return Some(Err(NalError::Param));
                }

                if !src[*pos..].starts_with(&START_CODE) {
                    *done = true;
                    return Some(Err(NalError::Param));
                }

                let start = *pos;
                let next = find_next_start_code(src, *pos + 4);

                if next < start + 4 {
                    *done = true;
                    return Some(Err(NalError::Param));
                }

                if next == src.len() {
                    *done = true;
                    *pos = src.len();
                } else {
                    *pos = next;
                }

                Some(Ok(NalChunk {
                    prefix_start: start,
                    prefix_len: 4,
                    data_size: next - (start + 4),
                }))
            }
            Self::LengthPrefixed { src, nal_size_len, offset } => {
                if *offset >= src.len() {
                    return None;
                }

                let start = *offset;
                match read_length_prefixed_nal_len(src, *offset, *nal_size_len) {
                    Ok((nal_len, payload_start)) => {
                        *offset = payload_start + nal_len;
                        Some(Ok(NalChunk {
                            prefix_start: start,
                            prefix_len: *nal_size_len,
                            data_size: nal_len,
                        }))
                    }
                    Err(e) => {
                        *offset = src.len();
                        Some(Err(e))
                    }
                }
            }
        }
    }
}

pub fn chunk_payload_bounds(src: &[u8], chunk: &NalChunk) -> Result<(usize, usize), NalError> {
    if chunk.prefix_len == 0 {
        return Err(NalError::Param);
    }

    let payload_start = chunk
        .prefix_start
        .checked_add(chunk.prefix_len)
        .ok_or(NalError::Param)?;
    let payload_end = payload_start
        .checked_add(chunk.data_size)
        .ok_or(NalError::Param)?;

    if payload_end > src.len() {
        return Err(NalError::Param);
    }

    Ok((payload_start, payload_end))
}

pub fn read_length_prefixed_nal_len(src: &[u8], offset: usize, nal_size_len: usize) -> Result<(usize, usize), NalError> {
    if offset + nal_size_len > src.len() {
        return Err(NalError::Underrun);
    }

    let nal_len = match nal_size_len {
        1 => src[offset] as usize,
        2 => u16::from_be_bytes([src[offset], src[offset + 1]]) as usize,
        3 => ((src[offset] as usize) << 16) | ((src[offset + 1] as usize) << 8) | (src[offset + 2] as usize),
        4 => u32::from_be_bytes([src[offset], src[offset + 1], src[offset + 2], src[offset + 3]]) as usize,
        _ => return Err(NalError::Param),
    };

    let payload_start = offset + nal_size_len;
    if payload_start + nal_len > src.len() {
        return Err(NalError::Underrun);
    }

    Ok((nal_len, payload_start))
}

fn find_next_start_code(buf: &[u8], from: usize) -> usize {
    let len = buf.len();
    if from + 4 > len {
        return len;
    }

    let mut one_idx = from + 3;
    while one_idx < len {
        if buf[one_idx] == 1 && buf[one_idx - 1] == 0 && buf[one_idx - 2] == 0 && buf[one_idx - 3] == 0 {
            return one_idx - 3;
        }
        one_idx += 1;
    }
    len
}
