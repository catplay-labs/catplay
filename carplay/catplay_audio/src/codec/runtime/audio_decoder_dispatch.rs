#[cfg(feature = "aac_dec")]
use crate::codec::aac::AacDecoder;
#[cfg(any(feature = "aac_enc", feature = "aac_dec"))]
use crate::codec::aac::AacError;
#[cfg(feature = "alac")]
use crate::codec::alac::{AlacDecoder, AlacError};
#[cfg(feature = "opus_dec")]
use crate::codec::opus::OpusDecoder;
#[cfg(any(feature = "opus_enc", feature = "opus_dec"))]
use crate::codec::opus::OpusError;
use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{
        AudioDecoder, AudioDecoderFactory,
        pcm::{PcmDecoder, PcmError},
    },
};

pub struct AudioDecoderDispatch {
    codec: Dispatch,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum CodecDispatchError {
    #[cfg(any(feature = "aac_enc", feature = "aac_dec"))]
    #[error("{0}")]
    Aac(#[from] AacError),
    #[cfg(feature = "alac")]
    #[error("{0}")]
    Alac(#[from] AlacError),
    #[cfg(any(feature = "opus_enc", feature = "opus_dec"))]
    #[error("{0}")]
    Opus(#[from] OpusError),
    #[error("{0}")]
    Pcm(#[from] PcmError),

    #[error("Unsupported codec for {0:?} -> {1:?}")]
    UnsupportedCodec(AudioCodec, AudioCodec),
}
enum Dispatch {
    #[cfg(feature = "aac_dec")]
    Aac(AacDecoder),
    #[cfg(feature = "alac")]
    Alac(AlacDecoder),
    #[cfg(feature = "opus_dec")]
    Opus(OpusDecoder),
    Pcm(PcmDecoder),
}

impl AudioDecoder for Dispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        match self {
            #[cfg(feature = "aac_dec")]
            Dispatch::Aac(dec) => dec.output_type(),
            #[cfg(feature = "alac")]
            Dispatch::Alac(dec) => dec.output_type(),
            #[cfg(feature = "opus_dec")]
            Dispatch::Opus(dec) => dec.output_type(),
            Dispatch::Pcm(dec) => dec.output_type(),
        }
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        match self {
            #[cfg(feature = "aac_dec")]
            Dispatch::Aac(dec) => Ok(dec.decode(data, output)?),
            #[cfg(feature = "alac")]
            Dispatch::Alac(dec) => Ok(dec.decode(data, output)?),
            #[cfg(feature = "opus_dec")]
            Dispatch::Opus(dec) => Ok(dec.decode(data, output)?),
            Dispatch::Pcm(dec) => Ok(dec.decode(data, output)?),
        }
    }

    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        match self {
            #[cfg(feature = "aac_dec")]
            Dispatch::Aac(dec) => Ok(dec.conceal_lost_packet(output)?),
            #[cfg(feature = "alac")]
            Dispatch::Alac(dec) => Ok(dec.conceal_lost_packet(output)?),
            #[cfg(feature = "opus_dec")]
            Dispatch::Opus(dec) => Ok(dec.conceal_lost_packet(output)?),
            Dispatch::Pcm(dec) => Ok(dec.conceal_lost_packet(output)?),
        }
    }
}

impl AudioDecoderDispatch {
    pub fn has_runtime_support(codec: AudioCodec) -> bool {
        match codec {
            AudioCodec::LinearPcm => true,
            AudioCodec::AppleLossless => cfg!(feature = "alac"),
            AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => cfg!(feature = "aac_dec"),
            AudioCodec::Opus => cfg!(feature = "opus_dec"),
        }
    }
}

impl AudioDecoderFactory for AudioDecoderDispatch {
    type Error = CodecDispatchError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let codec = match asbd.format {
            AudioCodec::LinearPcm => Dispatch::Pcm(PcmDecoder::new(asbd)?),
            #[cfg(feature = "alac")]
            AudioCodec::AppleLossless => Dispatch::Alac(AlacDecoder::new(asbd)?),
            #[cfg(feature = "aac_dec")]
            AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => Dispatch::Aac(AacDecoder::new(asbd)?),
            #[cfg(feature = "opus_dec")]
            AudioCodec::Opus => Dispatch::Opus(OpusDecoder::new(asbd)?),
            #[cfg(not(all(feature = "aac_dec", feature = "opus_dec", feature = "alac")))]
            _ => return Err(CodecDispatchError::UnsupportedCodec(asbd.format, AudioCodec::LinearPcm)),
        };
        Ok(Self { codec })
    }
}

impl AudioDecoder for AudioDecoderDispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.codec.output_type()
    }

    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        self.codec.conceal_lost_packet(output)
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        self.codec.decode(data, output)
    }
}
