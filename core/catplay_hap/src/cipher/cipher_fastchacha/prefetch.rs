//! ChaCha payload provider for the shared keystream prefetcher.
use super::{ChaChaPrefetchStats, ChaChaPrefetchStatus, FastChaCha20, HomeKitChaChaNonce, nonce_to_bytes};
use crate::prefetch::{KeystreamPrefetch, KeystreamProvider, Schedule};
use log::warn;

const CHUNK_SIZE: usize = 16 * 1024;

#[derive(Clone)]
struct ChaChaProvider {
    key: [u8; 32],
}

impl KeystreamProvider for ChaChaProvider {
    type Error = core::convert::Infallible;

    fn generate_keystream(&mut self, nonce: u64, offset: u128, out: &mut [u8]) -> Result<(), Self::Error> {
        generate(&self.key, nonce, offset as usize, out);
        Ok(())
    }

    fn apply_keystream(&mut self, nonce: u64, offset: u128, data: &mut [u8]) -> Result<(), Self::Error> {
        super::apply_keystream_at_offset(self.key, nonce_to_bytes(nonce), offset as usize, data);
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct Prefetch {
    inner: Option<KeystreamPrefetch<ChaChaProvider>>,
    spawn_failed: bool,
}

impl Prefetch {
    pub(super) fn reset(&mut self, key: [u8; 32], nonce: HomeKitChaChaNonce, size: usize) -> Option<ChaChaPrefetchStats> {
        let provider = ChaChaProvider { key };
        let previous = if let Some(inner) = &mut self.inner {
            Some(inner.reset(provider, nonce.0, Schedule::Finite { target: size }, 0))
        } else {
            self.inner = Some(KeystreamPrefetch::new(
                provider,
                nonce.0,
                Schedule::Finite { target: size },
                0,
                "chacha-prefetch",
                CHUNK_SIZE,
            ));
            None
        };
        if size > 0 && !self.spawn_failed {
            if let Err(error) = self.inner.as_mut().unwrap().start_worker() {
                self.spawn_failed = true;
                warn!("Cannot start ChaCha prefetch worker: {error}; using synchronous cipher");
            }
        }
        previous
    }

    pub(super) fn status(&self) -> Option<ChaChaPrefetchStatus> {
        self.inner.as_ref().map(KeystreamPrefetch::status)
    }

    /// Apply cached bytes and generate the missing suffix synchronously.
    pub(super) fn apply(&mut self, nonce: HomeKitChaChaNonce, offset: usize, data: &mut [u8]) {
        // ChaCha's valid payload range was checked by the caller.
        self.inner
            .as_mut()
            .unwrap()
            .apply(nonce.0, offset as u128, data)
            .unwrap();
    }
}

fn generate(key: &[u8; 32], nonce: u64, offset: usize, out: &mut [u8]) {
    let mut cipher = FastChaCha20::new_with_counter(*key, nonce_to_bytes(nonce), 1 + (offset / 64) as u32);
    let skip = offset % 64;
    let mut done = 0;
    if skip != 0 {
        let mut block = [0u8; 64];
        cipher.keystream_only(&mut block);
        done = (64 - skip).min(out.len());
        out[..done].copy_from_slice(&block[skip..skip + done]);
    }
    cipher.keystream_only(&mut out[done..]);
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
    use std::{
        thread,
        time::{Duration, Instant},
        vec,
    };

    const KEY: [u8; 32] = [0x93; 32];

    fn reference(nonce: u64, offset: usize, data: &mut [u8]) {
        let bytes = nonce_to_bytes(nonce);
        let mut cipher = chacha20::ChaCha20::new((&KEY).into(), (&bytes).into());
        cipher.seek(64 + offset as u64);
        cipher.apply_keystream(data);
    }

    fn wait(prefetch: &Prefetch, available: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while prefetch.status().unwrap().available_bytes != available {
            assert!(Instant::now() < deadline, "worker did not fill target");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn provider_matches_rustcrypto_at_arbitrary_offsets() {
        for nonce in [0, 1, 0x123456789abcdef0, u64::MAX] {
            for offset in [0, 1, 63, 64, 65, CHUNK_SIZE - 1, CHUNK_SIZE + 17] {
                for len in [0, 1, 3, 63, 64, 65, 129, CHUNK_SIZE + 1] {
                    let mut actual = vec![0xa7; len];
                    generate(&KEY, nonce, offset, &mut actual);
                    let mut expected = vec![0; len];
                    reference(nonce, offset, &mut expected);
                    assert_eq!(actual, expected, "nonce={nonce} offset={offset} len={len}");
                }
            }
        }
    }

    #[test]
    fn reset_reports_used_and_dropped_bytes() {
        let mut prefetch = Prefetch::default();
        assert!(prefetch.reset(KEY, HomeKitChaChaNonce(7), 73).is_none());
        wait(&prefetch, 73);
        let mut actual = [0x12; 100];
        let mut expected = actual;
        reference(7, 0, &mut expected);
        prefetch.apply(HomeKitChaChaNonce(7), 0, &mut actual);
        assert_eq!(actual, expected);
        let stats = prefetch.reset(KEY, HomeKitChaChaNonce(8), 0).unwrap();
        assert_eq!((stats.consumed_bytes, stats.used_bytes, stats.synchronous_bytes), (100, 73, 27));
        assert_eq!(stats.dropped_bytes, 0);
    }

    #[test]
    fn stale_nonce_does_not_consume_current_cache() {
        let mut prefetch = Prefetch::default();
        prefetch.reset(KEY, HomeKitChaChaNonce(7), 80);
        wait(&prefetch, 80);
        let mut old = [0x12; 9];
        let mut expected_old = old;
        reference(6, 0, &mut expected_old);
        prefetch.apply(HomeKitChaChaNonce(6), 0, &mut old);
        assert_eq!(old, expected_old);
        assert_eq!(prefetch.status().unwrap().available_bytes, 80);
        let mut current = [0x34; 20];
        let mut expected_current = current;
        reference(7, 0, &mut expected_current);
        prefetch.apply(HomeKitChaChaNonce(7), 0, &mut current);
        assert_eq!(current, expected_current);
        let stats = prefetch.reset(KEY, HomeKitChaChaNonce(8), 0).unwrap();
        assert_eq!((stats.used_bytes, stats.dropped_bytes), (20, 60));
    }
}
