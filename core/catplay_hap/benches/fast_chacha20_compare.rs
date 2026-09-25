use catplay_hap::fast_chacha::FastChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const DATA_LEN: usize = 128 * 1024;

fn bench_fast_chacha20_compare(c: &mut Criterion) {
    let key = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x10, 0x21, 0x32, 0x43, 0x54, 0x65,
        0x76, 0x87, 0x98, 0xa9, 0xba, 0xcb, 0xdc, 0xed, 0xfe, 0x0f,
    ];
    let nonce = [0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x0f, 0xed, 0xcb, 0xa9];
    let plain = make_plaintext();

    let use_asm = catplay_hap::fast_chacha::is_asm_available_chacha20();
    let reference = encrypt_rustcrypto(&key, &nonce, &plain);
    let fast = encrypt_fast_best(&key, &nonce, &plain, use_asm);
    let fallback = encrypt_fast_fallback(&key, &nonce, &plain, 10);
    let chacha6 = encrypt_fast_fallback(&key, &nonce, &plain, 3);

    assert_eq!(fast, reference, "FastChaCha20 output differs from RustCrypto");
    assert_eq!(fallback, reference, "FastChaCha20 fallback output differs from RustCrypto");
    assert_ne!(chacha6, reference, "ChaCha6 unexpectedly matched ChaCha20 reference");

    let mut decrypt_check = fast.clone();
    let mut fast_cipher_dec = FastChaCha20::new(&key, &nonce);
    fast_cipher_dec.seek(64);
    if use_asm {
        fast_cipher_dec.apply_keystream(&mut decrypt_check);
    } else {
        fast_cipher_dec.apply_keystream_pure(&mut decrypt_check, 10);
    }
    assert_eq!(decrypt_check, plain, "FastChaCha20 failed to decrypt its own ciphertext");

    let mut group = c.benchmark_group("fast_chacha20_compare");
    group.throughput(Throughput::Bytes(DATA_LEN as u64));

    group.bench_function(BenchmarkId::new("rustcrypto", "chacha20"), |b| {
        b.iter(|| {
            let _ = encrypt_rustcrypto(&key, &nonce, &plain);
        });
    });

    group.bench_function(
        BenchmarkId::new("fast_chacha", if use_asm { "asm" } else { "best-fallback" }),
        |b| {
            b.iter(|| {
                let _ = encrypt_fast_best(&key, &nonce, &plain, use_asm);
            });
        },
    );

    group.bench_function(BenchmarkId::new("fast_chacha", "fallback"), |b| {
        b.iter(|| {
            let _ = encrypt_fast_fallback(&key, &nonce, &plain, 10);
        });
    });

    group.bench_function(BenchmarkId::new("fast_chacha", "chacha6"), |b| {
        b.iter(|| {
            let _ = encrypt_fast_fallback(&key, &nonce, &plain, 3);
        });
    });

    group.finish();
}

fn make_plaintext() -> Vec<u8> {
    (0..DATA_LEN)
        .map(|i| (i as u8).wrapping_mul(37).wrapping_add(11))
        .collect()
}

fn encrypt_rustcrypto(key: &[u8; 32], nonce: &[u8; 12], plain: &[u8]) -> Vec<u8> {
    let mut data = plain.to_vec();
    let mut cipher = chacha20::ChaCha20::new(key.into(), nonce.into());
    cipher.seek(64);
    cipher.apply_keystream(&mut data);
    data
}

fn encrypt_fast_best(key: &[u8; 32], nonce: &[u8; 12], plain: &[u8], use_asm: bool) -> Vec<u8> {
    let mut data = plain.to_vec();
    let mut cipher = FastChaCha20::new(key, nonce);
    cipher.seek(64);
    if use_asm {
        cipher.apply_keystream(&mut data);
    } else {
        cipher.apply_keystream_pure(&mut data, 10);
    }
    data
}

fn encrypt_fast_fallback(key: &[u8; 32], nonce: &[u8; 12], plain: &[u8], double_rounds: usize) -> Vec<u8> {
    let mut data = plain.to_vec();
    let mut cipher = FastChaCha20::new(key, nonce);
    cipher.seek(64);
    cipher.apply_keystream_pure(&mut data, double_rounds);
    data
}

criterion_group! {
    name = benches;
    config = Criterion::default().without_plots();
    targets = bench_fast_chacha20_compare
}
criterion_main!(benches);
