#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod cipher;

mod backend;
pub mod prefetch;
#[cfg(any(test, not(feature = "openssl")))]
mod ring_sha512;
mod ring_util;
mod storage;
mod xor_cipher;

pub use backend::*;
#[cfg(any(test, not(feature = "openssl")))]
pub(crate) use ring_sha512::*;
pub(crate) use ring_util::*;
pub use storage::*;

pub mod aes;
pub mod fast_chacha;
