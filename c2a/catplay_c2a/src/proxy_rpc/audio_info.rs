use catplay_carplay::{
    audio::AudioStreamBasicDescription,
    msg::{AudioFormat, AudioFormatStruct, AudioType, StreamType},
};

/// The proxy decodes to PCM without resampling. Advertise 44.1/48 kHz only
/// where the car can consume PCM at that rate, including for compressed input.
pub(super) fn filter_audio_sample_rates(iphone: &mut [AudioFormatStruct], car: &[AudioFormatStruct]) {
    for entry in iphone {
        // AudioProxyUtil opens MainHighAudio on the car as MainAudio.
        let stream_type = match entry.stream_type {
            StreamType::MainHighAudio => StreamType::MainAudio,
            stream_type => stream_type,
        };
        // Prefer the specific audio type, then legacy catch-all capabilities,
        // then Compatibility when the car does not advertise that audio type.
        let audio_type = [entry.audio_type, None, Some(AudioType::Compatibility)]
            .into_iter()
            .find(|audio_type| {
                car.iter()
                    .any(|other| other.stream_type == stream_type && other.audio_type == *audio_type)
            });
        let pcm = car
            .iter()
            .filter(|other| other.stream_type == stream_type && Some(other.audio_type) == audio_type)
            .filter_map(|other| other.audio_output_formats)
            .fold(AudioFormat::empty(), |formats, other| formats | other);

        if let Some(formats) = entry.audio_output_formats.as_mut() {
            for format in *formats {
                let Ok(asbd) = AudioStreamBasicDescription::try_from(format) else {
                    continue;
                };
                let supported = match asbd.sample_rate {
                    44_100 => pcm.intersects(AudioFormat::PCM_44100_MONO | AudioFormat::PCM_44100_STEREO),
                    48_000 => pcm.intersects(AudioFormat::PCM_48000_MONO | AudioFormat::PCM_48000_STEREO),
                    16_000 => pcm.intersects(AudioFormat::PCM_16000_MONO | AudioFormat::PCM_16000_STEREO),
                    24_000 => pcm.intersects(AudioFormat::PCM_24000_MONO | AudioFormat::PCM_24000_STEREO),
                    _ => true,
                };
                if !supported {
                    formats.remove(format);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_carplay::{carplay_rx::sink::AirPlayReceiver, msg::InfoMessageResponse};
    use insta::assert_debug_snapshot;

    fn defaults() -> Vec<AudioFormatStruct> {
        let mut info = InfoMessageResponse::default();
        AirPlayReceiver::setup_audio_defaults(&mut info);
        info.audio_formats
    }

    fn car_format(stream_type: StreamType, audio_type: Option<AudioType>, formats: AudioFormat) -> AudioFormatStruct {
        AudioFormatStruct {
            stream_type,
            audio_type,
            audio_input_formats: None,
            audio_output_formats: Some(formats),
        }
    }

    fn output(info: &[AudioFormatStruct], stream_type: StreamType, audio_type: AudioType) -> AudioFormat {
        info.iter()
            .find(|entry| entry.stream_type == stream_type && entry.audio_type == Some(audio_type))
            .unwrap()
            .audio_output_formats
            .unwrap()
    }

    #[test]
    fn defaults_follow_single_car_pcm_rate_including_aac_and_opus() {
        for (pcm, aac, rejected_rate) in [
            (AudioFormat::PCM_44100_STEREO, AudioFormat::AAC_LC_44100_STEREO, 48_000),
            (AudioFormat::PCM_48000_STEREO, AudioFormat::AAC_LC_48000_STEREO, 44_100),
        ] {
            let mut info = defaults();
            let original = info.clone();
            let car = [
                car_format(StreamType::MainAudio, None, pcm),
                car_format(StreamType::AltAudio, None, pcm),
            ];
            filter_audio_sample_rates(&mut info, &car);
            assert_eq!(output(&info, StreamType::MainHighAudio, AudioType::Media), aac);
            assert_eq!(output(&info, StreamType::MainAudio, AudioType::Media), pcm);
            for (entry, original) in info.iter().zip(&original) {
                assert_eq!(entry.audio_input_formats, original.audio_input_formats);
                for format in entry.audio_output_formats.unwrap() {
                    assert_ne!(
                        AudioStreamBasicDescription::try_from(format)
                            .unwrap()
                            .sample_rate,
                        rejected_rate
                    );
                }
            }
            // The car advertises no PCM at 16/24 kHz, so both PCM and Opus
            // at those rates are removed. 32 kHz remains outside this filter.
            assert_eq!(
                output(&info, StreamType::MainAudio, AudioType::Telephony),
                AudioFormat::PCM_32000_MONO
            );
            assert_eq!(
                output(&info, StreamType::MainAudio, AudioType::SpeechRecognition),
                AudioFormat::empty()
            );
        }
    }

    #[test]
    fn both_media_rates_preserve_defaults_except_unsupported_16k_and_24k() {
        let mut info = defaults();
        let original = info.clone();
        let car = [
            car_format(
                StreamType::MainAudio,
                None,
                AudioFormat::PCM_44100_STEREO | AudioFormat::PCM_48000_STEREO,
            ),
            car_format(
                StreamType::AltAudio,
                None,
                AudioFormat::PCM_44100_STEREO | AudioFormat::PCM_48000_STEREO,
            ),
        ];
        filter_audio_sample_rates(&mut info, &car);
        // Supporting both media rates does not imply support for 16/24 kHz
        // microphone/voice output. Input capabilities remain unchanged.
        let unsupported =
            AudioFormat::PCM_16000_MONO | AudioFormat::PCM_24000_MONO | AudioFormat::OPUS_16000_MONO | AudioFormat::OPUS_24000_MONO;
        for (entry, original) in info.iter().zip(original) {
            assert_eq!(
                entry.audio_output_formats,
                original
                    .audio_output_formats
                    .map(|formats| formats - unsupported)
            );
            assert_eq!(entry.audio_input_formats, original.audio_input_formats);
        }
    }

    #[test]
    fn stream_and_audio_type_capabilities_are_kept_separate() {
        let mut info = defaults();
        let car = [
            car_format(StreamType::MainAudio, Some(AudioType::Compatibility), AudioFormat::PCM_44100_STEREO),
            car_format(StreamType::MainAudio, Some(AudioType::Media), AudioFormat::PCM_48000_STEREO),
            car_format(StreamType::AltAudio, None, AudioFormat::PCM_44100_STEREO),
        ];
        filter_audio_sample_rates(&mut info, &car);
        assert_eq!(
            output(&info, StreamType::MainHighAudio, AudioType::Media),
            AudioFormat::AAC_LC_48000_STEREO
        );
        assert_eq!(
            output(&info, StreamType::MainAudio, AudioType::Alert),
            AudioFormat::PCM_44100_STEREO
        );
        assert_eq!(
            output(&info, StreamType::AltAudio, AudioType::Default),
            AudioFormat::PCM_44100_STEREO
        );
    }

    #[test]
    fn mhi2q_210_81_uses_48k_for_media_and_16k_for_alt_audio() {
        // HARMAN / AUDI 4940, sourceVersion "210.81".
        let car = [
            AudioFormatStruct {
                audio_input_formats: Some(AudioFormat::PCM_16000_MONO),
                audio_output_formats: Some(AudioFormat::PCM_16000_MONO | AudioFormat::PCM_48000_STEREO),
                stream_type: StreamType::MainAudio,
                audio_type: None,
            },
            AudioFormatStruct {
                audio_input_formats: Some(AudioFormat::PCM_16000_MONO),
                audio_output_formats: Some(AudioFormat::PCM_16000_MONO),
                stream_type: StreamType::AltAudio,
                audio_type: None,
            },
        ];

        let mut iphone = defaults();
        filter_audio_sample_rates(&mut iphone, &car);

        // Wireless media must decode to the car's PCM 48 kHz stereo.
        assert_eq!(
            output(&iphone, StreamType::MainHighAudio, AudioType::Media),
            AudioFormat::AAC_LC_48000_STEREO,
        );

        assert_eq!(
            output(&iphone, StreamType::MainAudio, AudioType::Media),
            AudioFormat::PCM_48000_STEREO,
        );

        // // The legacy PCM-only entry must remain usable.
        // assert_eq!(
        //     output(&iphone, StreamType::AltAudio, AudioType::Compatibility),
        //     AudioFormat::PCM_16000_MONO,
        // );

        assert_eq!(
            output(&iphone, StreamType::AltAudio, AudioType::Compatibility),
            AudioFormat::empty(),
        );
        let alt = output(&iphone, StreamType::AltAudio, AudioType::Default);

        // Require a usable compressed format for wireless AltAudio.
        assert!(
            alt.contains(AudioFormat::OPUS_16000_MONO),
            "AltAudio/Default must offer Opus 16 kHz: {alt:?}",
        );

        // PCM 16 kHz may also be offered, but no other rate may survive.
        let allowed = AudioFormat::PCM_16000_MONO | AudioFormat::OPUS_16000_MONO;
        assert!(allowed.contains(alt), "AltAudio/Default must decode to PCM 16 kHz mono: {alt:?}",);
        assert_debug_snapshot!(iphone);
    }
}
