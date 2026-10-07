#[cfg(feature = "iap2")]
pub mod iap2;
#[cfg(all(feature = "iap2", feature = "project"))]
pub mod iap2_strings;
#[cfg(feature = "iap2")]
pub use iap2::*;

#[cfg(feature = "iap2")]
mod iap2_ext;
