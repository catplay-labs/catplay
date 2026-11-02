use core::fmt;
use std::{path::PathBuf, time::Instant};

use ab_glyph::InvalidFont;
use image::ImageError;
use log::{debug, warn};

use crate::{Anchor, AssetCache, AssetCacheError, Canvas, DirtyRect, Surface, TextAlign, UiRect, resolve_rect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    Image,
    Font,
}

#[derive(Clone, PartialEq)]
pub enum RenderOp {
    SetBackground {
        rgba: u32,
    },

    LoadAsset {
        id: String,
        path: PathBuf,
        asset_type: AssetType,
    },
    ReleaseAsset {
        id: String,
    },
    ReleaseFont {
        id: String,
    },

    LoadAssetConst {
        id: &'static str,
        buf: &'static [u8],
        asset_type: AssetType,
    },

    BlitAsset {
        id: String,
        rect: UiRect,
    },

    SetTextColor {
        argb: u32,
    },

    DrawText {
        text: String,
        rect: UiRect,

        align: TextAlign,
        font_id: String,
        font_size_pt: f32,
        color: u32, // 0xRRGGBBAA
    },
}

impl fmt::Debug for RenderOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetBackground { rgba } => f.debug_struct("SetBackground").field("rgba", rgba).finish(),
            Self::LoadAsset { id, path, asset_type } => f
                .debug_struct("LoadAsset")
                .field("id", id)
                .field("path", path)
                .field("asset_type", asset_type)
                .finish(),
            Self::LoadAssetConst { id, asset_type, .. } => f
                .debug_struct("LoadAssetConst")
                .field("id", id)
                .field("asset_type", asset_type)
                .finish(),
            Self::BlitAsset { id, rect } => f
                .debug_struct("BlitAsset")
                .field("id", id)
                .field("rect", rect)
                .finish(),
            Self::SetTextColor { argb } => f.debug_struct("SetTextColor").field("argb", argb).finish(),
            Self::DrawText {
                text,
                rect,
                align,
                font_id,
                font_size_pt,
                color,
            } => f
                .debug_struct("DrawText")
                .field("text", text)
                .field("rect", rect)
                .field("align", align)
                .field("font_id", font_id)
                .field("font_size_pt", font_size_pt)
                .field("color", color)
                .finish(),
            Self::ReleaseAsset { id } => f.debug_struct("ReleaseAsset").field("id", id).finish(),
            Self::ReleaseFont { id } => f.debug_struct("ReleaseFont").field("id", id).finish(),
        }
    }
}
/// Widest a single line of text may get, as a fraction of the screen width.
const TEXT_MAX_SCREEN_FRACTION: f32 = 0.95;

pub struct Renderer {
    fb: Surface,
    dpi: f32,
    assets: AssetCache,

