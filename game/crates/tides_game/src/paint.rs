//! Procedural pixel-art tiles: each 48px cell is painted at 16px into one
//! RGBA canvas, then shown scaled ×3 with nearest filtering.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use tides_core::map::{Layout, Tile, cell_noise, cell_seed};

pub const SRC: u32 = 16;

/// Plain RGBA8 pixel buffer; converted to a Bevy `Image` once painted.
pub struct Canvas {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

pub type Rgb = [u8; 3];

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h, data: vec![0; (w * h * 4) as usize] }
    }

    pub fn px(&mut self, x: u32, y: u32, c: Rgb) {
        if x < self.w && y < self.h {
            let i = ((y * self.w + x) * 4) as usize;
            self.data[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }

    pub fn fill(&mut self, x: u32, y: u32, w: u32, h: u32, c: Rgb) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.px(xx, yy, c);
            }
        }
    }

    pub fn into_image(self) -> Image {
        Image::new(
            Extent3d { width: self.w, height: self.h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            self.data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// Per-cell painting context: origin in the canvas plus deterministic noise.
pub struct Cell {
    pub ox: u32,
    pub oy: u32,
    pub col: u32,
    pub row: u32,
    seed: i64,
}

impl Cell {
    pub fn noise(&self, n: u32) -> f32 {
        cell_noise(self.seed, n)
    }

    /// A noise-driven offset in `lo..lo+span` pixels.
    pub fn jitter(&self, n: u32, lo: u32, span: u32) -> u32 {
        lo + (self.noise(n) * span as f32) as u32
    }
}

/// Paint a whole layout with the given per-tile painter.
pub fn bake(layout: &impl Layout, paint: impl Fn(&mut Canvas, &Cell, Tile)) -> Image {
    let mut canvas = Canvas::new(layout.cols() * SRC, layout.rows() * SRC);
    for row in 0..layout.rows() {
        for col in 0..layout.cols() {
            let cell = Cell { ox: col * SRC, oy: row * SRC, col, row, seed: cell_seed(col, row) };
            paint(&mut canvas, &cell, layout.tile(col, row));
        }
    }
    canvas.into_image()
}

/// Nearest-neighbour downsample so painted portraits/visions read as pixel
/// art (longest side becomes `low_res` px). Returns None if not RGBA8 data.
pub fn pixelate(src: &Image, low_res: u32) -> Option<Image> {
    let (w, h) = (src.width(), src.height());
    let data = src.data.as_ref()?;
    if w == 0 || h == 0 || data.len() < (w * h * 4) as usize {
        return None;
    }
    let s = low_res as f32 / w.max(h) as f32;
    let (nw, nh) = (((w as f32 * s) as u32).max(1), ((h as f32 * s) as u32).max(1));
    let mut out = Canvas::new(nw, nh);
    for y in 0..nh {
        for x in 0..nw {
            let (sx, sy) = ((x * w / nw).min(w - 1), (y * h / nh).min(h - 1));
            let i = ((sy * w + sx) * 4) as usize;
            let o = ((y * nw + x) * 4) as usize;
            out.data[o..o + 4].copy_from_slice(&data[i..i + 4]);
        }
    }
    Some(out.into_image())
}
