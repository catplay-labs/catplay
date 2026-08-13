use bytes::BytesMut;
use std::os::fd::OwnedFd;

#[allow(unused)]
use crate::screen::ScreenFrame;
use crate::video::{AvccConfigExtended, NalChunk, Pts};
/// Represents incoming H264/HEVC video frame.
#[derive(Debug, Clone)]
pub struct EncodedVideoFrame {
    /// Optimal presentation timestamp, already adjusted by media clock
    pub pts: Pts,
    /// Width as described by last VideoConfig opcode
    pub width: u32,
    /// Height as described by last VideoConfig opcode
    pub height: u32,
    /// Annex-B encoded (or in special cases - AVCC/HEVC encoded) H264/HEVC data that are _expected_ to produce 1 full frame
    pub data: BytesMut,
    /// Optional copy of [AvccConfigExtended] relevant to this frame
    pub config: Option<AvccConfigExtended>,
    /// Optional list of offsets where NAL segments really start, without AnnexB/AVCC header (if such analysis was performed).
    ///
    /// Can be used to inspect the packet before passing it to decoder.
    pub nal_offsets: Option<Vec<NalChunk>>,
    /// Keyframe status of this frame, if known
    pub is_keyframe: Option<bool>,
    /// Set when `data` already carries AVCC/HVCC length prefixes, so the transmit path leaves it
    /// alone instead of converting from AnnexB.
    pub length_prefixed: bool,

    /// 128b [ScreenFrame] header slice (optional)
    pub header_buf: BytesMut,
    /// 16b [ScreenFrame] ChaCha tag suffix slice (optional)
    pub chacha_tag_buf: BytesMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YuvLayout {
    I420,
    Nv12,
}

#[derive(Debug)]
pub struct DmaBufPlane {
    pub fd: OwnedFd,
    pub offset: u32,
    pub pitch: u32,
    pub modifier: u64,
}

#[derive(Debug)]
pub struct DmaBufFrame {
    pub width: u32,
    pub height: u32,
    pub drm_format: u32,
    pub planes: Vec<DmaBufPlane>,
}

impl EncodedVideoFrame {
    pub fn is_known_keyframe(&self) -> bool {
        self.is_keyframe == Some(true)
    }
}

/// Represents structs which allow access to YUV-decoded buffers of previously-encoded H264 frames.
pub trait YuvDecoded {
    fn pts(&self) -> Pts;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn layout(&self) -> YuvLayout;

    fn yuv(&self) -> [&[u8]; 3];

    fn strides(&self) -> [usize; 3];

    fn plane_ptrs(&self) -> [*const u8; 3] {
        let yuv = self.yuv();
        [yuv[0].as_ptr(), yuv[1].as_ptr(), yuv[2].as_ptr()]
    }

    fn dmabuf(&self) -> Option<&DmaBufFrame> {
        None
    }
}
