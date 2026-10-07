#[cfg(test)]
mod tests {
    use catplay_csm::decoder::*;
    use core::fmt::Debug;

    fn assert_tlv_round_trip<T>(value: T, expected: &[u8])
    where
        T: CsmPayloadEncode + CsmDecode + PartialEq + Debug,
    {
        let encoded = CsmWriter::serialize(|writer| value.encode_param(0x1234, writer));

        assert_eq!(encoded, expected);
        assert_eq!(value.measure(), expected.len() - 4);
        assert_eq!(T::decode_from_bytes(&encoded[4..]), value);
    }

    #[test]
    fn reader_counts_and_streams_with_callback_supplied_at_stream_time() {
        let data = [
            0, 5, 0, 1, 0xAA, // id 1
            0, 4, 0, 2, // id 2
            0, 5, 0, 1, 0xBB, // id 1
            0,    // incomplete final header
        ];
        let mut reader = CsmReader::new(&data);
        assert_eq!(reader.count_repeating(1), 2);
        assert_eq!(reader.count_repeating(2), 1);
        assert_eq!(reader.count_repeating(3), 0);

        let mut values = Vec::new();
        reader.stream_all(|param| values.push((param.id, param.value)));
        assert_eq!(values, [(1, &data[4..5]), (2, &data[9..9]), (1, &data[13..14])]);
        assert_eq!(reader.take_error(), Some(CsmError::Underflow));
    }

    #[test]
    fn test_packed_array_decode() {
        assert_eq!(<[u8; 4]>::decode_from_bytes(&[]), [0; 4]);
        assert_eq!(<[u8; 4]>::decode_from_bytes(&[1, 2]), [1, 2, 0, 0]);
        assert_eq!(<[u8; 4]>::decode_from_bytes(&[1, 2, 3, 4]), [1, 2, 3, 4]);
        assert_eq!(<[u8; 2]>::decode_from_bytes(&[1, 2, 3, 4]), [1, 2]);
        assert_eq!(<[u8; 0]>::decode_from_bytes(&[1, 2]), []);
        assert_eq!(CsmPackedTight::<u8, 4>::decode_from_bytes(&[1, 2]).data, [1, 2, 0, 0]);
        assert_eq!(<[u16; 3]>::decode_from_bytes(&[0x12, 0x34, 0x56]), [0x1234, 0, 0]);
    }

    #[test]
    fn test_byte_array_encodes_as_single_tlv() {
        let value = CsmByteArray::from([0x54, 0x45, 0x53, 0x54, 0x00, 0xFF]);
        let encoded = CsmWriter::serialize(|writer| value.encode_param(0x1234, writer));

        // One header for the entire buffer, not a separate TLV for each byte.
        assert_eq!(encoded, [0x00, 0x0A, 0x12, 0x34, 0x54, 0x45, 0x53, 0x54, 0x00, 0xFF]);
        assert_eq!(CsmByteArray::decode_from_bytes(&encoded[4..]), value);

        let empty = CsmByteArray::default();
        let encoded = CsmWriter::serialize(|writer| empty.encode_param(0x1234, writer));
        assert_eq!(encoded, [0x00, 0x04, 0x12, 0x34]);
        assert_eq!(CsmByteArray::decode_from_bytes(&encoded[4..]), empty);
    }

    #[test]
    fn test_byte_and_string_payload_codecs() {
        fn check<T: CsmPayloadEncode>(value: T, payload: &[u8]) {
            assert_eq!(value.measure(), payload.len());
            assert_eq!(value.serialize(), payload);
            let encoded = CsmWriter::serialize(|writer| value.encode_param(0x1234, writer));
            assert_eq!(&encoded[..2], &((payload.len() + 4) as u16).to_be_bytes());
            assert_eq!(&encoded[2..4], &[0x12, 0x34]);
            assert_eq!(&encoded[4..], payload);
        }

        check([0x00u8, 0xFF], &[0x00, 0xFF]);
        check(&[0x00u8, 0xFF][..], &[0x00, 0xFF]);
        check([0u8; 0], &[]);
        check(&[] as &[u8], &[]);
        check("", &[0]);
        check("ą", &[0xC4, 0x85, 0]);
    }

