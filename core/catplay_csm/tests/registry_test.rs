#[cfg(test)]
mod tests {
    use catplay_csm::decoder::*;
    use catplay_csm::msg::iap2::PowerUpdate;

    #[test]
    fn registry_should_not_be_empty_at_runtime() {
        let registry: &'static CsmPacketRegistry = CsmPacketRegistry::static_registry();
        let ids: Vec<String> = registry
            .all_known_ids()
            .iter()
            .map(|x| format!("{:#04x}", *x))
            .collect();
        assert_eq!(ids.len(), 144);
        println!("Known IDs: {:?}", ids)
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

    #[cfg(feature = "clone_box")]
    #[test]
    fn boxed_packet_clone_preserves_type_and_payload() {
        let original: CsmPacketBox = CsmUnknownPacket(0x1234, vec![1, 2, 3]).into();
        let cloned = original.clone();
        drop(original);

        assert_eq!(cloned.cast::<CsmUnknownPacket>(), Some(&CsmUnknownPacket(0x1234, vec![1, 2, 3])));
    }

    #[test]
    fn registry_creates_default_and_decodes_payload() {
        let registry = CsmPacketRegistry::static_registry();
        let id = PowerUpdate::PACKET_ID;

        let empty = registry.create_by_id(id).unwrap();
        assert_eq!(empty.cast::<PowerUpdate>(), Some(&PowerUpdate::default()));

        let value = PowerUpdate {
            maximum_current_drawn_from_accessory: Some(42),
            ..PowerUpdate::default()
        };
        let decoded = registry
            .create_by_id_from_bytes(id, &value.serialize())
            .unwrap();
        assert_eq!(decoded.cast::<PowerUpdate>(), Some(&value));
    }
}
