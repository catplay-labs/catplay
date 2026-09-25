use bytes::BytesMut;

use crate::video::{AnnexBIter, NalChunk, NalError};

const START_CODE: [u8; 4] = [0, 0, 0, 1];

/// Used to recreate AnnexB/AVCC/HVCC buffers during conversion when `nal_size_len` is incompatible between targets.
pub struct AnnexBBuilder {
    out: BytesMut,
    chunks: Vec<NalChunk>,
    pending: Option<NalChunk>,
}

impl AnnexBBuilder {
    pub fn new(out: BytesMut) -> Self {
        Self {
            out,
            chunks: Vec::new(),
            pending: None,
        }
    }

    pub fn push_start_code(&mut self, data_size: usize) -> Result<(), NalError> {
        self.begin_chunk(4, data_size)?;
        self.out.extend_from_slice(&START_CODE);
        Ok(())
    }

    pub fn push_length_prefix(&mut self, nal_size_len: usize, data_size: usize) -> Result<(), NalError> {
        self.begin_chunk(nal_size_len, data_size)?;
        match nal_size_len {
            1 if data_size <= u8::MAX as usize => self.out.extend_from_slice(&[data_size as u8]),
            2 if data_size <= u16::MAX as usize => self
                .out
                .extend_from_slice(&(data_size as u16).to_be_bytes()),
            3 if data_size <= 0x00FF_FFFF => self.out.extend_from_slice(&[
                ((data_size >> 16) & 0xFF) as u8,
                ((data_size >> 8) & 0xFF) as u8,
                (data_size & 0xFF) as u8,
            ]),
            4 => self
                .out
                .extend_from_slice(&(data_size as u32).to_be_bytes()),
            _ => return Err(NalError::Param),
        }
        Ok(())
    }

    pub fn push_data(&mut self, data: &[u8]) -> Result<(), NalError> {
        let chunk = self.pending.take().ok_or(NalError::Param)?;
        if data.len() != chunk.data_size {
            return Err(NalError::Param);
        }
        self.out.extend_from_slice(data);
        self.chunks.push(chunk);
        Ok(())
    }

    pub fn finish(mut self) -> Result<(BytesMut, Vec<NalChunk>), NalError> {
        if self.pending.is_some() {
            return Err(NalError::Param);
        }
        Ok((self.out.split(), self.chunks))
    }

    fn begin_chunk(&mut self, prefix_len: usize, data_size: usize) -> Result<(), NalError> {
        if self.pending.is_some() || prefix_len == 0 {
            return Err(NalError::Param);
        }
        self.pending = Some(NalChunk {
            prefix_start: self.out.len(),
            prefix_len,
            data_size,
        });
        Ok(())
    }

    pub fn rebuild(
        src: &mut BytesMut,
        iter: AnnexBIter<'_>,
        to_length_prefixed: bool,
        nal_size_len: usize,
    ) -> Result<(BytesMut, Vec<NalChunk>), NalError> {
        if src.is_empty() {
            return Ok((BytesMut::new(), Vec::new()));
        }

        let chunks = iter.collect::<Result<Vec<_>, _>>()?;
        if chunks.is_empty() {
            return Ok((BytesMut::new(), Vec::new()));
        }

        Self::rebuild_from_chunks(src, chunks, to_length_prefixed, nal_size_len)
    }

    pub fn rebuild_from_chunks(
        src: &mut BytesMut,
        chunks: Vec<NalChunk>,
        to_length_prefixed: bool,
        nal_size_len: usize,
    ) -> Result<(BytesMut, Vec<NalChunk>), NalError> {
        if src.is_empty() || chunks.is_empty() {
            return Ok((BytesMut::new(), Vec::new()));
        }

        // Fast path: both source and target use 4-byte prefixes.
        if nal_size_len == 4 && chunks.iter().all(|c| c.prefix_len == 4) {
            let mut out_chunks = Vec::with_capacity(chunks.len());

            for chunk in chunks {
                let prefix_end = chunk.prefix_start + 4;
                if to_length_prefixed {
                    src[chunk.prefix_start..prefix_end].copy_from_slice(&(chunk.data_size as u32).to_be_bytes());
                } else {
                    src[chunk.prefix_start..prefix_end].copy_from_slice(&START_CODE);
                }

                out_chunks.push(NalChunk {
                    prefix_start: chunk.prefix_start,
                    prefix_len: 4,
                    data_size: chunk.data_size,
                });
            }

            return Ok((src.split(), out_chunks));
        }

        // Slow path: rebuild from scratch
        let mut builder = AnnexBBuilder::new(BytesMut::with_capacity(if to_length_prefixed { src.len() } else { src.len() * 2 }));
        for chunk in chunks {
            let payload_start = chunk.prefix_start + chunk.prefix_len;
            let payload_end = payload_start + chunk.data_size;

            if to_length_prefixed {
                builder.push_length_prefix(nal_size_len, chunk.data_size)?;
            } else {
                builder.push_start_code(chunk.data_size)?;
            }
            builder.push_data(&src[payload_start..payload_end])?;
        }

        builder.finish()
    }
}