    #[test]
    fn test_packed_measure_uses_element_measure() {
        struct Measured;

        impl CsmPayloadEncode for Measured {
            fn encode_to_bytes(&self, _: &mut CsmWriter) {
                panic!("measurement must not encode the payload");
            }

            fn measure(&self) -> usize {
                2
            }
        }

        let packed = CsmPackedTight::new([Measured, Measured, Measured]);
        assert_eq!(packed.measure(), 6);
        assert_eq!(packed.as_ref().measure(), 6);
        assert_eq!((&[] as &[Measured]).measure(), 0);
    }

    #[test]
    fn test_oversized_primitive_payload_emits_no_tlv() {
        fn check<T: CsmPayloadEncode>(value: T) {
            let mut encoded = Vec::new();
            let mut callback = |chunk: &[u8]| encoded.extend_from_slice(chunk);
            let mut writer = CsmWriter::new(&mut callback);
            value.encode_param(0x1234, &mut writer);
            assert!(writer.overflow);
            assert_eq!(writer.written, 0);
            assert!(encoded.is_empty());
        }

        let bytes = vec![0u8; u16::MAX as usize - 3];
        let text = "a".repeat(u16::MAX as usize - 4);
        check(bytes.as_slice());
        check(text.as_str());
    }

    #[test]
    fn test_signed_primitive_codecs() {
        assert_tlv_round_trip(-1i8, &[0x00, 0x05, 0x12, 0x34, 0xFF]);
        assert_tlv_round_trip(i16::MIN, &[0x00, 0x06, 0x12, 0x34, 0x80, 0x00]);
        assert_tlv_round_trip(-0x0102_0304i32, &[0x00, 0x08, 0x12, 0x34, 0xFE, 0xFD, 0xFC, 0xFC]);
        assert_tlv_round_trip(i64::MIN, &[0x00, 0x0C, 0x12, 0x34, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_signed_primitive_underflow_returns_zero() {
        assert_eq!(i8::decode_from_bytes(&[]), 0);
        assert_eq!(i16::decode_from_bytes(&[0xFF]), 0);
        assert_eq!(i32::decode_from_bytes(&[0xFF; 3]), 0);
        assert_eq!(i64::decode_from_bytes(&[0xFF; 7]), 0);
    }

    #[test]
    fn test_rational_codecs() {
        assert_tlv_round_trip(
            CsmRat16 {
                numerator: -2,
                denominator: 3,
            },
            &[0x00, 0x08, 0x12, 0x34, 0xFF, 0xFE, 0x00, 0x03],
        );
        assert_tlv_round_trip(
            CsmRat32 {
                numerator: -0x0102_0304,
                denominator: 0x1020_3040,
            },
            &[0x00, 0x0C, 0x12, 0x34, 0xFE, 0xFD, 0xFC, 0xFC, 0x10, 0x20, 0x30, 0x40],
        );
        assert_tlv_round_trip(
            CsmURat16 {
                numerator: 0x1234,
                denominator: 0xABCD,
            },
            &[0x00, 0x08, 0x12, 0x34, 0x12, 0x34, 0xAB, 0xCD],
        );
        assert_tlv_round_trip(
            CsmURat32 {
                numerator: 0x1234_5678,
                denominator: 0x9ABC_DEF0,
            },
            &[0x00, 0x0C, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0],
        );
    }

    #[test]
    fn test_rational_underflow_returns_default() {
        assert_eq!(CsmRat16::decode_from_bytes(&[0x00, 0x01, 0x00]), CsmRat16::default());
        assert_eq!(CsmRat32::decode_from_bytes(&[0; 7]), CsmRat32::default());
        assert_eq!(CsmURat16::decode_from_bytes(&[]), CsmURat16::default());
        assert_eq!(CsmURat32::decode_from_bytes(&[0; 7]), CsmURat32::default());
    }
}
