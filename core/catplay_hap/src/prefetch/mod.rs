//! Shared scheduling and reporting for keystream prefetchers.

use core::fmt;

/// A snapshot of the active prefetch target.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PrefetchStatus {
    pub nonce: u64,
    /// Continuous cache capacity or finite target length.
    pub requested_bytes: usize,
    pub available_bytes: usize,
    pub consumed_position: u128,
    pub generated_position: u128,
    /// Cached bytes used so far in the active cycle.
    pub used_bytes: usize,
}

/// Counters for one completed prefetch cycle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PrefetchStats {
    pub nonce: u64,
    pub requested_bytes: usize,
    /// Payload bytes processed in this cycle, including synchronous fallback.
    pub consumed_bytes: usize,
    pub generated_bytes: usize,
    pub used_bytes: usize,
    pub synchronous_bytes: usize,
    /// Completed output discarded at finite-cycle reset; zero for continuous cycles.
    pub dropped_bytes: usize,
    pub discarded_in_flight_bytes: usize,
}

impl fmt::Display for PrefetchStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hit = 100.0 * self.used_bytes as f64 / self.consumed_bytes.max(1) as f64;
        let dropped = 100.0 * self.dropped_bytes as f64 / self.generated_bytes.max(1) as f64;
        write!(
            f,
            "consumed={} cached={} synchronous={} cache_hit={hit:.1}% generated={} dropped={} drop={dropped:.1}% discarded_in_flight={} target={}",
            self.consumed_bytes,
            self.used_bytes,
            self.synchronous_bytes,
            self.generated_bytes,
            self.dropped_bytes,
            self.discarded_in_flight_bytes,
            self.requested_bytes
        )
    }
}

/// Provides keystream bytes and applies them to caller-owned data.
pub(crate) trait KeystreamProvider: Clone + Send + 'static {
    type Error;

    /// Write the raw keystream at the requested nonce and byte offset.
    /// Implementations must not assume that `out` is initially zeroed.
    fn generate_keystream(&mut self, nonce: u64, offset: u128, out: &mut [u8]) -> Result<(), Self::Error>;

    /// XOR the keystream at the requested position into `data` in place.
    fn apply_keystream(&mut self, nonce: u64, offset: u128, data: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy)]
pub(crate) enum Schedule {
    /// Keep this many useful bytes queued ahead of the consumer.
    Continuous { capacity: usize },
    /// Generate offsets only up to `target` for the current nonce.
    Finite { target: usize },
}

#[derive(Debug)]
pub(crate) enum ApplyError<E> {
    Provider(E),
    PositionOverflow,
}

/// How many bytes of one request came from the cache or synchronous generation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PrefetchResult {
    pub cached_bytes: usize,
    pub synchronous_bytes: usize,
}

/// Failure to start a background worker. This cannot occur without `std`.
#[derive(Debug, Clone, Copy)]
pub struct PrefetchStartError;

impl fmt::Display for PrefetchStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("prefetch worker could not start")
    }
}

#[cfg(feature = "std")]
mod threaded;
#[cfg(feature = "std")]
pub(crate) use threaded::KeystreamPrefetch;
#[cfg(not(feature = "std"))]
mod synchronous;
#[cfg(not(feature = "std"))]
pub(crate) use synchronous::KeystreamPrefetch;
