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
            let seg_len = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
            (d(a) + d(b) - seg_len).abs() < 26.0
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

// ============================ 灰灯镇 ============================

/// A building: footprint + display name. Its door sits at the bottom centre.
#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub rect: Rect,
    pub name: &'static str,
}

impl Building {
    /// Just outside the front wall, where the player stands to enter.
    pub fn door(&self) -> Pt {
        (self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h + 18.0)
    }
}

pub struct Town;

impl Town {
    pub const SIZE: (f32, f32) = (1300.0, 820.0);
    /// South gate gap in the border wall (x range), where the mine road arrives.
    pub const GATE: (f32, f32) = (580.0, 720.0);
    pub const ENTRANCE: Pt = (650.0, 760.0);
    pub const BUILDINGS: [Building; 5] = [
        Building { rect: Rect::new(120.0, 110.0, 300.0, 170.0), name: "自治会会堂" },
        Building { rect: Rect::new(520.0, 90.0, 240.0, 150.0), name: "杂货铺" },
        Building { rect: Rect::new(880.0, 130.0, 280.0, 180.0), name: "民居" },
        Building { rect: Rect::new(150.0, 520.0, 260.0, 160.0), name: "民居" },
        Building { rect: Rect::new(900.0, 520.0, 260.0, 170.0), name: "铁匠铺" },
    ];
    pub const PLAZA: Rect = Rect::new(470.0, 350.0, 420.0, 230.0);
    /// The gray lamps the town clings to.
    pub const LAMPS: [Pt; 6] =
        [(470.0, 350.0), (890.0, 350.0), (470.0, 580.0), (890.0, 580.0), (680.0, 300.0), (680.0, 700.0)];
    pub const ANVIL: Pt = (1080.0, 716.0);
    pub const FOUNTAIN: Pt = (680.0, 430.0);
    pub const NOTICE: Pt = (540.0, 600.0);
    /// Front wall height: the rest of a building's footprint is roof.
    const FACADE_H: f32 = 72.0;

    fn in_border(x: f32, y: f32) -> bool {
        let (w, h) = Self::SIZE;
        let edge = x < 40.0 || x > w - 40.0 || y < 40.0 || y > h - 40.0;
        let gate = y > h - 60.0 && x > Self::GATE.0 && x < Self::GATE.1;
        edge && !gate
    }

    /// Dirt paths: gate → plaza, and plaza → every door.
    fn on_path(x: f32, y: f32) -> bool {
        let near = |a: Pt, b: Pt| {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len2 = dx * dx + dy * dy;
            let t = (((x - a.0) * dx + (y - a.1) * dy) / len2).clamp(0.0, 1.0);
            let (px, py) = (a.0 + t * dx, a.1 + t * dy);
            (x - px).powi(2) + (y - py).powi(2) < 26.0 * 26.0
        };
        let (cx, cy) = Self::PLAZA.center();
        near((650.0, Self::SIZE.1), (cx, cy)) || Self::BUILDINGS.iter().any(|b| near(b.door(), (cx, cy)))
    }
}

impl Layout for Town {
    fn size(&self) -> (f32, f32) {
        Self::SIZE
    }

    fn classify_point(&self, x: f32, y: f32) -> Tile {
        if Self::in_border(x, y) {
            return Tile::Wall;
        }
        if let Some(b) = Self::BUILDINGS.iter().find(|b| b.rect.contains(x, y)) {
            let facade_top = b.rect.y + b.rect.h - Self::FACADE_H;
            return if y >= facade_top { Tile::Facade } else { Tile::Roof };
        }
        if Self::PLAZA.contains(x, y) {
            Tile::Plank
        } else if Self::on_path(x, y) {
            Tile::Ground
        } else {
            Tile::Grass
        }
    }
}

#[cfg(test)]
mod town_tests {
    use super::*;

    #[test]
    fn town_entrance_doors_and_props_are_walkable() {
        let mut spots = vec![Town::ENTRANCE, Town::ANVIL, Town::NOTICE];
        spots.extend(Town::BUILDINGS.iter().map(|b| b.door()));
        for p in spots {
            assert!(walkable(&Town, p.0, p.1), "{p:?} blocked");
        }
    }

    #[test]
    fn buildings_have_roof_above_facade() {
        let b = Town::BUILDINGS[1].rect;
        assert_eq!(Town.classify_point(b.x + 20.0, b.y + 10.0), Tile::Roof);
        assert_eq!(Town.classify_point(b.x + 20.0, b.y + b.h - 10.0), Tile::Facade);
    }

    #[test]
    fn gate_is_open() {
        assert!(walkable(&Town, 650.0, 800.0));
        assert!(!walkable(&Town, 300.0, 800.0));
    }
}
