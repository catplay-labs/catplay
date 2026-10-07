//! Compile-time string pools.

pub(super) mod helpers;
mod macros;

use super::common::StaticHandle;

/// A statically known pool of UTF-8 strings. The handle is the pool type itself;
/// its `Raw` width selects the compact representation stored by clients.
pub trait StringPool: StaticHandle<Target = str> + core::fmt::Debug {
    type Handle: StaticHandle<Target = str, Raw = Self::Raw>;
}

impl<T> StringPool for T
where
    T: StaticHandle<Target = str> + core::fmt::Debug,
{
    type Handle = T;
}
