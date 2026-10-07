mod codec;

use super::{DecodeContext, DecodeError, EncodeContext, EncodeError, Iap1MessageMetadata, Writer};
use alloc::vec::Vec;

pub use codec::*;

/// Static dispatch for the command IDs known by one lingo.
pub trait Registry {
    const LINGO_ID: u8;
    type Message;

    fn decode(command: u16, ctx: &DecodeContext, payload: &[u8]) -> Result<Self::Message, RegistryDecodeError>;
    fn encode(message: &Self::Message, ctx: &EncodeContext, writer: &mut Writer<'_>) -> Result<(), EncodeError>;
    fn command_id(message: &Self::Message) -> u16;
    fn metadata(command: u16) -> Option<Iap1MessageMetadata>;
}

/// A message type with a known lingo registry and frame codec.
pub trait RegisteredMessage: Sized {
    type Registry: Registry<Message = Self>;

    fn decode_registered(command: u16, data: &[u8]) -> Result<(Self, Option<u16>), RegisteredError> {
        RegisteredCodec::<Self::Registry>::decode(command, data)
    }

    /// Encode from 0x55 through the body; the caller supplies any sync byte and checksum.
    fn encode_registered(&self, transaction_id: Option<u16>, context: &EncodeContext) -> Result<Vec<u8>, RegisteredError> {
        RegisteredCodec::<Self::Registry>::encode(self, transaction_id, context)
    }
}

/// Dispatches messages across all enabled lingos.
pub trait ProtocolRegistry {
    type Message;

    fn decode(lingo: u8, command: u16, ctx: &DecodeContext, payload: &[u8]) -> Result<Self::Message, RegistryDecodeError>;
    fn encode(message: &Self::Message, ctx: &EncodeContext, writer: &mut Writer<'_>) -> Result<(), EncodeError>;
    fn lingo_id(message: &Self::Message) -> u8;
    fn command_id(message: &Self::Message) -> u16;
    fn metadata(lingo: u8, command: u16) -> Result<Option<Iap1MessageMetadata>, RegistryDecodeError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryDecodeError {
    #[error("unknown lingo {0:#04x}")]
    UnknownLingo(u8),
    #[error("unknown command {command:#06x} for lingo {lingo:#04x}")]
    UnknownCommand { lingo: u8, command: u16 },
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
