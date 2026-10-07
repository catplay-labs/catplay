//! Background worker, cache state, and consumer path.
extern crate std;

mod consumer;
mod state;
#[cfg(test)]
mod tests;
mod wake;
mod worker;

pub(crate) use consumer::KeystreamPrefetch;
