use catplay_lingo::{
    DecodeContext, DecodeError, EncodeContext, EncodeError, FrameCodec, Iap1Message, RegisteredError, RegisteredMessage, Registry, Writer,
    lingo_0x0::*,
};
use insta::assert_debug_snapshot;

#[allow(dead_code)]
#[derive(Debug)]
struct TestSnapshot<M = LingoMessage> {
    case: &'static str,
    message: M,
    bytes: Vec<u8>,
}

enum PayloadInput<'a> {
    Message(LingoMessage),
    Wire { command: u16, bytes: &'a [u8] },
}

fn snapshot_payload(
    snapshots: &mut Vec<TestSnapshot<catplay_lingo::lingos::LingoMessage>>,
    case: &'static str,
    input: PayloadInput<'_>,
    context: DecodeContext,
) {
    let (message, observed) = match input {
        PayloadInput::Message(message) => (message, None),
        PayloadInput::Wire { command, bytes } => (LingoRegistry::decode(command, &context, bytes).unwrap(), Some(bytes)),
    };
    let mut encoded = Vec::new();
    LingoRegistry::encode(
        &message,
        &EncodeContext {
            adjusted_payload_length: context.adjusted_payload_length,
            has_transaction_id: context.has_transaction_id,
        },
        &mut Writer::new(&mut encoded),
    )
    .unwrap();
    if let Some(bytes) = observed {
        assert_eq!(encoded, bytes, "wire round-trip mismatch for {case}");
    }
    assert_eq!(
        LingoRegistry::decode(LingoRegistry::command_id(&message), &context, &encoded).unwrap(),
        message,
        "message round-trip mismatch for {case}"
    );
    snapshots.push(TestSnapshot {
        case,
        message: message.into(),
        bytes: encoded,
    });
}

// Exercise encoding without a payload-length hint, then decode using actual bytes.
fn snapshot_length_selected<T>(
    snapshots: &mut Vec<TestSnapshot<catplay_lingo::lingos::LingoMessage>>,
    case: &'static str,
    value: T,
    expected: &[u8],
    wrap: impl FnOnce(T) -> catplay_lingo::lingos::LingoMessage,
) where
    T: catplay_lingo::Iap1Message + PartialEq + std::fmt::Debug,
{
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut catplay_lingo::Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, expected, "wire mismatch for {case}");
    let decoded = T::decode(
        &DecodeContext {
            adjusted_payload_length: bytes.len(),
            has_transaction_id: false,
        },
        &bytes,
    )
    .unwrap();
    assert_eq!(decoded, value, "message mismatch for {case}");
    snapshots.push(TestSnapshot {
        case,
        message: wrap(decoded),
        bytes,
    });
}

fn snapshot_registered(
    snapshots: &mut Vec<TestSnapshot>,
    case: &'static str,
    message: LingoMessage,
    transaction_id: Option<u16>,
    context: EncodeContext,
) {
    let packet = message.encode_registered(transaction_id, &context).unwrap();
    let (frame, consumed) = FrameCodec::decode(&packet).unwrap().unwrap();
    assert_eq!(consumed, packet.len(), "expected one frame for {case}");
    let (command, data) = frame.command().unwrap();
    let (decoded, decoded_transaction_id) = LingoMessage::decode_registered(command, data).unwrap();
    assert_eq!(decoded_transaction_id, transaction_id, "transaction ID mismatch for {case}");
    assert_eq!(decoded, message, "message round-trip mismatch for {case}");
    assert_eq!(
        decoded
            .encode_registered(decoded_transaction_id, &context)
            .unwrap(),
        packet,
        "wire round-trip mismatch for {case}"
    );
    snapshots.push(TestSnapshot {
        case,
        message: decoded,
        bytes: packet,
    });
}

