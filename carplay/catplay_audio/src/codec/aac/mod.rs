#[cfg(feature = "aac_dec")]
mod aac_decoder;
#[cfg(feature = "aac_enc")]
mod aac_encoder;

#[cfg(feature = "aac_dec")]
pub use aac_decoder::*;
#[cfg(feature = "aac_enc")]
pub use aac_encoder::*;

#[derive(thiserror::Error, Debug, Clone)]
pub enum AacError {
    #[cfg(feature = "aac_dec")]
    #[error("{0}")]
    Aac(fdk_aac::dec::DecoderError),
    #[cfg(feature = "aac_enc")]
    #[error("AAC encoder: {0}")]
    Encoder(String),
    #[error("Unknown")]
    Unknown,
}

#[cfg(feature = "aac_dec")]
impl From<fdk_aac::dec::DecoderError> for AacError {
    fn from(value: fdk_aac::dec::DecoderError) -> Self {
        Self::Aac(value)
    }
}
