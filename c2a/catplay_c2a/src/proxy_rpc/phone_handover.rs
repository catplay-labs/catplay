use std::future::Future;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct PhoneState {
    live: AtomicBool,
    recorded: AtomicBool,
}

#[derive(Clone)]
pub struct PhoneLease(Arc<PhoneState>);

impl PhoneLease {
    pub fn live(&self) -> bool {
        self.0.live.load(Ordering::Acquire)
    }
    pub fn end(&self) {
        self.0.live.store(false, Ordering::Release);
    }
    pub fn record(&self) {
        if self.live() {
            self.0.recorded.store(true, Ordering::Release);
        }
    }
}

#[derive(Default)]
struct Admission {
    paused: bool,
    current: Option<PhoneLease>,
}

#[derive(Clone, Default)]
pub struct PhoneGate(Arc<Mutex<Admission>>);

impl PhoneGate {
    pub fn acquire(&self) -> Option<PhoneLease> {
        let mut state = self.0.lock().unwrap();
        if state.paused || state.current.as_ref().is_some_and(PhoneLease::live) {
            return None;
        }
        let lease = PhoneLease(Arc::new(PhoneState {
            live: AtomicBool::new(true),
            recorded: AtomicBool::new(false),
        }));
        state.current = Some(lease.clone());
        Some(lease)
    }

    pub fn owns(&self, lease: &PhoneLease) -> bool {
        let state = self.0.lock().unwrap();
        !state.paused
            && state
                .current
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(&current.0, &lease.0))
    }

    pub fn pause(&self) {
        let mut state = self.0.lock().unwrap();
        state.paused = true;
        if let Some(old) = state.current.take() {
            old.end();
        }
    }

    pub fn paused(&self) -> bool {
        self.0.lock().unwrap().paused
    }
    pub fn recorded(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .current
            .as_ref()
            .is_some_and(|lease| lease.live() && lease.0.recorded.load(Ordering::Acquire))
    }
    pub fn resume(&self) {
        self.0.lock().unwrap().paused = false;
    }
}

/// Admission opens only after the real radio stream-release barrier succeeds.
pub(crate) async fn finish_handover<E>(gate: PhoneGate, release: impl Future<Output = Result<(), E>>) -> Result<(), E> {
    release.await?;
    gate.resume();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_phone_waits_for_confirmed_release_and_old_events_stay_retired() {
        let gate = PhoneGate::default();
        let old = gate.acquire().unwrap();
        gate.pause();
        let (done, wait) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(finish_handover(gate.clone(), async { wait.await.map_err(|_| ()) }));
        tokio::task::yield_now().await;
        assert!(gate.acquire().is_none());
        assert!(!gate.owns(&old));
        assert!(!old.live());
        done.send(()).unwrap();
        task.await.unwrap().unwrap();
        let new = gate.acquire().unwrap();
        old.end(); // delayed drop from A must not revoke B
        assert!(gate.owns(&new) && new.live());
        assert!(!gate.owns(&old));
    }

    #[tokio::test]
    async fn failed_release_does_not_admit_a_new_phone() {
        let gate = PhoneGate::default();
        gate.pause();
        assert_eq!(finish_handover(gate.clone(), async { Err::<(), _>(7) }).await, Err(7));
        assert!(gate.acquire().is_none());
    }

    #[test]
    fn disconnected_old_phone_cannot_own_a_replacement() {
        let gate = PhoneGate::default();
        let old = gate.acquire().unwrap();
        assert!(gate.acquire().is_none());
        old.end();
        assert!(gate.owns(&old)); // its disconnect may still be consumed
        let new = gate.acquire().unwrap();
        assert!(!gate.owns(&old));
        assert!(gate.owns(&new));
    }

    #[test]
    fn old_record_event_does_not_complete_the_new_phone() {
        let gate = PhoneGate::default();
        let old = gate.acquire().unwrap();
        old.record();
        assert!(gate.recorded());
        gate.pause();
        gate.resume();
        let new = gate.acquire().unwrap();
        old.record();
        assert!(!gate.recorded());
        new.record();
        assert!(gate.recorded());
    }
}
