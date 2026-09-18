use catplay_plist::{CachingSerializer, Dictionary, Value, from_bytes, to_writer_binary};

use crate::msg::{Command, CommandError, CommandRequestUI};

fn cache() -> CachingSerializer {
    CachingSerializer::new(CachingSerializer::CACHE_DEFAULT)
}

fn envelope(command_type: &str, params: Option<Value>) -> Dictionary {
    let mut map = Dictionary::new();
    map.insert("type".into(), command_type.into());
    if let Some(params) = params {
        map.insert("params".into(), params);
    }
    map
}

fn encode(map: &Dictionary) -> Vec<u8> {
    let mut bytes = Vec::new();
    to_writer_binary(&mut bytes, map).unwrap();
    bytes
}

fn assert_decodes(payload: &[u8], expected: &Command, cache: &mut CachingSerializer) {
    assert_eq!(&Command::deserialize(payload).unwrap(), expected);
    assert_eq!(&Command::deserialize_caching(payload, cache).unwrap(), expected);
}

#[test]
fn request_ui_captured_parameterless_binary_plist() {
    // 035.stdout:10723-10726 in the 2026-09-16 18:58:29 car capture.
    let payload = [
        0x62, 0x70, 0x6c, 0x69, 0x73, 0x74, 0x30, 0x30, 0xd1, 0x01, 0x02, 0x54, 0x74, 0x79, 0x70, 0x65, 0x59, 0x72, 0x65, 0x71, 0x75, 0x65,
        0x73, 0x74, 0x55, 0x49, 0x08, 0x0b, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a,
    ];
    assert_eq!(payload.len(), 61);
    assert_eq!(
        from_bytes::<Value>(&payload).unwrap(),
        Value::Dictionary(envelope("requestUI", None))
    );
    let mut cache = cache();
    for _ in 0..4 {
        assert_decodes(&payload, &Command::RequestUI(CommandRequestUI { url: None }), &mut cache);
    }
}

#[test]
fn request_ui_empty_and_url_params_preserve_serialization() {
    let mut cache = cache();
    for url in [None, Some("maps:/car/instrumentcluster"), Some(""), None] {
        let command = Command::RequestUI(CommandRequestUI {
            url: url.map(str::to_owned),
        });
        let mut params = Dictionary::new();
        if let Some(url) = url {
            params.insert("url".into(), url.into());
        }
        let expected = Value::Dictionary(envelope("requestUI", Some(params.into())));
        for payload in [
            command.serialize().unwrap(),
            command.serialize_caching(&mut cache).unwrap().to_vec(),
        ] {
            assert_eq!(from_bytes::<Value>(&payload).unwrap(), expected);
            assert_decodes(&payload, &command, &mut cache);
        }
    }
}

#[test]
fn request_ui_rejects_malformed_explicit_params_and_url() {
    let mut cache = cache();
    for invalid in [
        Value::Boolean(false),
        7.into(),
        "wrong".into(),
        Value::Array(vec![]),
        Value::Data(vec![]),
    ] {
        let mut params = Dictionary::new();
        params.insert("url".into(), invalid.clone());
        for params in [invalid, params.into()] {
            if let Value::Dictionary(ref map) = params {
                if matches!(map.get("url"), Some(Value::String(_))) {
                    continue; // Any string is a valid URL field at the protocol layer.
                }
            }
            let payload = encode(&envelope("requestUI", Some(params.clone())));
            let result = Command::deserialize(&payload);
            assert!(matches!(result, Err(CommandError::FailedDeserialize(..))), "{params:?}: {result:?}");
            assert!(matches!(
                Command::deserialize_caching(&payload, &mut cache),
                Err(CommandError::FailedDeserialize(..))
            ));
        }
        // A failed cached decode must not poison the next parameterless request.
        assert_decodes(
            &encode(&envelope("requestUI", None)),
            &Command::RequestUI(CommandRequestUI { url: None }),
            &mut cache,
        );
    }
}

#[test]
fn other_nested_commands_still_require_params() {
    let mut cache = cache();
    for command_type in [
        "duckAudio",
        "unduckAudio",
        "disableBluetooth",
        "changeModes",
        "modesChanged",
        "forceKeyFrame",
        "hidSetInputMode",
        "requestSiri",
        "setNightMode",
        "setLimitedUI",
        "iAPSendMessage",
    ] {
        let payload = encode(&envelope(command_type, None));
        for result in [Command::deserialize(&payload), Command::deserialize_caching(&payload, &mut cache)] {
            match result {
                Err(CommandError::FailedDeserialize(name, error)) => {
                    assert_eq!(name.as_str(), command_type);
                    assert!(error.to_string().contains("missing field `params`"), "{error}");
                }
                other => panic!("{command_type}: expected missing params, got {other:?}"),
            }
        }
    }
}
