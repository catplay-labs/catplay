use std::{mem, ops::Deref};

use crate::{Canvas, DirtyRect, FontPixelSink, PixmanCanvas, ReclaimableVec, ReclaimableVecBuilder, ReclaimableVecError, blend_argb};
use image::{ImageError, ImageResult, imageops::FilterType};
use pixman::{CreateFailed, FormatCode, Image, Operation};

pub struct Surface {
    pub img: Image<'static, 'static>,
    reclaimable: Option<ReclaimableVecBuilder>,
    pub img_bytes: usize,
    pub img_u32s: usize,
    pub width: i32,
    pub height: i32,
    pub pen_color: u32,
    pub dirty: DirtyRect,
}

pub struct ReclaimableSurface {
    img: Image<'static, 'static>,
    backing: ReclaimableVec,
    pub img_bytes: usize,
    pub img_u32s: usize,
    pub width: i32,
    pub height: i32,
}

pub trait SurfaceSource {
    fn pixman_image(&self) -> &Image<'static, 'static>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;

    fn validate_pixels(&self) -> Result<(), ReclaimableVecError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    #[error("reclaimable backing error: {0}")]
    Reclaimable(#[from] ReclaimableVecError),
    #[error("pixman alloc failed")]
    Pixman(#[from] CreateFailed),
    #[error("surface is not backed by reclaimable storage")]
    NotReclaimable,
}

impl Surface {
    #[allow(clippy::uninit_vec)]
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width <= 65535);
        assert!(height <= 65535);

        let size: usize = width as usize * height as usize;
        let img_bytes = size * 4;
        let img = Image::new(FormatCode::A8B8G8R8, width as _, height as _, false /* true */).expect("pixman alloc failed");

        let pen_color = 0;
        Surface {
            img,
            reclaimable: None,
            width,
            height,
            img_bytes,
            img_u32s: img_bytes / 4,
            pen_color,
            dirty: DirtyRect::default(),
        }
    }

    pub fn new_reclaimable(width: i32, height: i32) -> Result<Self, SurfaceError> {
        assert!(width <= 65535);
        assert!(height <= 65535);

        let size: usize = width as usize * height as usize;
        let img_bytes = size * 4;
        let mut reclaimable = ReclaimableVecBuilder::with_capacity(img_bytes)?;
        reclaimable.set_len(img_bytes)?;
        let img = unsafe {
            Image::from_raw_mut(
                FormatCode::A8B8G8R8,
                width as _,
                height as _,
                reclaimable.as_mut_ptr().cast::<u32>(),
                width as usize * 4,
                false,
            )?
        };

        Ok(Surface {
            img,
            reclaimable: Some(reclaimable),
            width,
            height,
            img_bytes,
            img_u32s: img_bytes / 4,
            pen_color: 0,
            dirty: DirtyRect::default(),
        })
    }

    pub fn from_image<C: Deref<Target = [u8]>>(img: image::ImageBuffer<image::Rgba<u8>, C>) -> Self {
        let (w, h) = img.dimensions();

        let mut me = Self::new(w as _, h as _);

        Canvas::as_rgba(&mut me).copy_from_slice(img.as_ref());
        me
    }

    pub fn as_image(&mut self) -> image::ImageBuffer<image::Rgba<u8>, &mut [u8]> {
        image::ImageBuffer::from_raw(self.width as _, self.height as _, Canvas::as_rgba(self)).expect("invalid buffer")
    }

    pub fn resized(&mut self, w: u32, h: u32) -> Surface {
        let resized = image::imageops::resize(&self.as_image(), w as _, h as _, FilterType::Nearest /*FilterType::Triangle*/);
        Self::from_image(resized)
    }

    pub fn into_reclaimable(self) -> Result<ReclaimableSurface, SurfaceError> {
        let Surface {
            img,
            reclaimable,
            img_bytes,
            img_u32s,
            width,
            height,
            ..
        } = self;

        drop(img);

        let mut backing = reclaimable.ok_or(SurfaceError::NotReclaimable)?;
        backing.set_len(img_bytes)?;
        let backing = backing.build()?;
        let img = unsafe {
            Image::from_raw_mut(
                FormatCode::A8B8G8R8,
                width as _,
                height as _,
                backing.as_ptr().cast::<u32>().cast_mut(),
                width as usize * 4,
                false,
            )?
        };

        Ok(ReclaimableSurface {
            img,
            backing,
            img_bytes,
            img_u32s,
            width,
            height,
        })
    }

