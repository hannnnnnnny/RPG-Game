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
