#[cfg(feature = "opus_dec")]
mod opus_decoder;
#[cfg(feature = "opus_enc")]
mod opus_encoder;

#[cfg(feature = "opus_dec")]
pub use opus_decoder::*;
#[cfg(feature = "opus_enc")]
pub use opus_encoder::*;

#[derive(thiserror::Error, Debug, Clone)]
pub enum OpusError {
    #[error("{0}: {1}")]
    Opus(&'static str, &'static str),
    #[cfg(feature = "opus_dec")]
    #[error("Overflow (channel mismatch?): {0} > {1}")]
    Overflow(usize, usize),
}

impl From<opus2::Error> for OpusError {
    fn from(value: opus2::Error) -> Self {
        Self::Opus(value.function(), value.description())
    }
}
