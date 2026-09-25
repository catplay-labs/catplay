use std::ptr;

use bytes::BytesMut;
use catplay_tracing_macro::trace_time;
use log::debug;
use x264_sys::*;

use crate::{H264Encoder, H264FrameBufferError, YuvShadowBuffer, carplay_helpers::CarPlayNalRewriter};

const CARPLAY_LEVELS: [((i32, i32, i32), i32); 8] = [
    ((800, 480, 30), 31),
    ((800, 480, 60), 31),
    ((960, 540, 30), 31),
    ((960, 540, 60), 32),
    ((1280, 720, 30), 31),
    ((1280, 720, 60), 32),
    ((1920, 720, 30), 40),
    ((1920, 720, 60), 42),
];

pub struct X264FrameBuffer {
    encoder: *mut x264_t,
    yuv: YuvShadowBuffer,
}

unsafe impl Send for X264FrameBuffer {}

impl X264FrameBuffer {
    pub fn new(width: i32, height: i32, fps: i32) -> Result<Self, H264FrameBufferError> {
        let mut param: x264_param_t = unsafe { std::mem::zeroed() };
        let ret = unsafe { x264_param_default_preset(&mut param, c"ultrafast".as_ptr() as _, c"zerolatency".as_ptr() as _) };

        if ret < 0 {
            return Err(H264FrameBufferError::X264DefaultPreset(ret));
        }

        param.i_keyint_max = 60;
        param.i_keyint_min = 1;
        param.i_scenecut_threshold = 0;
        param.b_intra_refresh = 0;
        param.i_bframe = 0;
        param.i_bframe_adaptive = 0;
        param.i_frame_reference = 1;
        param.i_slice_count = 1;
        param.i_slice_count_max = 1;
        param.i_dpb_size = 1;

        param.b_cabac = 0;
        param.b_aud = 0;
        param.b_repeat_headers = 0;
        param.b_annexb = 1;
        param.b_deblocking_filter = 0;
        param.b_open_gop = 0;
        param.b_vfr_input = 0;

        param.i_sync_lookahead = 0;
        param.rc.i_lookahead = 0;
        param.rc.i_rc_method = X264_RC_CQP as _;
        param.rc.i_qp_constant = 26;
        param.rc.b_mb_tree = 0;
        param.rc.i_aq_mode = 0;

        param.i_width = width;
        param.i_height = height;
        param.i_csp = X264_CSP_I420 as _;
        param.i_bframe_pyramid = 0;

        param.analyse.intra = 0;
        param.analyse.inter = 0;
        param.analyse.i_me_method = 0;
        param.analyse.i_subpel_refine = 0;
        param.analyse.i_trellis = 0;
        param.analyse.b_transform_8x8 = 1;
        param.analyse.b_psy = 0;
        param.analyse.b_psnr = 0;
        param.analyse.b_ssim = 0;
        param.analyse.i_direct_mv_pred = 0;
        param.analyse.b_chroma_me = 0;
        param.analyse.b_fast_pskip = 0;
        param.analyse.b_dct_decimate = 0;
        param.analyse.b_mixed_references = 0;

        param.i_threads = 1;
        param.i_lookahead_threads = 0;
        param.b_sliced_threads = 0;

        debug!("x264 params pre-match {param:?}");
        Self::tune_carplay_match(&mut param, width, height, fps);
        debug!("x264 params post-match {param:?}");

        let ret = unsafe { x264_param_apply_profile(&mut param, c"high".as_ptr() as _) };
        if ret < 0 {
            return Err(H264FrameBufferError::X264ApplyProfile(ret));
        }

        // Keep the source parameters honest even though the NAL rewriter also
        // emits the CarPlay VUI explicitly. x264 may derive a timebase again
        // internally for CFR input, so the rewritten SPS remains authoritative.
        param.b_vfr_input = 0;
        param.i_timebase_num = 0;
        param.i_timebase_den = 0;

        let encoder = unsafe { x264_encoder_open(&mut param) };
        if encoder.is_null() {
            return Err(H264FrameBufferError::X264EncoderOpenNull);
        }

        Ok(Self {
            encoder,
            yuv: YuvShadowBuffer::i420(width as _, height as _),
        })
    }

