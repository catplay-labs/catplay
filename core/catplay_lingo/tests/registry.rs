pub use catplay_lingo::*;

mod strings {
    pub use catplay_lingo::lingos::strings::LingoString;
}

#[path = "../src/lingos/lingo_0x5.rs"]
mod lingo_0x5;

use lingo_0x5::{LingoMessage, LingoRegistry};

#[test]
fn generated_registry_matches_known_ids_without_dynamic_registration() {
    assert_eq!(LingoRegistry::LINGO_ID, 0x05);
    let metadata = LingoRegistry::metadata(0x0002).unwrap();
    assert_eq!(metadata.command, 0x0002);
    assert!(metadata.deprecated);
    assert_eq!(LingoRegistry::metadata(0xffff), None);

    let message = LingoRegistry::decode(0x0002, &DecodeContext::default(), &[]).unwrap();
    assert!(matches!(message, LingoMessage::BeginHighPower(_)));
    assert!(message.cast::<lingo_0x5::BeginHighPower>().is_some());
    assert!(message.cast::<lingo_0x5::EndHighPower>().is_none());
    assert_eq!(LingoRegistry::command_id(&message), 0x0002);
    let mut payload = Vec::new();
    LingoRegistry::encode(&message, &EncodeContext::default(), &mut Writer::new(&mut payload)).unwrap();
    assert!(payload.is_empty());

    assert_eq!(
        LingoRegistry::decode(0xffff, &DecodeContext::default(), &[]),
        Err(RegistryDecodeError::UnknownCommand {
            lingo: 0x05,
            command: 0xffff
        })
    );
    let RegistryDecodeError::Decode(error) = LingoRegistry::decode(0x0002, &DecodeContext::default(), &[0x01]).unwrap_err() else {
        panic!("expected a payload decoding error");
    };
    assert_eq!(error.path().unwrap().as_str(), "BeginHighPower");
    assert_eq!(error.cause(), &DecodeError::TrailingData { remaining: 1 });
}
