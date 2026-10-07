//! Keystream prefetch with no background worker or cache.

use super::{ApplyError, KeystreamProvider, PrefetchResult, PrefetchStartError, PrefetchStats, PrefetchStatus, Schedule};
use core::sync::atomic::{AtomicBool, Ordering};

static WARNED: AtomicBool = AtomicBool::new(false);

pub(crate) struct KeystreamPrefetch<P: KeystreamProvider> {
    provider: P,
    nonce: u64,
    schedule: Schedule,
    consumed: u128,
    cycle: PrefetchStats,
}

impl<P: KeystreamProvider> KeystreamPrefetch<P> {
    pub fn new(provider: P, nonce: u64, schedule: Schedule, start: u128, _name: &'static str, _chunk_size: usize) -> Self {
        Self {
            provider,
            nonce,
            schedule,
            consumed: start,
            cycle: PrefetchStats::default(),
        }
    }

    pub fn start_worker(&mut self) -> Result<(), PrefetchStartError> {
        if !WARNED.swap(true, Ordering::Relaxed) {
            log::warn!("Keystream prefetch is off without std; using synchronous generation");
        }
        Ok(())
    }

    fn target(&self) -> usize {
        match self.schedule {
            Schedule::Continuous { capacity } => capacity,
            Schedule::Finite { target } => target,
        }
    }

    pub fn status(&self) -> PrefetchStatus {
        PrefetchStatus {
            nonce: self.nonce,
            requested_bytes: self.target(),
            available_bytes: 0,
            consumed_position: self.consumed,
            generated_position: self.consumed,
            used_bytes: 0,
        }
    }

    pub fn finish_cycle(&mut self) -> PrefetchStats {
        let mut stats = core::mem::take(&mut self.cycle);
        stats.nonce = self.nonce;
        stats.requested_bytes = self.target();
        stats
    }

    pub fn reset(&mut self, provider: P, nonce: u64, schedule: Schedule, start: u128) -> PrefetchStats {
        let previous = self.finish_cycle();
        self.provider = provider;
        self.nonce = nonce;
        self.schedule = schedule;
        self.consumed = start;
        previous
    }

    pub fn resize(&mut self, schedule: Schedule) {
        self.schedule = schedule;
    }

    pub fn apply(&mut self, nonce: u64, offset: u128, data: &mut [u8]) -> Result<PrefetchResult, ApplyError<P::Error>> {
        let end = offset
            .checked_add(data.len() as u128)
            .ok_or(ApplyError::PositionOverflow)?;
        if data.is_empty() {
            return Ok(PrefetchResult::default());
        }
        self.provider
            .apply_keystream(nonce, offset, data)
            .map_err(ApplyError::Provider)?;
        if nonce == self.nonce {
            self.consumed = self.consumed.max(end);
            self.cycle.consumed_bytes += data.len();
            self.cycle.synchronous_bytes += data.len();
        }
        Ok(PrefetchResult {
            cached_bytes: 0,
            synchronous_bytes: data.len(),
        })
    }

    pub fn apply_next(&mut self, data: &mut [u8]) -> Result<PrefetchResult, ApplyError<P::Error>> {
        self.apply(self.nonce, self.consumed, data)
    }
}