    pub fn tune_carplay_match(param: &mut x264_param_t, width: i32, height: i32, fps: i32) {
        param.b_cabac = 1;

        // Apple advertises four decoded reference slots in SPS, but only one
        // default active L0 reference in PPS. x264 derives those independently
        // from i_dpb_size and i_frame_reference, respectively.
        param.i_frame_reference = 1;
        param.i_dpb_size = 4;

        param.i_bframe = 0;
        param.i_bframe_adaptive = 0;
        param.i_bframe_pyramid = 0;

        param.b_interlaced = 0;
        param.b_fake_interlaced = 0;
        param.b_tff = 0;

        param.i_level_idc = CARPLAY_LEVELS
            .iter()
            .find_map(|&((candidate_width, candidate_height, candidate_fps), level)| {
                (candidate_width == width && candidate_height == height && candidate_fps == fps).then_some(level)
            })
            .unwrap_or(param.i_level_idc);

        param.vui.i_vidformat = 5;
        param.vui.b_fullrange = 1;
        param.vui.i_colorprim = 1;
        param.vui.i_transfer = 13;
        param.vui.i_colmatrix = 6;

        param.vui.i_chroma_loc = 0;
        param.vui.i_sar_width = 0;
        param.vui.i_sar_height = 0;

        // x264 restores the FPS timebase for CFR input. VFR would suppress the
        // fixed-rate flag but also adds one frame of delay, which breaks the
        // one-shot UI encoder. Removing timing_info entirely needs SPS rewriting
        // or a patched x264.
        param.b_vfr_input = 0;

        param.analyse.i_weighted_pred = 0;
        param.analyse.b_weighted_bipred = 0;

        param.b_deblocking_filter = 0;
        param.analyse.b_transform_8x8 = 1;

        param.b_open_gop = 0;
        param.b_intra_refresh = 0;

        param.i_keyint_min = 1;
        param.i_keyint_max = 60;
        param.i_scenecut_threshold = 0;
    }

    pub fn update_rgba(&mut self, rgba: &[u8], out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        <Self as H264Encoder>::update_rgba(self, rgba, out)
    }

    pub fn get_headers(&mut self, out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        <Self as H264Encoder>::get_headers(self, out)
    }

    fn transfer_nals(&mut self, pp_nal: *mut x264_nal_t, pi_nal: usize, out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        let mut size = 0;
        for i in 0..pi_nal {
            let nal = unsafe { *pp_nal.add(i) };
            size += nal.i_payload as usize;
        }

        out.reserve(size);

        for i in 0..pi_nal {
            let nal = unsafe { *pp_nal.add(i) };
            let slice = unsafe { std::slice::from_raw_parts(nal.p_payload, nal.i_payload as usize) };
            match nal.i_type as u32 {
                6 => {}
                7 => out.extend_from_slice(&CarPlayNalRewriter::rewrite_sps(slice)?),
                5 => out.extend_from_slice(&CarPlayNalRewriter::rewrite_idr(slice)?),
                _ => out.extend_from_slice(slice),
            }
        }
        Ok(())
    }
}

impl H264Encoder for X264FrameBuffer {
    #[trace_time]
    fn update_rgba(&mut self, rgba: &[u8], out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        self.yuv.update(rgba)?;
        let layout = self.yuv.layout();
        let mut pic_in: x264_picture_t = unsafe { std::mem::zeroed() };

        pic_in.img.i_stride[0] = layout.width as _;
        pic_in.img.i_stride[1] = layout.chroma_width as _;
        pic_in.img.i_stride[2] = layout.chroma_width as _;

        let [y, u, v] = self.yuv.yuv_slices_mut();

        pic_in.img.plane[0] = y.as_ptr() as _;
        pic_in.img.plane[1] = u.as_ptr() as _;
        pic_in.img.plane[2] = v.as_ptr() as _;

        pic_in.img.i_csp = X264_CSP_I420 as _;
        pic_in.img.i_plane = 3;
        pic_in.i_type = X264_TYPE_IDR as _;

        let mut pic_out: x264_picture_t = unsafe { std::mem::zeroed() };

        let mut pp_nal: *mut x264_nal_t = ptr::null_mut();
        let mut pi_nal = 0;

        let ret = unsafe { x264_encoder_encode(self.encoder, &mut pp_nal, &mut pi_nal, &mut pic_in, &mut pic_out) };
        if ret < 0 {
            return Err(H264FrameBufferError::X264EncoderEncode(ret));
        }

        self.transfer_nals(pp_nal, pi_nal as _, out)?;
        Ok(())
    }

