use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
    ptr,
};

use crate::{
    AudioCodec, AudioFormatFlags, AudioStreamBasicDescription,
    codec::{AudioEncoder, AudioEncoderFactory},
};

use super::AacError;

pub struct AacEncoder {
    encoder: FdkEncoder,
    input: AudioStreamBasicDescription,
    output: AudioStreamBasicDescription,
    frame_samples: usize,
    max_output_bytes: usize,
    pending: Vec<i16>,
}

struct FdkEncoder {
    handle: fdk_aac_sys::HANDLE_AACENCODER,
}

// The encoder handle is owned by one AacEncoder and calls require exclusive
// access through AudioEncoder::encode(&mut self).
unsafe impl Send for FdkEncoder {}

impl FdkEncoder {
    fn new(aot: fdk_aac_sys::AUDIO_OBJECT_TYPE, rate: u32, channels: u8, eld: bool) -> Result<Self, AacError> {
        let mut handle = ptr::null_mut();
        let result = unsafe { fdk_aac_sys::aacEncOpen(&mut handle, 0, 2) };
        check_fdk(result)?;
        let encoder = Self { handle };

        let params: [(fdk_aac_sys::AACENC_PARAM, u32); 7] = [
            (fdk_aac_sys::AACENC_PARAM_AACENC_AOT, aot as u32),
            (fdk_aac_sys::AACENC_PARAM_AACENC_BITRATE, 128_000),
            (fdk_aac_sys::AACENC_PARAM_AACENC_BITRATEMODE, 0),
            (fdk_aac_sys::AACENC_PARAM_AACENC_SAMPLERATE, rate),
            (fdk_aac_sys::AACENC_PARAM_AACENC_TRANSMUX, 0),
            (fdk_aac_sys::AACENC_PARAM_AACENC_SBR_MODE, 0),
            (fdk_aac_sys::AACENC_PARAM_AACENC_CHANNELMODE, if channels == 2 { 2 } else { 1 }),
        ];
        for (param, value) in params {
            check_fdk(unsafe { fdk_aac_sys::aacEncoder_SetParam(encoder.handle, param, value) })?;
        }
        if eld {
            check_fdk(unsafe { fdk_aac_sys::aacEncoder_SetParam(encoder.handle, fdk_aac_sys::AACENC_PARAM_AACENC_GRANULE_LENGTH, 480) })?;
        }
        check_fdk(unsafe { fdk_aac_sys::aacEncEncode(encoder.handle, ptr::null(), ptr::null(), ptr::null(), ptr::null_mut()) })?;
        Ok(encoder)
    }

    fn info(&self) -> Result<fdk_aac_sys::AACENC_InfoStruct, AacError> {
        let mut info = MaybeUninit::uninit();
        check_fdk(unsafe { fdk_aac_sys::aacEncInfo(self.handle, info.as_mut_ptr()) })?;
        Ok(unsafe { info.assume_init() })
    }

    fn encode(&self, input: &[i16], output: &mut [u8]) -> Result<(usize, usize), AacError> {
        let input_len = input.len().min(i32::MAX as usize) as i32;
        let mut input_ptr = input.as_ptr() as *mut i16 as *mut c_void;
        let mut input_ident = fdk_aac_sys::AACENC_BufferIdentifier_IN_AUDIO_DATA as i32;
        let mut input_size = input_len;
        let mut input_element_size = mem::size_of::<i16>() as i32;
        let input_desc = fdk_aac_sys::AACENC_BufDesc {
            numBufs: 1,
            bufs: &mut input_ptr,
            bufferIdentifiers: &mut input_ident,
            bufSizes: &mut input_size,
            bufElSizes: &mut input_element_size,
        };

        let mut output_ptr = output.as_mut_ptr() as *mut c_void;
        let mut output_ident = fdk_aac_sys::AACENC_BufferIdentifier_OUT_BITSTREAM_DATA as i32;
        let mut output_size = output.len().min(i32::MAX as usize) as i32;
        let mut output_element_size = mem::size_of::<u8>() as i32;
        let output_desc = fdk_aac_sys::AACENC_BufDesc {
            numBufs: 1,
            bufs: &mut output_ptr,
            bufferIdentifiers: &mut output_ident,
            bufSizes: &mut output_size,
            bufElSizes: &mut output_element_size,
        };
        let input_args = fdk_aac_sys::AACENC_InArgs {
            numInSamples: input_len,
            numAncBytes: 0,
        };
        let mut output_args = MaybeUninit::<fdk_aac_sys::AACENC_OutArgs>::zeroed();

        check_fdk(unsafe { fdk_aac_sys::aacEncEncode(self.handle, &input_desc, &output_desc, &input_args, output_args.as_mut_ptr()) })?;
        let output_args = unsafe { output_args.assume_init() };
        Ok((output_args.numOutBytes as usize, output_args.numInSamples as usize))
    }
}

