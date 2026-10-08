//! Allocation-free binary plist wire format, independent of Serde.

pub mod decode;
pub mod encode;
mod format;
pub mod scalar;
pub mod xml;
