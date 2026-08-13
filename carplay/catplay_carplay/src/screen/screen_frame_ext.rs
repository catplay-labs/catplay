use std::time::{Duration, Instant};

use bytes::BytesMut;
use catplay_plist::{PlistSerializable, plist_struct};
use log::debug;

use crate::{
    clock::{MediaClock, NtpU64},
    screen::{ScreenFlag, ScreenFrame, ScreenOpCode, Value64},
    video::{
        AnnexBConverter, AvccConfig, AvccConfigExtended, EncodedVideoFrame, HevcNalType, NalChunk, NalError, NalType,
        VideoView,
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
        let mut me = if other.length_prefixed {
            Self::video_avcc(other.data, other.pts.0, clock)
        } else {
            Self::video_annexb_cached(other.data, nal_size_len, other.pts.0, clock, other.nal_offsets.unwrap_or_default())?
        };
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
            length_prefixed: false,
            chacha_tag_buf: self.chacha_tag_buf,
            header_buf: self.header_buf,
        }))
    }

    /// `pts` is the timestamp of the frame this configures the decoder for - an iPhone stamps the
    /// config frame with it rather than leaving the media clock at zero.
    pub fn config(avcc: &AvccConfigExtended, pts: Instant, clock: &dyn MediaClock) -> Self {
        let mut frame = ScreenFrame::default();
        let header = &mut frame.header;
        header.opcode = ScreenOpCode::VideoConfig;
        header.params[0] = Value64::from_u64(clock.encode_local(pts).0);
        header.params[1] = Value64::from_f32_floor(avcc.width, avcc.height);

        // What an iPhone sets, less the ones that depend on the caller. `Bit4` and `NoDisplaySleep`
        // are copied from its config frame rather than reasoned about - a receiver that keeps the
        // panel asleep shows exactly the black screen we were chasing.
        let mut flags = ScreenFlag::Encrypted | ScreenFlag::Bit4 | ScreenFlag::NoDisplaySleep;

        if avcc.respect_timestamps {
            flags |= ScreenFlag::RespectTimestamps;
        }

        // An iPhone fills both pairs with the same rect, so mirror that rather than guessing which
        // one a receiver reads.
        if let Some(view) = avcc.view {
            let origin = Value64::from_f32(view.origin_x, view.origin_y);
            let size = Value64::from_f32(view.width, view.height);
            header.params[3] = origin;
            header.params[4] = size;
            header.params[5] = origin;
            header.params[6] = size;
        }

        if avcc.hevc {
            if let Some(hvcc) = hvcc_config_serialize(&avcc.avcc) {
                flags |= ScreenFlag::UseFormatDescription;
                let stsd = hvcc_write_stsd_atom_from_old_format_with_tags(avcc.width, avcc.height, &hvcc, *b"hvc1", *b"hvcC");
                frame.data.extend_from_slice(&stsd);
            }
        } else if let Some(data) = avcc_config_serialize(&avcc.avcc) {
            // From AnnexB
            frame.data.extend_from_slice(&data);
        }

        frame.header.set_flags(flags);
        frame
    }

    /// Mark a frame the stream can open on. An iPhone sets this on the video frame following a
    /// config frame and on no other, so a receiver may well be waiting for it before it decodes.
    pub fn mark_opening_frame(&mut self) {
        self.header.small_param[0] = 0x10;
    }

    pub fn flags(&self) -> ScreenFlag {
        self.header.flags()
    }

    pub fn config_decode(&self, video_latency: Duration) -> Option<AvccConfigExtended> {
        if self.header.opcode != ScreenOpCode::VideoConfig {
            return None;
        }

        let (width, height) = self.header.params[1].as_f32_floor();
        let (origin_x, origin_y) = self.header.params[3].as_f32();
        let (view_width, view_height) = self.header.params[4].as_f32();
        let view = (view_width > 0.0 && view_height > 0.0).then_some(VideoView {
            origin_x,
            origin_y,
            width: view_width,
            height: view_height,
        });

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
            view,
        })
    }

    pub fn keep_alive() -> Self {
        let mut frame = ScreenFrame::default();
        let header = &mut frame.header;
        header.opcode = ScreenOpCode::KeepAlive;
        frame
    }

    /// The keep-alive an iPhone actually sends: the sender's own view of the stream, as a binary
    /// plist body. `params[14]` repeats the body length, as the phone's do.
    pub fn keep_alive_with_stats(stats: &ScreenSenderStats) -> Self {
        let Ok(body) = stats.pencode() else {
            return Self::keep_alive();
        };
        let mut frame = ScreenFrame::default();
        frame.header.opcode = ScreenOpCode::KeepAliveWithBody;
        frame.header.params[14] = Value64::from_f32(0.0, body.len() as f32);
        frame.data = body;
        frame
    }
}

// What the sender reports about itself alongside a keep-alive, named and cased as an iPhone names
// them. Two fields it also sends - `rttAvg` and `txCapacityAvg` - are absent because nothing here
// measures round-trip time or link capacity, and a zero would read as a perfect link rather than
// as no answer.
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

#[cfg(test)]
mod reference_tests {
    use super::*;
    use crate::clock::MediaClockSession;
    use crate::video::{AvccConfig, EncodedVideoFrame, Pts, VideoView, hvcc_config_serialize};

