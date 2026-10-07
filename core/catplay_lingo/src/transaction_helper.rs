//! Session-level iAP1 transaction-ID handling for either protocol role.
//!
//! This helper decides whether an ID is present, allocates local IDs and reserves
//! IDs at the caller's request. The caller decides which response completes a
//! transaction and calls `finish_transaction` explicitly.

use alloc::collections::BTreeMap;

use crate::{
    CommandKey, Iap1Message, Iap1MessageMetadata, Iap1Source, ProtocolCodec, ProtocolRegistry, RegisteredError, TransactionIdPolicy,
    lingo_0x0::StartIDPS,
    lingos::{LingoMessage, LingoRegistry},
    wire::{Frame, Packet, TransactionIntent},
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransactionError {
    #[error("iAP1 frame is missing a lingo or command")]
    MalformedFrame,
    #[error(transparent)]
    Codec(#[from] RegisteredError),
    #[error("unknown iAP1 command {lingo:#04x}/{command:#06x}")]
    UnknownCommand { lingo: u8, command: u16 },
    #[error("iAP1 command {lingo:#04x}/{command:#06x} has the wrong direction for this role")]
    WrongDirection { lingo: u8, command: u16 },
    #[error("transaction IDs are disabled for {lingo:#04x}/{command:#06x}")]
    IdsDisabled { lingo: u8, command: u16 },
    #[error("transaction ID is required for {lingo:#04x}/{command:#06x}")]
    MissingId { lingo: u8, command: u16 },
    #[error("transaction ID is prohibited for {lingo:#04x}/{command:#06x}")]
    UnexpectedId { lingo: u8, command: u16 },
    #[error("transaction ID {0:#06x} is already reserved by a local transaction")]
    IdAlreadyPending(u16),
    #[error("all transaction IDs have outstanding requests")]
    IdsExhausted,
}

/// `server = true` is the Apple-device role. A new accessory ID starts at zero
/// per R46 §2.6.1.1; the first Apple-device example starts at one. Neither
/// counter changes when the caller supplies an existing ID.
#[derive(Clone)]
pub struct TransactionHelper {
    server: bool,
    enabled: bool,
    next_local_id: u16,
    pending: BTreeMap<u16, CommandKey>,
}

impl TransactionHelper {
    pub fn new(server: bool) -> Self {
        Self {
            server,
            enabled: false,
            next_local_id: if server { 1 } else { 0 },
            pending: BTreeMap::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.server);
    }

    pub fn ids_enabled(&self) -> bool {
        self.enabled
    }

    /// The session owner can change ID support explicitly. Disabling it drops
    /// reservations, while retaining the counter until `reset` is called.
    pub fn set_ids_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.pending.clear();
        }
    }

    pub fn pending_command(&self, id: u16) -> Option<CommandKey> {
        self.pending.get(&id).copied()
    }

    /// Number of locally started iAP1 transactions awaiting session completion.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Release a local reservation when the caller considers it complete,
    /// cancelled or timed out. This helper never infers completion from packets.
    pub fn finish_transaction(&mut self, id: u16) -> Option<CommandKey> {
        self.pending.remove(&id)
    }

    /// Resolve `Permitted` from the known session state, before payload decode.
    pub fn incoming_has_id(&self, lingo: u8, command: u16) -> Result<bool, TransactionError> {
        let key = CommandKey { lingo, command };
        let meta = Self::metadata(key)?;
        self.check_direction(key, meta, false)?;
        match meta.transaction_id {
            TransactionIdPolicy::Prohibited => Ok(false),
            TransactionIdPolicy::Required => {
                if !self.enabled && key != StartIDPS::KEY {
                    Err(TransactionError::IdsDisabled { lingo, command })
                } else {
                    Ok(true)
                }
            }
            TransactionIdPolicy::Permitted => Ok(self.enabled),
        }
    }

    /// Decode once with an ID decision supplied by session state. Incoming
    /// responses do not change the caller's outstanding transaction set.
    pub fn decode(&mut self, frame: &Frame<'_>) -> Result<Packet, TransactionError> {
        let lingo = frame.lingo().ok_or(TransactionError::MalformedFrame)?;
        let (command, data) = frame.command().ok_or(TransactionError::MalformedFrame)?;
        let has_id = self.incoming_has_id(lingo, command)?;
        let (message, transaction_id) = ProtocolCodec::<LingoRegistry>::decode_with_transaction_id(lingo, command, data, has_id)?;
        let packet = Packet { message, transaction_id };
        self.observe_incoming(&packet)?;
        Ok(packet)
    }

    /// Validate an already decoded packet and apply only the StartIDPS ID-state
    /// transition. The session owner handles every other protocol transition.
    pub fn observe_incoming(&mut self, packet: &Packet) -> Result<(), TransactionError> {
        let key = Self::key(&packet.message);
        let meta = Self::metadata(key)?;
        self.check_direction(key, meta, false)?;
        self.check_id(
            key,
            meta.transaction_id,
            packet.transaction_id,
            self.enabled || key == StartIDPS::KEY,
        )?;
        if key == StartIDPS::KEY {
            self.enabled = true;
        }
        Ok(())
    }

    /// `New` always advances the local counter, even when untracked. `Existing`
    /// uses the supplied ID without advancing it; this covers replies and
    /// later commands in a transaction that intentionally reuses its ID.
    pub fn prepare_outgoing(&mut self, message: &LingoMessage, intent: TransactionIntent) -> Result<Option<u16>, TransactionError> {
        let key = Self::key(message);
        let meta = Self::metadata(key)?;
        self.check_direction(key, meta, true)?;
        let enabled_for_packet = self.enabled || key == StartIDPS::KEY;
        let start_transaction = match intent {
            TransactionIntent::None => false,
            TransactionIntent::New { start_transaction } | TransactionIntent::Existing { start_transaction, .. } => start_transaction,
        };
        let id = match intent {
            TransactionIntent::None => None,
            TransactionIntent::Existing { id, .. } => Some(id),
            TransactionIntent::New { .. } => {
                // Validate before allocation so an invalid command cannot
                // consume a counter value.
                self.check_id(key, meta.transaction_id, Some(0), enabled_for_packet)?;
                Some(self.allocate_id()?)
            }
        };
        self.check_id(key, meta.transaction_id, id, enabled_for_packet)?;
        if start_transaction {
            let id = id.expect("tracked transaction has an ID");
            if self.pending.contains_key(&id) {
                return Err(TransactionError::IdAlreadyPending(id));
            }
            self.pending.insert(id, key);
        }
        if key == StartIDPS::KEY {
            self.enabled = true;
        }
        Ok(id)
    }

    fn check_id(
        &self,
        key: CommandKey,
        policy: TransactionIdPolicy,
        id: Option<u16>,
        enabled_for_packet: bool,
    ) -> Result<(), TransactionError> {
        match (policy, id) {
            (TransactionIdPolicy::Prohibited, Some(_)) => Err(TransactionError::UnexpectedId {
                lingo: key.lingo,
                command: key.command,
            }),
            (TransactionIdPolicy::Required, None) => Err(TransactionError::MissingId {
                lingo: key.lingo,
                command: key.command,
            }),
            (TransactionIdPolicy::Permitted, None) if enabled_for_packet => Err(TransactionError::MissingId {
                lingo: key.lingo,
                command: key.command,
            }),
            (_, Some(_)) if !enabled_for_packet => Err(TransactionError::IdsDisabled {
                lingo: key.lingo,
                command: key.command,
            }),
            _ => Ok(()),
        }
    }

    fn key(message: &LingoMessage) -> CommandKey {
        CommandKey {
            lingo: LingoRegistry::lingo_id(message),
            command: LingoRegistry::command_id(message),
        }
    }

    fn metadata(key: CommandKey) -> Result<Iap1MessageMetadata, TransactionError> {
        LingoRegistry::metadata(key.lingo, key.command)
            .ok()
            .flatten()
            .ok_or(TransactionError::UnknownCommand {
                lingo: key.lingo,
                command: key.command,
            })
    }

    fn check_direction(&self, key: CommandKey, meta: Iap1MessageMetadata, outgoing: bool) -> Result<(), TransactionError> {
        let local_source = if self.server { Iap1Source::Device } else { Iap1Source::Accessory };
        if (meta.source == local_source) != outgoing {
            Err(TransactionError::WrongDirection {
                lingo: key.lingo,
                command: key.command,
            })
        } else {
            Ok(())
        }
    }

    fn allocate_id(&mut self) -> Result<u16, TransactionError> {
        for _ in 0..=u16::MAX {
            let id = self.next_local_id;
            self.next_local_id = self.next_local_id.wrapping_add(1);
            if !self.pending.contains_key(&id) {
                return Ok(id);
            }
        }
        Err(TransactionError::IdsExhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameCodec, Writer, lingo_0x0 as general};

    fn frame(body: &[u8]) -> alloc::vec::Vec<u8> {
        let mut bytes = alloc::vec::Vec::new();
        FrameCodec::encode(body, &mut Writer::new(&mut bytes)).unwrap();
        bytes
    }

    fn decoded_body(bytes: &[u8]) -> alloc::vec::Vec<u8> {
        FrameCodec::decode(bytes).unwrap().unwrap().0.body.to_vec()
    }

    #[test]
    fn start_idps_uses_zero_and_only_caller_finishes_transaction() {
        let mut accessory = TransactionHelper::new(false);
        let start: LingoMessage = general::LingoMessage::StartIDPS(general::StartIDPS {}).into();
        let id = accessory
            .prepare_outgoing(&start, TransactionIntent::New { start_transaction: true })
            .unwrap();
        assert_eq!(id, Some(0));
        assert_eq!(accessory.pending_count(), 1);
        assert!(accessory.ids_enabled());
        let bytes = frame(&FrameCodec::command_body(0, 2, id, &[0, 0x38]).unwrap());
        let body = decoded_body(&bytes);
        let wire = Frame { body: &body };
        let ack = accessory.decode(&wire).unwrap();
        assert_eq!(ack.transaction_id, id);
        assert_eq!(accessory.pending_command(0), Some(StartIDPS::KEY));
        assert_eq!(accessory.finish_transaction(0), Some(StartIDPS::KEY));
        assert_eq!(accessory.pending_command(0), None);
        assert_eq!(accessory.pending_count(), 0);
    }

    #[test]
    fn optional_pre_idps_query_does_not_consume_first_id() {
        let mut accessory = TransactionHelper::new(false);
        let query: LingoMessage = general::LingoMessage::RequestiPodSoftwareVersion(general::RequestiPodSoftwareVersion {}).into();
        assert_eq!(
            accessory
                .prepare_outgoing(&query, TransactionIntent::None)
                .unwrap(),
            None
        );
        assert!(!accessory.ids_enabled());

        let start: LingoMessage = general::LingoMessage::StartIDPS(general::StartIDPS {}).into();
        assert_eq!(
            accessory
                .prepare_outgoing(&start, TransactionIntent::New { start_transaction: true })
                .unwrap(),
            Some(0)
        );
    }

    #[test]
    fn legacy_identification_decodes_without_start_idps() {
        let mut iphone = TransactionHelper::new(true);
        let mut accessory = TransactionHelper::new(false);

        let request: LingoMessage = general::LingoMessage::RequestIdentify(general::RequestIdentify {}).into();
        let request = Packet {
            transaction_id: iphone
                .prepare_outgoing(&request, TransactionIntent::None)
                .unwrap(),
            message: request,
        };
        let body = request.encode_body().unwrap();
        let wire = Frame { body: &body };
        let decoded = accessory.decode(&wire).unwrap();
        assert_eq!(decoded, request);

        let identify: LingoMessage = general::LingoMessage::IdentifyDeviceLingoes(general::IdentifyDeviceLingoes {
            device_lingoes_spoken: general::IdentifyDeviceLingoesDeviceLingoesSpoken::from_bits_retain(5),
            options: general::IdentifyDeviceLingoesOptions::from_raw(2),
            device_id: 512,
        })
        .into();
        let identify = Packet {
            transaction_id: accessory
                .prepare_outgoing(&identify, TransactionIntent::None)
                .unwrap(),
            message: identify,
        };
        let body = identify.encode_body().unwrap();
        let wire = Frame { body: &body };
        let decoded = iphone.decode(&wire).unwrap();
        assert_eq!(decoded, identify);

        let ack: LingoMessage = general::LingoMessage::IPodAck(general::IPodAck {
            command_result: general::IPodAckCommandResult::OK,
            acked_command_id: general::IdentifyDeviceLingoes::KEY.command as u8,
            maximum_pending_wait: None,
            session_id: None,
            num_bytes_dropped: None,
        })
        .into();
        let ack = Packet {
            transaction_id: iphone
                .prepare_outgoing(&ack, TransactionIntent::None)
                .unwrap(),
            message: ack,
        };
        let body = ack.encode_body().unwrap();
        let wire = Frame { body: &body };
        let decoded = accessory.decode(&wire).unwrap();
        assert_eq!(decoded, ack);

        assert!(!iphone.ids_enabled());
        assert!(!accessory.ids_enabled());
    }

    #[test]
    fn untracked_new_ids_advance_and_existing_ids_do_not() {
        use crate::lingos::lingo_0x2 as remote;

        let mut accessory = TransactionHelper::new(false);
        accessory.set_ids_enabled(true);
        let button: LingoMessage = remote::LingoMessage::ContextButtonStatus(remote::ContextButtonStatus {
            button_states0: remote::ContextButtonStatusButtonStates0::empty(),
            button_states1: None,
            button_states2: None,
            button_states3: None,
        })
        .into();
        assert_eq!(
            accessory
                .prepare_outgoing(&button, TransactionIntent::New { start_transaction: false })
                .unwrap(),
            Some(0)
        );
        assert_eq!(accessory.pending_command(0), None);
        assert_eq!(
            accessory
                .prepare_outgoing(
                    &button,
                    TransactionIntent::Existing {
                        id: 42,
                        start_transaction: false,
                    },
                )
                .unwrap(),
            Some(42)
        );
        assert_eq!(
            accessory
                .prepare_outgoing(&button, TransactionIntent::New { start_transaction: false })
                .unwrap(),
            Some(1)
        );
    }

    #[test]
    fn existing_id_can_start_a_new_local_transaction_after_finish() {
        let mut iphone = TransactionHelper::new(true);
        iphone.set_ids_enabled(true);
        let get_info: LingoMessage =
            general::LingoMessage::GetAccessoryAuthenticationInfo(general::GetAccessoryAuthenticationInfo {}).into();
        let id = iphone
            .prepare_outgoing(&get_info, TransactionIntent::New { start_transaction: true })
            .unwrap()
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(
            iphone.prepare_outgoing(
                &get_info,
                TransactionIntent::Existing {
                    id,
                    start_transaction: true
                }
            ),
            Err(TransactionError::IdAlreadyPending(id))
        );
        assert_eq!(iphone.pending_command(id), Some(general::GetAccessoryAuthenticationInfo::KEY));
        iphone.finish_transaction(id);
        let get_signature: LingoMessage =
            general::LingoMessage::GetAccessoryAuthenticationSignature(general::GetAccessoryAuthenticationSignature {
                challenge: None,
                authentication_retry_counter: None,
            })
            .into();
        assert_eq!(
            iphone
                .prepare_outgoing(
                    &get_signature,
                    TransactionIntent::Existing {
                        id,
                        start_transaction: true
                    }
                )
                .unwrap(),
            Some(1)
        );
        iphone.finish_transaction(id);
        assert_eq!(
            iphone
                .prepare_outgoing(&get_info, TransactionIntent::New { start_transaction: true })
                .unwrap(),
            Some(2)
        );
    }

    #[test]
    fn permitted_button_is_decoded_using_session_state() {
        let mut iphone = TransactionHelper::new(true);
        let bytes = frame(&[2, 0, 0, 1, 2]);
        let body = decoded_body(&bytes);
        let wire = Frame { body: &body };
        assert_eq!(iphone.decode(&wire).unwrap().transaction_id, None);
        iphone.set_ids_enabled(true);
        assert_eq!(iphone.decode(&wire).unwrap().transaction_id, Some(1));
    }

    #[test]
    fn prohibited_omits_id_and_modern_session_rejects_missing_permitted_id() {
        let mut iphone = TransactionHelper::new(true);
        iphone.set_ids_enabled(true);
        let identify: LingoMessage = general::LingoMessage::RequestIdentify(general::RequestIdentify {}).into();
        assert_eq!(
            iphone
                .prepare_outgoing(&identify, TransactionIntent::None)
                .unwrap(),
            None
        );
        assert!(iphone.ids_enabled());
        let no_id_ack: LingoMessage = general::LingoMessage::IPodAck(general::IPodAck {
            command_result: general::IPodAckCommandResult::OK,
            acked_command_id: 0x38,
            maximum_pending_wait: None,
            session_id: None,
            num_bytes_dropped: None,
        })
        .into();
        assert_eq!(
            iphone.prepare_outgoing(&no_id_ack, TransactionIntent::None),
            Err(TransactionError::MissingId { lingo: 0, command: 2 })
        );
    }

    #[test]
    fn rollover_skips_a_pending_local_id() {
        let mut accessory = TransactionHelper::new(false);
        accessory.set_ids_enabled(true);
        accessory.next_local_id = 0;
        let start: LingoMessage = general::LingoMessage::StartIDPS(general::StartIDPS {}).into();
        accessory
            .prepare_outgoing(&start, TransactionIntent::New { start_transaction: true })
            .unwrap();
        accessory.next_local_id = u16::MAX;
        assert_eq!(
            accessory
                .prepare_outgoing(&start, TransactionIntent::New { start_transaction: false })
                .unwrap(),
            Some(u16::MAX)
        );
        assert_eq!(
            accessory
                .prepare_outgoing(&start, TransactionIntent::New { start_transaction: false })
                .unwrap(),
            Some(1)
        );
    }
}
