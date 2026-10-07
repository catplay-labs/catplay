//! Typed iAP1 messages on top of the transport-independent frame codec.

use super::{Frame, ProtocolCodec, RegisteredError};
use crate::lingos::{LingoMessage, LingoRegistry};

/// Identifies a command independently of its payload or transaction ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandKey {
    pub lingo: u8,
    pub command: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PacketError {
    #[error("decoded iAP1 frame is missing its lingo or command")]
    MalformedFrame,
    #[error(transparent)]
    Registered(#[from] RegisteredError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub message: LingoMessage,
    pub transaction_id: Option<u16>,
}

/// ID source and explicit tracking request for an outgoing command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionIntent {
    /// Send without an ID, when the command policy and session allow it.
    None,
    /// Allocate a fresh local ID and advance the counter.
    New { start_transaction: bool },
    /// Reuse the given ID without advancing the counter, including replies.
    Existing { id: u16, start_transaction: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outgoing {
    pub message: LingoMessage,
    pub transaction: TransactionIntent,
}

impl Outgoing {
    pub fn new(message: impl Into<LingoMessage>, transaction: TransactionIntent) -> Self {
        Self {
            message: message.into(),
            transaction,
        }
    }
}

impl Packet {
    pub fn decode(frame: &Frame<'_>) -> Result<Self, PacketError> {
        let lingo = frame.lingo().ok_or(PacketError::MalformedFrame)?;
        let (command, data) = frame.command().ok_or(PacketError::MalformedFrame)?;
        let (message, transaction_id) = ProtocolCodec::<LingoRegistry>::decode(lingo, command, data)?;
        Ok(Self { message, transaction_id })
    }

    /// Encode only the lingo, command, optional transaction ID and payload.
    /// Pass this to the link layer as a legacy packet body.
    pub fn encode_body(&self) -> Result<alloc::vec::Vec<u8>, RegisteredError> {
        ProtocolCodec::<LingoRegistry>::encode_body(&self.message, self.transaction_id)
    }
}