#[test]
fn general_lingo_payload_round_trips() {
    assert_eq!(LingoRegistry::LINGO_ID, 0);
    let mut snapshots = Vec::new();

    let identify = LingoMessage::IdentifyDeviceLingoes(IdentifyDeviceLingoes {
        device_lingoes_spoken: IdentifyDeviceLingoesDeviceLingoesSpoken::from_bits_retain(5),
        options: IdentifyDeviceLingoesOptions::from_raw(2),
        device_id: 512,
    });
    snapshot_payload(
        &mut snapshots,
        "identify device lingoes",
        PayloadInput::Message(identify),
        DecodeContext::default(),
    );

    let mut challenge_payload = [0u8; 17];
    challenge_payload[..16].copy_from_slice(&std::array::from_fn::<_, 16, _>(|index| index as u8));
    challenge_payload[16] = 2;
    snapshot_payload(
        &mut snapshots,
        "fixed-size authentication challenge",
        PayloadInput::Wire {
            command: 0x17,
            bytes: &challenge_payload,
        },
        DecodeContext {
            adjusted_payload_length: challenge_payload.len(),
            has_transaction_id: true,
        },
    );
    let tokens_payload = [
        2, // two tokens
        13, 0, 0, 2, 0, 2, 0, 0, 0, 2, 0, 0, 2, 0, // identify
        10, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, // capabilities
    ];
    snapshot_payload(
        &mut snapshots,
        "FID token values",
        PayloadInput::Wire {
            command: 0x39,
            bytes: &tokens_payload,
        },
        DecodeContext::default(),
    );

    snapshot_payload(
        &mut snapshots,
        "FID token acknowledgement",
        PayloadInput::Wire {
            command: 0x3a,
            bytes: &[1, 3, 0, 0, 0],
        },
        DecodeContext::default(),
    );
    snapshot_payload(
        &mut snapshots,
        "open authentication status enum",
        PayloadInput::Wire {
            command: 0x19,
            bytes: &[10],
        },
        DecodeContext::default(),
    );
    use catplay_lingo::Iap1Message;
    use catplay_lingo::lingos::lingo_0x2::{ContextButtonStatus, LingoMessage as RemoteMessage};
    for (case, challenge, expected) in [
        (
            "16-byte challenge, zero length hint, maximum retry counter",
            GetAccessoryAuthenticationSignatureChallengeDisjoint::Value16ByteChallenge([0xa5; 16]),
            [vec![0xa5; 16], vec![255]].concat(),
        ),
        (
            "20-byte challenge, zero length hint, maximum retry counter",
            GetAccessoryAuthenticationSignatureChallengeDisjoint::Value20ByteChallenge([0xa5; 20]),
            [vec![0xa5; 20], vec![255]].concat(),
        ),
    ] {
        let value = GetAccessoryAuthenticationSignature {
            challenge: Some(challenge),
            authentication_retry_counter: Some(255),
        };
        snapshot_length_selected(&mut snapshots, case, value.clone(), &expected, |value| {
            LingoMessage::GetAccessoryAuthenticationSignature(value).into()
        });
        let mut incomplete = value;
        incomplete.authentication_retry_counter = None;
        assert!(matches!(
            incomplete.encode(&EncodeContext::default(), &mut catplay_lingo::Writer::new(&mut Vec::new())),
            Err(EncodeError::RoundTripDecode(_))
        ));
    }
    for (case, payload) in [
        ("ContextButtonStatus: mandatory byte only", &[0x01][..]),
        ("ContextButtonStatus: first optional byte", &[0x01, 0x02][..]),
        ("ContextButtonStatus: two optional bytes", &[0x01, 0x02, 0x04][..]),
        ("ContextButtonStatus: all optional bytes", &[0x01, 0x02, 0x04, 0x08][..]),
    ] {
        let value = ContextButtonStatus::decode(
            &DecodeContext {
                adjusted_payload_length: payload.len(),
                has_transaction_id: false,
            },
            payload,
        )
        .unwrap();
        snapshot_length_selected(&mut snapshots, case, value.clone(), payload, |value| {
            RemoteMessage::ContextButtonStatus(value).into()
        });
        if value.button_states2.is_some() {
            let mut hole = value;
            hole.button_states1 = None;
            assert_eq!(
                hole.encode(&EncodeContext::default(), &mut catplay_lingo::Writer::new(&mut Vec::new())),
                Err(EncodeError::RoundTripMismatch)
            );
        }
    }
    assert_debug_snapshot!(snapshots);
}

