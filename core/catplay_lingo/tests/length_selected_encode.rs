use catplay_lingo::lingo_0x0::{GetAccessoryAuthenticationSignature as Challenge, LingoMessage as General};
use catplay_lingo::lingos::lingo_0x2::ContextButtonStatus as Buttons;
use catplay_lingo::{DecodeContext, EncodeContext, EncodeError, Frame, Iap1Message, Packet, Writer};
use catplay_reflect::static_strings;

fn encode<T: Iap1Message>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut bytes = vec![0xaa];
    let result = value.encode(&EncodeContext::default(), &mut Writer::new(&mut bytes));
    if result.is_err() {
        assert_eq!(bytes, [0xaa], "validation must not append invalid bytes");
    }
    result.map(|_| bytes[1..].to_vec())
}

#[test]
fn challenge_variants_encode_without_length_context() {
    for length in [17, 21] {
        let mut payload = vec![0x42; length];
        payload[length - 1] = 3;
        let value = Challenge::decode(
            &DecodeContext {
                adjusted_payload_length: length,
                has_transaction_id: false,
            },
            &payload,
        )
        .unwrap();
        assert_eq!(encode(&value).unwrap(), payload);
        let packet = Packet {
            message: General::GetAccessoryAuthenticationSignature(value).into(),
            transaction_id: Some(7),
        };
        let body = packet.encode_body().unwrap();
        let frame = Frame { body: &body };
        assert_eq!(Packet::decode(&frame).unwrap(), packet);
    }
}

#[test]
fn incomplete_challenge_is_rejected() {
    let mut value = Challenge::decode(
        &DecodeContext {
            adjusted_payload_length: 17,
            has_transaction_id: false,
        },
        &[1; 17],
    )
    .unwrap();
    value.authentication_retry_counter = None;
    assert!(matches!(encode(&value), Err(EncodeError::RoundTripDecode(_))));
}

#[test]
fn button_prefixes_round_trip_without_length_context() {
    for length in 1..=4 {
        let payload = vec![1; length];
        let value = Buttons::decode(
            &DecodeContext {
                adjusted_payload_length: length,
                has_transaction_id: false,
            },
            &payload,
        )
        .unwrap();
        assert_eq!(encode(&value).unwrap(), payload);
    }
}

#[test]
fn button_holes_are_rejected_even_when_bytes_decode_successfully() {
    let full = Buttons::decode(
        &DecodeContext {
            adjusted_payload_length: 4,
            has_transaction_id: false,
        },
        &[1; 4],
    )
    .unwrap();
    for mask in [2, 4, 5, 6] {
        let mut value = full.clone();
        if mask & 1 == 0 {
            value.button_states1 = None;
        }
        if mask & 2 == 0 {
            value.button_states2 = None;
        }
        if mask & 4 == 0 {
            value.button_states3 = None;
        }
        assert_eq!(encode(&value), Err(EncodeError::RoundTripMismatch));
    }
}

static_strings! {
    pub struct TestLingoString(u8) {
        S1 = "value",
        S2 = "HighestFieldBit",
    }
}

catplay_lingo::iap1! {
    #[iap1(context = ctx, strings = TestLingoString, lingo = 0, command = 1, source = device,
        response = false, ack = false, deprecated = false, transaction_id = prohibited)]
    struct HighestFieldBit {
        fields {
            length_optional pub value: u8 => { bit: 127, key: "value", when: ctx.encoding() || ctx.adjusted_payload_length == 1, predicate_value: false },
        }
        steps {
            1 => length_optional value: u8 => { bit: 127, key: "value", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || ctx.adjusted_payload_length == 1, program: [catplay_lingo::FlatPredicateToken::Or, catplay_lingo::FlatPredicateToken::Encoding, catplay_lingo::FlatPredicateToken::Eq, catplay_lingo::FlatPredicateToken::AdjustedPayloadLength, catplay_lingo::FlatPredicateToken::Integer(1u8)] } },
            2 => length_optional value: u8 => { bit: 127, key: "value", wire: [scalar], project_wire: catplay_lingo::Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding(), program: [catplay_lingo::FlatPredicateToken::Encoding] } },
        }
    }
}

#[test]
fn highest_bit_deduplicates_repeated_steps() {
    assert_eq!(encode(&HighestFieldBit { value: Some(42) }).unwrap(), [42]);
}
