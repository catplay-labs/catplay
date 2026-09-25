use catplay_hap::aes::{Aes128CtrKernelStream, Aes128CtrOpenSsl, Aes128CtrSoft};

use std::hint::black_box;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const BUFFER_LEN: usize = 128 * 1024;
const EXT_BUFFER_LEN: usize = 256 * 1024;
const KEY: [u8; 16] = [0x42; 16];
const IV: [u8; 16] = [0x24; 16];

const TEST_KEY: [u8; 16] = [
    0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
];
const TEST_IV: [u8; 16] = [
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];
const TEST_PLAINTEXT: [u8; 64] = [
    0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03,
    0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19,
    0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
];
const TEST_CIPHERTEXT: [u8; 64] = [
    0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce, 0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70,
    0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff, 0x5a, 0xe4, 0xdf, 0x3e, 0xdb, 0xd5, 0xd3, 0x5e, 0x5b, 0x4f, 0x09, 0x02,
    0x0d, 0xb0, 0x3e, 0xab, 0x1e, 0x03, 0x1d, 0xda, 0x2f, 0xbe, 0x03, 0xd1, 0x79, 0x21, 0x70, 0xa0, 0xf3, 0x00, 0x9c, 0xee,
];

const STRESS_SIZES: &[usize] = &[
    1,
    2,
    3,
    7,
    15,
    16,
    17,
    31,
    32,
    33,
    63,
    64,
    65,
    111,
    112,
    113,
    127,
    128,
    255,
    256,
    511,
    512,
    1024,
    1536,
    4096,
    8192,
    12288,
    16384 - 16,
    16384,
    16384 + 16,
    16384 + 112,
    32768 + 112,
    65536 + 112,
];

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn print_vector_result(name: &str, output: &[u8]) {
    if output == TEST_CIPHERTEXT {
        eprintln!("aes_ctr/{name} test-vector: ok output={}", hex(output));
    } else {
        eprintln!(
            "aes_ctr/{name} test-vector: err expected={} output={}",
            hex(&TEST_CIPHERTEXT),
            hex(output)
        );
    }
}

fn warn_if_fake_encryption(name: &str, input: &[u8], output: &[u8]) {
    if output == input {
        eprintln!("aes_ctr/{name} ext-round-trip: warn fake-encryption output==input");
    }
}

fn stress_enabled() -> bool {
    true
    // matches!(std::env::var_os("CATPLAY_AES_CTR_STRESS"), Some(val) if val != "0")
}

fn stress_rounds() -> usize {
    std::env::var("CATPLAY_AES_CTR_STRESS_ROUNDS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(500)
}

fn stress_busy_threads() -> usize {
    std::env::var("CATPLAY_AES_CTR_STRESS_BUSY_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get().saturating_sub(1))
                .unwrap_or(1)
                .max(1)
        })
}

fn make_stress_buffer(len: usize, seed: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    for (i, byte) in out.iter_mut().enumerate() {
        let x = (i as u32)
            .wrapping_mul(0x45d9_f3b)
            .wrapping_add((seed as u32).wrapping_mul(0x9e37_79b9))
            ^ 0xa5a5_5a5a;
        *byte = (x as u8) ^ ((x >> 8) as u8) ^ ((x >> 16) as u8) ^ ((x >> 24) as u8);
    }
    out
}

