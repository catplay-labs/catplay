use super::{BitReader, BitWriter, RewriteResult};
use crate::H264FrameBufferError;

/// Adapts x264's SPS and IDR slice syntax to the parameter-set contract used
/// by CarPlay while preserving the encoded CABAC payload.
pub(crate) struct CarPlayNalRewriter;

impl CarPlayNalRewriter {
    pub(crate) fn rewrite_sps(nal: &[u8]) -> RewriteResult<Vec<u8>> {
        let prefix_len = Self::annexb_prefix_len(nal)?;
        let header = *nal
            .get(prefix_len)
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("missing SPS NAL header"))?;
        if header & 0x1f != 7 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected SPS NAL"));
        }

        let rbsp = Self::remove_emulation_prevention(&nal[prefix_len + 1..]);
        let mut reader = BitReader::new(&rbsp);
        let mut writer = BitWriter::default();

        let profile_idc = reader.bits(8)?;
        writer.bits(profile_idc, 8);
        writer.bits(reader.bits(8)?, 8);
        writer.bits(reader.bits(8)?, 8);
        writer.ue(reader.ue()?);

        if profile_idc != 100 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected High Profile SPS"));
        }
        let chroma_format_idc = reader.ue()?;
        writer.ue(chroma_format_idc);
        if chroma_format_idc == 3 {
            writer.bit(reader.bit()?);
        }
        writer.ue(reader.ue()?);
        writer.ue(reader.ue()?);
        writer.bit(reader.bit()?);
        let scaling_matrix_present = reader.bit()?;
        writer.bit(scaling_matrix_present);
        if scaling_matrix_present != 0 {
            return Err(H264FrameBufferError::X264BitstreamRewrite(
                "custom SPS scaling matrices are unsupported",
            ));
        }

        let source_frame_num_minus4 = reader.ue()?;
        let source_poc_type = reader.ue()?;
        if source_frame_num_minus4 != 0 || source_poc_type != 2 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("unexpected x264 frame_num/POC model"));
        }
        writer.ue(8); // log2_max_frame_num_minus4: Apple uses a 12-bit frame_num.
        writer.ue(0); // pic_order_cnt_type.
        writer.ue(9); // log2_max_pic_order_cnt_lsb_minus4: Apple uses a 13-bit POC LSB.

        let max_num_ref_frames = reader.ue()?;
        if max_num_ref_frames != 4 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected four decoded reference frames"));
        }
        writer.ue(max_num_ref_frames);
        writer.bit(reader.bit()?);
        writer.ue(reader.ue()?);
        writer.ue(reader.ue()?);
        let frame_mbs_only = reader.bit()?;
        writer.bit(frame_mbs_only);
        if frame_mbs_only == 0 {
            writer.bit(reader.bit()?);
        }
        writer.bit(reader.bit()?);
        let frame_cropping = reader.bit()?;
        writer.bit(frame_cropping);
        if frame_cropping != 0 {
            for _ in 0..4 {
                writer.ue(reader.ue()?);
            }
        }

        if reader.bit()? == 0 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected VUI parameters"));
        }
        writer.bit(1);
        Self::write_carplay_vui(&mut writer);
        writer.rbsp_trailing_bits();

        Ok(Self::rebuild_nal(nal, prefix_len, header, &writer.data))
    }

    pub(crate) fn rewrite_idr(nal: &[u8]) -> RewriteResult<Vec<u8>> {
        let prefix_len = Self::annexb_prefix_len(nal)?;
        let header = *nal
            .get(prefix_len)
            .ok_or(H264FrameBufferError::X264BitstreamRewrite("missing IDR NAL header"))?;
        if header & 0x1f != 5 || header >> 5 == 0 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected reference IDR NAL"));
        }

        let rbsp = Self::remove_emulation_prevention(&nal[prefix_len + 1..]);
        let mut reader = BitReader::new(&rbsp);
        let first_mb = reader.ue()?;
        let slice_type = reader.ue()?;
        let pps_id = reader.ue()?;
        if slice_type % 5 != 2 || pps_id != 0 {
            return Err(H264FrameBufferError::X264BitstreamRewrite("expected I slice using PPS 0"));
        }
        let frame_num = reader.bits(4)?;
        let idr_pic_id = reader.ue()?;
        let no_output_of_prior_pics = reader.bit()?;
        let long_term_reference = reader.bit()?;
        let slice_qp_delta = reader.se()?;
        let disable_deblocking_filter_idc = reader.ue()?;
        let deblock_offsets = if disable_deblocking_filter_idc != 1 {
            Some((reader.se()?, reader.se()?))
        } else {
            None
        };

        while reader.pos % 8 != 0 {
            if reader.bit()? != 1 {
                return Err(H264FrameBufferError::X264BitstreamRewrite("invalid CABAC alignment"));
            }
        }
        let cabac_start = reader.pos / 8;

        let mut writer = BitWriter::default();
        writer.ue(first_mb);
        writer.ue(slice_type);
        writer.ue(pps_id);
        writer.bits(frame_num, 12);
        writer.ue(idr_pic_id);
        writer.bits(0, 13); // pic_order_cnt_lsb for the generated IDR.
        writer.bit(no_output_of_prior_pics);
        writer.bit(long_term_reference);
        writer.se(slice_qp_delta);
        writer.ue(disable_deblocking_filter_idc);
        if let Some((alpha, beta)) = deblock_offsets {
            writer.se(alpha);
            writer.se(beta);
        }
        writer.align_one();
        writer.data.extend_from_slice(&rbsp[cabac_start..]);

        Ok(Self::rebuild_nal(nal, prefix_len, header, &writer.data))
    }

    fn rebuild_nal(nal: &[u8], prefix_len: usize, header: u8, rbsp: &[u8]) -> Vec<u8> {
        let escaped = Self::add_emulation_prevention(rbsp);
        let mut out = Vec::with_capacity(prefix_len + 1 + escaped.len());
        out.extend_from_slice(&nal[..prefix_len]);
        out.push(header);
        out.extend_from_slice(&escaped);
        out
    }

    pub(super) fn remove_emulation_prevention(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        let mut zeroes = 0;
        for &byte in data {
            if zeroes >= 2 && byte == 3 {
                zeroes = 0;
                continue;
            }
            out.push(byte);
            zeroes = if byte == 0 { zeroes + 1 } else { 0 };
        }
        out
    }

    fn add_emulation_prevention(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        let mut zeroes = 0;
        for &byte in data {
            if zeroes >= 2 && byte <= 3 {
                out.push(3);
                zeroes = 0;
            }
            out.push(byte);
            zeroes = if byte == 0 { zeroes + 1 } else { 0 };
        }
        out
    }

    fn write_carplay_vui(writer: &mut BitWriter) {
        writer.bit(0); // aspect_ratio_info_present_flag
        writer.bit(0); // overscan_info_present_flag
        writer.bit(1); // video_signal_type_present_flag
        writer.bits(5, 3); // video_format: unspecified
        writer.bit(1); // video_full_range_flag
        writer.bit(1); // colour_description_present_flag
        writer.bits(1, 8); // colour_primaries: BT.709
        writer.bits(13, 8); // transfer_characteristics: sRGB
        writer.bits(6, 8); // matrix_coefficients: SMPTE 170M
        writer.bit(0); // chroma_loc_info_present_flag
        writer.bit(0); // timing_info_present_flag
        writer.bit(0); // nal_hrd_parameters_present_flag
        writer.bit(0); // vcl_hrd_parameters_present_flag
        writer.bit(0); // pic_struct_present_flag
        writer.bit(1); // bitstream_restriction_flag
        writer.bit(1); // motion_vectors_over_pic_boundaries_flag
        writer.ue(2); // max_bytes_per_pic_denom
        writer.ue(1); // max_bits_per_mb_denom
        writer.ue(16); // log2_max_mv_length_horizontal
        writer.ue(16); // log2_max_mv_length_vertical
        writer.ue(0); // num_reorder_frames
        writer.ue(4); // max_dec_frame_buffering
    }

    pub(super) fn annexb_prefix_len(nal: &[u8]) -> RewriteResult<usize> {
        if nal.starts_with(&[0, 0, 0, 1]) {
            Ok(4)
        } else if nal.starts_with(&[0, 0, 1]) {
            Ok(3)
        } else {
            Err(H264FrameBufferError::X264BitstreamRewrite("missing Annex-B start code"))
        }
    }
}
