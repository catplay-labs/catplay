use crate::{
    carplay_rx::sink::AirPlayReceiver,
    msg::{AudioFormat, AudioFormatStruct, AudioLatency, AudioType, InfoMessageResponse, StreamType},
};

impl AirPlayReceiver {
    pub fn setup_audio_defaults(r: &mut InfoMessageResponse) {
        // --- Main Audio - Compatibility ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: Some(AudioFormat::PCM_8000_MONO | AudioFormat::PCM_16000_MONO | AudioFormat::PCM_24000_MONO),
            audio_output_formats: Some(
                AudioFormat::PCM_8000_MONO
                    | AudioFormat::PCM_16000_MONO
                    | AudioFormat::PCM_24000_MONO
                    | AudioFormat::PCM_44100_STEREO
                    | AudioFormat::PCM_48000_STEREO,
            ),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Compatibility),
        });

        // --- Alt Audio - Compatibility ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: None,
            audio_output_formats: Some(AudioFormat::PCM_44100_STEREO | AudioFormat::PCM_48000_STEREO),
            stream_type: StreamType::AltAudio,
            audio_type: Some(AudioType::Compatibility),
        });

        // --- Main Audio - Alert ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: None,
            audio_output_formats: Some(AudioFormat::PCM_44100_STEREO | AudioFormat::PCM_48000_STEREO | AudioFormat::OPUS_48000_MONO),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Alert),
        });

        // --- Main Audio - Default ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: Some(
                AudioFormat::PCM_24000_MONO | AudioFormat::PCM_16000_MONO | AudioFormat::OPUS_24000_MONO | AudioFormat::OPUS_16000_MONO,
            ),
            audio_output_formats: Some(
                AudioFormat::PCM_24000_MONO | AudioFormat::PCM_16000_MONO | AudioFormat::OPUS_24000_MONO | AudioFormat::OPUS_16000_MONO,
            ),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Default),
        });

        // --- Main Audio - Media ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: None,
            audio_output_formats: Some(AudioFormat::PCM_44100_STEREO | AudioFormat::PCM_48000_STEREO),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Media),
        });

        // --- Main Audio - Telephony ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: Some(
                AudioFormat::PCM_16000_MONO
                    | AudioFormat::PCM_24000_MONO
                    | AudioFormat::PCM_32000_MONO
                    | AudioFormat::OPUS_16000_MONO
                    | AudioFormat::OPUS_24000_MONO,
            ),
            audio_output_formats: Some(
                AudioFormat::PCM_16000_MONO
                    | AudioFormat::PCM_24000_MONO
                    | AudioFormat::PCM_32000_MONO
                    | AudioFormat::OPUS_16000_MONO
                    | AudioFormat::OPUS_24000_MONO,
            ),

            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::Telephony),
        });

        // --- Main Audio - SpeechRecognition (Siri) ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: Some(AudioFormat::PCM_24000_MONO | AudioFormat::OPUS_24000_MONO),
            audio_output_formats: Some(AudioFormat::PCM_24000_MONO | AudioFormat::OPUS_24000_MONO),
            stream_type: StreamType::MainAudio,
            audio_type: Some(AudioType::SpeechRecognition),
        });

        // --- Alt Audio - Default ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: None,
            audio_output_formats: Some(
                AudioFormat::PCM_44100_STEREO
                    | AudioFormat::PCM_48000_STEREO
                    | AudioFormat::OPUS_48000_MONO
                    | AudioFormat::OPUS_24000_MONO
                    | AudioFormat::OPUS_16000_MONO,
            ),
            stream_type: StreamType::AltAudio,
            audio_type: Some(AudioType::Default),
        });

        // --- Main High Audio - Media ---
        r.audio_formats.push(AudioFormatStruct {
            audio_input_formats: None,
            audio_output_formats: Some(AudioFormat::AAC_LC_48000_STEREO | AudioFormat::AAC_LC_44100_STEREO),
            stream_type: StreamType::MainHighAudio,
            audio_type: Some(AudioType::Media),
        });

        // --- Audio Latencies ---
        for (stype, atype, input_latency, output_latency) in [
            // -------- MainAudio --------

            // MainAudio catch-all (duplex)
            (StreamType::MainAudio, None, Some(0), Some(0)),
            // MainAudio Default (duplex)
            (StreamType::MainAudio, Some(AudioType::Default), Some(0), Some(0)),
            // MainAudio Media (output-only)
            (StreamType::MainAudio, Some(AudioType::Media), None, Some(0)),
            // MainAudio Telephony (duplex)
            (StreamType::MainAudio, Some(AudioType::Telephony), Some(0), Some(0)),
            // MainAudio SpeechRecognition (duplex)
            (StreamType::MainAudio, Some(AudioType::SpeechRecognition), Some(0), Some(0)),
            // MainAudio Alert (output-only)
            (StreamType::MainAudio, Some(AudioType::Alert), None, Some(0)),
            // -------- AltAudio --------

            // AltAudio catch-all (output-only)
            (StreamType::AltAudio, None, None, Some(0)),
            // AltAudio Default (output-only)
            (StreamType::AltAudio, Some(AudioType::Default), None, Some(0)),
            // -------- MainHighAudio --------

            // MainHighAudio Media (output-only)
            (StreamType::MainHighAudio, Some(AudioType::Media), None, Some(0)),
        ] {
            r.audio_latencies.push(AudioLatency {
                stream_type: Some(stype),
                audio_type: atype,
                sr: None,
                ss: None,
                ch: None,
                input_latency_micros: input_latency,
                output_latency_micros: output_latency,
            });
        }
    }
}
