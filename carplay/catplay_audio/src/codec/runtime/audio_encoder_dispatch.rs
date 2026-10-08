#[cfg(feature = "aac_enc")]
use crate::codec::aac::AacEncoder;
#[cfg(feature = "opus_enc")]
use crate::codec::opus::OpusEncoder;
use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{AudioEncoder, AudioEncoderFactory, pcm::PcmEncoder, runtime::CodecDispatchError},
};

pub struct AudioEncoderDispatch {
    codec: Dispatch,
}

enum Dispatch {
    #[cfg(feature = "aac_enc")]
    Aac(AacEncoder),
    #[cfg(feature = "opus_enc")]
    Opus(OpusEncoder),
    Pcm(PcmEncoder),
}

impl AudioEncoder for Dispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        match self {
            #[cfg(feature = "aac_enc")]
            Dispatch::Aac(enc) => enc.output_type(),
            #[cfg(feature = "opus_enc")]
            Dispatch::Opus(dec) => dec.output_type(),
            Dispatch::Pcm(dec) => dec.output_type(),
        }
    }

    fn input_type(&self) -> AudioStreamBasicDescription {
        match self {
            #[cfg(feature = "aac_enc")]
            Dispatch::Aac(enc) => enc.input_type(),
            #[cfg(feature = "opus_enc")]
            Dispatch::Opus(dec) => dec.input_type(),
            Dispatch::Pcm(dec) => dec.input_type(),
        }
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        match self {
            #[cfg(feature = "aac_enc")]
            Dispatch::Aac(enc) => Ok(enc.encode(samples, output)?),
            #[cfg(feature = "opus_enc")]
            Dispatch::Opus(dec) => Ok(dec.encode(samples, output)?),
            Dispatch::Pcm(dec) => Ok(dec.encode(samples, output)?),
        }
    }
}

impl AudioEncoderDispatch {
    pub fn has_runtime_support(codec: AudioCodec) -> bool {
        match codec {
            AudioCodec::LinearPcm => true,
            AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => cfg!(feature = "aac_enc"),
            AudioCodec::Opus => cfg!(feature = "opus_enc"),
            AudioCodec::AppleLossless => false,
        }
    }
}

impl AudioEncoderFactory for AudioEncoderDispatch {
    type Error = CodecDispatchError;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        if input.format != AudioCodec::LinearPcm {
            return Err(CodecDispatchError::UnsupportedCodec(input.format, output.format));
        }

        let codec = match output.format {
            AudioCodec::LinearPcm => Dispatch::Pcm(PcmEncoder::new(input, output)?),
            #[cfg(feature = "aac_enc")]
            AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => Dispatch::Aac(AacEncoder::new(input, output)?),
            #[cfg(feature = "opus_enc")]
            AudioCodec::Opus => Dispatch::Opus(OpusEncoder::new(input, output)?),
            _ => return Err(CodecDispatchError::UnsupportedCodec(input.format, output.format)),
        };
        Ok(Self { codec })
    }
}

impl AudioEncoder for AudioEncoderDispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn input_type(&self) -> AudioStreamBasicDescription {
        self.codec.input_type()
    }

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.codec.output_type()
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        self.codec.encode(samples, output)
    }
}
