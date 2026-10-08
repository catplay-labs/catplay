use alloc::string::String;
#[cfg(feature = "serde")]
use alloc::string::ToString;
use core::fmt;

/// Error returned by the local plist value and compatibility APIs.
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn message(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl core::error::Error for Error {}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::message(msg.to_string())
    }
}
