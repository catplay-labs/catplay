use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use catplay_audio::{
    AudioStreamBasicDescription,
    codec::{
        AudioDecoder, AudioDecoderFactory, AudioEncoder, AudioEncoderFactory,
        aac::{AacDecoder, AacEncoder},
        opus::{OpusDecoder, OpusEncoder},
    },
};

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
const SAMPLES_PER_SAMPLE: usize = 5;
const SAMPLE_TIME: Duration = Duration::from_millis(200);

fn measure(mut operation: impl FnMut()) -> Vec<f64> {
    for _ in 0..100 {
        operation();
    }

    (0..SAMPLES_PER_SAMPLE)
        .map(|_| {
            let start = Instant::now();
            let mut iterations = 0u64;
            while start.elapsed() < SAMPLE_TIME {
                operation();
                iterations += 1;
            }
            start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64
        })
        .collect()
}

fn report(name: &str, frame_ms: f64, samples: &[f64]) {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let median_us = sorted[sorted.len() / 2];
    let mean_us = samples.iter().sum::<f64>() / samples.len() as f64;
    let throughput = 1_000_000.0 / median_us;
    println!(
        "codec_bench name={name} mean_us={mean_us:.2} median_us={median_us:.2} p95_us={:.2} frames_per_sec={throughput:.1} realtime_x={:.2}",
        sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)],
        throughput * frame_ms / 1000.0,
    );
}

fn signal(samples: usize) -> Vec<i16> {
    (0..samples)
        .map(|index| {
            let phase = (index % 480) as i32;
            ((phase - 240) * 96) as i16
        })
        .collect()
}

fn opus_packet() -> Vec<u8> {
    let input_asbd = AudioStreamBasicDescription::fill_pcm(SAMPLE_RATE, 16, 16, CHANNELS as u8, false);
    let output_asbd = AudioStreamBasicDescription::fill_opus(SAMPLE_RATE, CHANNELS);
    let mut encoder = OpusEncoder::new(input_asbd, output_asbd).expect("create Opus encoder");
    let pcm = signal(output_asbd.frames_per_packet as usize * CHANNELS as usize);
    let mut packet = vec![0u8; 4_000];
    let (written, consumed) = encoder
        .encode(&pcm, &mut packet)
        .expect("encode Opus fixture");
    assert_eq!(consumed, pcm.len(), "Opus fixture did not consume one complete frame");
    assert!(written > 0, "Opus fixture encoder emitted an empty packet");
    packet.truncate(written);
    packet
}

fn aac_packets(eld: bool) -> Vec<Vec<u8>> {
    let input_asbd = AudioStreamBasicDescription::fill_pcm(SAMPLE_RATE, 16, 16, CHANNELS as u8, false);
    let output_asbd = if eld {
        AudioStreamBasicDescription::fill_aac_eld(SAMPLE_RATE, CHANNELS)
    } else {
        AudioStreamBasicDescription::fill_aac_lc(SAMPLE_RATE, CHANNELS)
    };
    let mut encoder = AacEncoder::new(input_asbd, output_asbd)
        .unwrap_or_else(|error| panic!("create CatPlay AAC {} fixture encoder: {error}", if eld { "ELD" } else { "LC" }));
    let frame_samples = encoder.output_type().frames_per_packet as usize;
    let pcm = signal(frame_samples * CHANNELS as usize);
    let mut packet = vec![0u8; 8_192];

    let mut packets = Vec::with_capacity(512);
    for _ in 0..520 {
        let (written, consumed) = encoder
            .encode(&pcm, &mut packet)
            .expect("encode AAC fixture");
        assert_eq!(consumed, pcm.len(), "AAC fixture encoder did not consume a complete frame");
        if written > 0 {
            packets.push(packet[..written].to_vec());
            if packets.len() == 512 {
                return packets;
            }
        }
    }
    panic!(
        "FDK AAC {} fixture encoder emitted only {} packets",
        if eld { "ELD" } else { "LC" },
        packets.len()
    );
}

fn bench_opus_encode() {
    let input_asbd = AudioStreamBasicDescription::fill_pcm(SAMPLE_RATE, 16, 16, CHANNELS as u8, false);
    let output_asbd = AudioStreamBasicDescription::fill_opus(SAMPLE_RATE, CHANNELS);
    let mut encoder = OpusEncoder::new(input_asbd, output_asbd).expect("create Opus encoder");
    let pcm = signal(output_asbd.frames_per_packet as usize * CHANNELS as usize);
    let mut packet = vec![0u8; 4_000];
    let samples = measure(|| {
        let (written, consumed) = encoder
            .encode(black_box(&pcm), black_box(&mut packet))
            .expect("Opus encode");
        assert!(written > 0 && consumed == pcm.len());
        black_box(written);
    });
    report("opus_encode_48k_stereo_20ms", 20.0, &samples);
}

