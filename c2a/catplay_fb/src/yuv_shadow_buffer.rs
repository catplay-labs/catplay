use catplay_tracing_macro::trace_time;
use yuv::{BufferStoreMut, YuvError, YuvPlanarImageMut, YuvRange, YuvStandardMatrix, rgba_to_yuv420};

use crate::{DirtyRect, libyuv};

const ALIGN: usize = 64;

fn align_offset(base: usize, offset: usize) -> usize {
    let addr = base + offset;
    offset + ((ALIGN - (addr % ALIGN)) % ALIGN)
}

/// YUV shadow of a reference RGB buffer.
#[derive(Clone, Copy, Debug)]
pub struct Yuv420Layout {
    pub width: usize,
    pub height: usize,
    pub chroma_width: usize,
    pub chroma_height: usize,
    pub y_len: usize,
    pub uv_len: usize,
    total_len: usize,
}

#[derive(Clone, Copy, Debug)]
struct Yuv420Offsets {
    y_off: usize,
    u_off: usize,
    v_off: usize,
}

pub struct Yuv420Buffer<'a> {
    layout: Yuv420Layout,
    offsets: Yuv420Offsets,
    yuv: &'a mut [u8],
}

/// YUV shadow of a reference RGB buffer.
pub struct YuvShadowBuffer {
    pub layout: Yuv420Layout,
    yuv: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum YuvShadowBufferError {
    #[error("YuvError: {0:?}")]
    YuvConversion(YuvError),
    #[error("libyuv error code={0}")]
    LibYuv(i32),
    #[error("Invalid dirty rectangle provided")]
    InvalidDirtyRect,
    #[error("Buffer too small for in-place conversion")]
    BufferToSmall,
}

impl YuvShadowBuffer {
    pub fn i420(width: usize, height: usize) -> Self {
        let layout = Yuv420Layout::i420(width, height);
        let yuv = vec![0u8; layout.total_len];

        Self { layout, yuv }
    }

    #[inline(always)]
    pub fn as_mut_buffer(&mut self) -> Yuv420Buffer<'_> {
        Yuv420Buffer::from_slice(self.layout, self.yuv.as_mut_slice()).expect("owned YUV storage must match layout")
    }

    #[inline(always)]
    pub fn layout(&self) -> Yuv420Layout {
        self.layout
    }

    #[inline(always)]
    pub fn yuv_slices_mut(&mut self) -> [&mut [u8]; 3] {
        let offsets = self.layout.offsets_for_base(self.yuv.as_ptr() as usize);
        self.layout
            .yuv_slices_mut_with_offsets(offsets, self.yuv.as_mut_slice())
    }

    #[inline(always)]
    pub fn yuv_slices(&self) -> [&[u8]; 3] {
        let offsets = self.layout.offsets_for_base(self.yuv.as_ptr() as usize);
        self.layout.yuv_slices_with_offsets(offsets, &self.yuv)
    }

    pub fn update(&mut self, rgba: &[u8]) -> Result<(), YuvShadowBufferError> {
        self.as_mut_buffer().update(rgba)
    }

    #[trace_time]
    pub fn update_dirty(&mut self, rgba: &[u8], rect: DirtyRect) -> Result<(), YuvShadowBufferError> {
        self.as_mut_buffer().update_dirty(rgba, rect)
    }
}

impl Yuv420Layout {
    pub fn i420(width: usize, height: usize) -> Self {
        let chroma_width = width.div_ceil(2);
        let chroma_height = height.div_ceil(2);

        let y_len = width * height;
        let uv_len = chroma_width * chroma_height;

        let total_len = y_len + uv_len + uv_len;
        let total_len = total_len + (ALIGN - 1) * 3;

        Self {
            width,
            height,
            chroma_width,
            chroma_height,
            y_len,
            uv_len,
            total_len,
        }
    }

    #[inline(always)]
    pub fn storage_len(&self) -> usize {
        self.total_len
    }

    #[inline(always)]
    fn offsets_for_base(&self, base: usize) -> Yuv420Offsets {
        let y_off = align_offset(base, 0);
        let u_off = align_offset(base, y_off + self.y_len);
        let v_off = align_offset(base, u_off + self.uv_len);

        Yuv420Offsets { y_off, u_off, v_off }
    }

    #[inline(always)]
    fn yuv_slices_mut_with_offsets<'a>(&self, offsets: Yuv420Offsets, yuv: &'a mut [u8]) -> [&'a mut [u8]; 3] {
        let (_, after_y) = yuv.split_at_mut(offsets.y_off);
        let (y, after_y) = after_y.split_at_mut(self.y_len);

        let u_gap = offsets.u_off - (offsets.y_off + self.y_len);
        let (_, after_u_gap) = after_y.split_at_mut(u_gap);
        let (u, after_u) = after_u_gap.split_at_mut(self.uv_len);

        let v_gap = offsets.v_off - (offsets.u_off + self.uv_len);
        let (_, after_v_gap) = after_u.split_at_mut(v_gap);
        let (v, _) = after_v_gap.split_at_mut(self.uv_len);

        [y, u, v]
    }

    #[inline(always)]
    fn yuv_slices_with_offsets<'a>(&self, offsets: Yuv420Offsets, yuv: &'a [u8]) -> [&'a [u8]; 3] {
        let y = &yuv[offsets.y_off..offsets.y_off + self.y_len];
        let u = &yuv[offsets.u_off..offsets.u_off + self.uv_len];
        let v = &yuv[offsets.v_off..offsets.v_off + self.uv_len];
        [y, u, v]
    }
}

