use super::state::State;
use super::std;
use super::wake::{Wake, Worker};
use super::worker::CoreWorker;
use crate::prefetch::{ApplyError, KeystreamProvider, PrefetchResult, PrefetchStartError, PrefetchStats, PrefetchStatus, Schedule};
use crate::xor_cipher::xor;
use std::collections::VecDeque;
use std::sync::Arc;
use std::vec::Vec;

/// Single-consumer cache. Generation happens outside the state lock; a reset
/// invalidates any in-flight result by advancing `generation`.
pub(crate) struct KeystreamPrefetch<P: KeystreamProvider> {
    shared: Arc<Wake<State<P>>>,
    worker: Option<Worker<CoreWorker<P>>>,
    name: &'static str,
    chunk_size: usize,
}

impl<P: KeystreamProvider> KeystreamPrefetch<P> {
    pub fn new(provider: P, nonce: u64, schedule: Schedule, start: u128, name: &'static str, chunk_size: usize) -> Self {
        let shared = Arc::new(Wake::new(State {
            provider,
            nonce,
            schedule,
            consumed: start,
            scheduled: start,
            generated_position: start,
            available: 0,
            generation: 0,
            in_flight: None,
            chunks: VecDeque::new(),
            cycle: PrefetchStats::default(),
            failed: false,
            stop: false,
        }));
        Self {
            shared,
            worker: None,
            name,
            chunk_size,
        }
    }

    pub fn start_worker(&mut self) -> Result<(), PrefetchStartError> {
        if self.worker.is_none() {
            self.worker = Some(
                Worker::spawn(
                    self.name,
                    CoreWorker {
                        shared: Arc::clone(&self.shared),
                        chunk_size: self.chunk_size,
                        provider: None,
                        active_generation: u64::MAX,
                    },
                )
                .map_err(|_| PrefetchStartError)?,
            );
        }
        Ok(())
    }

    pub fn status(&self) -> PrefetchStatus {
        self.shared.lock().status()
    }

    /// Ends accounting for a cycle while keeping the continuous cache warm.
    pub fn finish_cycle(&mut self) -> PrefetchStats {
        let mut state = self.shared.lock();
        state.close_cycle(false)
    }

    /// Ends the cycle, changes nonce and/or target, and invalidates queued work.
    pub fn reset(&mut self, provider: P, nonce: u64, schedule: Schedule, start: u128) -> PrefetchStats {
        let mut state = self.shared.lock();
        let count_dropped = matches!(state.schedule, Schedule::Finite { .. });
        let previous = state.close_cycle(count_dropped);
        state.provider = provider;
        state.nonce = nonce;
        state.schedule = schedule;
        state.consumed = start;
        state.restart_from_consumed();
        drop(state);
        self.shared.notify();
        previous
    }

    /// Resize without ending accounting for the current cycle.
    pub fn resize(&mut self, schedule: Schedule) {
        let mut state = self.shared.lock();
        state.schedule = schedule;
        state.restart_from_consumed();
        drop(state);
        self.shared.notify();
    }

    /// XOR cached bytes and apply a freshly generated suffix in place. The caller
    /// serializes calls to this method.
    pub fn apply(&mut self, nonce: u64, offset: u128, data: &mut [u8]) -> Result<PrefetchResult, ApplyError<P::Error>> {
        let end = offset
            .checked_add(data.len() as u128)
            .ok_or(ApplyError::PositionOverflow)?;
        if data.is_empty() {
            return Ok(PrefetchResult::default());
        }
        let mut cached = Vec::new();
        let (mut provider, cached_len, tracked) = {
            let mut state = self.shared.lock();
            let tracked = state.nonce == nonce;
            let mut cursor = offset;
            if tracked && offset >= state.consumed {
                while let Some(chunk) = state.chunks.front() {
                    let chunk_end = chunk.position + chunk.bytes.len() as u128;
                    if chunk_end <= cursor {
                        state.chunks.pop_front();
                        continue;
                    }
                    if chunk.position > cursor || cursor == end {
                        break;
                    }
                    let len = (chunk_end - cursor).min(end - cursor) as usize;
                    cached.push((Arc::clone(&chunk.bytes), (cursor - chunk.position) as usize, len));
                    cursor += len as u128;
                    if cursor == chunk_end {
                        state.chunks.pop_front();
                    }
                }
                state.consumed = end;
                while state
                    .chunks
                    .front()
                    .is_some_and(|chunk| chunk.position + chunk.bytes.len() as u128 <= end)
                {
                    state.chunks.pop_front();
                }
                if matches!(state.schedule, Schedule::Continuous { .. }) && cursor < end {
                    // A continuous cache refills from the end of this synchronous
                    // suffix. In-flight output may overlap bytes just consumed.
                    state.restart_from_consumed();
                } else {
                    // A finite target keeps useful later offsets. The worker may
                    // still finish the remaining range, but never past `target`.
                    state.recount_ready_bytes();
                }
            }
            let cached_len = (cursor - offset) as usize;
            (state.provider.clone(), cached_len, tracked)
        };
        self.shared.notify();
        let fallback_len = data.len() - cached_len;
        if fallback_len > 0 {
            if let Err(error) = provider.apply_keystream(nonce, offset + cached_len as u128, &mut data[cached_len..]) {
                if tracked {
                    let mut state = self.shared.lock();
                    state.consumed = offset;
                    state.restart_from_consumed();
                    drop(state);
                    self.shared.notify();
                }
                return Err(ApplyError::Provider(error));
            }
        }
        let mut at = 0;
        for (bytes, from, len) in cached {
            xor(&mut data[at..at + len], &bytes[from..from + len]);
            at += len;
        }
        if tracked {
            let mut state = self.shared.lock();
            state.cycle.consumed_bytes += data.len();
            state.cycle.used_bytes += cached_len;
            state.cycle.synchronous_bytes += fallback_len;
        }
        Ok(PrefetchResult {
            cached_bytes: cached_len,
            synchronous_bytes: fallback_len,
        })
    }

    /// Apply at the current continuous-stream cursor.
    pub fn apply_next(&mut self, data: &mut [u8]) -> Result<PrefetchResult, ApplyError<P::Error>> {
        let (nonce, offset) = {
            let state = self.shared.lock();
            (state.nonce, state.consumed)
        };
        self.apply(nonce, offset, data)
    }
}
