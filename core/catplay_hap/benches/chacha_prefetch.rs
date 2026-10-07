//! RX latency and total process CPU, with completed waste separate from in-flight work.
//! `cargo bench -p catplay_hap --bench chacha_prefetch`
use catplay_hap::cipher::{ChaChaPrefetchStats, HomeKitChaChaNonce, HomeKitCipherFast, HomeKitCipherRing};
use std::{
    hint::black_box,
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
enum Mode {
    Off,
    Ready,
    Partial,
    Immediate,
    Estimated(u64),
}

#[cfg(target_os = "linux")]
fn cpu_seconds() -> f64 {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    assert_eq!(unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut time) }, 0);
    time.tv_sec as f64 + time.tv_nsec as f64 * 1e-9
}
#[cfg(not(target_os = "linux"))]
fn cpu_seconds() -> f64 {
    f64::NAN
}

fn percentile(times: &mut [Duration], percent: usize) -> f64 {
    times.sort_unstable();
    times[(times.len() * percent).div_ceil(100).saturating_sub(1)].as_secs_f64() * 1000.0
}

fn run(mode: Mode, skip_poly: bool, frames: &[(Vec<u8>, Vec<u8>)]) {
    let mut rx = HomeKitCipherFast::new([0x53; 32]);
    if skip_poly {
        rx.set_prefer_poly_skip();
    }
    let mut timings = Vec::new();
    let mut resets = Vec::new();
    let mut totals = ChaChaPrefetchStats::default();
    let mut collect = |s: Option<ChaChaPrefetchStats>| {
        if let Some(s) = s {
            totals.generated_bytes += s.generated_bytes;
            totals.used_bytes += s.used_bytes;
            totals.dropped_bytes += s.dropped_bytes;
            totals.synchronous_bytes += s.synchronous_bytes;
            totals.discarded_in_flight_bytes += s.discarded_in_flight_bytes;
        }
    };
    let cpu_start = cpu_seconds();
    let mut due = Instant::now();
    for (i, (plain, wire)) in frames.iter().enumerate() {
        let target = match mode {
            Mode::Off => 0,
            Mode::Ready | Mode::Immediate => plain.len(),
            Mode::Partial => plain.len() / 2,
            Mode::Estimated(_) => {
                if i == 0 {
                    0
                } else {
                    frames[i - 1].0.len() * 3 / 2
                }
            }
        };
        let start = Instant::now();
        collect(rx.reset_prefetch(HomeKitChaChaNonce(i as u64), target));
        resets.push(start.elapsed());
        match mode {
            Mode::Ready | Mode::Partial => {
                let deadline = Instant::now() + Duration::from_secs(5);
                while rx.prefetch_status().unwrap().available_bytes != target {
                    assert!(Instant::now() < deadline, "prefetch timeout");
                    thread::sleep(Duration::from_millis(1));
                }
            }
            Mode::Estimated(ms) => {
                due += Duration::from_millis(ms);
                if let Some(left) = due.checked_duration_since(Instant::now()) {
                    thread::sleep(left);
                }
            }
            _ => {}
        }
        let mut data = wire.clone();
        let start = Instant::now();
        let payload = rx
            .decrypt(black_box(&mut data), b"screen", HomeKitChaChaNonce(i as u64))
            .unwrap();
        timings.push(start.elapsed());
        assert_eq!(payload, plain);
        let used = rx.prefetch_status().unwrap().used_bytes;
        match mode {
            Mode::Ready => assert_eq!(used, plain.len()),
            Mode::Partial => assert_eq!(used, target),
            Mode::Off => assert_eq!(used, 0),
            _ => {} // Immediate/estimated report observed hits, never assume a miss.
        }
    }
    collect(rx.reset_prefetch(HomeKitChaChaNonce(frames.len() as u64), 0));
    drop(rx); // Include outstanding worker CPU in the process total.
    let cpu = cpu_seconds() - cpu_start;
    println!(
        "{mode:?} poly={} p50={:.3}ms p95={:.3}ms p99={:.3}ms reset_p95={:.3}ms cpu={cpu:.4}s hit={:.1}% generated={} dropped={} in_flight={}",
        !skip_poly,
        percentile(&mut timings, 50),
        percentile(&mut timings, 95),
        percentile(&mut timings, 99),
        percentile(&mut resets, 95),
        100.0 * totals.used_bytes as f64 / (totals.used_bytes + totals.synchronous_bytes).max(1) as f64,
        totals.generated_bytes,
        totals.dropped_bytes,
        totals.discarded_in_flight_bytes,
    );
}

fn main() {
    let mut tx = HomeKitCipherRing::new([0x53; 32]);
    let frames: Vec<_> = (0..40)
        .map(|i| {
            let len = [64 * 1024, 128 * 1024 + 1, 192 * 1024, 32 * 1024 + 7][i % 4];
            let plain: Vec<_> = (0..len).map(|j| (j * 31) as u8).collect();
            let mut wire = plain.clone();
            let tag = tx
                .encrypt(&mut wire, b"screen", HomeKitChaChaNonce(i as u64))
                .unwrap();
            wire.extend_from_slice(&tag);
            (plain, wire)
        })
        .collect();
    for skip_poly in [false, true] {
        for mode in [
            Mode::Off,
            Mode::Ready,
            Mode::Partial,
            Mode::Immediate,
            Mode::Estimated(16),
            Mode::Estimated(33),
        ] {
            run(mode, skip_poly, &frames);
        }
    }
}
