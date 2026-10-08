use super::AacError;
use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{AudioDecoder, AudioDecoderFactory},
};

pub struct AacDecoder {
    aac: fdk_aac::dec::Decoder,
    rate: u32,
    channels: u32,
}

fn find_frequency_index(sample_rate: u32) -> Option<u8> {
    Some(match sample_rate {
        96000 => 0,
        88200 => 1,
        64000 => 2,
        48000 => 3,
        44100 => 4,
        32000 => 5,
        24000 => 6,
        22050 => 7,
        16000 => 8,
        12000 => 9,
        11025 => 10,
        8000 => 11,
        7350 => 12,
        _ => return None,
    })
}

fn make_asc_lc(object_type: u8, channel_config: u8, frequency_index: u8) -> [u8; 2] {
    let mut asc = [0u8; 2];
    asc[0] = (object_type << 3) | ((frequency_index & 0x0E) >> 1);
    asc[1] = ((frequency_index & 0x01) << 7) | ((channel_config & 0x0F) << 3);
    asc
}

fn make_asc_eld(_object_type: u8, channel_config: u8, frequency_index: u8) -> [u8; 4] {
    [
        0xF8,
        0xE0 | ((frequency_index & 0x0F) << 1) | ((channel_config & 0x08) >> 3),
        ((channel_config & 0x07) << 5) | (1 << 4),
        0,
    ]
}

impl AudioDecoder for AacDecoder {
    type Error = AacError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        AudioStreamBasicDescription::fill_pcm(self.rate, 16, 16, self.channels as _, false)
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        let consumed_bytes = self.aac.fill(data)?;
        self.aac.decode_frame(output)?;
        let samples = self.aac.decoded_frame_size();
        Ok((samples, consumed_bytes))
    }
}

impl AudioDecoderFactory for AacDecoder {
    type Error = AacError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let is_eld = match asbd.format {
            AudioCodec::Mpeg4Aac => false,
            AudioCodec::Mpeg4AacEld => true,
            _ => return Err(AacError::Unknown),
        };

        let channels = asbd.channels();
        if channels != 1 && channels != 2 {
            return Err(AacError::Unknown);
        }

        const AOT_AAC_LC: u8 = 2;
        const AOT_ER_AAC_ELD: u8 = 39;

        let freq_index = find_frequency_index(asbd.sample_rate).ok_or(AacError::Unknown)?;
        let mut aac = fdk_aac::dec::Decoder::new(fdk_aac::dec::Transport::Raw);
        if !is_eld {
            aac.config_raw(&make_asc_lc(AOT_AAC_LC, channels, freq_index))?;
        } else {
            aac.config_raw(&make_asc_eld(AOT_ER_AAC_ELD, channels, freq_index))?;
        }

        Ok(Self {
            aac,
            rate: asbd.sample_rate,
            channels: channels as _,
        })
    }
}
