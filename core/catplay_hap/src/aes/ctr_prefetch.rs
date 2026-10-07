//! AES-CTR keystream prefetch with synchronous fallback.
//!
//! With `std`, a worker encrypts zeroes ahead of the consumer. Without `std`,
//! every request generates keystream synchronously at its exact byte position.
//! The cache is an optimization and never changes the CTR stream.

#[cfg(feature = "openssl")]
use super::Aes128CtrOpenSsl;
use super::{Aes128CtrKernelStream, Aes128CtrSoft, AfAlgError};
use crate::prefetch::{
    ApplyError, KeystreamPrefetch, KeystreamProvider, PrefetchResult, PrefetchStartError, PrefetchStats, PrefetchStatus, Schedule,
};
use core::convert::Infallible;
#[cfg(all(test, feature = "std"))]
use std::thread;
#[cfg(all(test, feature = "std"))]
use std::vec;

const CHUNK_SIZE: usize = 64 * 1024;
pub const DEFAULT_PREFETCH_CAPACITY: usize = 512 * 1024;

/// A sequential AES-128-CTR implementation usable by the prefetch layer.
pub trait Aes128CtrEngine: Send + 'static {
    type Error;

    fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Write raw keystream to `out`, regardless of its initial contents.
    fn generate_keystream(&mut self, out: &mut [u8]) -> Result<(), Self::Error>;

    /// XOR keystream into existing data in place.
    fn apply_keystream(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

impl Aes128CtrEngine for Aes128CtrKernelStream {
    type Error = AfAlgError;

    fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self, Self::Error> {
        Aes128CtrKernelStream::new(key, iv)
    }

    fn generate_keystream(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
        Aes128CtrKernelStream::generate_keystream(self, out)
    }

    fn apply_keystream(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        Aes128CtrKernelStream::apply_keystream(self, data)
    }
}

impl Aes128CtrEngine for Aes128CtrSoft {
    type Error = Infallible;

    fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self, Self::Error> {
        Ok(Aes128CtrSoft::new(key, iv))
    }

    fn generate_keystream(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
        out.fill(0);
        Aes128CtrSoft::apply_keystream(self, out);
        Ok(())
    }

    fn apply_keystream(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        Aes128CtrSoft::apply_keystream(self, data);
        Ok(())
    }
}

#[cfg(feature = "openssl")]
impl Aes128CtrEngine for Aes128CtrOpenSsl {
    type Error = Infallible;

    fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self, Self::Error> {
        Ok(Aes128CtrOpenSsl::new(key, iv))
    }

    fn generate_keystream(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
        out.fill(0);
        Aes128CtrOpenSsl::apply_keystream(self, out);
        Ok(())
    }

    fn apply_keystream(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        Aes128CtrOpenSsl::apply_keystream(self, data);
        Ok(())
    }
}

#[derive(Debug)]
pub enum PrefetchError<E> {
    Backend(E),
    PositionOverflow,
}

struct AesProvider<B: Aes128CtrEngine> {
    key: [u8; 16],
    iv: [u8; 16],
    backend: Option<B>,
    next_offset: u128,
}

impl<B: Aes128CtrEngine> AesProvider<B> {
    fn new(key: [u8; 16], iv: [u8; 16]) -> Self {
        Self {
            key,
            iv,
            backend: None,
            next_offset: 0,
        }
    }

    fn backend_at(&mut self, offset: u128) -> Result<&mut B, B::Error> {
        if self.backend.is_none() || self.next_offset != offset {
            self.backend = Some(positioned_backend::<B>(&self.key, &self.iv, offset)?);
        }
        Ok(self.backend.as_mut().unwrap())
    }
}

impl<B: Aes128CtrEngine> Clone for AesProvider<B> {
    fn clone(&self) -> Self {
        Self::new(self.key, self.iv)
    }
}

impl<B: Aes128CtrEngine> KeystreamProvider for AesProvider<B> {
    type Error = B::Error;

    fn generate_keystream(&mut self, _nonce: u64, offset: u128, out: &mut [u8]) -> Result<(), Self::Error> {
        self.backend_at(offset)?.generate_keystream(out)?;
        self.next_offset = offset + out.len() as u128;
        Ok(())
    }

    fn apply_keystream(&mut self, _nonce: u64, offset: u128, data: &mut [u8]) -> Result<(), Self::Error> {
        self.backend_at(offset)?.apply_keystream(data)?;
        self.next_offset = offset + data.len() as u128;
        Ok(())
    }
}

/// Single-consumer CTR wrapper. The worker fills up to `capacity` useful bytes.
pub struct Aes128CtrPrefetch<B: Aes128CtrEngine> {
    inner: KeystreamPrefetch<AesProvider<B>>,
}

impl<B: Aes128CtrEngine> Aes128CtrPrefetch<B> {
    pub fn new(key: [u8; 16], iv: [u8; 16], capacity: usize) -> Result<Self, PrefetchStartError> {
        let mut inner = KeystreamPrefetch::new(
            AesProvider::new(key, iv),
            0,
            Schedule::Continuous { capacity },
            0,
            "aes-ctr-prefetch",
            CHUNK_SIZE,
        );
        inner.start_worker()?;
        Ok(Self { inner })
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.inner.resize(Schedule::Continuous { capacity });
    }

    pub fn reset(&mut self, key: [u8; 16], iv: [u8; 16], byte_position: u128) -> PrefetchStats {
        self.inner.reset(
            AesProvider::new(key, iv),
            0,
            Schedule::Continuous {
                capacity: self.inner.status().requested_bytes,
            },
            byte_position,
        )
    }

    pub fn status(&self) -> PrefetchStatus {
        self.inner.status()
    }

    pub fn finish_cycle(&mut self) -> PrefetchStats {
        self.inner.finish_cycle()
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) -> Result<PrefetchResult, PrefetchError<B::Error>> {
        self.inner.apply_next(data).map_err(|error| match error {
            ApplyError::Provider(error) => PrefetchError::Backend(error),
            ApplyError::PositionOverflow => PrefetchError::PositionOverflow,
        })
    }
}

fn positioned_backend<B: Aes128CtrEngine>(key: &[u8; 16], iv: &[u8; 16], position: u128) -> Result<B, B::Error> {
    let counter = u128::from_be_bytes(*iv)
        .wrapping_add(position / 16)
        .to_be_bytes();
    let mut backend = B::new(key, &counter)?;
    let skip = (position % 16) as usize;
    if skip != 0 {
        backend.apply_keystream(&mut [0u8; 16][..skip])?;
    }
    Ok(backend)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    static SLOW_BACKEND_STARTED: AtomicBool = AtomicBool::new(false);

    struct SlowBackend(Aes128CtrSoft);

    impl Aes128CtrEngine for SlowBackend {
        type Error = Infallible;

        fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self, Self::Error> {
            Ok(Self(Aes128CtrSoft::new(key, iv)))
        }

        fn generate_keystream(&mut self, out: &mut [u8]) -> Result<(), Self::Error> {
            SLOW_BACKEND_STARTED.store(true, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(30));
            out.fill(0);
            self.0.apply_keystream(out);
            Ok(())
        }

        fn apply_keystream(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            SLOW_BACKEND_STARTED.store(true, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(30));
            self.0.apply_keystream(data);
            Ok(())
        }
    }

    fn wait_for_available<B: Aes128CtrEngine>(prefetch: &Aes128CtrPrefetch<B>, minimum: usize) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while prefetch.status().available_bytes < minimum {
            assert!(Instant::now() < deadline, "prefetch did not fill");
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn expected(key: [u8; 16], iv: [u8; 16], position: usize, data: &mut [u8]) {
        let mut cipher = Aes128CtrSoft::new(&key, &iv);
        let mut skipped = vec![0u8; position];
        cipher.apply_keystream(&mut skipped);
        cipher.apply_keystream(data);
    }

    #[test]
    fn provider_generates_raw_stream_over_nonzero_output_and_applies_in_place() {
        let key = [0x26; 16];
        let iv = [0x91; 16];
        let mut provider = AesProvider::<Aes128CtrSoft>::new(key, iv);

        let mut raw = [0xa5; 37];
        provider.generate_keystream(0, 11, &mut raw).unwrap();
        let mut expected_raw = [0; 37];
        expected(key, iv, 11, &mut expected_raw);
        assert_eq!(raw, expected_raw);

        let mut data = [0x5a; 29];
        provider.apply_keystream(0, 48, &mut data).unwrap();
        let mut expected_data = [0x5a; 29];
        expected(key, iv, 48, &mut expected_data);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn cache_hit_partial_miss_and_refill_match_continuous_ctr() {
        let key = [0x31; 16];
        let iv = [0x92; 16];
        let mut prefetch = Aes128CtrPrefetch::<Aes128CtrSoft>::new(key, iv, 31).unwrap();
        wait_for_available(&prefetch, 31);

        let mut first = vec![0x53; 47];
        let mut expected_first = first.clone();
        expected(key, iv, 0, &mut expected_first);
        let result = prefetch.apply_keystream(&mut first).unwrap();
        assert_eq!(
            result,
            PrefetchResult {
                cached_bytes: 31,
                synchronous_bytes: 16
            }
        );
        assert_eq!(first, expected_first);

        wait_for_available(&prefetch, 31);
        let mut second = vec![0x26; 13];
        let mut expected_second = second.clone();
        expected(key, iv, 47, &mut expected_second);
        assert_eq!(prefetch.apply_keystream(&mut second).unwrap().cached_bytes, 13);
        assert_eq!(second, expected_second);
        assert_eq!(prefetch.status().consumed_position, 60);
    }

    #[test]
    fn resize_zero_and_reset_at_unaligned_position() {
        let key = [0x11; 16];
        let iv = [0x22; 16];
        let mut prefetch = Aes128CtrPrefetch::<Aes128CtrSoft>::new(key, iv, 0).unwrap();
        assert_eq!(prefetch.status().available_bytes, 0);
        let mut first = [0x45; 9];
        let mut reference = first;
        expected(key, iv, 0, &mut reference);
        assert_eq!(
            prefetch
                .apply_keystream(&mut first)
                .unwrap()
                .synchronous_bytes,
            9
        );
        assert_eq!(first, reference);

        prefetch.set_capacity(20);
        wait_for_available(&prefetch, 20);
        assert_eq!(prefetch.status().requested_bytes, 20);
        prefetch.set_capacity(0);
        assert_eq!(prefetch.status().available_bytes, 0);

        let new_key = [0x73; 16];
        let new_iv = [0x84; 16];
        prefetch.reset(new_key, new_iv, 5);
        prefetch.set_capacity(18);
        wait_for_available(&prefetch, 18);
        let mut data = [0x55; 27];
        let mut reference = data;
        expected(new_key, new_iv, 5, &mut reference);
        assert_eq!(
            prefetch.apply_keystream(&mut data).unwrap(),
            PrefetchResult {
                cached_bytes: 18,
                synchronous_bytes: 9
            }
        );
        assert_eq!(data, reference);
    }

    #[test]
    fn reset_discards_in_flight_old_key_output() {
        SLOW_BACKEND_STARTED.store(false, Ordering::SeqCst);
        let mut prefetch = Aes128CtrPrefetch::<SlowBackend>::new([1; 16], [2; 16], 32).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !SLOW_BACKEND_STARTED.load(Ordering::SeqCst) {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
        let key = [3; 16];
        let iv = [4; 16];
        prefetch.reset(key, iv, 7);
        wait_for_available(&prefetch, 32);
        let mut data = [0x85; 32];
        let mut reference = data;
        expected(key, iv, 7, &mut reference);
        assert_eq!(prefetch.apply_keystream(&mut data).unwrap().cached_bytes, 32);
        assert_eq!(data, reference);
    }

    #[test]
    fn frame_stats_include_all_progressive_chunks() {
        let mut prefetch = Aes128CtrPrefetch::<Aes128CtrSoft>::new([7; 16], [8; 16], 16).unwrap();
        wait_for_available(&prefetch, 16);
        prefetch.apply_keystream(&mut [0; 9]).unwrap();
        prefetch.set_capacity(0);
        prefetch.apply_keystream(&mut [0; 11]).unwrap();
        let stats = prefetch.finish_cycle();
        assert_eq!((stats.consumed_bytes, stats.used_bytes, stats.synchronous_bytes), (20, 9, 11));
        assert_eq!(stats.dropped_bytes, 0);
        assert_eq!(prefetch.finish_cycle().consumed_bytes, 0);
    }
}
