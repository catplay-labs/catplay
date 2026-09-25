use std::collections::HashMap;

use ab_glyph::{FontArc, InvalidFont, PxScale};
use image::{ImageError, RgbaImage};
use log::debug;
use turbojpeg::PixelFormat;

use crate::{FontRenderer, Surface};

const JPEG_MAGIC: &[u8; 3] = &[0xFF, 0xD8, 0xFF];

pub struct AssetCache {
    img_originals: HashMap<String, Surface>,

    img_scaled: HashMap<(String, u32, u32), Surface>,

    fonts: HashMap<String, FontArc>,
    fonts_cache: HashMap<(String, u32), FontRenderer>,
}

#[derive(thiserror::Error, Debug)]
pub enum AssetCacheError {
    #[error("Failed to load image: {0}")]
    Image(#[from] ImageError),
    #[error("Failed to decode JPEG: {0}")]
    TurboJpeg(#[from] turbojpeg::Error),
    #[error("Failed to decode JPEG: empty buffer")]
    TurboJpegEmptyBuffer,
    #[error("Invalid font: {0}")]
    InvalidFont(#[from] InvalidFont),
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            img_originals: HashMap::new(),
            img_scaled: HashMap::new(),
            fonts: HashMap::new(),
            fonts_cache: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn load(&mut self, id: String, path: &std::path::Path) -> Result<(), AssetCacheError> {
        if self.img_originals.contains_key(&id) {
            return Ok(());
        }

        let buf = std::fs::read(path).map_err(ImageError::IoError)?;
        let surf = Self::decode_surface(&buf)?;
        self.img_originals.insert(id, surf);
        Ok(())
    }

    pub fn load_buf(&mut self, id: String, buf: &[u8]) -> Result<(), AssetCacheError> {
        if self.img_originals.contains_key(&id) {
            return Ok(());
        }

        let surf = Self::decode_surface(buf)?;
        self.img_originals.insert(id, surf);
        Ok(())
    }

    fn decode_surface(buf: &[u8]) -> Result<Surface, AssetCacheError> {
        if buf.starts_with(JPEG_MAGIC) {
            let img = turbojpeg::decompress(buf, PixelFormat::RGBA)?;
            let rgba = RgbaImage::from_raw(img.width as u32, img.height as u32, img.pixels).ok_or(AssetCacheError::TurboJpegEmptyBuffer)?;
            return Ok(Surface::from_image(rgba));
        }

        let img = image::load_from_memory(buf)?.to_rgba8();
        Ok(Surface::from_image(img))
    }

    pub fn release_img(&mut self, id: &str) {
        self.img_originals.retain(|k, _v| k != id);
        self.img_scaled.retain(|k, _v| k.0 != id);
        debug!("Releasing asset {id}");
    }

    pub fn release_font(&mut self, id: &str) {
        self.fonts.retain(|k, _v| *k != id);
        self.fonts_cache.retain(|k, _v| k.0 != id);
        debug!("Releasing font {id}");
    }

    pub fn get_scaled(&mut self, id: String, w: u32, h: u32) -> Result<&Surface, AssetCacheError> {
        let key = (id.clone(), w, h);

        if !self.img_scaled.contains_key(&key) {
            let orig = self
                .img_originals
                .get_mut(&id)
                .ok_or_else(|| AssetCacheError::AssetNotFound(id.clone()))?;
            let scaled = orig.resized(w, h);
            self.img_scaled.insert(key.clone(), scaled);
        }

        Ok(self.img_scaled.get_mut(&key).unwrap())
    }

    pub fn load_font(&mut self, id: String, data: Vec<u8>) -> Result<(), AssetCacheError> {
        if self.fonts.contains_key(&id) {
            return Ok(());
        }

        let font = FontArc::try_from_vec(data)?;

        self.fonts.insert(id.clone(), font);
        Ok(())
    }

    pub fn load_font_scale(&mut self, id: String, scale: u32) -> Result<&mut FontRenderer, AssetCacheError> {
        let font = self
            .fonts
            .get(&id)
            .ok_or_else(|| AssetCacheError::AssetNotFound(id.clone()))?
            .clone();

        use std::collections::hash_map::Entry;

        let entry = self.fonts_cache.entry((id, scale));

        Ok(match entry {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(v) => v.insert(FontRenderer::new(font, PxScale::from(scale as f32))),
        })
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}
