use catplay_lingo::{Iap1Decode, Iap1Encode, Reader, Writer, lingo_0x0::*};

#[test]
fn rf_certification_token_round_trips_zero_known_and_reserved_bits() {
    for mask in [0u32, 1, 1 << 6, 1 << 2, 0x8000_006b] {
        let mut bytes = vec![7, 0, 2, 12];
        bytes.extend_from_slice(&mask.to_be_bytes());
        let mut reader = Reader::new(&bytes);
        let token = SetFIDTokenValuesTokens::decode(&mut reader).unwrap();
        reader.finish().unwrap();
        let SetFIDTokenValuesTokens::AccessoryInfoToken(info) = &token else {
            panic!("expected accessory info token");
        };
        assert_eq!(
            info.acc_info_type,
            SetFIDTokenValuesTokensAccessoryInfoTokenAccInfoType::RFCertifications
        );
        assert_eq!(info.rf_certification_declaration.unwrap().bits(), mask);
        let mut encoded = Vec::new();
        token.encode(&mut Writer::new(&mut encoded)).unwrap();
        assert_eq!(encoded, bytes);
    }
}
