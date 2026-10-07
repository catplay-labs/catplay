#![cfg_attr(not(test), no_std)]

#[cfg(feature = "project")]
pub use catplay_reflect::field_project;

pub mod decoder;
pub mod files;
#[cfg(feature = "alloc")]
pub mod msg;