    current_text_rgb: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("Invalid font: {0}")]
    InvalidFont(#[from] InvalidFont),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown font: {0}")]
    UnknownFont(String),
    #[error("Unknown asset: {0}")]
    UnknownAsset(String),
    #[error("Assets caching error: {0}")]
    AssetsCache(#[from] AssetCacheError),
    #[error("No font selected")]
    NoFontSelected,

    #[error("Failed to export image: {0}")]
    ImageExportError(ImageError),
}

impl Renderer {
    pub fn new(width: i32, height: i32, dpi: f32) -> Self {
        Self {
            fb: Surface::new(width, height),
            dpi,
            assets: AssetCache::new(),
            current_text_rgb: 0,
        }
    }

    pub fn reset(&mut self, width: i32, height: i32, dpi: f32) {
        self.dpi = dpi;
        self.fb = Surface::new(width, height);
    }

    pub fn export_to_disk(&mut self, path: PathBuf) -> Result<(), RenderError> {
        self.fb
            .save_surface(path.to_str().unwrap())
            .map_err(RenderError::ImageExportError)
    }

    pub fn execute(&mut self, ops: &[RenderOp]) -> Result<(), RenderError> {
        for op in ops {
            let start = Instant::now();
            match op {
                RenderOp::SetBackground { rgba } => {
                    Canvas::fill(&mut self.fb, *rgba);

                    let dirty = DirtyRect {
                        x: 0,
                        y: 0,
                        w: self.fb.width as _,
                        h: self.fb.height as _,
                    };
                    debug!("Background dirty: {dirty:?}");
                    self.fb.add_dirty(dirty);
                }
                RenderOp::LoadAsset { id, path, asset_type } => match asset_type {
                    AssetType::Image => {
                        self.assets.load(id.clone(), path)?;
                    }
                    AssetType::Font => {
                        self.assets.load_font(id.clone(), std::fs::read(path)?)?;
                    }
                },
                RenderOp::LoadAssetConst { id, buf, asset_type } => match asset_type {
                    AssetType::Image => {
                        self.assets.load_buf((*id).into(), buf)?;
                    }
                    AssetType::Font => {
                        self.assets.load_font((*id).into(), (*buf).into())?;
                    }
                },
                RenderOp::BlitAsset { id, rect } => {
                    let rect = resolve_rect(*rect, self.fb.width, self.fb.height, self.dpi);
                    let asset = self.assets.get_scaled(id.clone(), rect.w, rect.h)?;
                    Canvas::blit(&mut self.fb, asset, rect.x, rect.y, rect.w, rect.h);

                    debug!("Blit asset dirty: {rect:?}");
                    self.fb.add_dirty(rect);
                }
                RenderOp::SetTextColor { argb } => {
                    self.current_text_rgb = *argb;
                    self.fb.set_pen_color(*argb);
                }
                RenderOp::DrawText {
                    text,
                    rect,
                    align,
                    font_id,
                    font_size_pt,
                    color,
                } => {
                    let r = resolve_rect(*rect, self.fb.width, self.fb.height, self.dpi);
                    let (rx, ry, rw, rh) = (r.x, r.y, r.w, r.h);
                    let mut px = font_size_pt * self.dpi / 72.0;

                    // Shrink text that would not fit on the screen at all, so that long
                    // strings (e.g. the pairing prompt with a long device name) stay
                    // readable on head units with a narrow CarPlay viewport.
                    // Text that already fits is left untouched.
                    let max_w = self.fb.width as f32 * TEXT_MAX_SCREEN_FRACTION;
                    let tw = self
                        .assets
                        .load_font_scale((*font_id).clone(), px.round() as _)?
                        .measure_text_width(text);
                    if tw > max_w && max_w > 0.0 {
                        px = (px.round() * max_w / tw).floor().max(1.0);
                    }

                    let font_renderer = self
                        .assets
                        .load_font_scale((*font_id).clone(), px.round() as _)?;

                    self.fb.set_pen_color(*color);

                    let (mut pen_x, baseline_y) = match rect.anchor {
                        Anchor::BaselineLeft => (rx as f32, ry as f32),
                        Anchor::BaselineCenter => (rx as f32 + rw as f32 / 2.0, ry as f32),
                        Anchor::BaselineRight => (rx as f32 + rw as f32, ry as f32),
                        _ => (rx as f32 + rw as f32 / 2.0, ry as f32 + rh as f32 / 2.0),
                    };

                    match align {
                        TextAlign::Left => {}
                        TextAlign::Center => {
                            let tw = font_renderer.measure_text_width(text);
                            pen_x -= tw / 2.0;
                        }
                        TextAlign::Right => {
                            let tw = font_renderer.measure_text_width(text);
                            pen_x -= tw;
                        }
                    }

                    let dirty = font_renderer.draw_text(&mut self.fb, pen_x, baseline_y, text);
                    if let Some(dirty) = dirty {
                        debug!("Text dirty: {dirty:?}");
                        self.fb.add_dirty(dirty);
                    }
                }
                RenderOp::ReleaseAsset { id } => self.assets.release_img(id),
                RenderOp::ReleaseFont { id } => self.assets.release_font(id),
            }

            warn!("Finished OP {op:?} in {:?}", Instant::now() - start);
        }

        Ok(())
    }

    pub fn framebuffer(&mut self) -> &mut Surface {
        &mut self.fb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unit;

    const FONT: &[u8] = include_bytes!("../../catplay_c2a/assets/Exo2-Light.otf");

    /// Renders `text` the way the C2A overlay does (30 pt, centered, 700 dp box) on a
    /// `width` x 200 screen at 64 dpi, and returns the first and last column that
    /// received any text pixels.
    fn ink_columns(width: i32, text: &str) -> (usize, usize) {
        let mut renderer = Renderer::new(width, 200, 64.0);
        renderer
            .execute(&[
                RenderOp::SetBackground { rgba: 0xFF000000 },
                RenderOp::LoadAssetConst {
                    id: "font",
                    buf: FONT,
                    asset_type: AssetType::Font,
                },
                RenderOp::DrawText {
                    text: text.into(),
                    rect: UiRect {
                        x: Unit::Percent(0.5),
                        y: Unit::Percent(0.3),
                        w: Unit::Dp(700.0),
                        h: Unit::Dp(0.0),
                        anchor: Anchor::BaselineCenter,
                    },
                    align: TextAlign::Center,
                    font_id: "font".into(),
                    font_size_pt: 30.0,
                    color: 0xFFFFFFFF,
                },
            ])
            .unwrap();

        let fb = renderer.framebuffer();
        let bg = Canvas::rgba_u32(fb)[0];
        let w = width as usize;
        let inked: Vec<usize> = Canvas::rgba_u32(fb)
            .iter()
            .enumerate()
            .filter(|(_, px)| **px != bg)
            .map(|(i, _)| i % w)
            .collect();
        (*inked.iter().min().unwrap(), *inked.iter().max().unwrap())
    }

    #[test]
    fn text_wider_than_screen_is_shrunk_to_fit() {
        let width = 320;
        let (first, last) = ink_columns(width, "Press any key to pair: Somebody's iPhone | 756615");

        // Unscaled (30 pt at 64 dpi = ~27 px) this string is about twice the screen width
        // and would be cut at both edges.
        assert!(first > 0, "text runs off the left edge (first ink column {first})");
        assert!(last < width as usize - 1, "text runs off the right edge (last ink column {last})");
    }

    #[test]
    fn text_that_fits_keeps_its_size() {
        let width = 1280;
        let (first, last) = ink_columns(width, "Waiting for connection...");
        let ink_w = last - first + 1;

        // At ~27 px this string is well over 200 px wide; a shrunk one would not be.
        assert!(ink_w > 200, "short text was shrunk (ink width {ink_w})");
    }
}
