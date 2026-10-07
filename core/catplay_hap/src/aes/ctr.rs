use super::{Aes128CtrBackend, Aes128CtrEngine, Aes128CtrPrefetch, PrefetchError, PrefetchResult, PrefetchStats, PrefetchStatus};

/// Thin crypto wrapper so we can swap implementations per-arch later.
pub struct Aes128Ctr {
    inner: Aes128CtrPrefetch<Aes128CtrBackend>,
}

impl Aes128Ctr {
    pub fn new(key: &[u8; 16], iv: &[u8; 16], prefetch: usize) -> Self {
        Self {
            inner: Aes128CtrPrefetch::new(*key, *iv, prefetch).expect("AES-CTR prefetch initialization failed"),
        }
    }

    pub fn apply_keystream(
        &mut self,
        data: &mut [u8],
    ) -> Result<PrefetchResult, PrefetchError<<Aes128CtrBackend as Aes128CtrEngine>::Error>> {
        self.inner.apply_keystream(data)
    }

    pub fn status(&self) -> PrefetchStatus {
        self.inner.status()
    }

    pub fn finish_prefetch_cycle(&mut self) -> PrefetchStats {
        self.inner.finish_cycle()
    }
}
