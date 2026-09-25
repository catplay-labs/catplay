#[cfg(test)]
mod tests {
    use catplay_csm::{decoder::*, msg::AuthenticationCertificate};

    #[test]
    fn registry_should_not_be_empty_at_runtime() {
        let registry: &'static CsmPacketRegistry = CsmPacketRegistry::static_registry();
        let ids: Vec<String> = registry
            .all_known_ids()
            .iter()
            .map(|x| format!("{:#04x}", *x))
            .collect();
        assert!(!ids.is_empty());
        println!("Known IDs: {:?}", ids)
    }

    #[test]
    fn registry_decode_and_downcast_ref() {
        let auth_cert_packet: &[u8] = &[
            0x40, 0x40, // Magic
            0x00, 0x0E, // Length = 14 bytes
            0xAA, 0x01, // Type = AuthenticationCertificate
            0x00, 0x08, // TLV Length = 8 (2 len + 2 id + 4 data)
            0x00, 0x00, // TLV ID = 0x0000
            0x54, 0x45, 0x53, 0x54, // "TEST"
        ];
        let expected = AuthenticationCertificate {
            authentication_certificate: vec![b'T', b'E', b'S', b'T'].into(),
        };
        let registry = CsmPacketRegistry::static_registry();
        let decoded = registry.decode(auth_cert_packet).expect("expected packet");

        if let Some(x) = AuthenticationCertificate::cast(&decoded) {
            assert_eq!(expected, *x);
        } else {
            panic!("Wrong type {:?}", decoded)
        }
        println!("{:?}", decoded)
    }

    #[test]
    fn registry_encode() {
        let auth_cert_packet: &[u8] = &[
            0x40, 0x40, // Magic
            0x00, 0x0E, // Length = 14 bytes
            0xAA, 0x01, // Type = AuthenticationCertificate
            0x00, 0x08, // TLV Length = 8 (2 len + 2 id + 4 data)
            0x00, 0x00, // TLV ID = 0x0000
            0x54, 0x45, 0x53, 0x54, // "TEST"
        ];

        let registry = CsmPacketRegistry::static_registry();
        let packet = AuthenticationCertificate {
            authentication_certificate: vec![b'T', b'E', b'S', b'T'].into(),
        };

        let bytes = registry.encode(&packet).expect("no packet encoded");

        let decoded_packet = &registry
            .decode(auth_cert_packet)
            .expect("no packet decoded");

        let decoded_packet_unpacked = AuthenticationCertificate::cast(decoded_packet).expect("no packet after downcast ref");

        let reserialized = registry
            .encode(decoded_packet_unpacked)
            .expect("no packet encoded #2");

        assert_eq!(auth_cert_packet, bytes);
        assert_eq!(auth_cert_packet, reserialized);
        println!("{:?}", decoded_packet);
        println!("{:?}", bytes);
    }

    #[test]
    fn registry_encode_decode_unknown() {
        let registry = CsmPacketRegistry::new();

        let packet = CsmPacketWithPayload::new(0x1234, &[1, 2, 3]).unwrap();
        let encoded = packet.serialize();

        let expected = CsmUnknownPacket(0x1234, vec![1, 2, 3]);
        let decoded = registry.decode(&encoded).unwrap();

        assert_eq!(Some(expected).as_ref(), CsmUnknownPacket::cast(&decoded));

        let encoded = registry.encode(&decoded).unwrap();
        assert_eq!(encoded, encoded);
    }
}
