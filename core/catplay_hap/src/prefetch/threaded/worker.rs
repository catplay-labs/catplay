use super::state::{Job, State};
use super::std;
use super::wake::{Chunk, PollWorker, Wake};
use crate::prefetch::{KeystreamProvider, Schedule};
use std::sync::Arc;

pub(super) struct CoreWorker<P: KeystreamProvider> {
    pub(super) shared: Arc<Wake<State<P>>>,
    pub(super) chunk_size: usize,
    pub(super) provider: Option<P>,
    pub(super) active_generation: u64,
}

impl<P: KeystreamProvider> PollWorker for CoreWorker<P> {
    type State = State<P>;
    fn shared(&self) -> &Arc<Wake<State<P>>> {
        &self.shared
    }
    fn idle(state: &State<P>) -> bool {
        !state.stop && !state.ready()
    }
    fn stop(state: &mut State<P>) {
        state.stop = true;
    }
    fn poll(&mut self) -> bool {
        let (provider, job) = {
            let mut state = self.shared.lock();
            if state.stop {
                return true;
            }
            if !state.ready() {
                return false;
            }
            let start = match state.schedule {
                Schedule::Continuous { .. } => state.scheduled,
                Schedule::Finite { .. } => state.scheduled.max(state.consumed),
            };
            let remaining = match state.schedule {
                Schedule::Continuous { capacity } => capacity.saturating_sub(state.available) as u128,
                Schedule::Finite { target } => (target as u128).saturating_sub(start),
            };
            let len = remaining
                .min(self.chunk_size as u128)
                .min(u128::MAX - start) as usize;
            if len == 0 {
                state.failed = true;
                return false;
            }
            let job = Job {
                generation: state.generation,
                nonce: state.nonce,
                offset: start,
                len,
            };
            state.scheduled = start + len as u128;
            state.in_flight = Some(job);
            (state.provider.clone(), job)
        };
        if self.active_generation != job.generation {
            self.provider = Some(provider);
            self.active_generation = job.generation;
        }
        // Allocate the final shared cache chunk uninitialized. KeystreamProvider
        // promises to write every output byte, so zero-filling a temporary Vec
        // first would be wasted work and converting Vec -> Arc<[u8]> would copy
        // the entire chunk into a second allocation.
        let mut bytes = Arc::<[u8]>::new_uninit_slice(job.len);
        let output = Arc::get_mut(&mut bytes).expect("new Arc allocation must be unique");
        let output = unsafe { core::slice::from_raw_parts_mut(output.as_mut_ptr().cast::<u8>(), output.len()) };
        let result = self
            .provider
            .as_mut()
            .unwrap()
            .generate_keystream(job.nonce, job.offset, output);
        let mut state = self.shared.lock();
        if state.stop {
            return true;
        }
        if state.generation != job.generation {
            return false;
        }
        state.in_flight = None;
        if result.is_err() {
            state.failed = true;
            self.provider = None;
            return false;
        }
        // `generate_keystream` succeeded and guarantees that it initialized the
        // full output slice above.
        let bytes = unsafe { bytes.assume_init() };
        state.cycle.generated_bytes += job.len;
        state.generated_position = job.offset + job.len as u128;
        let useful_start = job.offset.max(state.consumed);
        if useful_start < state.generated_position {
            state.available += (state.generated_position - useful_start) as usize;
            state.chunks.push_back(Chunk {
                bytes,
                position: job.offset,
            });
        }
        false
    }
}
