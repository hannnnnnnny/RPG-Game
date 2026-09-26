//! Map layouts on a 48px grid, in *map space*: origin top-left, y down
//! (the design docs' coordinates). The engine converts to world space.

pub const TILE: f32 = 48.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Floor,
    Path,
    Puddle,
    Ground,
    Plank,
}

impl Tile {
    pub fn is_solid(self) -> bool {
        self == Tile::Wall
    }
}

/// Axis-aligned rectangle in map space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// A map: its pixel size and what lies under each grid cell.
pub trait Layout {
    fn size(&self) -> (f32, f32);
    fn classify_point(&self, x: f32, y: f32) -> Tile;

    fn cols(&self) -> u32 {
        (self.size().0 / TILE).ceil() as u32
    }

    fn rows(&self) -> u32 {
        (self.size().1 / TILE).ceil() as u32
    }

    /// A cell is classified by its centre point.
    fn tile(&self, col: u32, row: u32) -> Tile {
        let half = TILE / 2.0;
        self.classify_point(col as f32 * TILE + half, row as f32 * TILE + half)
    }
}

/// Greedy horizontal merge: one rect per contiguous solid run per row, so a
/// map needs dozens of colliders instead of hundreds.
pub fn wall_rects(layout: &impl Layout) -> Vec<Rect> {
    let mut out = Vec::new();
    for row in 0..layout.rows() {
        let mut col = 0;
        while col < layout.cols() {
            if !layout.tile(col, row).is_solid() {
                col += 1;
                continue;
            }
            let start = col;
            while col < layout.cols() && layout.tile(col, row).is_solid() {
                col += 1;
            }
            let w = (col - start) as f32 * TILE;
            out.push(Rect::new(start as f32 * TILE, row as f32 * TILE, w, TILE));
        }
    }
    out
}

/// Deterministic per-cell noise in [0,1): the same map every run.
pub fn cell_noise(seed: i64, n: u32) -> f32 {
    ((seed as f64) * f64::from(n + 1)).sin().abs().fract() as f32
}

pub fn cell_seed(col: u32, row: u32) -> i64 {
    (i64::from(col) * 73_856_093) ^ (i64::from(row) * 19_349_663)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Box3;
    impl Layout for Box3 {
        fn size(&self) -> (f32, f32) {
            (TILE * 3.0, TILE * 3.0)
        }
        fn classify_point(&self, x: f32, y: f32) -> Tile {
            if Rect::new(TILE, TILE, TILE, TILE).contains(x, y) { Tile::Floor } else { Tile::Wall }
        }
    }

    #[test]
    fn walls_merge_per_row() {
        let rects = wall_rects(&Box3);
        // Row 0: one full run, row 1: two single cells, row 2: one full run.
        assert_eq!(rects.len(), 4);
        assert_eq!(rects[0], Rect::new(0.0, 0.0, TILE * 3.0, TILE));
    }

    #[test]
    fn noise_is_deterministic_and_in_range() {
        let a = cell_noise(cell_seed(3, 4), 2);
        assert_eq!(a, cell_noise(cell_seed(3, 4), 2));
        assert!((0.0..1.0).contains(&a));
    }
}
