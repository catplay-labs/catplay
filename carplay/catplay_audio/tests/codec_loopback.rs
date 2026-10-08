use catplay_audio::{
    AudioCodec,
    codec::runtime::{AudioDecoderDispatch, AudioEncoderDispatch},
};
#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
use catplay_audio::{
    AudioStreamBasicDescription,
    codec::{AudioDecoder, AudioDecoderFactory, AudioEncoder, AudioEncoderFactory},
};

#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
const SAMPLE_RATE: u32 = 48_000;
#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
const CHANNELS: u32 = 2;
#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
const FRAMES: usize = 32;

#[test]
fn runtime_support_matches_features() {
    assert!(AudioEncoderDispatch::has_runtime_support(AudioCodec::LinearPcm));
    assert!(AudioDecoderDispatch::has_runtime_support(AudioCodec::LinearPcm));
    assert_eq!(
        AudioEncoderDispatch::has_runtime_support(AudioCodec::Mpeg4Aac),
        cfg!(feature = "aac_enc")
    );
    assert_eq!(
        AudioDecoderDispatch::has_runtime_support(AudioCodec::Mpeg4Aac),
        cfg!(feature = "aac_dec")
    );
    assert_eq!(
        AudioEncoderDispatch::has_runtime_support(AudioCodec::Opus),
        cfg!(feature = "opus_enc")
    );
    assert_eq!(
        AudioDecoderDispatch::has_runtime_support(AudioCodec::Opus),
        cfg!(feature = "opus_dec")
    );
    assert_eq!(
        AudioDecoderDispatch::has_runtime_support(AudioCodec::AppleLossless),
        cfg!(feature = "alac")
    );
}

#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
fn pcm_frame(frames_per_channel: usize) -> Vec<i16> {
    (0..frames_per_channel * CHANNELS as usize)
        .map(|index| {
            let phase = (index % 480) as i32;
            ((phase - 240) * 96) as i16
        })
        .collect()
}

#[cfg(any(all(feature = "aac_enc", feature = "aac_dec"), all(feature = "opus_enc", feature = "opus_dec")))]
fn round_trip(codec: AudioCodec, encoded: AudioStreamBasicDescription) {
    let input = AudioStreamBasicDescription::fill_pcm(SAMPLE_RATE, 16, 16, CHANNELS as u8, false);
    let mut encoder = AudioEncoderDispatch::new(input, encoded).expect("create encoder");
    let output = encoder.output_type();
    let pcm = pcm_frame(output.frames_per_packet as usize);
    let mut packets = Vec::new();
    let mut packet_buffer = vec![0u8; 16 * 1024];

    for _ in 0..FRAMES {
        let (written, consumed) = encoder
            .encode(&pcm, &mut packet_buffer)
            .expect("encode frame");
        assert_eq!(consumed, pcm.len(), "encoder did not consume the complete PCM frame");
        if written > 0 {
            packets.push(packet_buffer[..written].to_vec());
        }
    }
    assert!(!packets.is_empty(), "{codec:?} encoder produced no packets");

    let mut decoder = AudioDecoderDispatch::new(output).expect("create decoder");
    let mut decoded = vec![0i16; output.frames_per_packet as usize * CHANNELS as usize * 4];
    let mut decoded_frames = 0;
    for packet in &packets {
        let (written, consumed) = decoder.decode(packet, &mut decoded).expect("decode packet");
        assert_eq!(consumed, packet.len(), "{codec:?} decoder did not consume a complete packet");
        assert!(written > 0, "{codec:?} decoder produced no PCM samples");
        decoded_frames += 1;
    }
    assert!(decoded_frames > 0);
}

#[cfg(all(feature = "aac_enc", feature = "aac_dec"))]
#[test]
fn aac_lc_encode_decode_loopback() {
    round_trip(
        AudioCodec::Mpeg4Aac,
        AudioStreamBasicDescription::fill_aac_lc(SAMPLE_RATE, CHANNELS),
    );
}

#[cfg(all(feature = "aac_enc", feature = "aac_dec"))]
#[test]
fn aac_eld_encode_decode_loopback() {
    round_trip(
        AudioCodec::Mpeg4AacEld,
        AudioStreamBasicDescription::fill_aac_eld(SAMPLE_RATE, CHANNELS),
    );
}

#[cfg(all(feature = "opus_enc", feature = "opus_dec"))]
#[test]
fn opus_encode_decode_loopback() {
    round_trip(AudioCodec::Opus, AudioStreamBasicDescription::fill_opus(SAMPLE_RATE, CHANNELS));
}