    pub fn canvas(&mut self) -> PixmanCanvas<'_> {
        PixmanCanvas::new(&mut self.img, self.width, self.height, self.img_bytes)
    }

    pub fn save_surface(&mut self, path: &str) -> ImageResult<()> {
        let img: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(self.width as _, self.height as _, Canvas::as_rgba(self)).expect("invalid buffer");

        img.save(path)
    }

    pub fn blit_checked<S: SurfaceSource>(&mut self, other: &S, x: u32, y: u32, w: u32, h: u32) -> Result<(), ReclaimableVecError> {
        other.validate_pixels()?;
        self.img.composite(
            Operation::Src,
            other.pixman_image(),
            None,
            (0, 0),
            (0, 0),
            (x as i16, y as i16),
            (w as u16, h as u16),
        );
        other.validate_pixels()
    }

    pub fn set_pen_color(&mut self, argb: u32) {
        self.pen_color = argb;
    }

    pub fn add_dirty(&mut self, dirty: DirtyRect) {
        self.dirty.merge(dirty)
    }

    pub fn take_dirty(&mut self) -> DirtyRect {
        mem::take(&mut self.dirty)
    }
}

impl SurfaceSource for Surface {
    #[inline(always)]
    fn pixman_image(&self) -> &Image<'static, 'static> {
        &self.img
    }

    #[inline(always)]
    fn width(&self) -> i32 {
        self.width
    }

    #[inline(always)]
    fn height(&self) -> i32 {
        self.height
    }
}

impl Canvas for Surface {
    #[inline(always)]
    fn width(&self) -> i32 {
        self.width
    }

    #[inline(always)]
    fn height(&self) -> i32 {
        self.height
    }

    #[inline(always)]
    fn img_bytes(&self) -> usize {
        self.img_bytes
    }

    #[inline(always)]
    fn img_u32s(&self) -> usize {
        self.img_u32s
    }

    fn rgba(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.img.data() as *const u8, self.img_bytes) }
    }

    fn rgba_u32(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.img.data(), self.img_u32s) }
    }

    fn as_rgba(&mut self) -> &mut [u8] {
        self.canvas().into_rgba()
    }

    fn as_rgba_u32(&mut self) -> &mut [u32] {
        self.canvas().into_rgba_u32()
    }

    fn fill(&mut self, rgba: u32) {
        self.canvas().fill(rgba);
    }

    fn blit_region<C: Canvas>(&mut self, other: &C, src_x: u32, src_y: u32, dst_x: u32, dst_y: u32, w: u32, h: u32) {
        self.canvas()
            .blit_region(other, src_x, src_y, dst_x, dst_y, w, h);
    }
}

impl ReclaimableSurface {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.img_bytes
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.img_bytes == 0
    }

    pub fn copy_rgba_to_vec(&self) -> Result<Vec<u8>, ReclaimableVecError> {
        self.backing.copy_to_vec()
    }

    pub fn validate(&self) -> Result<(), ReclaimableVecError> {
        self.backing.validate()
    }

    pub fn save_surface(&self, path: &str) -> Result<(), SurfaceSaveError> {
        let rgba = self.copy_rgba_to_vec()?;
        let img: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(self.width as _, self.height as _, rgba).expect("invalid buffer");

        img.save(path).map_err(SurfaceSaveError::Image)
    }
}

impl SurfaceSource for ReclaimableSurface {
    #[inline(always)]
    fn pixman_image(&self) -> &Image<'static, 'static> {
        &self.img
    }

    #[inline(always)]
    fn width(&self) -> i32 {
        self.width
    }

    #[inline(always)]
    fn height(&self) -> i32 {
        self.height
    }

    fn validate_pixels(&self) -> Result<(), ReclaimableVecError> {
        self.validate()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceSaveError {
    #[error("reclaimable backing error: {0}")]
    Reclaimable(#[from] ReclaimableVecError),
    #[error("failed to export image: {0}")]
    Image(ImageError),
}

impl FontPixelSink for Surface {
    fn blend_pixel(&mut self, x: i32, y: i32, coverage: u8) -> bool {
        if x < 0 || y < 0 {
            return false;
        }

        let alpha: u8 = coverage;
        if alpha == 0 {
            return false;
        }

        let src = ((alpha as u32) << 24) | self.pen_color;
        if let Some(dst) = Canvas::get_pixel(self, x as _, y as _) {
            *dst = blend_argb(*dst, src);
            return true;
        }

        false
    }
}

#[inline(always)]
pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reclaimable_surface_copies_pixels_out_with_crc_check() {
        let mut surface = Surface::new_reclaimable(2, 1).unwrap();
        Canvas::as_rgba(&mut surface).copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

        let surface = surface.into_reclaimable().unwrap();

        assert_eq!(surface.copy_rgba_to_vec().unwrap(), [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn reclaimable_surface_can_be_blitted_as_source() {
        let mut source = Surface::new_reclaimable(1, 1).unwrap();
        Canvas::as_rgba(&mut source).copy_from_slice(&[11, 22, 33, 44]);
        let source = source.into_reclaimable().unwrap();

        let mut dst = Surface::new(1, 1);
        dst.blit_checked(&source, 0, 0, 1, 1).unwrap();

        assert_eq!(Canvas::as_rgba(&mut dst), &[11, 22, 33, 44]);
    }
}
