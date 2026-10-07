//! Static string and object pools and their shared handle infrastructure.

mod common;
mod object;
mod string;

pub use common::StaticHandle;
pub use object::{HandleShared, PoolHandle, StaticObjectPool};
pub use string::StringPool;

/// Implementation details used by exported macros; not a stable public API.
#[doc(hidden)]
pub mod __private {
    pub use super::common::index::{Index, Index8, Index16};
    pub use super::string::helpers::*;
}
