//! iAP1 wire types, primitive payload codecs, framing and registered packet codecs.

mod bitflags;
mod context;
mod error;
mod frame;
pub mod packet;
mod predicate;
mod project;
mod reader;
pub mod registry;
mod traits;
mod writer;

pub use crate::prim::predicate::{PredicateValue, decode_length, encode_length, predicate_eval};
pub use bitflags::*;
pub use context::*;
pub use error::*;
pub use frame::*;
pub use packet::*;
pub use predicate::*;
pub use project::*;
pub use reader::*;
pub use registry::*;
pub use traits::*;
pub use writer::*;

#[cfg(test)]
mod tests;
