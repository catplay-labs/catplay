use pixman::{FormatCode, Image, Operation};

pub trait Canvas {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn img_bytes(&self) -> usize;
    fn img_u32s(&self) -> usize;
    fn rgba(&self) -> &[u8];
    fn rgba_u32(&self) -> &[u32];
    fn as_rgba(&mut self) -> &mut [u8];
    fn as_rgba_u32(&mut self) -> &mut [u32];
    fn fill(&mut self, rgba: u32);
    fn blit_region<C: Canvas>(&mut self, other: &C, src_x: u32, src_y: u32, dst_x: u32, dst_y: u32, w: u32, h: u32);

    fn blit<C: Canvas>(&mut self, other: &C, x: u32, y: u32, w: u32, h: u32) {
        self.blit_region(other, 0, 0, x, y, w, h);
    }

    #[inline(always)]
    fn get_pixel(&mut self, x: usize, y: usize) -> Option<&mut u32> {
        // The buffer is a flat row-major array, so an x past the right edge would
        // otherwise alias onto the start of the next row.
        if x >= self.width() as usize || y >= self.height() as usize {
            return None;
        }

        let i = y * self.width() as usize + x;
        self.as_rgba_u32().get_mut(i)
    }

    #[inline(always)]
    fn set_pixel(&mut self, x: usize, y: usize, argb: u32) {
        if let Some(pixel) = self.get_pixel(x, y) {
            *pixel = argb;
        }
    }
}

pub struct PixmanCanvas<'img> {
    img: &'img mut Image<'static, 'static>,
    width: i32,
    height: i32,
    img_bytes: usize,
    img_u32s: usize,
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn fill32_unroll8(mut p: *mut u32, value: u32, mut n: usize) {
    unsafe {
        while n >= 8 {
            p.add(0).write(value);
            p.add(1).write(value);
            p.add(2).write(value);
            p.add(3).write(value);
            p.add(4).write(value);
            p.add(5).write(value);
            p.add(6).write(value);
            p.add(7).write(value);
            p = p.add(8);
            n -= 8;
        }

        while n != 0 {
            p.write(value);
            p = p.add(1);
            n -= 1;
        }
    }
}
impl<'img> PixmanCanvas<'img> {
    pub fn new(img: &'img mut Image<'static, 'static>, width: i32, height: i32, img_bytes: usize) -> Self {
        Self {
            img,
            width,
            height,
            img_bytes,
            img_u32s: img_bytes / 4,
        }
    }

    pub fn into_rgba(self) -> &'img mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.img.data() as *mut u8, self.img_bytes) }
    }

    pub fn into_rgba_u32(self) -> &'img mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.img.data(), self.img_u32s) }
    }

    #[inline(always)]
    pub fn into_pixel_mut(self, x: usize, y: usize) -> Option<&'img mut u32> {
        let i = y * self.width as usize + x;
        self.into_rgba_u32().get_mut(i)
    }

    #[inline(always)]
    pub fn set_pixel_at(self, x: usize, y: usize, argb: u32) {
        if let Some(pixel) = self.into_pixel_mut(x, y) {
            *pixel = argb;
        }
    }
}

impl Canvas for PixmanCanvas<'_> {
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
        unsafe { std::slice::from_raw_parts_mut(self.img.data() as *mut u8, self.img_bytes) }
    }

    fn as_rgba_u32(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.img.data(), self.img_u32s) }
    }

    fn fill(&mut self, rgba: u32) {
        let pixel = rgba8_to_pixman_a8b8g8r8(rgba);
        if pixel == 0 || pixel == 0xff {
            self.as_rgba().fill((pixel & 0xff) as u8);
            return;
        }

        let dst = self.as_rgba_u32();
        #[cfg(target_arch = "mips")]
        unsafe {
            return fill32_unroll8(dst.as_mut_ptr(), pixel, dst.len());
        }

        #[cfg(not(any(target_arch = "mips")))]
        {
            dst.fill(pixel);
        }
    }

    fn blit_region<C: Canvas>(&mut self, other: &C, src_x: u32, src_y: u32, dst_x: u32, dst_y: u32, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }

        if let Ok(src_img) = unsafe {
            Image::from_raw_mut(
                FormatCode::A8B8G8R8,
                other.width().max(0) as usize,
                other.height().max(0) as usize,
                other.rgba_u32().as_ptr().cast_mut(),
                other.width().max(0) as usize * 4,
                false,
            )
        } {
            self.img.composite32(
                Operation::Src,
                &src_img,
                None,
                (src_x as i32, src_y as i32),
                (0, 0),
                (dst_x as i32, dst_y as i32),
                (w as i32, h as i32),
            );
            return;
        }

        let src_width = other.width().max(0) as usize;
        let src_height = other.height().max(0) as usize;
        let dst_width = self.width.max(0) as usize;
        let dst_height = self.height.max(0) as usize;

        let src_x = src_x as usize;
        let src_y = src_y as usize;
        let dst_x = dst_x as usize;
        let dst_y = dst_y as usize;

        if src_x >= src_width || src_y >= src_height || dst_x >= dst_width || dst_y >= dst_height {
            return;
        }

        let copy_w = (w as usize).min(src_width - src_x).min(dst_width - dst_x);
        let copy_h = (h as usize).min(src_height - src_y).min(dst_height - dst_y);

        let src = other.rgba_u32();
        let dst = self.as_rgba_u32();

        for row in 0..copy_h {
            let src_start = (src_y + row) * src_width + src_x;
            let dst_start = (dst_y + row) * dst_width + dst_x;
            let src_row = &src[src_start..src_start + copy_w];
            let dst_row = &mut dst[dst_start..dst_start + copy_w];
            dst_row.copy_from_slice(src_row);
        }
    }
}

#[inline(always)]
pub(crate) fn blend_argb(dst: u32, src: u32) -> u32 {
    let sa = (src >> 24) & 0xFF;
    if sa == 0 {
        return dst;
    }

    let inv = 255 - sa;

    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;

    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    let da = (dst >> 24) & 0xFF;

    let r = (sr * sa + dr * inv) / 255;
    let g = (sg * sa + dg * inv) / 255;
    let b = (sb * sa + db * inv) / 255;
    let a = sa + (da * inv) / 255;

    (a << 24) | (r << 16) | (g << 8) | b
}

#[inline(always)]
fn rgba8_to_pixman_a8b8g8r8(rgba: u32) -> u32 {
    let r = ((rgba >> 24) & 0xFF) as u8;
    let g = ((rgba >> 16) & 0xFF) as u8;
    let b = ((rgba >> 8) & 0xFF) as u8;
    let a = (rgba & 0xFF) as u8;

    let r = premultiply_channel(r, a);
    let g = premultiply_channel(g, a);
    let b = premultiply_channel(b, a);

    u32::from_le_bytes([r, g, b, a])
}

#[inline(always)]
fn premultiply_channel(channel: u8, alpha: u8) -> u8 {
    (((channel as u16) * (alpha as u16) + 127) / 255) as u8
}