fn spawn_cpu_burners(stop: Arc<AtomicBool>, count: usize) -> Vec<thread::JoinHandle<()>> {
    let mut handles = Vec::with_capacity(count);

    for worker_id in 0..count {
        let stop = Arc::clone(&stop);
        handles.push(thread::spawn(move || {
            let mut acc = (worker_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            while !stop.load(Ordering::Relaxed) {
                for _ in 0..4096 {
                    acc = acc.rotate_left(7).wrapping_add(0x9e37_79b9_7f4a_7c15);
                    acc ^= acc.wrapping_mul(0x94d0_49bb_1331_11eb);
                    black_box(acc);
                }
                thread::yield_now();
            }
            black_box(acc);
        }));
    }

    handles
}

fn run_kernel_ctr_stress() {
    if !stress_enabled() {
        return;
    }

    let rounds = stress_rounds();
    let burner_count = stress_busy_threads();
    eprintln!(
        "aes_ctr/kernel stress: enabled rounds={rounds} busy_threads={burner_count} sizes={}",
        STRESS_SIZES.len()
    );

    let stop = Arc::new(AtomicBool::new(false));
    let burners = spawn_cpu_burners(Arc::clone(&stop), burner_count);

    for round in 0..rounds {
        for &size in STRESS_SIZES {
            let mut plain = make_stress_buffer(size, round ^ size);
            let original = plain.clone();

            thread::yield_now();
            let mut cipher = Aes128CtrKernelStream::new(&TEST_KEY, &TEST_IV).expect("AF_ALG ctr(aes) init failed");
            cipher
                .apply_keystream(&mut plain)
                .expect("AF_ALG ctr(aes) apply failed");

            thread::yield_now();
            let mut cipher = Aes128CtrKernelStream::new(&TEST_KEY, &TEST_IV).expect("AF_ALG ctr(aes) init failed");
            cipher
                .apply_keystream(&mut plain)
                .expect("AF_ALG ctr(aes) apply failed");

            if plain != original {
                eprintln!(
                    "aes_ctr/kernel stress: err round={round} size={size} original={} output={}",
                    hex(&original),
                    hex(&plain)
                );
                panic!("kernel ctr stress round-trip mismatch");
            }

            thread::yield_now();
        }
    }

    stop.store(true, Ordering::Relaxed);
    for handle in burners {
        let _ = handle.join();
    }

    eprintln!("aes_ctr/kernel stress: OK");
}

fn verify_test_vectors() {
    let mut output = TEST_PLAINTEXT;
    let mut cipher = Aes128CtrOpenSsl::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    print_vector_result("openssl", &output);

    let mut output = TEST_PLAINTEXT;
    let mut cipher = Aes128CtrSoft::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    print_vector_result("soft", &output);

    let mut output = TEST_PLAINTEXT;
    match Aes128CtrKernelStream::new(&TEST_KEY, &TEST_IV) {
        Ok(mut cipher) => match cipher.apply_keystream(&mut output) {
            Ok(()) => print_vector_result("kernel", &output),
            Err(err) => eprintln!("aes_ctr/kernel test-vector: err apply={err:?} output={}", hex(&output)),
        },
        Err(err) => eprintln!("aes_ctr/kernel test-vector: err init={err:?}"),
    }
}

fn verify_test_vectors_ext() {
    let expected = vec![0x11u8; EXT_BUFFER_LEN];

    let mut output = expected.clone();
    let mut cipher = Aes128CtrOpenSsl::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    warn_if_fake_encryption("openssl", &expected, &output);
    let mut cipher = Aes128CtrOpenSsl::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    if output == expected {
        eprintln!("aes_ctr/openssl ext-round-trip: OK");
    } else {
        eprintln!(
            "aes_ctr/openssl ext-round-trip: err expected={} output={}",
            hex(&expected),
            hex(&output)
        );
    }

    let mut output = expected.clone();
    let mut cipher = Aes128CtrSoft::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    warn_if_fake_encryption("soft", &expected, &output);
    let mut cipher = Aes128CtrSoft::new(&TEST_KEY, &TEST_IV);
    cipher.apply_keystream(&mut output);
    if output == expected {
        eprintln!("aes_ctr/soft ext-round-trip: OK");
    } else {
        eprintln!(
            "aes_ctr/soft ext-round-trip: err expected={} output={}",
            hex(&expected),
            hex(&output)
        );
    }

    let mut output = expected.clone();
    let mut cipher = Aes128CtrKernelStream::new(&TEST_KEY, &TEST_IV).expect("AF_ALG ctr(aes) init failed");
    cipher
        .apply_keystream(&mut output)
        .expect("AF_ALG ctr(aes) apply failed");
    warn_if_fake_encryption("kernel", &expected, &output);
    let mut cipher = Aes128CtrKernelStream::new(&TEST_KEY, &TEST_IV).expect("AF_ALG ctr(aes) init failed");
    cipher
        .apply_keystream(&mut output)
        .expect("AF_ALG ctr(aes) apply failed");
    if output == expected {
        eprintln!("aes_ctr/kernel ext-round-trip: OK");
    } else {
        eprintln!(
            "aes_ctr/kernel ext-round-trip: err expected={} output={}",
            hex(&expected),
            hex(&output)
        );
    }
}

fn aes_ctr_throughput(c: &mut Criterion) {
    verify_test_vectors();
    for _ in 0..10 {
        verify_test_vectors_ext();
    }
    run_kernel_ctr_stress();

    let mut group = c.benchmark_group("aes_ctr");
    group.throughput(Throughput::Bytes(BUFFER_LEN as u64));

    group.bench_function("openssl", |b| {
        let mut cipher = Aes128CtrOpenSsl::new(&KEY, &IV);
        let mut data = vec![0x11u8; BUFFER_LEN];

        b.iter(|| {
            cipher.apply_keystream(black_box(data.as_mut_slice()));
        });
    });

    group.bench_function("soft", |b| {
        let mut cipher = Aes128CtrSoft::new(&KEY, &IV);
        let mut data = vec![0x11u8; BUFFER_LEN];

        b.iter(|| {
            cipher.apply_keystream(black_box(data.as_mut_slice()));
        });
    });

    group.bench_function("kernel", |b| {
        let mut cipher = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");
        let mut data = vec![0x11u8; BUFFER_LEN];

        b.iter(|| {
            cipher
                .apply_keystream(black_box(data.as_mut_slice()))
                .expect("AF_ALG ctr(aes) apply failed");
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().without_plots();
    targets = aes_ctr_throughput
}
criterion_main!(benches);
