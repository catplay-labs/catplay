use std::str::FromStr;

use bytes::BytesMut;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    modes::ChangeModes,
    msg::{
        CommandChangeMapZoomLevel, CommandDisableBluetooth, CommandDuckAudio, CommandForceKeyFrame, CommandHidSendReport,
        CommandHidSetInputMode, CommandIApSendMessage, CommandModesChanged, CommandRequestSiri, CommandRequestUI, CommandSetLimitedUI,
        CommandSetNightMode, CommandType, CommandUnduckAudio,
    },
    rtsp_frame::RtspString,
};
use catplay_plist::{CachingSerializer, Dictionary, PlistResult, Value, from_bytes, from_value, to_writer_binary};

use super::command_registry::CommandError;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    DuckAudio(CommandDuckAudio),
    UnduckAudio(CommandUnduckAudio),
    DisableBluetooth(CommandDisableBluetooth),
    ChangeModes(ChangeModes),
    ModesChanged(CommandModesChanged),
    ForceKeyFrame(CommandForceKeyFrame),
    HidSendReport(CommandHidSendReport),
    HidSetInputMode(CommandHidSetInputMode),
    RequestSiri(CommandRequestSiri),
    RequestUI(CommandRequestUI),
    SetNightMode(CommandSetNightMode),
    SetLimitedUI(CommandSetLimitedUI),
    ChangeMapZoomLevel(CommandChangeMapZoomLevel),

    IApSendMessage(CommandIApSendMessage),
}

impl Command {
    pub fn get_type(&self) -> CommandType {
        match self {
            Command::DuckAudio(_) => CommandType::DuckAudio,
            Command::UnduckAudio(_) => CommandType::UnduckAudio,
            Command::DisableBluetooth(_) => CommandType::DisableBluetooth,
            Command::ChangeModes(_) => CommandType::ChangeModes,
            Command::ModesChanged(_) => CommandType::ModesChanged,
            Command::ForceKeyFrame(_) => CommandType::ForceKeyFrame,
            Command::HidSendReport(_) => CommandType::HidSendReport,
            Command::HidSetInputMode(_) => CommandType::HidSetInputMode,
            Command::RequestSiri(_) => CommandType::RequestSiri,
            Command::RequestUI(_) => CommandType::RequestUI,
            Command::SetNightMode(_) => CommandType::SetNightMode,
            Command::SetLimitedUI(_) => CommandType::SetLimitedUI,
            Command::ChangeMapZoomLevel(_) => CommandType::ChangeMapZoomLevel,
            Command::IApSendMessage(_) => CommandType::IApSendMessage,
        }
    }

    pub fn serialize_caching(&self, serializer: &mut CachingSerializer) -> PlistResult<BytesMut> {
        serializer.serialize(&self)
    }

    pub fn serialize(&self) -> PlistResult<Vec<u8>> {
        let mut payload = Vec::new();
        to_writer_binary(&mut payload, self)?;
        Ok(payload)
    }

    pub fn deserialize(payload: &[u8]) -> Result<Command, CommandError> {
        let command_type = from_bytes::<CommandRawTypeRef>(payload)
            .map_err(|e| CommandError::InvalidPayload(e.into()))?
            .command_type;

        let Ok(_cmd_type) = CommandType::from_str(&command_type) else {
            return Err(CommandError::UnknownCommand(command_type.into()));
        };

        from_bytes::<Command>(payload).map_err(|e| CommandError::FailedDeserialize(command_type.into(), e.into()))
    }

