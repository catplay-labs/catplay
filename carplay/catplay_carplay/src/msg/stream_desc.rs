use std::time::Duration;

use crate::{
    common::AirPlayCompressionType,
    msg::{AudioFormat, AudioType, StreamType},
};
use catplay_plist::{downcast_enum_to_legacy_u64, plist_enum_untagged, plist_struct, u64_as_i64};

plist_struct! {
    pub struct StreamDescriptionAudio {
        #[serde(
            rename = "streamConnectionID",
            with = "u64_as_i64"
        )]
        pub stream_connection_id: u64,
        #[serde(rename = "type")]
        pub stream_type: StreamType,

        #[serde(with = "downcast_enum_to_legacy_u64")]
        pub audio_format: AudioFormat,
        pub audio_latency_ms: u64,
        pub audio_loopback: Option<bool>,
        #[serde(default)]
        pub audio_type: AudioType,
        pub control_port: Option<u16>,
        pub data_port: Option<u16>,

        #[serde(default)]
        pub input: bool,
        pub spf: Option<u64>,
        pub vocoder_info: Option<VocoderInfo>,

        // Modern CarPlay
        #[serde(rename = "supportsRTPPacketRedundancy")]
        #[serde(default)]
        pub supports_rtp_packet_redundancy: bool,
        #[serde(default)]
        pub supports_high_accuracy_timestamps: bool
    }
}

plist_struct! {
    pub struct StreamDescriptionAudioLegacy {
        pub ct: AirPlayCompressionType, // ALAC (2)
        pub spf: u64,                   // 352
        pub latency_min: u64,           // 11025
        pub latency_max: u64,           // 88200
        pub sr: u32,

        #[serde(default)]
        pub audio_mode: String, // "default"

        pub control_port: u16,
        pub is_media: bool, // true

        #[serde(rename = "type")]
        pub stream_type: StreamType, // LegacyAudio (96)

        #[serde(rename = "streamConnectionID", with = "u64_as_i64", default)]
        pub stream_connection_id: u64, // default 0

        #[serde(default)]
        pub supports_dynamic_stream_id: bool,

        #[serde(default)]
        pub redundant_audio: u32, // default 0

        #[serde(default)]
        pub using_screen: bool, // default false

        #[serde(with = "downcast_enum_to_legacy_u64")]
        pub audio_format: AudioFormat,
    }
}

impl StreamDescriptionAudioLegacy {
    pub fn as_modern(&self) -> StreamDescriptionAudio {
        StreamDescriptionAudio {
            stream_connection_id: self.stream_connection_id,
            stream_type: self.stream_type,
            audio_format: self.audio_format,
            // Enforce a minimum of 250ms latency when streaming from AirPlay or AirPlay mirroring; 32ms minimum is not stable.
            audio_latency_ms: (self.latency_min.saturating_mul(1000) / u64::from(self.sr.max(1))).max(250),
            audio_loopback: None,
            audio_type: AudioType::Default,
            control_port: Some(self.control_port),
            data_port: None,
            input: false,
            spf: Some(self.spf),
            vocoder_info: None,
            supports_rtp_packet_redundancy: false,
            supports_high_accuracy_timestamps: false,
        }
    }
}

plist_struct! {
    pub struct StreamDescriptionScreen {
        #[serde(
            rename = "streamConnectionID",
            with = "u64_as_i64"
        )]
        pub stream_connection_id: u64,
        #[serde(rename = "type")]
        pub stream_type: StreamType,
        pub latency_ms: Option<u64>,
        pub uuid: Option<String>
    }
}

plist_enum_untagged! {
    pub enum StreamDescription {
        AudioLegacy(StreamDescriptionAudioLegacy),
        Audio(StreamDescriptionAudio),
        Screen(StreamDescriptionScreen),
        #[default]
        Invalid,
    }
}

plist_struct! {
    pub struct VocoderInfo {
        pub sample_rate: f64
    }
}

plist_struct! {
    pub struct StreamDescriptionResponse {
        pub data_port: u16,
        #[serde(rename = "type")]
        pub stream_type: StreamType,
        #[serde(
            rename = "streamConnectionID",
            with = "u64_as_i64",
            default
        )]
        pub stream_connection_id: u64,
        pub control_port: Option<u16>,

        pub sample_time: Option<u64>,
        pub timestamp: Option<u64>,
        pub timestamp_raw_ns: Option<u64>,

        // Modern CarPlay
        #[serde(rename = "supportsRTPPacketRedundancy")]
        #[serde(default)]
        pub supports_rtp_packet_redundancy: bool,
        #[serde(default)]
        pub supports_high_accuracy_timestamps: bool,
    }
}

plist_struct! {
    pub struct StreamFeedback {
        pub sr: Option<f32>,
        #[serde(rename = "type")]
        pub stream_type: StreamType,
        #[serde(
            rename = "streamConnectionID",
            with = "u64_as_i64"
        )]
        pub stream_connection_id: u64,

        pub sample_time: Option<u64>,
        pub timestamp: Option<u64>,
        pub timestamp_raw_ns: Option<u64>,
    }
}

impl StreamDescriptionScreen {
    pub fn new(stream_connection_id: u64, latency: Duration, uuid: &str) -> Self {
        Self {
            stream_connection_id,
            stream_type: StreamType::Screen,
            latency_ms: Some(latency.as_millis() as _),
            uuid: Some(uuid.into()),
        }
    }
}

impl StreamDescriptionAudio {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stream_connection_id: u64,
        latency: Duration,
        stream_type: StreamType,
        audio_format: AudioFormat,
        audio_type: AudioType,

        data_port: Option<u16>,
        control_port: Option<u16>,
        input: bool,

        supports_rtp_packet_redundancy: bool,
        supports_high_accuracy_timestamps: bool,
    ) -> Self {
        Self {
            stream_connection_id,
            audio_latency_ms: latency.as_millis() as _,
            stream_type,
            audio_format,
            audio_type,
            data_port,
            control_port,
            input,
            supports_high_accuracy_timestamps,
            supports_rtp_packet_redundancy,

            // vocoder_info
            ..Default::default()
        }
    }
}

impl StreamDescriptionResponse {
    pub fn new_audio(
        stream_type: StreamType,
        stream_connection_id: u64,
        data_port: u16,
        control_port: u16,
        supports_rtp_packet_redundancy: bool,
        supports_high_accuracy_timestamps: bool,
    ) -> Self {
        Self {
            data_port,
            control_port: Some(control_port),
            stream_type,
            stream_connection_id,
            supports_rtp_packet_redundancy,
            supports_high_accuracy_timestamps,
            ..Default::default()
        }
    }

    pub fn new_screen(data_port: u16) -> Self {
        Self {
            data_port,
            stream_type: StreamType::Screen,
            ..Default::default()
        }
    }
}

impl From<StreamDescriptionAudio> for StreamDescription {
    fn from(value: StreamDescriptionAudio) -> Self {
        Self::Audio(value)
    }
}

impl From<StreamDescriptionScreen> for StreamDescription {
    fn from(value: StreamDescriptionScreen) -> Self {
        Self::Screen(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_without_stream_connection_id_deserializes_as_audio() {
        let response: StreamDescriptionResponse = serde_json::from_str(r#"{"dataPort":65526,"type":100}"#).unwrap();

        assert_eq!(response.data_port, 65526);
        assert_eq!(response.stream_type, StreamType::MainAudio);
        assert_eq!(response.stream_connection_id, 0);
    }
}
