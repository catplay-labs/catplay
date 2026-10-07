//! LingoKit schema generator and static iAP1 payload codecs.
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod lingos;
pub use lingos::lingo_0x0;
pub mod link_layer;
mod macros;
mod prim;
pub mod transaction_helper;
pub mod wire;
pub use wire::*;

#[doc(hidden)]
pub use catplay_reflect as __field_project;

#[doc(hidden)]
pub use alloc::string::String as __String;
#[doc(hidden)]
pub use alloc::vec::Vec as __Vec;
#[doc(hidden)]
pub use bitflags as __bitflags;

/// Internal formatting hook used to rename token records without nesting them.
#[doc(hidden)]
pub trait __NamedDebug {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
}

macro_rules! named_scalar_debug {
    ($($ty:ty),* $(,)?) => {
        $(impl __NamedDebug for $ty {
            fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(name).field(self).finish()
            }
        })*
    };
}
named_scalar_debug!(u8, u16, u32, u64, i8, i16, i32, i64, bool);

impl<const N: usize> __NamedDebug for [u8; N] {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl __NamedDebug for alloc::string::String {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl<T: core::fmt::Debug> __NamedDebug for alloc::vec::Vec<T> {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}
