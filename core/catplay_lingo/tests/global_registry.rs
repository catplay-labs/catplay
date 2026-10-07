use catplay_lingo::{
    DecodeContext, EncodeContext, ProtocolRegistry, RegistryDecodeError, Writer,
    lingos::{self, LingoMessage, LingoRegistry},
};

#[test]
fn global_registry_delegates_to_lingo_registries() {
    let general = LingoRegistry::decode(0x00, 0x0000, &DecodeContext::default(), &[]).unwrap();
    assert!(matches!(&general, LingoMessage::General(_)));
    assert!(
        general
            .cast::<lingos::lingo_0x0::RequestIdentify>()
            .is_some()
    );
    assert_eq!(LingoRegistry::lingo_id(&general), 0x00);
    assert_eq!(LingoRegistry::command_id(&general), 0x0000);
    assert_eq!(
        LingoRegistry::metadata(0x00, 0x0000)
            .unwrap()
            .unwrap()
            .lingo,
        0x00
    );

    let power = LingoRegistry::decode(0x05, 0x0002, &DecodeContext::default(), &[]).unwrap();
    assert!(matches!(&power, LingoMessage::AccessoryPower(_)));
    assert!(power.cast::<lingos::lingo_0x5::BeginHighPower>().is_some());
    assert_eq!(LingoRegistry::lingo_id(&power), 0x05);
    assert_eq!(LingoRegistry::command_id(&power), 0x0002);
    assert!(
        LingoRegistry::metadata(0x05, 0x0002)
            .unwrap()
            .unwrap()
            .deprecated
    );
    let mut payload = Vec::new();
    LingoRegistry::encode(&power, &EncodeContext::default(), &mut Writer::new(&mut payload)).unwrap();
    assert!(payload.is_empty());
    assert_eq!(LingoRegistry::metadata(0x05, 0xffff), Ok(None));
}

#[test]
fn global_registry_distinguishes_unknown_lingo_from_unknown_command() {
    assert_eq!(
        LingoRegistry::decode(0xfe, 0x0000, &DecodeContext::default(), &[]),
        Err(RegistryDecodeError::UnknownLingo(0xfe))
    );
    assert_eq!(LingoRegistry::metadata(0xfe, 0x0000), Err(RegistryDecodeError::UnknownLingo(0xfe)));
    assert_eq!(
        LingoRegistry::decode(0x05, 0xffff, &DecodeContext::default(), &[]),
        Err(RegistryDecodeError::UnknownCommand {
            lingo: 0x05,
            command: 0xffff,
        })
    );
}
