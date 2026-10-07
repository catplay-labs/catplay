// Let everything here live as catplay_csm::decoder:*

#[cfg(feature = "alloc")]
mod alloc;
mod packets;
mod prim;
mod wire;

#[cfg(feature = "alloc")]
pub use alloc::*;
pub use packets::*;
pub use prim::*;
pub use wire::*;
