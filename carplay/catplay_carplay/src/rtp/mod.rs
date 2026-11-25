mod rtcp_session;
mod rtp_cipher;
mod rtp_cipher_aes;
mod rtp_cipher_chacha;
mod rtp_header;
mod rtp_packet;
mod rtp_receiver;

pub use rtcp_session::*;
pub use rtp_cipher::*;
pub use rtp_cipher_aes::*;
pub use rtp_cipher_chacha::*;
pub use rtp_header::*;
pub use rtp_packet::*;
pub use rtp_receiver::*;

#[derive(Clone, Copy)]
pub(super) enum AudioThreadDirection {
    Rx,
    Tx,
}

#[derive(Clone, Copy)]
pub(super) enum AudioCodecRole {
    Decode,
    Encode,
}

pub(super) fn audio_thread_name(stream_type: crate::msg::StreamType, direction: AudioThreadDirection, role: AudioCodecRole) -> String {
    let direction = match direction {
        AudioThreadDirection::Rx => "rx",
        AudioThreadDirection::Tx => "tx",
    };
    let role = match role {
        AudioCodecRole::Decode => "d",
        AudioCodecRole::Encode => "e",
    };
    format!("{stream_type:?}_{direction}{role}")
}

#[cfg(test)]
mod audio_thread_name_tests {
    use super::{AudioCodecRole, AudioThreadDirection, audio_thread_name};
    use crate::msg::StreamType;

    #[test]
    fn names_audio_threads_by_stream_direction_and_codec_role() {
        assert_eq!(
            audio_thread_name(StreamType::AltAudio, AudioThreadDirection::Rx, AudioCodecRole::Decode),
            "AltAudio_rxd"
        );
        assert_eq!(
            audio_thread_name(StreamType::AltAudio, AudioThreadDirection::Tx, AudioCodecRole::Encode),
            "AltAudio_txe"
        );
    }
}

pub mod play;
pub mod record;
