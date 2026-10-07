//! Frame latency benchmark for the optional AES-CTR prefetch layer.
//!
//! Run with `cargo bench -p catplay_hap --bench aes_ctr_prefetch`.
//! Set CATPLAY_AES_CTR_PREFETCH_BACKEND=soft|openssl|kernel to select the engine.
//! Frame/capacity/count can be overridden with CATPLAY_AES_CTR_FRAME_BYTES,
//! CATPLAY_AES_CTR_CACHE_BYTES and CATPLAY_AES_CTR_FRAMES.

#[cfg(feature = "openssl")]
use catplay_hap::aes::Aes128CtrOpenSsl;
use catplay_hap::aes::{Aes128CtrEngine, Aes128CtrKernelStream, Aes128CtrPrefetch, Aes128CtrSoft, DEFAULT_PREFETCH_CAPACITY};
use std::fmt::Debug;
use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

const KEY: [u8; 16] = [0x51; 16];
const IV: [u8; 16] = [0x62; 16];
const DEFAULT_FRAME_BYTES: usize = 1024 * 1024;
const DEFAULT_FRAMES: usize = 40;

struct Outcome {
    latencies: Vec<Duration>,
    cached_bytes: usize,
    frame_bytes: usize,
    last_frame: Vec<u8>,
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&value| value > 0)
        .unwrap_or(default)
}

fn run<B: Aes128CtrEngine>(enabled: bool, cadence: Duration, frame_bytes: usize, capacity: usize, frames: usize) -> Outcome
where
    B::Error: Debug,
{
    let mut prefetch = enabled.then(|| Aes128CtrPrefetch::<B>::new(KEY, IV, capacity).expect("spawn prefetch worker"));
    let mut direct = (!enabled).then(|| B::new(&KEY, &IV).expect("initialize CTR engine"));
    let mut latencies = Vec::with_capacity(frames);
    let mut cached_bytes = 0;
    let mut frame = vec![0u8; frame_bytes];
    let mut next_frame = Instant::now() + cadence;

    for _ in 0..frames {
        let now = Instant::now();
        if now < next_frame {
            thread::sleep(next_frame - now);
        }
        frame.fill(0x5a);
        let start = Instant::now();
        if let Some(prefetch) = prefetch.as_mut() {
            let result = prefetch
                .apply_keystream(black_box(&mut frame))
                .expect("prefetched CTR frame");
            cached_bytes += result.cached_bytes;
        } else {
            direct
                .as_mut()
                .unwrap()
                .apply_keystream(black_box(&mut frame))
                .expect("direct CTR frame");
        }
        latencies.push(start.elapsed());
        next_frame += cadence;
    }

    // Dropping the worker is outside the frame latency measurement.
    drop(prefetch);
    Outcome {
        latencies,
        cached_bytes,
        frame_bytes,
        last_frame: frame,
    }
}

fn print_result(prefetch: bool, prefer_keystream_generation: bool, cadence_ms: u64, outcome: &Outcome) {
    let mut samples: Vec<f64> = outcome
        .latencies
        .iter()
        .map(|time| time.as_secs_f64() * 1000.0)
        .collect();
    samples.sort_by(f64::total_cmp);
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let median = samples[samples.len() / 2];
    let p95 = samples[((samples.len() * 95).div_ceil(100)).saturating_sub(1)];
    let hit = 100.0 * outcome.cached_bytes as f64 / (outcome.frame_bytes * samples.len()) as f64;
    println!(
        "prefetch={} prefer_keystream_generation={} {cadence_ms:>2} ms  mean={mean:>8.3} ms  median={median:>8.3} ms  p95={p95:>8.3} ms  cache_hit={hit:>5.1}%",
        if prefetch { "on" } else { "off" },
        if prefer_keystream_generation { "on" } else { "off" },
    );
}

fn benchmark<B: Aes128CtrEngine>(name: &str, frame_bytes: usize, capacity: usize, frames: usize, set_preference: fn(bool))
where
    B::Error: Debug,
{
    println!(
        "backend={name} frame={} KiB cache={} KiB frames={frames}",
        frame_bytes / 1024,
        capacity / 1024
    );
    if let Ok(value) = std::env::var("CATPLAY_AES_CTR_PREFETCH_PROFILE_CADENCE_MS") {
        let cadence_ms = value
            .parse::<u64>()
            .expect("profile cadence must be an integer");
        assert!(matches!(cadence_ms, 16 | 33), "profile cadence must be 16 or 33 ms");
        set_preference(true);
        println!("profile_path=prefetch-consumer keystream_only=on cadence={cadence_ms} ms");
        let outcome = run::<B>(true, Duration::from_millis(cadence_ms), frame_bytes, capacity, frames);
        print_result(true, true, cadence_ms, &outcome);
        return;
    }

    for cadence_ms in [16, 33] {
        let cadence = Duration::from_millis(cadence_ms);
        // Reverse preference order for the second cadence to reduce order bias.
        let preferences = if cadence_ms == 16 { [false, true] } else { [true, false] };
        let mut reference_output = None;
        for prefer_keystream_generation in preferences {
            set_preference(prefer_keystream_generation);
            let direct = run::<B>(false, cadence, frame_bytes, capacity, frames);
            let prefetched = run::<B>(true, cadence, frame_bytes, capacity, frames);
            assert_eq!(
                direct.last_frame, prefetched.last_frame,
                "direct/prefetch CTR output differs at {cadence_ms} ms cadence with keystream generation preference={prefer_keystream_generation}"
            );
            if let Some(reference) = reference_output.as_ref() {
                assert_eq!(
                    &direct.last_frame, reference,
                    "CTR output differs when toggling keystream generation preference at {cadence_ms} ms cadence"
                );
            } else {
                reference_output = Some(direct.last_frame.clone());
            }
            print_result(false, prefer_keystream_generation, cadence_ms, &direct);
            print_result(true, prefer_keystream_generation, cadence_ms, &prefetched);
        }
    }
}

fn ignore_preference(_: bool) {}

fn main() {
    let frame_bytes = env_usize("CATPLAY_AES_CTR_FRAME_BYTES", DEFAULT_FRAME_BYTES);
    let capacity = env_usize("CATPLAY_AES_CTR_CACHE_BYTES", DEFAULT_PREFETCH_CAPACITY);
    let frames = env_usize("CATPLAY_AES_CTR_FRAMES", DEFAULT_FRAMES);
    #[cfg(feature = "openssl")]
    let default_backend = "openssl";
    #[cfg(not(feature = "openssl"))]
    let default_backend = "soft";
    let backend = std::env::var("CATPLAY_AES_CTR_PREFETCH_BACKEND").unwrap_or_else(|_| default_backend.into());

    match backend.as_str() {
        "soft" => benchmark::<Aes128CtrSoft>("soft", frame_bytes, capacity, frames, ignore_preference),
        #[cfg(feature = "openssl")]
        "openssl" => benchmark::<Aes128CtrOpenSsl>("openssl", frame_bytes, capacity, frames, ignore_preference),
        "kernel" => benchmark::<Aes128CtrKernelStream>(
            "kernel",
            frame_bytes,
            capacity,
            frames,
            Aes128CtrKernelStream::set_prefer_keystream_generation,
        ),
        _ => panic!("unknown/unavailable backend: {backend}"),
    }
}
