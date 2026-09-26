//! Hand-authored area layouts (map space, y down). Pure data + geometry so
//! the engine just renders and spawns what these describe.

use crate::map::{Layout, Rect, TILE, Tile};

/// A point in map space.
pub type Pt = (f32, f32);

// ============================ 黑潮矿区 ============================

pub struct Mine;

impl Mine {
    pub const ROOMS: [Rect; 5] = [
        Rect::new(60.0, 80.0, 350.0, 180.0),
        Rect::new(360.0, 185.0, 520.0, 170.0),
        Rect::new(760.0, 310.0, 350.0, 210.0),
        Rect::new(1040.0, 470.0, 240.0, 260.0),
        Rect::new(420.0, 540.0, 520.0, 180.0),
    ];
    /// Ellipses of black-tide water: (cx, cy, rx, ry).
    pub const PUDDLES: [(f32, f32, f32, f32); 3] =
        [(685.0, 260.0, 95.0, 35.0), (980.0, 445.0, 110.0, 45.0), (555.0, 640.0, 85.0, 29.0)];
    /// The old rail path the dead mine lamps still mark.
    pub const PATH: [Pt; 5] =
        [(200.0, 170.0), (620.0, 270.0), (930.0, 415.0), (1160.0, 600.0), (680.0, 630.0)];

    pub const PLAYER_START: Pt = (155.0, 165.0);
    /// Safe point after the totem vision, just short of Grom's room.
    pub const TOTEM_CHECKPOINT: Pt = (870.0, 400.0);
    pub const ENEMIES: [Pt; 3] = [(510.0, 245.0), (825.0, 395.0), (1075.0, 575.0)];
    pub const INJURED_DWARF: Pt = (395.0, 230.0);
    pub const TOTEM: Pt = (910.0, 382.0);
    pub const EXIT: Pt = (1220.0, 650.0);
    pub const BOSS: Pt = (1090.0, 520.0);

    fn in_puddle(x: f32, y: f32) -> bool {
        Self::PUDDLES.iter().any(|&(cx, cy, rx, ry)| {
            let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
            dx * dx + dy * dy <= 1.0
        })
    }

    /// Near any segment of the rail path (ellipse test on the segment ends).
    fn on_path(x: f32, y: f32) -> bool {
        Self::PATH.windows(2).any(|seg| {
            let (a, b) = (seg[0], seg[1]);
            let d = |p: Pt| ((x - p.0).powi(2) + (y - p.1).powi(2)).sqrt();
            let len = d(a).max(0.0).min(f32::MAX);
            let seg_len = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
            (len + d(b) - seg_len).abs() < 26.0
        })
    }
}

impl Layout for Mine {
    fn size(&self) -> (f32, f32) {
        (1450.0, 900.0)
    }

    fn classify_point(&self, x: f32, y: f32) -> Tile {
        if !Self::ROOMS.iter().any(|r| r.contains(x, y)) {
            Tile::Wall
        } else if Self::in_puddle(x, y) {
            Tile::Puddle
        } else if Self::on_path(x, y) {
            Tile::Path
        } else {
            Tile::Floor
        }
    }
}

/// True if a point is walkable (inside the map and not in a wall cell).
pub fn walkable(layout: &impl Layout, x: f32, y: f32) -> bool {
    let (w, h) = layout.size();
    if x < 0.0 || y < 0.0 || x >= w || y >= h {
        return false;
    }
    !layout.tile((x / TILE) as u32, (y / TILE) as u32).is_solid()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mine_spawn_is_walkable() {
        let spots = [Mine::PLAYER_START, Mine::TOTEM_CHECKPOINT, Mine::INJURED_DWARF, Mine::TOTEM, Mine::BOSS];
        for p in spots.iter().chain(Mine::ENEMIES.iter()) {
            assert!(walkable(&Mine, p.0, p.1), "{p:?} is inside a wall");
        }
    }

    #[test]
    fn outside_rooms_is_wall() {
        assert_eq!(Mine.classify_point(5.0, 5.0), Tile::Wall);
        assert_eq!(Mine.classify_point(685.0, 260.0), Tile::Puddle);
    }
}
