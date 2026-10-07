use catplay_csm::{
    decoder::{AsCsmPacket, CsmFlag, CsmPacked, CsmPackedTight, CsmPacket, CsmPacketId, CsmPacketRegistry, CsmPacketUtil},
    msg::iap2::{
        AccessoryWiFiConfigurationInformation, AuthenticationCertificate, BluetoothPairingAccessoryInformation,
        BluetoothTransportComponent, EngineTypes, IdentificationInformation, IdentificationRejected,
        PlaybackQueueListContentTransferInfoRequest, RequestAuthenticationChallengeResponse, SetNowPlayingInformation,
        StartNowPlayingUpdates, StartNowPlayingUpdatesMediaItemAttributes, StartNowPlayingUpdatesPlaybackAttributes, StopNowPlayingUpdates,
        VehicleInformationComponent,
    },
};
use insta::assert_debug_snapshot;

#[derive(Debug, PartialEq, Eq)]
struct TestSnapshot {
    packet: String,
    bytes: Vec<u8>,
}

#[test]
fn test_risky_packets() {
    fn assert_packet<T: CsmPacket + PartialEq + AsCsmPacket>(snap: &mut Vec<TestSnapshot>, packet: &T) {
        let registry = CsmPacketRegistry::static_registry();
        let encoded = registry.encode(packet).expect("failed to encode packet");
        let decoded = registry.decode(&encoded).expect("failed to decode packet");
        let decoded = T::cast(&decoded).expect("failed to cast packet");

        assert_eq!(decoded, packet, "round-trip mismatch for {packet:?}");
        snap.push(TestSnapshot {
            packet: format!("{decoded:?}"),
            bytes: encoded,
        });
    }

    let mut snap = Vec::new();

    assert_packet(
        &mut snap,
        &AuthenticationCertificate {
            authentication_certificate: vec![b'T', b'E', b'S', b'T'].into(),
        },
    );

    assert_packet(
        &mut snap,
        &IdentificationInformation {
            name: "Name".into(),
            model_identifier: "Model".into(),
            manufacturer: "Manufacturer".into(),
            serial_number: "Serial".into(),
            firmware_version: "FirmwareVersion".into(),
            messages_sent_by_accessory: CsmPacked::from(vec![
                AccessoryWiFiConfigurationInformation::PACKET_ID,
                StartNowPlayingUpdates::PACKET_ID,
                StopNowPlayingUpdates::PACKET_ID,
                SetNowPlayingInformation::PACKET_ID,
            ]),
            vehicle_information_component: Some(VehicleInformationComponent {
                identifier: 1,
                name: "Auto Box".into(),
                display_name: "DisplayName".into(),
                engine_type: vec![EngineTypes::Gasoline],
                ..VehicleInformationComponent::default()
            }),
            supported_language: vec!["en".into()],
            bluetooth_transport_component: vec![BluetoothTransportComponent {
                transport_component_identifier: 1,
                transport_component_name: "Blue".into(),
                transport_supports_iap2_connection: CsmFlag::Yes,
                bluetooth_transport_media_access_control_address: [1, 2, 3, 4, 5, 6],
            }],
            ..IdentificationInformation::default()
        },
    );

    assert_packet(
        &mut snap,
        &StartNowPlayingUpdates {
            media_item_attributes: Some(StartNowPlayingUpdatesMediaItemAttributes::default()),
            playback_attributes: Some(StartNowPlayingUpdatesPlaybackAttributes::default()),
            playback_queue_list_content_transfer_info_request: Some(PlaybackQueueListContentTransferInfoRequest::default()),
        },
    );

    assert_packet(
        &mut snap,
        &IdentificationRejected {
            name: CsmFlag::Yes,
            ..IdentificationRejected::default()
        },
    );

    assert_packet(
        &mut snap,
        &RequestAuthenticationChallengeResponse {
            authentication_challenge: vec![0x42u8; 20].into(),
        },
    );

    assert_packet(
        &mut snap,
        &BluetoothPairingAccessoryInformation {
            bluetooth_transport_component_identifier: 7,
            pairing_data_p192: CsmPackedTight::new([CsmPackedTight::new([0x11; 16]), CsmPackedTight::new([0x22; 16])]),
            pairing_data_p256: None,
        },
    );

    assert_packet(
        &mut snap,
        &IdentificationInformation {
            messages_sent_by_accessory: CsmPacked::from(&[0x1234, 0xABCD]),
            messages_received_from_device: CsmPacked::from(&[0x5678]),
            ..Default::default()
        },
    );

    assert_debug_snapshot!(snap);
}