    fn get_headers(&mut self, out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        let mut pp_nal: *mut x264_nal_t = ptr::null_mut();
        let mut pi_nal = 0;

        let ret = unsafe { x264_encoder_headers(self.encoder, &mut pp_nal, &mut pi_nal) };

        if ret < 0 {
            return Err(H264FrameBufferError::X264EncoderHeaders(ret));
        }

        self.transfer_nals(pp_nal, pi_nal as _, out)?;
        Ok(())
    }
}

impl Drop for X264FrameBuffer {
    fn drop(&mut self) {
        if !self.encoder.is_null() {
            unsafe { x264_encoder_close(self.encoder) };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use insta::assert_debug_snapshot;

    use super::*;

    #[allow(dead_code)]
    #[derive(Debug)]
    struct SpsPpsSnapshot {
        width: i32,
        height: i32,
        fps: i32,
        level: i32,
        sps: String,
        pps: String,
    }

    fn annexb_nals(data: &[u8]) -> Vec<(&[u8], usize)> {
        let mut starts = Vec::new();
        let mut offset = 0;
        while offset + 3 <= data.len() {
            if data[offset..].starts_with(&[0, 0, 0, 1]) {
                starts.push((offset, 4));
                offset += 4;
            } else if data[offset..].starts_with(&[0, 0, 1]) {
                starts.push((offset, 3));
                offset += 3;
            } else {
                offset += 1;
            }
        }

        starts
            .iter()
            .enumerate()
            .map(|(index, &(start, prefix_len))| {
                let end = starts
                    .get(index + 1)
                    .map(|&(next_start, _)| next_start)
                    .unwrap_or(data.len());
                (&data[start..end], prefix_len)
            })
            .collect()
    }

    fn hex(data: &[u8]) -> String {
        let mut hex = String::with_capacity(data.len() * 2);
        for byte in data {
            write!(hex, "{byte:02x}").unwrap();
        }
        hex
    }

    #[test]
    fn carplay_sps_pps() {
        let mut snapshot = Vec::new();
        for &((width, height, fps), level) in &CARPLAY_LEVELS {
            let mut encoder = X264FrameBuffer::new(width, height, fps).expect("failed to create x264 encoder");
            let mut headers = BytesMut::new();
            encoder
                .get_headers(&mut headers)
                .expect("failed to generate SPS/PPS");

            let nals = annexb_nals(&headers);
            assert!(
                nals.iter()
                    .all(|(nal, prefix_len)| nal.get(*prefix_len).is_none_or(|byte| byte & 0x1f != 6)),
                "SEI leaked from x264 for {width}x{height}@{fps}"
            );
            let sps = nals
                .iter()
                .find(|(nal, prefix_len)| nal.get(*prefix_len).is_some_and(|byte| byte & 0x1f == 7))
                .copied()
                .unwrap_or_else(|| panic!("missing SPS for {width}x{height}@{fps}"));
            let (pps, pps_prefix_len) = nals
                .iter()
                .find(|(nal, prefix_len)| nal.get(*prefix_len).is_some_and(|byte| byte & 0x1f == 8))
                .map(|(nal, prefix_len)| (*nal, *prefix_len))
                .unwrap_or_else(|| panic!("missing PPS for {width}x{height}@{fps}"));

            // Apple uses a different nal_ref_idc (0x28 instead of x264's
            // 0x68), but the PPS RBSP that controls slice parsing must match.
            assert_eq!(
                &pps[pps_prefix_len + 1..],
                &[0xee, 0x3c, 0xb0],
                "PPS payload differs from Apple for {width}x{height}@{fps}"
            );
            snapshot.push(SpsPpsSnapshot {
                width,
                height,
                fps,
                level,
                sps: hex(sps.0),
                pps: hex(pps),
            });
        }

        assert_debug_snapshot!(snapshot);
    }
}
