use catplay_lingo::{
    DecodeContext, DecodeError, EncodeContext, EncodeError, Iap1Decode, Iap1DecodeCounted, Iap1DecodeRemaining, Iap1Encode,
    Iap1EncodeCounted, Iap1Message, Reader, Writer,
    lingo_0x0::{IPodNotification, IPodNotificationNotificationType, ReturniPodName},
};

#[test]
fn projected_raw_string_includes_nul_in_encoded_length() {
    for name in ["", "CatPlay", "żółw"] {
        let message = ReturniPodName { i_pod_name: name.into() };
        let mut payload = Vec::new();
        message
            .encode(&EncodeContext::default(), &mut Writer::new(&mut payload))
            .unwrap();
        assert_eq!(payload, [name.as_bytes(), &[0]].concat());
        assert_eq!(
            ReturniPodName::decode(
                &DecodeContext {
                    adjusted_payload_length: payload.len(),
                    ..Default::default()
                },
                &payload
            )
            .unwrap(),
            message,
        );
    }
}

#[test]
fn projected_optional_raw_string_encodes_bundle_notification() {
    let message = IPodNotification {
        notification_type: IPodNotificationNotificationType::NowPlayingAppBundleName,
        bundle_seed_id: Some("com.apple.mobileipod".into()),
        ..Default::default()
    };
    let mut payload = Vec::new();
    message
        .encode(&EncodeContext::default(), &mut Writer::new(&mut payload))
        .unwrap();
    assert_eq!(payload, [b"\x0acom.apple.mobileipod".as_slice(), &[0]].concat());
    assert_eq!(
        IPodNotification::decode(
            &DecodeContext {
                adjusted_payload_length: payload.len(),
                ..Default::default()
            },
            &payload
        )
        .unwrap(),
        message,
    );
}

#[test]
fn strings_strip_the_wire_terminator_and_encode_it_once() {
    for text in ["", "hello", "żółw"] {
        let mut wire = Vec::new();
        text.encode(&mut Writer::new(&mut wire)).unwrap();
        assert_eq!(wire.last(), Some(&0));
        assert_eq!(&wire[..wire.len() - 1], text.as_bytes());
        assert_eq!(String::decode(&mut Reader::new(&wire)).unwrap(), text);
        assert_eq!(String::decode_remaining(&mut Reader::new(&wire), "text").unwrap(), text);
        assert_eq!(String::decode_counted(&mut Reader::new(&wire), wire.len()).unwrap(), text);
        let mut counted = Vec::new();
        text.encode_counted(&mut Writer::new(&mut counted), wire.len(), "text")
            .unwrap();
        assert_eq!(counted, wire);
        assert_eq!(
            text.encode_counted(&mut Writer::new(&mut Vec::new()), text.len(), "text"),
            Err(EncodeError::InvalidFieldValue { field: "text" })
        );
    }
}

#[test]
fn missing_nul_and_bad_utf8_are_distinct_errors() {
    for wire in [&[][..], &b"hello"[..], &[0xff][..]] {
        assert_eq!(String::decode(&mut Reader::new(wire)), Err(DecodeError::MissingNullTerminator));
        assert_eq!(
            String::decode_counted(&mut Reader::new(wire), wire.len()),
            Err(DecodeError::MissingNullTerminator)
        );
    }
    assert_eq!(String::decode(&mut Reader::new(&[0xff, 0])), Err(DecodeError::InvalidUtf8));
    let mut reader = Reader::new(&[b'a', 0, 42]);
    assert_eq!(String::decode_counted(&mut reader, 2).unwrap(), "a");
    assert_eq!(reader.read_u8().unwrap(), 42);
    reader.finish().unwrap();
}

#[test]
fn borrowed_counted_string_borrows_input_and_releases_reader() {
    let wire = b"hello\0\x2a";
    let text = {
        let mut reader = Reader::new(wire);
        let text = <&str>::decode_counted(&mut reader, 6).unwrap();
        assert_eq!(reader.read_u8().unwrap(), 42);
        reader.finish().unwrap();
        text
    };
    assert_eq!(text, "hello");
    assert_eq!(text.as_ptr(), wire.as_ptr());
}

#[test]
fn borrowed_counted_string_preserves_validation() {
    for wire in [&[][..], &b"hello"[..], &[0xff][..]] {
        assert_eq!(
            <&str>::decode_counted(&mut Reader::new(wire), wire.len()),
            Err(DecodeError::MissingNullTerminator)
        );
    }
    assert_eq!(
        <&str>::decode_counted(&mut Reader::new(&[0xff, 0]), 2),
        Err(DecodeError::InvalidUtf8)
    );
    assert_eq!(<&str>::decode_counted(&mut Reader::new(&[0]), 1), Ok(""));
    assert_eq!(
        <&str>::decode_counted(&mut Reader::new(&[0]), 2),
        Err(DecodeError::UnexpectedEof { needed: 2, remaining: 1 })
    );
}
