use super::std;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

pub(super) struct Chunk {
    pub bytes: Arc<[u8]>,
    pub position: u128,
}

pub(super) struct Wake<S> {
    state: Mutex<S>,
    wake: Condvar,
}

impl<S> Wake<S> {
    pub fn new(state: S) -> Self {
        Self {
            state: Mutex::new(state),
            wake: Condvar::new(),
        }
    }

    pub fn notify(&self) {
        self.wake.notify_one();
    }

    pub fn lock(&self) -> MutexGuard<'_, S> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn wait_while<'a>(&self, state: MutexGuard<'a, S>, idle: impl FnMut(&mut S) -> bool) -> MutexGuard<'a, S> {
        self.wake
            .wait_while(state, idle)
            .unwrap_or_else(|e| e.into_inner())
    }
}

impl<S: Default> Default for Wake<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

/// The cipher owns one bounded unit of work; this adapter owns its thread.
pub(super) trait PollWorker: Send + 'static {
    type State: Send + 'static;
    fn shared(&self) -> &Arc<Wake<Self::State>>;
    fn idle(state: &Self::State) -> bool;
    fn stop(state: &mut Self::State);
    /// Sleep until work or shutdown is visible under the same lock as notify.
    fn wait(&self) {
        let state = self.shared().lock();
        drop(self.shared().wait_while(state, |state| Self::idle(state)));
    }
    /// Returns true only when the worker must exit.
    fn poll(&mut self) -> bool;
}

pub(super) struct Worker<W: PollWorker> {
    shared: Arc<Wake<W::State>>,
    handle: Option<JoinHandle<()>>,
}

impl<W: PollWorker> Worker<W> {
    const YIELD_AFTER_CHUNKS: usize = 4;

    pub fn spawn(name: &str, mut worker: W) -> std::io::Result<Self> {
        let shared = Arc::clone(worker.shared());
        let handle = thread::Builder::new().name(name.into()).spawn(move || {
            #[cfg(target_os = "linux")]
            unsafe {
                libc::nice(10);
            }
            let mut polls_since_yield = 0usize;
            loop {
                worker.wait();
                if worker.poll() {
                    break;
                }

                polls_since_yield += 1;
                if polls_since_yield == Self::YIELD_AFTER_CHUNKS {
                    polls_since_yield = 0;
                    thread::yield_now();
                }
            }
        })?;
        Ok(Self {
            shared,
            handle: Some(handle),
        })
    }
}

impl<W: PollWorker> Drop for Worker<W> {
    fn drop(&mut self) {
        W::stop(&mut self.shared.lock());
        self.shared.notify();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
