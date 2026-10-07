use super::std;
use super::wake::Chunk;
use crate::prefetch::{KeystreamProvider, PrefetchStats, PrefetchStatus, Schedule};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(super) struct Job {
    pub(super) generation: u64,
    pub(super) nonce: u64,
    pub(super) offset: u128,
    pub(super) len: usize,
}

pub(super) struct State<P: KeystreamProvider> {
    pub(super) provider: P,
    pub(super) nonce: u64,
    pub(super) schedule: Schedule,
    pub(super) consumed: u128,
    pub(super) scheduled: u128,
    pub(super) generated_position: u128,
    pub(super) available: usize,
    pub(super) generation: u64,
    pub(super) in_flight: Option<Job>,
    pub(super) chunks: VecDeque<Chunk>,
    pub(super) cycle: PrefetchStats,
    pub(super) failed: bool,
    pub(super) stop: bool,
}

impl<P: KeystreamProvider> State<P> {
    pub(super) fn target(&self) -> usize {
        match self.schedule {
            Schedule::Continuous { capacity } => capacity,
            Schedule::Finite { target } => target,
        }
    }

    pub(super) fn ready(&self) -> bool {
        if self.failed {
            return false;
        }
        match self.schedule {
            Schedule::Continuous { capacity } => self.available < capacity && self.scheduled < u128::MAX,
            Schedule::Finite { target } => self.scheduled.max(self.consumed) < target as u128,
        }
    }

    /// Discard queued and in-flight work, then refill from the consumer cursor.
    /// The generation counter prevents an old worker result from being published.
    pub(super) fn restart_from_consumed(&mut self) {
        self.chunks.clear();
        self.available = 0;
        self.scheduled = self.consumed;
        self.generated_position = self.consumed;
        self.in_flight = None;
        self.generation = self.generation.wrapping_add(1);
        self.failed = false;
    }

    /// Count the part of queued chunks still ahead of `consumed`. This does not
    /// discard in-flight work or move the finite target.
    pub(super) fn recount_ready_bytes(&mut self) {
        self.available = self
            .chunks
            .iter()
            .map(|chunk| {
                let end = chunk.position + chunk.bytes.len() as u128;
                end.saturating_sub(chunk.position.max(self.consumed)) as usize
            })
            .sum();
    }

    pub(super) fn status(&self) -> PrefetchStatus {
        PrefetchStatus {
            nonce: self.nonce,
            requested_bytes: self.target(),
            available_bytes: self.available,
            consumed_position: self.consumed,
            generated_position: self.generated_position,
            used_bytes: self.cycle.used_bytes,
        }
    }

    pub(super) fn close_cycle(&mut self, count_dropped: bool) -> PrefetchStats {
        let mut stats = core::mem::take(&mut self.cycle);
        stats.nonce = self.nonce;
        stats.requested_bytes = self.target();
        if count_dropped {
            stats.dropped_bytes = stats.generated_bytes.saturating_sub(stats.used_bytes);
            stats.discarded_in_flight_bytes = self.in_flight.map_or(0, |job| job.len);
        }
        stats
    }
}
