use super::KeystreamPrefetch;
use crate::prefetch::{KeystreamProvider, PrefetchStats, Schedule};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct BlockingProvider {
    block_once: Arc<AtomicBool>,
    entered: Arc<Barrier>,
    resume: Arc<Barrier>,
}

impl KeystreamProvider for BlockingProvider {
    type Error = core::convert::Infallible;

    fn generate_keystream(&mut self, nonce: u64, offset: u128, out: &mut [u8]) -> Result<(), Self::Error> {
        if out.len() == 16 && self.block_once.swap(false, Ordering::SeqCst) {
            self.entered.wait();
            self.resume.wait();
        }
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = (nonce as u8).wrapping_add((offset as u8).wrapping_add(i as u8));
        }
        Ok(())
    }

    fn apply_keystream(&mut self, nonce: u64, offset: u128, data: &mut [u8]) -> Result<(), Self::Error> {
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= (nonce as u8).wrapping_add((offset as u8).wrapping_add(i as u8));
        }
        Ok(())
    }
}

fn blocked() -> (BlockingProvider, Arc<Barrier>, Arc<Barrier>) {
    let entered = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    (
        BlockingProvider {
            block_once: Arc::new(AtomicBool::new(true)),
            entered: Arc::clone(&entered),
            resume: Arc::clone(&resume),
        },
        entered,
        resume,
    )
}

fn wait_for(prefetch: &KeystreamPrefetch<BlockingProvider>, available: usize) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while prefetch.status().available_bytes != available {
        assert!(Instant::now() < deadline, "prefetch did not fill");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn formatting_calculates_hit_and_drop_rates() {
    let stats = PrefetchStats {
        consumed_bytes: 80,
        used_bytes: 60,
        synchronous_bytes: 20,
        generated_bytes: 75,
        dropped_bytes: 15,
        ..Default::default()
    };
    let text = stats.to_string();
    assert!(text.contains("cache_hit=75.0%"));
    assert!(text.contains("drop=20.0%"));
    assert!(text.contains("consumed=80"));
}

#[test]
fn finite_consumer_overtakes_in_flight_chunk() {
    let (provider, entered, resume) = blocked();
    let mut prefetch = KeystreamPrefetch::new(provider.clone(), 7, Schedule::Finite { target: 32 }, 0, "test-prefetch", 16);
    prefetch.start_worker().unwrap();
    entered.wait();
    let mut first = [0u8; 5];
    assert_eq!(prefetch.apply(7, 0, &mut first).unwrap().synchronous_bytes, 5);
    assert_eq!(first, [7, 8, 9, 10, 11]);
    resume.wait();
    wait_for(&prefetch, 27);
    let mut rest = [0u8; 27];
    assert_eq!(prefetch.apply(7, 5, &mut rest).unwrap().cached_bytes, 27);
    assert_eq!(rest[0], 12);
    assert_eq!(rest[26], 38);
    let stats = prefetch.reset(provider, 8, Schedule::Finite { target: 0 }, 0);
    assert_eq!((stats.consumed_bytes, stats.used_bytes, stats.dropped_bytes), (32, 27, 5));
}

#[test]
fn reset_discards_in_flight_old_nonce() {
    let (provider, entered, resume) = blocked();
    let mut prefetch = KeystreamPrefetch::new(provider.clone(), 7, Schedule::Finite { target: 16 }, 0, "test-prefetch", 16);
    prefetch.start_worker().unwrap();
    entered.wait();
    let old = prefetch.reset(provider, 8, Schedule::Finite { target: 8 }, 0);
    assert_eq!(old.discarded_in_flight_bytes, 16);
    resume.wait();
    wait_for(&prefetch, 8);
    let mut bytes = [0u8; 8];
    assert_eq!(prefetch.apply(8, 0, &mut bytes).unwrap().cached_bytes, 8);
    assert_eq!(bytes, [8, 9, 10, 11, 12, 13, 14, 15]);
}

#[test]
fn provider_error_keeps_input_and_cursor_unchanged() {
    #[derive(Clone)]
    struct FailingProvider;
    impl KeystreamProvider for FailingProvider {
        type Error = ();
        fn generate_keystream(&mut self, _: u64, _: u128, out: &mut [u8]) -> Result<(), Self::Error> {
            out.fill(0xff);
            Err(())
        }

        fn apply_keystream(&mut self, _: u64, _: u128, _: &mut [u8]) -> Result<(), Self::Error> {
            Err(())
        }
    }

    let mut prefetch = KeystreamPrefetch::new(FailingProvider, 0, Schedule::Continuous { capacity: 0 }, 17, "test-prefetch", 16);
    let mut data = [0x35; 9];
    assert!(prefetch.apply_next(&mut data).is_err());
    assert_eq!(data, [0x35; 9]);
    assert_eq!(prefetch.status().consumed_position, 17);
    assert_eq!(prefetch.finish_cycle().consumed_bytes, 0);
}
