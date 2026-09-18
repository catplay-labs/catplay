use std::time::{Duration, Instant};

use bytes::BytesMut;
use catplay_plist::{PlistSerializable, plist_struct};
use log::{debug, warn};

use crate::{
    clock::{MediaClock, NtpU64},
    screen::{ScreenFlag, ScreenFrame, ScreenOpCode, Value64},
    video::{
        AnnexBConverter, AvccConfig, AvccConfigExtended, EncodedVideoFrame, HevcNalType, NalChunk, NalError, NalType,
        avcc_config_deserialize, avcc_config_serialize, hvcc_config_deserialize, hvcc_config_serialize,
        hvcc_sample_entry_extract_codec_config, hvcc_write_stsd_atom_from_old_format_with_tags,
    },
};

impl ScreenFrame {
    fn scan(&mut self, nal_size_len: usize, from_length_prefixed: bool) -> Result<(), NalError> {
        self.nal_offset_cache = AnnexBConverter::scan_only(&self.data, nal_size_len, from_length_prefixed)?;
        Ok(())
    }

    fn convert_startcodes(&mut self, nal_size_len: usize, to_annexb: bool) -> Result<(), NalError> {
        if to_annexb {
            // Direction: HVCC/AVCC -> AnnexB
            debug!("HVCC/AVCC -> AnnexB; cache size {}", self.nal_offset_cache.len());
            let mut converter = AnnexBConverter::with_cache(nal_size_len, self.nal_offset_cache.clone());
            self.data = converter.convert(&mut self.data, false)?;
        } else {
            // Direction: AnnexB -> HVCC/AVCC
            if nal_size_len != 4 {
                // In transmit mode, we don't support legacy nal_size_len configurations != 4
                return Err(NalError::Param);
            }
            debug!("AnnexB -> HVCC/AVCC; cache size {}", self.nal_offset_cache.len());

            let mut converter = AnnexBConverter::with_cache(nal_size_len, self.nal_offset_cache.clone());
            self.data = converter.convert(&mut self.data, true)?;
        }
        Ok(())
    }

    /// Convert [EncodedVideoFrame] back to [ScreenFrame] in an optimal fashion.
    pub fn video_proxied(other: EncodedVideoFrame, nal_size_len: usize, clock: &dyn MediaClock) -> Result<Self, NalError> {
        let mut me = Self::video_annexb_cached(other.data, nal_size_len, other.pts.0, clock, other.nal_offsets.unwrap_or_default())?;
        me.header_buf = other.header_buf;
        me.chacha_tag_buf = other.chacha_tag_buf;

        debug!(
            "<- ScreenFrame header={:?} data={:?} chacha={:?} nals={}",
            me.header_buf.as_ptr(),
            me.data.as_ptr(),
            me.chacha_tag_buf.as_ptr(),
            me.nal_offset_cache.len()
        );

        Ok(me)
    }

    /// Create video frame from AnnexB buffer, converting it to AVCC/HVCC.
    pub fn video_annexb(data: BytesMut, nal_size_len: usize, pts: Instant, clock: &dyn MediaClock) -> Result<Self, NalError> {
        Self::video_annexb_cached(data, nal_size_len, pts, clock, Vec::new())
    }

    /// Create video frame from AnnexB buffer, converting it to AVCC/HVCC using cached NAL offsets.
    pub fn video_annexb_cached(
        data: BytesMut,
        nal_size_len: usize,
        pts: Instant,
        clock: &dyn MediaClock,
        cache: Vec<NalChunk>,
    ) -> Result<Self, NalError> {
        let mut me = Self::video_avcc(data, pts, clock);
        me.nal_offset_cache = cache;
        me.convert_startcodes(nal_size_len, false)?;
        Ok(me)
    }

    /// Create video frame from AVCC/HVCC buffer.
    pub fn video_avcc(data: BytesMut, pts: Instant, clock: &dyn MediaClock) -> Self {
        let mut frame: ScreenFrame = ScreenFrame::default();
        let header = &mut frame.header;
        header.opcode = ScreenOpCode::VideoFrame;
        header.params[0] = Value64::from_u64(clock.encode_local(pts).0);

        frame.data = data;
        frame
    }

    /// Decode AVCC/HVCC video frame and convert payload to AnnexB.
    pub fn video_decode(mut self, config: &AvccConfigExtended, clock: &dyn MediaClock) -> Result<Option<EncodedVideoFrame>, NalError> {
        if self.header.opcode != ScreenOpCode::VideoFrame || self.data.is_empty() {
            return Ok(None);
        }

        let nal_size_len = config.avcc.nal_size_len;
        let pts = self.header.params[0].as_u64();
        let now = Instant::now();
        let mut pts_decoded = clock.decode_remote(NtpU64(pts));
        if !config.respect_timestamps {
            pts_decoded.replace(now);
        }

        let nal_byte = self.data.get(nal_size_len).copied().unwrap_or(0);
        let is_keyframe = match config.hevc {
            true => {
                let is_keyframe = HevcNalType::from_byte(nal_byte).is_irap();
                let nal_type = NalType::from_byte(nal_byte);
                debug!("nal_type: {nal_type:?} is_keyframe={is_keyframe}");
                is_keyframe
            }
            false => {
                let nal_byte = self.data.get(config.avcc.nal_size_len).copied().unwrap_or(0);
                let nal_type = NalType::from_byte(nal_byte);
                let is_keyframe = nal_type.is_keyframe();
                debug!("nal_type: {nal_type:?} is_keyframe={is_keyframe}");
                is_keyframe
            }
        };

        self.scan(nal_size_len, true)?;
        self.convert_startcodes(nal_size_len, true)?;

        debug!(
            "-> ScreenFrame header={:?} data={:?} chacha={:?} nals={}",
            self.header_buf.as_ptr(),
            self.data.as_ptr(),
            self.chacha_tag_buf.as_ptr(),
            self.nal_offset_cache.len()
        );

        Ok(Some(EncodedVideoFrame {
            pts: pts_decoded.unwrap_or(now).into(),
            width: config.width,
            height: config.height,
            data: self.data,
            config: Some(config.clone()),
            nal_offsets: Some(self.nal_offset_cache),
            is_keyframe: Some(is_keyframe),
            chacha_tag_buf: self.chacha_tag_buf,
            header_buf: self.header_buf,
        }))
    }

