use super::super::{FrameCodec, LinkError};
use crate::{DecodeContext, EncodeContext, EncodeError, ProtocolRegistry, Registry, RegistryDecodeError, TransactionIdPolicy, Writer};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegisteredError {
    #[error(transparent)]
    Decode(RegistryDecodeError),
    #[error(transparent)]
    Encode(EncodeError),
    #[error(transparent)]
    Link(LinkError),
    #[error("transaction ID is required")]
    MissingTransactionId,
    #[error("transaction ID is prohibited")]
    UnexpectedTransactionId,
    #[error("ambiguous transaction ID for command {command:#06x}")]
    AmbiguousTransactionId { command: u16 },
}

/// Decode one known command. Transaction IDs are interpreted per command policy,
/// not as a framing mode: some commands require them before IDPS completes,
/// while other commands prohibit them throughout the session.
pub struct RegisteredCodec<R>(PhantomData<R>);

impl<R: Registry> RegisteredCodec<R> {
    pub fn decode(command: u16, data: &[u8]) -> Result<(R::Message, Option<u16>), RegisteredError> {
        let meta = R::metadata(command).ok_or(RegisteredError::Decode(RegistryDecodeError::UnknownCommand {
            lingo: R::LINGO_ID,
            command,
        }))?;
        TransactionCodec::decode(command, data, meta.transaction_id, |payload, has_transaction_id| {
            R::decode(
                command,
                &DecodeContext {
                    adjusted_payload_length: payload.len(),
                    has_transaction_id,
                },
                payload,
            )
        })
    }
}

struct TransactionCodec;

impl TransactionCodec {
    fn decode<M>(
        command: u16,
        data: &[u8],
        policy: TransactionIdPolicy,
        decode: impl Fn(&[u8], bool) -> Result<M, RegistryDecodeError>,
    ) -> Result<(M, Option<u16>), RegisteredError> {
        let with_id = || {
            let [hi, lo, payload @ ..] = data else {
                return Err(RegisteredError::MissingTransactionId);
            };
            decode(payload, true)
                .map(|message| (message, Some(u16::from_be_bytes([*hi, *lo]))))
                .map_err(RegisteredError::Decode)
        };
        match policy {
            TransactionIdPolicy::Prohibited => decode(data, false)
                .map(|message| (message, None))
                .map_err(RegisteredError::Decode),
            TransactionIdPolicy::Required => with_id(),
            TransactionIdPolicy::Permitted => {
                // Without session state, ID presence can only be guessed from the payload.
                // Both interpretations may decode, e.g. for `ContextButtonStatus`
                // TransactionHelper uses the session state and calls
                // `decode_with_transaction_id` instead of taking this path.
                let without = decode(data, false);
                let with = if data.len() >= 2 { Some(with_id()) } else { None };
                match (without, with) {
                    (Ok(message), Some(Ok(_))) => {
                        let _ = message;
                        Err(RegisteredError::AmbiguousTransactionId { command })
                    }
                    (Ok(message), _) => Ok((message, None)),
                    (Err(_), Some(Ok(value))) => Ok(value),
                    (Err(error), _) => Err(RegisteredError::Decode(error)),
                }
            }
        }
    }

    fn check_policy(policy: TransactionIdPolicy, transaction_id: Option<u16>) -> Result<(), RegisteredError> {
        match (policy, transaction_id) {
            (TransactionIdPolicy::Required, None) => Err(RegisteredError::MissingTransactionId),
            (TransactionIdPolicy::Prohibited, Some(_)) => Err(RegisteredError::UnexpectedTransactionId),
            _ => Ok(()),
        }
    }
}

/// Typed packet codec for the registry of all enabled lingoes.
pub struct ProtocolCodec<R>(PhantomData<R>);

impl<R: ProtocolRegistry> ProtocolCodec<R> {
    pub fn decode(lingo: u8, command: u16, data: &[u8]) -> Result<(R::Message, Option<u16>), RegisteredError> {
        let meta = R::metadata(lingo, command)
            .map_err(RegisteredError::Decode)?
            .ok_or(RegisteredError::Decode(RegistryDecodeError::UnknownCommand { lingo, command }))?;
        TransactionCodec::decode(command, data, meta.transaction_id, |payload, has_transaction_id| {
            R::decode(
                lingo,
                command,
                &DecodeContext {
                    adjusted_payload_length: payload.len(),
                    has_transaction_id,
                },
                payload,
            )
        })
    }

    /// Decode with the transaction-ID presence already determined by the session.
    /// This avoids guessing from the payload of a `Permitted` command.
    pub fn decode_with_transaction_id(
        lingo: u8,
        command: u16,
        data: &[u8],
        has_transaction_id: bool,
    ) -> Result<(R::Message, Option<u16>), RegisteredError> {
        let meta = R::metadata(lingo, command)
            .map_err(RegisteredError::Decode)?
            .ok_or(RegisteredError::Decode(RegistryDecodeError::UnknownCommand { lingo, command }))?;
        TransactionCodec::check_policy(meta.transaction_id, has_transaction_id.then_some(0))?;
        let (payload, transaction_id) = if has_transaction_id {
            let [hi, lo, payload @ ..] = data else {
                return Err(RegisteredError::MissingTransactionId);
            };
            (payload, Some(u16::from_be_bytes([*hi, *lo])))
        } else {
            (data, None)
        };
        let message = R::decode(
            lingo,
            command,
            &DecodeContext {
                adjusted_payload_length: payload.len(),
                has_transaction_id,
            },
            payload,
        )
        .map_err(RegisteredError::Decode)?;
        Ok((message, transaction_id))
    }

    pub fn encode_body(message: &R::Message, transaction_id: Option<u16>) -> Result<Vec<u8>, RegisteredError> {
        let lingo = R::lingo_id(message);
        let command = R::command_id(message);
        let meta = R::metadata(lingo, command)
            .map_err(RegisteredError::Decode)?
            .expect("registry message has metadata");
        TransactionCodec::check_policy(meta.transaction_id, transaction_id)?;
        let mut payload = Vec::new();
        R::encode(
            message,
            &EncodeContext {
                adjusted_payload_length: 0,
                has_transaction_id: transaction_id.is_some(),
            },
            &mut Writer::new(&mut payload),
        )
        .map_err(RegisteredError::Encode)?;
        FrameCodec::command_body(lingo, command, transaction_id, &payload).map_err(RegisteredError::Link)
    }
}

/// Encode one registered command with the transaction policy from its schema.
impl<R: Registry> RegisteredCodec<R> {
    pub fn encode(message: &R::Message, transaction_id: Option<u16>, context: &EncodeContext) -> Result<Vec<u8>, RegisteredError> {
        let command = R::command_id(message);
        let meta = R::metadata(command).expect("registry command has metadata");
        TransactionCodec::check_policy(meta.transaction_id, transaction_id)?;
        let mut payload = Vec::new();
        R::encode(
            message,
            &EncodeContext {
                adjusted_payload_length: context.adjusted_payload_length,
                has_transaction_id: transaction_id.is_some(),
            },
            &mut Writer::new(&mut payload),
        )
        .map_err(RegisteredError::Encode)?;
        let body = FrameCodec::command_body(R::LINGO_ID, command, transaction_id, &payload).map_err(RegisteredError::Link)?;
        let mut packet = Vec::with_capacity(body.len() + 6);
        FrameCodec::encode(&body, &mut Writer::new(&mut packet)).map_err(RegisteredError::Link)?;
        Ok(packet)
    }
}
