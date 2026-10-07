use super::Writer;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<'a> {
    /// Bytes after the packet length, including lingo, command and any transaction ID.
    pub body: &'a [u8],
}

impl Frame<'_> {
    pub fn lingo(&self) -> Option<u8> {
        self.body.first().copied()
    }

    /// Return the command and its remaining wire bytes, including any transaction ID.
    pub fn command(&self) -> Option<(u16, &[u8])> {
        let lingo = self.lingo()?;
        if lingo == 4 {
            Some((u16::from_be_bytes([*self.body.get(1)?, *self.body.get(2)?]), self.body.get(3..)?))
        } else {
            Some((u16::from(*self.body.get(1)?), self.body.get(2..)?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LinkError {
    #[error("invalid iAP1 frame sync byte")]
    InvalidSync,
    #[error("iAP1 frame body is empty")]
    EmptyBody,
    #[error("iAP1 frame is missing a command ID")]
    TruncatedCommand,
    #[error("invalid iAP1 packet length")]
    InvalidLength,
    #[error("iAP1 frame body is too long: {0}")]
    BodyTooLong(usize),
    #[error("one-byte iAP1 command does not fit in one byte: {0:#06x}")]
    CommandOutOfRange(u16),
}

/// Encode the short or extended length format. The length counts the lingo,
/// command ID and command data, so the short format holds up to 255 body bytes.
pub struct FrameCodec;

impl FrameCodec {
    /// Decode one iAP1 frame starting at 0x55. Scanning, the trailing checksum
    /// byte and protocol selection belong to the caller; the body is not interpreted.
    pub fn decode(src: &[u8]) -> Result<Option<(Frame<'_>, usize)>, LinkError> {
        let Some(&first) = src.first() else {
            return Ok(None);
        };
        if first != 0x55 {
            return Err(LinkError::InvalidSync);
        }
        let Some(&length_byte) = src.get(1) else {
            return Ok(None);
        };
        let (header_len, body_len, extended) = if length_byte == 0 {
            let (Some(&hi), Some(&lo)) = (src.get(2), src.get(3)) else {
                return Ok(None);
            };
            (4, u16::from_be_bytes([hi, lo]) as usize, true)
        } else {
            (2, length_byte as usize, false)
        };
        if body_len == 0 {
            return Err(LinkError::InvalidLength);
        }
        let total_len = header_len + body_len;
        if src.len() < total_len {
            return Ok(None);
        }
        if extended && body_len <= u8::MAX as usize {
            return Err(LinkError::InvalidLength);
        }
        if body_len < if src[header_len] == 4 { 3 } else { 2 } {
            return Err(LinkError::TruncatedCommand);
        }
        Ok(Some((
            Frame {
                body: &src[header_len..header_len + body_len],
            },
            total_len,
        )))
    }

    /// Encode a frame starting at 0x55. The caller appends its checksum and may
    /// prefix the frame with a 0xFF sync byte when its transport needs one.
    pub fn encode(body: &[u8], writer: &mut Writer<'_>) -> Result<(), LinkError> {
        if body.is_empty() {
            return Err(LinkError::EmptyBody);
        }
        if body.len() < if body[0] == 4 { 3 } else { 2 } {
            return Err(LinkError::TruncatedCommand);
        }
        if body.len() > u16::MAX as usize {
            return Err(LinkError::BodyTooLong(body.len()));
        }
        writer.write_u8(0x55);
        if body.len() <= u8::MAX as usize {
            writer.write_u8(body.len() as u8);
        } else {
            writer.write_u8(0);
            writer.write_bytes(&(body.len() as u16).to_be_bytes());
        }
        writer.write_bytes(body);
        Ok(())
    }

    pub fn command_body(lingo: u8, command: u16, transaction_id: Option<u16>, payload: &[u8]) -> Result<Vec<u8>, LinkError> {
        let mut body = Vec::with_capacity(3 + usize::from(transaction_id.is_some()) * 2 + payload.len());
        body.push(lingo);
        if lingo != 4 {
            let command = u8::try_from(command).map_err(|_| LinkError::CommandOutOfRange(command))?;
            body.push(command);
        } else {
            body.extend_from_slice(&command.to_be_bytes());
        }
        if let Some(id) = transaction_id {
            body.extend_from_slice(&id.to_be_bytes());
        }
        body.extend_from_slice(payload);
        Ok(body)
    }
}
