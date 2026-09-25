use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{AudioDecoder, AudioDecoderFactory},
};

pub struct AlacDecoder {
    alac: alac::Decoder,
    output: AudioStreamBasicDescription,
    max_samples_per_packet: usize,
    needs_i32_scratch: bool,
    scratch_i32: Vec<i32>,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum AlacError {
    #[error("{0}")]
    InvalidData(String),
    #[error("Unsupported ALAC stream: {0:?}")]
    UnsupportedFormat(AudioStreamBasicDescription),
    #[error("Output buffer too small: need at least {required} samples, got {actual}")]
    OutputTooSmall { required: usize, actual: usize },
}

impl From<alac::InvalidData> for AlacError {
    fn from(value: alac::InvalidData) -> Self {
        Self::InvalidData(value.to_string())
    }
}

impl AlacDecoder {
    fn stream_info_from_asbd(asbd: AudioStreamBasicDescription) -> Result<alac::StreamInfo, AlacError> {
        if asbd.format != AudioCodec::AppleLossless {
            return Err(AlacError::UnsupportedFormat(asbd));
        }

        if !matches!(asbd.channels_per_frame, 1 | 2) {
            return Err(AlacError::UnsupportedFormat(asbd));
        }

        if !matches!(asbd.bits_per_channel, 16 | 24) {
            return Err(AlacError::UnsupportedFormat(asbd));
        }

        let frame_length = if asbd.frames_per_packet == 0 { 352 } else { asbd.frames_per_packet };
        let fmtp = format!(
            "{} 0 {} 40 10 14 {} 255 0 0 {}",
            frame_length, asbd.bits_per_channel, asbd.channels_per_frame, asbd.sample_rate
        );
        alac::StreamInfo::from_sdp_format_parameters(&fmtp).map_err(AlacError::from)
    }

    fn decode_into_output(&mut self, data: &[u8], output: &mut [i16]) -> Result<usize, AlacError> {
        if output.len() < self.max_samples_per_packet {
            return Err(AlacError::OutputTooSmall {
                required: self.max_samples_per_packet,
                actual: output.len(),
            });
        }

        if !self.needs_i32_scratch {
            let decoded = self
                .alac
                .decode_packet(data, &mut output[..self.max_samples_per_packet])?;
            return Ok(decoded.len());
        }

        let decoded = self
            .alac
            .decode_packet(data, &mut self.scratch_i32[..self.max_samples_per_packet])?;
        for (dst, &src) in output.iter_mut().zip(decoded.iter()) {
            let v = (src >> 8).clamp(i16::MIN as i32, i16::MAX as i32);
            *dst = v as i16;
        }
        Ok(decoded.len())
    }
}

impl AudioDecoderFactory for AlacDecoder {
    type Error = AlacError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let stream_info = Self::stream_info_from_asbd(asbd)?;
        let max_samples_per_packet = stream_info.max_samples_per_packet() as usize;
        let needs_i32_scratch = stream_info.bit_depth() > 16;
        let scratch_i32 = if needs_i32_scratch {
            vec![0; max_samples_per_packet]
        } else {
            Vec::new()
        };

        Ok(Self {
            alac: alac::Decoder::new(stream_info),
            output: AudioStreamBasicDescription::fill_pcm(asbd.sample_rate, 16, 16, asbd.channels() as _, false),
            max_samples_per_packet,
            needs_i32_scratch,
            scratch_i32,
        })
    }
}

impl AudioDecoder for AlacDecoder {
    type Error = AlacError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.output
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        match self.decode_into_output(data, output) {
            Ok(samples) => Ok((samples, data.len())),
            Err(_) if data.len() > 4 => {
                let samples = self.decode_into_output(&data[4..], output)?;
                Ok((samples, data.len()))
            }
            Err(err) => Err(err),
        }
    }
}