    pub fn deserialize_caching(payload: &[u8], serializer: &mut CachingSerializer) -> Result<Command, CommandError> {
        let command_type = serializer
            .deserialize::<CommandRawTypeRef>(payload)
            .map_err(CommandError::InvalidPayload)?
            .command_type;

        let Ok(_cmd_type) = CommandType::from_str(&command_type) else {
            return Err(CommandError::UnknownCommand(command_type.into()));
        };

        serializer
            .deserialize::<Command>(payload)
            .map_err(|e| CommandError::FailedDeserialize(command_type.into(), e))
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", self.get_type().as_str())?;

        match self {
            Command::HidSendReport(data) => {
                // `hidSendReport` is flattened (no `params` envelope).
                map.serialize_entry("hidReport", &data.hid_report)?;
                if let Some(timestamp) = data.timestamp {
                    map.serialize_entry("timestamp", &timestamp)?;
                }
                map.serialize_entry("uuid", &data.uuid)?;
            }
            Command::DuckAudio(data) => map.serialize_entry("params", data)?,
            Command::UnduckAudio(data) => map.serialize_entry("params", data)?,
            Command::DisableBluetooth(data) => map.serialize_entry("params", data)?,
            Command::ChangeModes(data) => map.serialize_entry("params", data)?,
            Command::ModesChanged(data) => map.serialize_entry("params", data)?,
            Command::ForceKeyFrame(data) => map.serialize_entry("params", data)?,
            Command::HidSetInputMode(data) => map.serialize_entry("params", data)?,
            Command::RequestSiri(data) => map.serialize_entry("params", data)?,
            Command::RequestUI(data) => map.serialize_entry("params", data)?,
            Command::SetNightMode(data) => map.serialize_entry("params", data)?,
            Command::SetLimitedUI(data) => map.serialize_entry("params", data)?,
            Command::ChangeMapZoomLevel(data) => map.serialize_entry("params", data)?,
            Command::IApSendMessage(data) => map.serialize_entry("params", data)?,
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = <Dictionary as Deserialize>::deserialize(deserializer)?;

        let Some(command_type_val) = map.remove("type") else {
            return Err(serde::de::Error::missing_field("type"));
        };
        let command_type: String = from_value(&command_type_val).map_err(serde::de::Error::custom)?;
        let cmd_type = CommandType::from_str(&command_type)
            .map_err(|_| serde::de::Error::custom(format!("unknown AirPlay command type: {command_type}")))?;

        let from_nested = |map: &mut Dictionary| -> Result<Value, D::Error> {
            Ok(map.remove("params").unwrap_or_else(|| Value::Dictionary(Dictionary::new())))
        };
        let from_flat = |map: Dictionary| -> Value { Value::Dictionary(map) };

        match cmd_type {
            CommandType::DuckAudio => from_value(&from_nested(&mut map)?).map(Command::DuckAudio),
            CommandType::UnduckAudio => from_value(&from_nested(&mut map)?).map(Command::UnduckAudio),
            CommandType::DisableBluetooth => from_value(&from_nested(&mut map)?).map(Command::DisableBluetooth),
            CommandType::ChangeModes => from_value(&from_nested(&mut map)?).map(Command::ChangeModes),
            CommandType::ModesChanged => from_value(&from_nested(&mut map)?).map(Command::ModesChanged),
            CommandType::ForceKeyFrame => from_value(&from_nested(&mut map)?).map(Command::ForceKeyFrame),
            CommandType::HidSetInputMode => from_value(&from_nested(&mut map)?).map(Command::HidSetInputMode),
            CommandType::RequestSiri => from_value(&from_nested(&mut map)?).map(Command::RequestSiri),
            CommandType::RequestUI => from_value(&from_nested(&mut map)?).map(Command::RequestUI),
            CommandType::SetNightMode => from_value(&from_nested(&mut map)?).map(Command::SetNightMode),
            CommandType::SetLimitedUI => from_value(&from_nested(&mut map)?).map(Command::SetLimitedUI),
            CommandType::ChangeMapZoomLevel => from_value(&from_nested(&mut map)?).map(Command::ChangeMapZoomLevel),
            CommandType::IApSendMessage => from_value(&from_nested(&mut map)?).map(Command::IApSendMessage),
            CommandType::HidSendReport => from_value(&from_flat(map)).map(Command::HidSendReport),
        }
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
struct CommandRawTypeRef {
    #[serde(rename = "type")]
    command_type: RtspString,
}

#[cfg(test)]
mod tests {
    use catplay_plist::{Dictionary, Value, from_bytes, to_writer_binary};

    use crate::msg::{Command, CommandChangeMapZoomLevel, CommandForceKeyFrame, CommandRequestUI};

    #[test]
    fn test_serialize_payload_direct_hid() {
        let report = Command::HidSendReport(super::super::CommandHidSendReport {
            hid_report: vec![1, 2, 3, 4].into(),
            timestamp: Some(123),
            uuid: "test".into(),
        });
        let mut expected = Dictionary::new();
        expected.insert("type".into(), "hidSendReport".into());
        expected.insert("hidReport".into(), Value::Data(vec![1, 2, 3, 4]));
        expected.insert("timestamp".into(), Value::Integer(123.into()));
        expected.insert("uuid".into(), "test".into());

        let payload = report.serialize().unwrap();
        let as_value = from_bytes::<Value>(&payload).unwrap();
        assert_eq!(as_value, expected.into());

        let deser = Command::deserialize(&payload).unwrap();
        assert_eq!(deser, report);
    }

    #[test]
    fn test_serialize_payload_direct_duck() {
        let report = Command::DuckAudio(super::super::CommandDuckAudio {
            duration_ms: 100.0,
            volume: 101.0,
        });

        let mut expected = Dictionary::new();
        expected.insert("type".into(), "duckAudio".into());
        let mut expected_params = Dictionary::new();
        expected_params.insert("durationMs".into(), 100.0.into());
        expected_params.insert("volume".into(), 101.0.into());
        expected.insert("params".into(), expected_params.into());

        let payload = report.serialize().unwrap();
        let as_value = from_bytes::<Value>(&payload).unwrap();
        assert_eq!(as_value, expected.into());

        let deser = Command::deserialize(&payload).unwrap();
        assert_eq!(deser, report);
    }

    #[test]
    fn test_deserialize_request_ui_params_variants() {
        let cases = [
            {
                let mut params = Dictionary::new();
                params.insert("url".into(), "https://example.com".into());

                let mut command = Dictionary::new();
                command.insert("type".into(), "requestUI".into());
                command.insert("params".into(), params.into());
                command
            },
            {
                let mut command = Dictionary::new();
                command.insert("type".into(), "requestUI".into());
                command.insert("params".into(), Dictionary::new().into());
                command
            },
            {
                let mut command = Dictionary::new();
                command.insert("type".into(), "requestUI".into());
                command
            },
        ];
        let expected_urls = [Some("https://example.com"), None, None];

        for (command, expected_url) in cases.into_iter().zip(expected_urls) {
            let mut payload = Vec::new();
            to_writer_binary(&mut payload, &command).unwrap();

            let deserialized = Command::deserialize(&payload).unwrap();
            assert_eq!(
                deserialized,
                Command::RequestUI(CommandRequestUI {
                    url: expected_url.map(str::to_owned),
                })
            );
        }
    }

    #[test]
    fn test_deserialize_change_map_zoom_level() {
        // Sent by a VW MMX head unit when zooming the map on the instrument cluster.
        let mut params = Dictionary::new();
        params.insert("zoomDirection".into(), 1u64.into());
        params.insert("uuid".into(), "b7e6c5a0-2222-4000-8000-000000000002".into());

        let mut command = Dictionary::new();
        command.insert("type".into(), "changeMapZoomLevel".into());
        command.insert("params".into(), params.into());

        let mut payload = Vec::new();
        to_writer_binary(&mut payload, &command).unwrap();

        let expected = Command::ChangeMapZoomLevel(CommandChangeMapZoomLevel {
            zoom_direction: 1,
            uuid: "b7e6c5a0-2222-4000-8000-000000000002".into(),
        });
        let deserialized = Command::deserialize(&payload).unwrap();
        assert_eq!(deserialized, expected);

        // The command is proxied to the iPhone, so it must survive serialization.
        let reserialized = Command::deserialize(&deserialized.serialize().unwrap()).unwrap();
        assert_eq!(reserialized, expected);
    }

    #[test]
    fn test_deserialize_force_key_frame_with_uuid() {
        // Sent by a VW MMX head unit to request a keyframe for the instrument cluster (alt screen).
        let mut params = Dictionary::new();
        params.insert("uuid".into(), "b7e6c5a0-2222-4000-8000-000000000002".into());

        let mut command = Dictionary::new();
        command.insert("type".into(), "forceKeyFrame".into());
        command.insert("params".into(), params.into());

        let mut payload = Vec::new();
        to_writer_binary(&mut payload, &command).unwrap();

        let expected = Command::ForceKeyFrame(CommandForceKeyFrame {
            uuid: Some("b7e6c5a0-2222-4000-8000-000000000002".into()),
        });
        let deserialized = Command::deserialize(&payload).unwrap();
        assert_eq!(deserialized, expected);

        // The command is proxied to the iPhone, so the display UUID must survive serialization.
        let reserialized = from_bytes::<Value>(&deserialized.serialize().unwrap()).unwrap();
        assert_eq!(reserialized, command.into());
    }

    #[test]
    fn test_serialize_force_key_frame_without_uuid() {
        let mut command = Dictionary::new();
        command.insert("type".into(), "forceKeyFrame".into());
        command.insert("params".into(), Dictionary::new().into());

        let payload = Command::ForceKeyFrame(CommandForceKeyFrame { uuid: None })
            .serialize()
            .unwrap();
        assert_eq!(from_bytes::<Value>(&payload).unwrap(), command.into());
    }
}