    /// The header and sample-entry bytes an iPhone sends this CFMOTO head unit, from
    /// `carjack/logs/cfmoto-iphone.pcapng`. Ours has to agree with it everywhere the value is not
    /// stream-specific - a config frame is the only thing the receiver sets its decoder up from,
    /// and a session where every other layer looks healthy but nothing appears is what a
    /// disagreement here costs.
    #[test]
    fn config_frame_matches_the_iphone_reference() {
        let config = AvccConfigExtended {
            hevc: true,
            avcc: AvccConfig {
                nal_size_len: 4,
                // VPS, SPS then PPS, Annex-B, as an encoder emits them.
                sps_pps: [
                    &[0u8, 0, 0, 1, 0x40, 0x01, 0x0c, 0x01][..],
                    &[0, 0, 0, 1, 0x42, 0x01, 0x01, 0x01][..],
                    &[0, 0, 0, 1, 0x44, 0x01, 0xc0, 0x76][..],
                ]
                .concat(),
            },
            video_latency: Duration::ZERO,
            width: 800,
            height: 1280,
            respect_timestamps: true,
            view: Some(VideoView {
                origin_x: 0.0,
                origin_y: 224.0,
                width: 800.0,
                height: 1056.0,
            }),
        };

        let frame = ScreenFrame::config(&config, Instant::now(), &MediaClockSession::new());

        // `1e 01` on the wire: RespectTimestamps | Encrypted | UseFormatDescription | Bit4 |
        // NoDisplaySleep, the last two only expressible since the field became 16-bit.
        assert_eq!(
            frame.header.small_param[1..],
            [0x1e, 0x01],
            "flags must match the iPhone's config frame"
        );

        // An `hvc1` entry naming HEVC, not the AVC compressor name this used to write.
        // idSize+cType+resvd+dataRefIndex+version+revision+vendor+quality*2+w+h+res*2+dataSize+frameCount
        let name_at = 50;
        assert_eq!(&frame.data[name_at..name_at + 5], b"\x04HEVC");

        // ... and the colour box the phone appends after the codec config.
        assert!(
            frame.data.windows(4).any(|w| w == b"colr"),
            "expected a colr atom"
        );
        assert!(frame.data.windows(4).any(|w| w == b"hvcC"));
    }

    /// The `hvcC` from the same capture. A receiver sets its decoder up from this record and
    /// nothing else, so round-tripping the phone's own parameter sets through our writer has to
    /// reproduce it exactly - profile, tier and level included, which the skeleton left at zero.
    #[test]
    fn hvcc_round_trips_the_iphone_reference() {
        const REFERENCE: &str = "010160000000b0000000000078f000fcfdf8f800000b03a00001001840010c01ffff0160000\
            00300b000000300000300780cc090a10001003e420101016000000300b00000030000030078a0064200501620\
            33b914862e7f13f0bfa1bf50ffaa08fd5413faaa0afd55417faaaa0cfd5554a6e021a02010a2000100074401c\
            072f05b24";
        let reference: Vec<u8> = REFERENCE
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();

        let parsed = hvcc_config_deserialize(&reference).expect("the phone's record parses");
        let ours = hvcc_config_serialize(&AvccConfig {
            nal_size_len: parsed.nal_size_len,
            sps_pps: parsed.vps_sps_pps,
        })
        .expect("and serializes back");

        assert_eq!(ours, reference, "hvcC must match the iPhone's byte for byte");
    }

    /// A sender whose encoder already emits AVCC/HVCC hands us the sample as it is; converting it
    /// again would read the length prefixes as start codes and corrupt every frame.
    #[test]
    fn a_length_prefixed_frame_is_passed_through() {
        let sample = [0u8, 0, 0, 4, 0x26, 0x01, 0xaf, 0xe0];
        let frame = EncodedVideoFrame {
            pts: Pts(Instant::now()),
            width: 800,
            height: 1280,
            data: BytesMut::from(&sample[..]),
            config: None,
            nal_offsets: None,
            is_keyframe: Some(true),
            length_prefixed: true,
            header_buf: BytesMut::new(),
            chacha_tag_buf: BytesMut::new(),
        };

        let screen = ScreenFrame::video_proxied(frame, 4, &MediaClockSession::new()).unwrap();

        assert_eq!(&screen.data[..], &sample[..]);
    }

    /// The phone's keep-alives carry its own view of the stream as a binary plist, with the body
    /// length repeated in `params[14]`. An empty `KeepAlive` is what this used to send.
    #[test]
    fn keep_alive_carries_stats_like_the_iphone() {
        let stats = ScreenSenderStats {
            tx_usage_avg: 4096.0,
            encoder_current_fps: 30,
            sent_frames_avg: 29,
            queued_frames_avg: 2,
            loss_avg: 0.0,
        };

        let frame = ScreenFrame::keep_alive_with_stats(&stats);

        assert_eq!(frame.header.opcode, ScreenOpCode::KeepAliveWithBody);
        assert_eq!(frame.header.params[14].as_f32(), (0.0, frame.data.len() as f32));
        assert!(frame.data.starts_with(b"bplist00"));
        assert_eq!(ScreenSenderStats::pdecode(&frame.data).unwrap(), stats);
        assert!(
            frame.data.windows(17).any(|w| w == b"encoderCurrentFPS"),
            "keys keep the phone's casing"
        );
    }
}