#[test]
fn generated_predicate_program_reads_prior_field_values() {
    let value = IPodAck {
        command_result: IPodAckCommandResult::CommandPending,
        acked_command_id: 0x33,
        maximum_pending_wait: Some(0x0102_0304),
        session_id: None,
        num_bytes_dropped: None,
    };
    let mut bytes = Vec::new();
    value
        .encode(&EncodeContext::default(), &mut Writer::new(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [6, 0x33, 1, 2, 3, 4]);
    assert_eq!(IPodAck::decode(&DecodeContext::default(), &bytes).unwrap(), value);

    let invalid = IPodAck {
        command_result: IPodAckCommandResult::OK,
        acked_command_id: 0x33,
        maximum_pending_wait: Some(1),
        session_id: None,
        num_bytes_dropped: None,
    };
    assert_eq!(
        invalid.encode(&EncodeContext::default(), &mut Writer::new(&mut Vec::new())),
        Err(EncodeError::UnexpectedConditionalField {
            field: "maximumPendingWait"
        })
    );
}

#[test]
fn general_lingo_registered_round_trips() {
    let mut snapshots = Vec::new();

    let (ack, transaction_id) = LingoMessage::decode_registered(0x02, &[0, 1, 0, 0x38]).unwrap();
    snapshot_registered(
        &mut snapshots,
        "iPod ack with transaction ID",
        ack,
        transaction_id,
        EncodeContext::default(),
    );

    let (ack, transaction_id) = LingoMessage::decode_registered(0x02, &[0, 0x13]).unwrap();
    snapshot_registered(
        &mut snapshots,
        "iPod ack without transaction ID",
        ack,
        transaction_id,
        EncodeContext::default(),
    );

    let mut challenge_body = vec![0, 7];
    challenge_body.extend(0..16);
    challenge_body.push(2);
    let (challenge, transaction_id) = LingoMessage::decode_registered(0x17, &challenge_body).unwrap();
    snapshot_registered(
        &mut snapshots,
        "challenge with transaction ID",
        challenge,
        transaction_id,
        EncodeContext {
            adjusted_payload_length: 17,
            has_transaction_id: true,
        },
    );

    snapshot_registered(
        &mut snapshots,
        "start IDPS",
        LingoMessage::StartIDPS(StartIDPS {}),
        Some(2),
        EncodeContext::default(),
    );
    snapshot_registered(
        &mut snapshots,
        "second authentication challenge",
        LingoMessage::GetAccessoryAuthenticationSignature(GetAccessoryAuthenticationSignature {
            challenge: Some(GetAccessoryAuthenticationSignatureChallengeDisjoint::Value20ByteChallenge(
                [0x5a; 20],
            )),
            authentication_retry_counter: Some(1),
        }),
        Some(9),
        EncodeContext {
            adjusted_payload_length: 21,
            has_transaction_id: true,
        },
    );

    assert_debug_snapshot!(snapshots);
}

#[test]
fn enum_ranges_reject_outside_values() {
    let invalid = LingoMessage::AckAccessoryAuthenticationStatus(AckAccessoryAuthenticationStatus {
        status: AckAccessoryAuthenticationStatusStatus::Other(0),
    });
    assert_eq!(
        LingoRegistry::encode(&invalid, &EncodeContext::default(), &mut Writer::new(&mut Vec::new())),
        Err(EncodeError::InvalidEnumValue { value: 0 })
    );
}

#[test]
fn fixed_bytes_and_token_lengths_are_checked() {
    fn without_path(error: catplay_lingo::RegistryDecodeError) -> catplay_lingo::RegistryDecodeError {
        match error {
            catplay_lingo::RegistryDecodeError::Decode(error) => catplay_lingo::RegistryDecodeError::Decode(error.cause().clone()),
            other => other,
        }
    }

    assert_eq!(
        LingoRegistry::decode(
            0x17,
            &DecodeContext {
                adjusted_payload_length: 17,
                has_transaction_id: false,
            },
            &[0; 15],
        )
        .map_err(without_path),
        Err(catplay_lingo::RegistryDecodeError::Decode(DecodeError::UnexpectedEof {
            needed: 16,
            remaining: 15,
        }))
    );
    assert_eq!(
        LingoRegistry::decode(0x39, &DecodeContext::default(), &[1, 1, 0]).map_err(without_path),
        Err(catplay_lingo::RegistryDecodeError::Decode(DecodeError::InvalidFieldValue {
            field: "token length"
        }))
    );
    assert_eq!(
        LingoRegistry::decode(0x39, &DecodeContext::default(), &[1, 4, 0xfe, 0xfd, 0xaa, 0xbb]).map_err(without_path),
        Err(catplay_lingo::RegistryDecodeError::Decode(DecodeError::UnknownToken {
            token: "SetFIDTokenValuesTokens",
            fid_type: 0xfe,
            fid_subtype: 0xfd,
        }))
    );
    assert_eq!(
        LingoRegistry::decode(0x3a, &DecodeContext::default(), &[1, 2, 0xfe, 0xfd]).map_err(without_path),
        Err(catplay_lingo::RegistryDecodeError::Decode(DecodeError::UnknownToken {
            token: "AckFIDTokenValuesTokenACKs",
            fid_type: 0xfe,
            fid_subtype: 0xfd,
        }))
    );
}

#[test]
fn registered_link_rejects_missing_transaction_id() {
    let start = LingoMessage::StartIDPS(StartIDPS {});
    assert_eq!(
        start.encode_registered(None, &EncodeContext::default()),
        Err(RegisteredError::MissingTransactionId)
    );
}
