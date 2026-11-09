use std::time::Duration;

use catplay_carplay::{
    audio::{AudioPlayerBox, AudioRecorderBox, AudioStreamBasicDescription},
    carplay_tx::{AirPlayTransmitter, AirPlayTransmitterProxyRef},
    msg::{AudioFormat, AudioType, StreamType},
    rtsp_frame::{RtspError, RtspResult},
};
use log::{debug, error};

use crate::proxy::pcm_proxy::PcmProxyPlayer;

pub struct AudioProxyUtil;

impl AudioProxyUtil {
    pub async fn open_audio(
        car: AirPlayTransmitterProxyRef,
        _latency: Duration,
        mut stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,
    ) -> RtspResult<(AudioPlayerBox<i16>, Option<AudioRecorderBox<i16>>)> {
        debug!(
            "Starting stream {stream_type:?} with {audio_format:?}; mapped pcm is {:?}",
            AudioFormat::try_from(pcm_format)
        );
        let audio_format = AudioFormat::try_from(pcm_format).map_err(|_| RtspError::Unknown)?;

        // Open a proxy audio stream towards the car

        let audio_pair = PcmProxyPlayer::pair();
        let mut mic_player = None;
        let mut pending_mic = None;

        if duplex {
            // If audio type is Telephony or SpeechRecognition, open a duplex stream with a microphone too
            // in such case, `open_microphone` will follow `open_audio` (that order is a small implementation detail)
            // and we will have an instance of AudioRecorder to return as a microphone proxy
            let p = PcmProxyPlayer::pair();
            pending_mic.replace(Box::new(p.1) as _);
            mic_player.replace(Box::new(p.0) as _);
        }

        let player_guard = audio_pair.0.guard.clone();

        if stream_type == StreamType::MainHighAudio {
            stream_type = StreamType::MainAudio;
        }

        let mut pcm_latency = Duration::from_millis(32); // 75 too small
        if stream_type == StreamType::MainHighAudio {
            pcm_latency = Duration::from_millis(75);
        }

        let guard_fut = car.setup_audio(
            pcm_latency,
            stream_type,
            audio_format,
            audio_type,
            mic_player,
            Box::new(audio_pair.1),
        );

        let guard = guard_fut
            .await // blocks RTSP thread with a 2s timeout
            .inspect_err(|err| error!("Failed to setup audio transmitter? {err}"))?;
        player_guard.lock().unwrap().replace(guard);

        Ok((Box::new(audio_pair.0), pending_mic))
    }
}