impl Drop for FdkEncoder {
    fn drop(&mut self) {
        unsafe { fdk_aac_sys::aacEncClose(&mut self.handle) };
    }
}

fn check_fdk(result: fdk_aac_sys::AACENC_ERROR) -> Result<(), AacError> {
    if result == fdk_aac_sys::AACENC_ERROR_AACENC_OK {
        Ok(())
    } else {
        Err(AacError::Encoder(format!("FDK AAC encoder error {result}")))
    }
}

impl AudioEncoderFactory for AacEncoder {
    type Error = AacError;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let (aot, eld) = match output.format {
            AudioCodec::Mpeg4Aac => (fdk_aac_sys::AUDIO_OBJECT_TYPE_AOT_AAC_LC, false),
            AudioCodec::Mpeg4AacEld => (fdk_aac_sys::AUDIO_OBJECT_TYPE_AOT_ER_AAC_ELD, true),
            _ => return Err(AacError::Unknown),
        };
        let channels = input.channels();
        if input.format != AudioCodec::LinearPcm
            || !input.format_flags.contains(AudioFormatFlags::IS_SIGNED_INT)
            || input.bits_per_channel != 16
            || output.channels() != channels
            || output.sample_rate != input.sample_rate
        {
            return Err(AacError::Unknown);
        }

        match channels {
            1 | 2 => (),
            _ => return Err(AacError::Unknown),
        }
        let encoder = FdkEncoder::new(aot, input.sample_rate, channels, eld)?;
        let info = encoder.info()?;
        if eld && (info.confSize < 3 || info.confBuf[2] & 0x10 == 0) {
            return Err(AacError::Encoder("FDK did not apply the AAC-ELD 480-sample frameLengthFlag".into()));
        }
        let frame_samples = info.frameLength as usize * channels as usize;
        let max_output_bytes = info.maxOutBufBytes as usize;
        if frame_samples == 0 || max_output_bytes == 0 {
            return Err(AacError::Unknown);
        }

        let mut output = output;
        output.frames_per_packet = info.frameLength;
        output.bytes_per_packet = 0;

        Ok(Self {
            encoder,
            input,
            output,
            frame_samples,
            max_output_bytes,
            pending: Vec::with_capacity(frame_samples),
        })
    }
}

impl AudioEncoder for AacEncoder {
    type Error = AacError;
    type Sample = i16;

    fn input_type(&self) -> AudioStreamBasicDescription {
        self.input
    }

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.output
    }

    fn encode(&mut self, samples: &[i16], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        let mut consumed = 0;
        let mut written = 0;

        loop {
            if self.pending.len() < self.frame_samples {
                let to_copy = (self.frame_samples - self.pending.len()).min(samples.len() - consumed);
                self.pending
                    .extend_from_slice(&samples[consumed..consumed + to_copy]);
                consumed += to_copy;
                if self.pending.len() < self.frame_samples {
                    break;
                }
            }

            if output.len() - written < self.max_output_bytes {
                break;
            }

            let (output_size, input_consumed) = self.encoder.encode(&self.pending, &mut output[written..])?;
            let accepted = input_consumed.min(self.pending.len());
            if accepted > 0 {
                self.pending.drain(..accepted);
            }
            written += output_size;
            if accepted == 0 {
                break;
            }
        }

        Ok((written, consumed))
    }
}
