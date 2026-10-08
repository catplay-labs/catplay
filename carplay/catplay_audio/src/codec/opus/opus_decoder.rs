use super::OpusError;
use crate::{
    AudioStreamBasicDescription,
    codec::{AudioDecoder, AudioDecoderFactory},
};

pub struct OpusDecoder {
    opus: opus2::Decoder,
    channels: usize,
    sample_rate: usize,
}

impl AudioDecoderFactory for OpusDecoder {
    type Error = OpusError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let channels = asbd.channels_per_frame.clamp(1, 2) as usize;

        let opus = opus2::Decoder::new(
            asbd.sample_rate,
            if channels == 2 {
                opus2::Channels::Stereo
            } else {
                opus2::Channels::Mono
            },
        )?;

        Ok(Self {
            opus,
            channels,
            sample_rate: asbd.sample_rate as _,
        })
    }
}

impl AudioDecoder for OpusDecoder {
    type Error = OpusError;
    type Sample = i16;

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        let samples_per_channel_decoded = self.opus.decode(data, output, false)?;
        let samples = samples_per_channel_decoded * self.channels;
        if samples > output.len() {
            return Err(OpusError::Overflow(samples, output.len()));
        }

        Ok((samples, data.len()))
    }

    fn output_type(&self) -> AudioStreamBasicDescription {
        AudioStreamBasicDescription::fill_pcm(self.sample_rate as _, 16, 16, self.channels as _, false)
    }

    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        let concealed_samples = self.decode(&[], output)?.0;
        Ok(concealed_samples)
    }
}
