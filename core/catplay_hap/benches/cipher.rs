use catplay_hap::cipher::{HomeKitChaChaNonce, HomeKitCipherFast, HomeKitCipherRing};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const ALIGNMENT: usize = 64;
const PAYLOAD_LEN: usize = 128 * 1024;
const FRAME_LEN: usize = PAYLOAD_LEN + 16;

fn make_aligned_storage(len: usize) -> (Vec<u8>, usize) {
    let storage = vec![44u8; len + ALIGNMENT];
    let start = storage.as_ptr().align_offset(ALIGNMENT);
    assert!(start != usize::MAX);
    assert!(start + len <= storage.len());
    assert_eq!((storage[start..].as_ptr() as usize) % ALIGNMENT, 0);
    (storage, start)
}

fn make_unaligned_storage(len: usize) -> (Vec<u8>, usize) {
    let (storage, aligned_start) = make_aligned_storage(len + 1);
    let start = aligned_start + 1;
    assert!(start + len <= storage.len());
    assert_ne!((storage[start..].as_ptr() as usize) % ALIGNMENT, 0);
    (storage, start)
}

fn cipher_encrypt_decrypt_fast(c: &mut Criterion) {
    let mut group = c.benchmark_group("cipher");
    group.throughput(Throughput::Bytes(PAYLOAD_LEN as u64));

    group.bench_function("decrypt_fast_aligned64", |b| {
        let mut cipher = HomeKitCipherFast::new([42u8; 32]);
        let (mut frame, frame_start) = make_aligned_storage(FRAME_LEN);
        let frame_slice = &mut frame[frame_start..frame_start + FRAME_LEN];
        let tag = cipher
            .encrypt(&mut frame_slice[..PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
            .unwrap();
        frame_slice[PAYLOAD_LEN..].copy_from_slice(&tag);

        let frame = frame.clone();
        b.iter(|| {
            let mut frame = frame.clone();
            cipher
                .decrypt(&mut frame[frame_start..frame_start + FRAME_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("decrypt_fast_unaligned", |b| {
        let mut cipher = HomeKitCipherFast::new([42u8; 32]);
        let (mut frame, frame_start) = make_unaligned_storage(FRAME_LEN);
        let frame_slice = &mut frame[frame_start..frame_start + FRAME_LEN];
        let tag = cipher
            .encrypt(&mut frame_slice[..PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
            .unwrap();
        frame_slice[PAYLOAD_LEN..].copy_from_slice(&tag);

        let frame = frame.clone();
        b.iter(|| {
            let mut frame = frame.clone();
            cipher
                .decrypt(&mut frame[frame_start..frame_start + FRAME_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("decrypt_ring_aligned64", |b| {
        let mut cipher = HomeKitCipherRing::new([42u8; 32]);
        let (mut frame, frame_start) = make_aligned_storage(FRAME_LEN);
        let frame_slice = &mut frame[frame_start..frame_start + FRAME_LEN];
        let tag = cipher
            .encrypt(&mut frame_slice[..PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
            .unwrap();
        frame_slice[PAYLOAD_LEN..].copy_from_slice(&tag);

        let frame = frame.clone();
        b.iter(|| {
            let mut frame = frame.clone();
            cipher
                .decrypt(&mut frame[frame_start..frame_start + FRAME_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("decrypt_ring_unaligned", |b| {
        let mut cipher = HomeKitCipherRing::new([42u8; 32]);
        let (mut frame, frame_start) = make_unaligned_storage(FRAME_LEN);
        let frame_slice = &mut frame[frame_start..frame_start + FRAME_LEN];
        let tag = cipher
            .encrypt(&mut frame_slice[..PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
            .unwrap();
        frame_slice[PAYLOAD_LEN..].copy_from_slice(&tag);

        let frame = frame.clone();
        b.iter(|| {
            let mut frame = frame.clone();
            cipher
                .decrypt(&mut frame[frame_start..frame_start + FRAME_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("encrypt_fast_aligned64", |b| {
        let mut cipher = HomeKitCipherFast::new([42u8; 32]);
        let (mut frame, frame_start) = make_aligned_storage(PAYLOAD_LEN);

        b.iter(|| {
            cipher
                .encrypt(&mut frame[frame_start..frame_start + PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("encrypt_fast_unaligned", |b| {
        let mut cipher = HomeKitCipherFast::new([42u8; 32]);
        let (mut frame, frame_start) = make_unaligned_storage(PAYLOAD_LEN);

        b.iter(|| {
            cipher
                .encrypt(&mut frame[frame_start..frame_start + PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("encrypt_ring_aligned64", |b| {
        let mut cipher = HomeKitCipherRing::new([42u8; 32]);
        let (mut frame, frame_start) = make_aligned_storage(PAYLOAD_LEN);

        b.iter(|| {
            cipher
                .encrypt(&mut frame[frame_start..frame_start + PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.bench_function("encrypt_ring_unaligned", |b| {
        let mut cipher = HomeKitCipherRing::new([42u8; 32]);
        let (mut frame, frame_start) = make_unaligned_storage(PAYLOAD_LEN);

        b.iter(|| {
            cipher
                .encrypt(&mut frame[frame_start..frame_start + PAYLOAD_LEN], &[], HomeKitChaChaNonce(1))
                .unwrap();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().without_plots();
    targets = cipher_encrypt_decrypt_fast
}
criterion_main!(benches);