impl<'a> Yuv420Buffer<'a> {
    pub fn from_slice(layout: Yuv420Layout, yuv: &'a mut [u8]) -> Result<Self, YuvShadowBufferError> {
        if yuv.len() < layout.storage_len() {
            return Err(YuvShadowBufferError::BufferToSmall);
        }

        let offsets = layout.offsets_for_base(yuv.as_ptr() as usize);

        Ok(Self { layout, offsets, yuv })
    }

    #[inline(always)]
    pub fn layout(&self) -> Yuv420Layout {
        self.layout
    }

    #[inline(always)]
    pub fn yuv_slices_mut(&mut self) -> [&mut [u8]; 3] {
        self.layout
            .yuv_slices_mut_with_offsets(self.offsets, self.yuv)
    }

    #[inline(always)]
    pub fn yuv_slices(&self) -> [&[u8]; 3] {
        self.layout.yuv_slices_with_offsets(self.offsets, self.yuv)
    }

    pub fn update(&mut self, rgba: &[u8]) -> Result<(), YuvShadowBufferError> {
        self.update_dirty(
            rgba,
            DirtyRect {
                x: 0,
                y: 0,
                w: self.layout.width as _,
                h: self.layout.height as _,
            },
        )
    }

    #[trace_time]
    pub fn update_dirty(&mut self, rgba: &[u8], rect: DirtyRect) -> Result<(), YuvShadowBufferError> {
        let full_w = self.layout.width as u32;
        let full_h = self.layout.height as u32;

        let y_stride = full_w as usize;
        let uv_stride = self.layout.chroma_width;
        let rgba_stride = (full_w * 4) as usize;

        let DirtyRect { x, y, w, h } = rect;

        if (x | y | w | h) & 1 != 0 {
            return Err(YuvShadowBufferError::InvalidDirtyRect);
        }

        if x + w > full_w || y + h > full_h {
            return Err(YuvShadowBufferError::InvalidDirtyRect);
        }

        let chroma_x = x / 2;
        let chroma_y = y / 2;
        let _chroma_w = w / 2;
        let _chroma_h = h / 2;

        let [y_plane, u_plane, v_plane] = self.yuv_slices_mut();

        let y_off = (y as usize * y_stride) + x as usize;
        let uv_off = (chroma_y as usize * uv_stride) + chroma_x as usize;
        let rgba_off = (y as usize * rgba_stride) + (x as usize * 4);

        let rgba_ptr = &rgba[rgba_off..];

        if cfg!(feature = "prefer_libyuv") {
            let ret = unsafe {
                libyuv::abgr_to_i420(
                    rgba_ptr.as_ptr(),
                    rgba_stride as _,
                    y_plane[y_off..].as_mut_ptr(),
                    y_stride as _,
                    u_plane[uv_off..].as_mut_ptr(),
                    uv_stride as _,
                    v_plane[uv_off..].as_mut_ptr(),
                    uv_stride as _,
                    w as _,
                    h as _,
                )
            };

            if ret != 0 {
                return Err(YuvShadowBufferError::LibYuv(ret));
            }

            return Ok(());
        }

        let mut planar = YuvPlanarImageMut {
            y_plane: BufferStoreMut::Borrowed(&mut y_plane[y_off..]),
            y_stride: y_stride as u32,
            u_plane: BufferStoreMut::Borrowed(&mut u_plane[uv_off..]),
            u_stride: uv_stride as u32,
            v_plane: BufferStoreMut::Borrowed(&mut v_plane[uv_off..]),
            v_stride: uv_stride as u32,
            width: w,
            height: h,
        };

        rgba_to_yuv420(
            &mut planar,
            rgba_ptr,
            rgba_stride as u32,
            YuvRange::Limited,
            YuvStandardMatrix::Bt601,
            yuv::YuvConversionMode::Balanced,
        )
        .map_err(YuvShadowBufferError::YuvConversion)
    }
}

#[cfg(feature = "openh264")]
impl openh264::formats::YUVSource for YuvShadowBuffer {
    fn dimensions(&self) -> (usize, usize) {
        (self.layout.width, self.layout.height)
    }

    fn strides(&self) -> (usize, usize, usize) {
        (self.layout.width, self.layout.chroma_width, self.layout.chroma_width)
    }

    fn y(&self) -> &[u8] {
        self.yuv_slices()[0]
    }

    fn u(&self) -> &[u8] {
        self.yuv_slices()[1]
    }

    fn v(&self) -> &[u8] {
        self.yuv_slices()[2]
    }
}

#[cfg(feature = "openh264")]
impl openh264::formats::YUVSource for Yuv420Buffer<'_> {
    fn dimensions(&self) -> (usize, usize) {
        (self.layout.width, self.layout.height)
    }

    fn strides(&self) -> (usize, usize, usize) {
        (self.layout.width, self.layout.chroma_width, self.layout.chroma_width)
    }

    fn y(&self) -> &[u8] {
        self.yuv_slices()[0]
    }

    fn u(&self) -> &[u8] {
        self.yuv_slices()[1]
    }

    fn v(&self) -> &[u8] {
        self.yuv_slices()[2]
    }
}