    pub fn config(avcc: &AvccConfigExtended) -> Self {
        let mut frame = ScreenFrame::default();
        let header = &mut frame.header;
        header.opcode = ScreenOpCode::VideoConfig;
        header.params[1] = Value64::from_f32_floor(avcc.width, avcc.height);
        header.small_param[1] |= ScreenFlag::Encrypted.bits();

        if avcc.respect_timestamps {
            header.small_param[1] |= ScreenFlag::RespectTimestamps.bits();
        }

        if avcc.hevc {
            if let Some(hvcc) = hvcc_config_serialize(&avcc.avcc) {
                header.small_param[1] |= ScreenFlag::UseFormatDescription.bits();
                let stsd = hvcc_write_stsd_atom_from_old_format_with_tags(avcc.width, avcc.height, &hvcc, *b"hvc1", *b"hvcC");
                frame.data.extend_from_slice(&stsd);
            }
        } else if let Some(data) = avcc_config_serialize(&avcc.avcc) {
            // From AnnexB
            frame.data.extend_from_slice(&data);
        }

        frame
    }

    pub fn flags(&self) -> ScreenFlag {
        ScreenFlag::from_bits_truncate(self.header.small_param[1])
    }

    pub fn config_decode(&self, video_latency: Duration) -> Option<AvccConfigExtended> {
        if self.header.opcode != ScreenOpCode::VideoConfig {
            return None;
        }

        let (width, height) = self.header.params[1].as_f32_floor();
        /*let (view_origin_x, view_origin_y) = self.header.params[3].as_f32();
        let (view_origin_x, view_origin_y) = (view_origin_x.floor() as u32, view_origin_y.floor() as u32);
        let (view_width, view_height) = self.header.params[4].as_f32();
        let (view_width, view_height) = (view_width.floor() as u32, view_height.floor() as u32);*/

        let flags = self.flags();
        let (avcc, hevc) = if flags.contains(ScreenFlag::UseFormatDescription) {
            let (tag, payload) = hvcc_sample_entry_extract_codec_config(&self.data)?;
            if &tag == b"avcC" {
                (avcc_config_deserialize(&payload)?, false)
            } else if &tag == b"hvcC" {
                let hvcc = hvcc_config_deserialize(&payload)?;
                (
                    AvccConfig {
                        nal_size_len: hvcc.nal_size_len,
                        sps_pps: hvcc.vps_sps_pps,
                    },
                    true,
                )
            } else {
                return None;
            }
        } else {
            (avcc_config_deserialize(&self.data)?, false)
        };

        Some(AvccConfigExtended {
            hevc,
            avcc,
            video_latency,
            width,
            height,
            respect_timestamps: flags.contains(ScreenFlag::RespectTimestamps),
        })
    }

    pub fn keep_alive() -> Self {
        let mut frame = ScreenFrame::default();
        let header = &mut frame.header;
        header.opcode = ScreenOpCode::KeepAlive;
        frame
    }

    pub fn keep_alive_with_stats(stats: &ScreenSenderStats) -> Self {
        let Ok(body) = stats.pencode() else {
            return Self::keep_alive();
        };
        let mut frame = ScreenFrame::default();
        frame.header.opcode = ScreenOpCode::KeepAliveWithBody;
        //Observed that iphone sends param[14] as frame len
        frame.header.params[14] = Value64::from_f32(0.0, body.len() as f32);
        frame.data = body;
        frame
    }

    pub fn sender_stats_decode(&self) -> Option<ScreenSenderStats> {
        if self.header.opcode != ScreenOpCode::KeepAliveWithBody {
            return None;
        }

        match ScreenSenderStats::pdecode(&self.data) {
            Ok(stats) => Some(stats),
            Err(e) => {
                warn!("Unreadable keep-alive body ({} bytes): {e:?}", self.data.len());
                None
            }
        }
    }
}

plist_struct! {
    pub struct ScreenSenderStats {
        /// Bytes per second written to the screen socket over the last interval.
        pub tx_usage_avg: f64,
        #[serde(rename = "encoderCurrentFPS")]
        pub encoder_current_fps: u32,
        pub sent_frames_avg: u32,
        pub queued_frames_avg: u32,
        pub loss_avg: f64,
    }
}
