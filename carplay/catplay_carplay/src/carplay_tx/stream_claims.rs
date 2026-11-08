use std::sync::Arc;

/// Identity is retained by queued old drops, so allocator reuse cannot alias it.
#[derive(Clone)]
pub(super) struct StreamLease(Arc<()>);

pub(super) struct StreamClaims<K>(Vec<(K, StreamLease)>);

impl<K> Default for StreamClaims<K> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K: Copy + Eq> StreamClaims<K> {
    pub fn claim(&mut self, kind: K) -> StreamLease {
        self.0.retain(|(key, _)| *key != kind);
        let lease = StreamLease(Arc::new(()));
        self.0.push((kind, lease.clone()));
        lease
    }

    pub fn release(&mut self, kind: K, lease: &StreamLease) -> bool {
        let index = self
            .0
            .iter()
            .position(|(key, current)| *key == kind && Arc::ptr_eq(&current.0, &lease.0));
        if let Some(index) = index {
            self.0.remove(index);
            true
        } else {
            false
        }
    }

    pub fn revoke_all(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delayed_audio_or_screen_drop_cannot_release_a_new_stream() {
        let mut streams = StreamClaims::default();
        for kind in [0, 1, 2, 3] {
            let old = streams.claim(kind);
            streams.revoke_all();
            let new = streams.claim(kind);
            assert!(!streams.release(kind, &old));
            assert!(streams.release(kind, &new));
            assert!(!streams.release(kind, &new));
        }
    }

    #[test]
    fn unrelated_stream_release_does_not_remove_another_owner() {
        let mut streams = StreamClaims::default();
        let main = streams.claim(0);
        let alt = streams.claim(1);
        assert!(!streams.release(0, &alt));
        assert!(streams.release(0, &main));
        assert!(streams.release(1, &alt));
    }
}