fn bench_opus_decode(packet: &[u8]) {
    let asbd = AudioStreamBasicDescription::fill_opus(SAMPLE_RATE, CHANNELS);
    let mut decoder = OpusDecoder::new(asbd).expect("create Opus decoder");
    let mut pcm = vec![0i16; asbd.frames_per_packet as usize * CHANNELS as usize];
    let samples = measure(|| {
        let (written, consumed) = decoder
            .decode(black_box(packet), black_box(&mut pcm))
            .expect("Opus decode");
        assert!(written > 0 && consumed == packet.len());
        black_box(pcm[written - 1]);
    });
    report("opus_decode_48k_stereo_20ms", 20.0, &samples);
}

fn bench_aac_decode(eld: bool, packets: &[Vec<u8>]) {
    let asbd = if eld {
        AudioStreamBasicDescription::fill_aac_eld(SAMPLE_RATE, CHANNELS)
    } else {
        AudioStreamBasicDescription::fill_aac_lc(SAMPLE_RATE, CHANNELS)
    };
    let mut decoder =
        AacDecoder::new(asbd).unwrap_or_else(|error| panic!("create FDK AAC {} decoder: {error}", if eld { "ELD" } else { "LC" }));
    let mut pcm = vec![0i16; 4_096];
    let mut packet_index = 0usize;
    let samples = measure(|| {
        if packet_index == packets.len() {
            decoder = AacDecoder::new(asbd)
                .unwrap_or_else(|error| panic!("recreate FDK AAC {} decoder: {error}", if eld { "ELD" } else { "LC" }));
            packet_index = 0;
        }
        let packet = &packets[packet_index];
        let (written, consumed) = decoder
            .decode(black_box(packet), black_box(&mut pcm))
            .unwrap_or_else(|error| panic!("FDK AAC {} decode packet {packet_index}: {error:?}", if eld { "ELD" } else { "LC" }));
        assert!(written > 0 && consumed > 0);
        black_box(pcm[written - 1]);
        packet_index += 1;
    });
    let frame_ms = if eld { 10.0 } else { 1024.0 * 1000.0 / SAMPLE_RATE as f64 };
    report(
        if eld {
            "fdk_aac_eld_decode_48k_stereo"
        } else {
            "fdk_aac_lc_decode_48k_stereo"
        },
        frame_ms,
        &samples,
    );
}

fn bench_aac_encode(eld: bool) {
    let input_asbd = AudioStreamBasicDescription::fill_pcm(SAMPLE_RATE, 16, 16, CHANNELS as u8, false);
    let output_asbd = if eld {
        AudioStreamBasicDescription::fill_aac_eld(SAMPLE_RATE, CHANNELS)
    } else {
        AudioStreamBasicDescription::fill_aac_lc(SAMPLE_RATE, CHANNELS)
    };
    let mut encoder = AacEncoder::new(input_asbd, output_asbd)
        .unwrap_or_else(|error| panic!("create FDK AAC {} encoder: {error}", if eld { "ELD" } else { "LC" }));
    let frame_samples = encoder.output_type().frames_per_packet as usize;
    let pcm = signal(frame_samples * CHANNELS as usize);
    let mut packet = vec![0u8; 8_192];
    let samples = measure(|| {
        let (written, consumed) = encoder
            .encode(black_box(&pcm), black_box(&mut packet))
            .unwrap_or_else(|error| panic!("FDK AAC {} encode: {error}", if eld { "ELD" } else { "LC" }));
        assert_eq!(consumed, pcm.len(), "FDK AAC encoder did not consume one complete frame");
        black_box(written);
    });
    let frame_ms = frame_samples as f64 * 1000.0 / SAMPLE_RATE as f64;
    report(
        if eld {
            "fdk_aac_eld_encode_48k_stereo"
        } else {
            "fdk_aac_lc_encode_48k_stereo"
        },
        frame_ms,
        &samples,
    );
}

fn main() {
    println!(
        "audio_codec_bench sample_count={SAMPLES_PER_SAMPLE} sample_time_ms={}",
        SAMPLE_TIME.as_millis()
    );

    let opus = opus_packet();
    bench_opus_encode();
    bench_opus_decode(&opus);

    let aac_lc = aac_packets(false);
    bench_aac_encode(false);
    bench_aac_decode(false, &aac_lc);

    let aac_eld = aac_packets(true);
    bench_aac_encode(true);
    bench_aac_decode(true, &aac_eld);
}
