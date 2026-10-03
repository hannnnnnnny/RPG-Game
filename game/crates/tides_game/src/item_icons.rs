//! 16px item icons: one silhouette per equipment slot, metal/cloth tinted by
//! quality so a glance at the bag tells rare from broken (Stardew-style).

use std::collections::HashMap;

use bevy::prelude::*;
use tides_core::item::{Quality, Slot};
use tides_core::forge::Material;
use tides_core::shop::Supply;

use crate::paint::Canvas;

type Rgb = [u8; 3];
const OUT: Rgb = [38, 22, 14];

/// Main tint + highlight for a quality.
fn palette(q: Quality) -> (Rgb, Rgb) {
    match q {
        Quality::Broken => ([120, 110, 100], [150, 140, 128]),
        Quality::Common => ([176, 180, 188], [226, 230, 236]),
        Quality::Rare => ([70, 120, 210], [140, 190, 250]),
        Quality::Corrupted => ([130, 60, 170], [200, 130, 240]),
        Quality::Relic => ([210, 150, 40], [250, 214, 110]),
        Quality::Mythic => ([210, 60, 60], [255, 150, 140]),
    }
}

#[derive(Resource, Default)]
pub struct ItemIcons {
    gear: HashMap<(Slot, Quality), Handle<Image>>,
    supplies: HashMap<Supply, Handle<Image>>,
    materials: HashMap<Material, Handle<Image>>,
}

impl ItemIcons {
    pub fn get(&mut self, slot: Slot, q: Quality, images: &mut Assets<Image>) -> Handle<Image> {
        self.gear.entry((slot, q)).or_insert_with(|| images.add(paint(slot, q).into_image())).clone()
    }

    pub fn supply(&mut self, s: Supply, images: &mut Assets<Image>) -> Handle<Image> {
        self.supplies.entry(s).or_insert_with(|| images.add(paint_supply(s).into_image())).clone()
    }

    pub fn material(&mut self, m: Material, images: &mut Assets<Image>) -> Handle<Image> {
        self.materials.entry(m).or_insert_with(|| images.add(paint_material(m).into_image())).clone()
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ItemIcons>();
}

/// Draw a shape given as rows of chars, colouring each char via `color`
/// (`None` = transparent).
fn stamp_with(rows: &[&str], color: impl Fn(char) -> Option<Rgb>) -> Canvas {
    let mut c = Canvas::new(16, 16);
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some(col) = color(ch) {
                c.px(x as u32, y as u32, col);
            }
        }
    }
    c
}

/// Gear chars: '#' outline, 'm' main, 'h' highlight, 'w' wood/leather, 'g' gem.
fn stamp(rows: &[&str], q: Quality) -> Canvas {
    let (m, h) = palette(q);
    stamp_with(rows, |ch| match ch {
        '#' => Some(OUT),
        'm' => Some(m),
        'h' => Some(h),
        'w' => Some([120, 76, 40]),
        'g' => Some([230, 60, 90]),
        _ => None,
    })
}

/// Supply chars: '#' outline, 'l' liquid, 'L' liquid shine, 'b' glass,
/// 'k' cork / clay, 's' steam.
pub fn paint_supply(s: Supply) -> Canvas {
    let (rows, liquid, shine): (&[&str], Rgb, Rgb) = match s {
        Supply::HealingDraught => (&[
            "................", "......####......", "......#kk#......", "......####......",
            ".......#b#......", ".......#b#......", ".....##bb##.....", "....#bbbbbb#....",
            "...#llllllll#...", "...#lLllllll#...", "...#lLllllll#...", "...#llllllll#...",
            "....#llllll#....", ".....######.....", "................", "................",
        ], [200, 40, 50], [255, 140, 140]),
        Supply::CalmingTea => (&[
            "................", "......s..s......", ".......s..s.....", "......s..s......",
            "................", "...##########...", "...#llllllll####", "...#kLLllllk#..#",
            "...#kkkkkkkk#..#", "...#kkkkkkkk####", "...#kkkkkkkk#...", "....#kkkkkk#....",
            ".....######.....", "...##########...", "................", "................",
        ], [120, 150, 90], [170, 200, 130]),
    };
    stamp_with(rows, |ch| match ch {
        '#' => Some(OUT),
        'l' => Some(liquid),
        'L' => Some(shine),
        'b' => Some([200, 220, 230]),
        'k' => Some([176, 120, 70]),
        's' => Some([230, 230, 236]),
        _ => None,
    })
}

pub fn paint(slot: Slot, q: Quality) -> Canvas {
    let rows: &[&str] = match slot {
        Slot::MainHand | Slot::OffHand => &[
            "............##..", "...........#hh#.", "..........#hm#..", ".........#hm#...",
            "........#hm#....", ".......#hm#.....", "......#hm#......", ".....#hm#.......",
            "..#.#hm#........", "..##hm#.........", "...##w#.........", "..#w##..........",
            ".#w#.#..........", "#w#.............", "##..............", "................",
        ],
        Slot::Chest => &[
            "................", "...##......##...", "..#hm######mh#..", ".#hmmmhhhhmmmh#.",
            ".#mmmmmmmmmmmm#.", ".##mmmmmmmmmm##.", "..#mmmmhhmmmm#..", "..#mmmmmmmmmm#..",
            "..#mmmmmmmmmm#..", "..#mmmmhhmmmm#..", "..#mmmmmmmmmm#..", "..#mmmmmmmmmm#..",
            "..#hhhhhhhhhh#..", "..############..", "................", "................",
        ],
        Slot::Hands | Slot::Head => &[
            "................", "....#.#.#.......", "...#h#h#h#......", "...#m#m#m#.#....",
            "...#m#m#m##h#...", "...#mmmmmm#m#...", "...#mmmmmmmm#...", "...#mmmmmmm#....",
            "...#mmmmmm#.....", "...#wwwwww#.....", "...#wwwwww#.....", "....######......",
            "................", "................", "................", "................",
        ],
        Slot::Boots => &[
            "................", "................", "....######......", "....#mmmm#......",
            "....#mhmm#......", "....#mmmm#......", "....#mmmm#......", "....#mmmm#......",
            "....#mmmm####...", "....#mmmmmmmm#..", "....#mmmmmmmmm#.", "....#hhhhhhhhh#.",
            "....#wwwwwwwww#.", "....###########.", "................", "................",
        ],
        Slot::Ring | Slot::Amulet => &[
            "................", "................", "......###.......", ".....#ggg#......",
            "....#hmgmh#.....", "....##mmm##.....", "...#m#...#m#....", "..#m#.....#m#...",
            "..#m#.....#m#...", "..#h#.....#h#...", "...#m#...#m#....", "....#mmmmm#.....",
            ".....#####......", "................", "................", "................",
        ],
        Slot::Totem => &[
            "................", ".......##.......", "......#hh#......", "......#hm#......",
            ".....#hmmm#.....", ".....#mmgm#.....", "....#hmmmmm#....", "....#mmmmmm#....",
            "....#mmgmmm#....", "...#hmmmmmmm#...", "...#mmmmmmmm#...", "...##########...",
            "...#wwwwwwww#...", "...##########...", "................", "................",
        ],
    };
    stamp(rows, q)
}

/// Material chars: '#' outline, 'm' main, 'h' highlight, 'd' shade.
pub fn paint_material(m: Material) -> Canvas {
    let (rows, main, hi, dark): (&[&str], Rgb, Rgb, Rgb) = match m {
        // A bundle of iron scraps tied with cord.
        Material::Common => (&[
            "................", "................", "................", "....##..........",
            "...#hm#...##....", "...#mmm#.#hm#...", "....#mmm#mmd#...", ".....#mmmmd#....",
            "....##wwwww##...", "...#hmmmmmmmd#..", "..#hmmmdmmmmmd#.", "..#mmmmmmmdmmd#.",
            "...#ddmmmmmdd#..", "....#########...", "................", "................",
        ], [150, 150, 158], [210, 212, 220], [100, 100, 108]),
        // A cut blue crystal.
        Material::Rare => (&[
            "................", ".......##.......", "......#hh#......", ".....#hhmm#.....",
            "....#hhmmmd#....", "...#hhmmmmdd#...", "...#hmmmmmdd#...", "...#hmmmmmdd#...",
            "...#hmmmmmdd#...", "...#hmmmmmdd#...", "....#mmmmdd#....", ".....#mmdd#.....",
            "......#dd#......", ".......##.......", "................", "................",
        ], [70, 130, 220], [170, 210, 255], [40, 80, 160]),
        // A blot of black-tide sludge.
        Material::CorruptResidue => (&[
            "................", "................", "................", "................",
            "......###.......", ".....#hmm#..##..", "....#hmmmm##hm#.", "...#hmmmmmmmmd#.",
            "..#mmmmmmmmmmd#.", "..#mmmmdmmmmdd#.", "...#mmmddmmdd#..", "....##dddddd#...",
            "......######....", "................", "................", "................",
        ], [90, 40, 110], [170, 100, 200], [50, 20, 64]),
    };
    stamp_with(rows, |ch| match ch {
        '#' => Some(OUT),
        'm' => Some(main),
        'h' => Some(hi),
        'd' => Some(dark),
        'w' => Some([176, 120, 70]),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_icon_is_16x16_and_not_empty() {
        for slot in [Slot::MainHand, Slot::Chest, Slot::Hands, Slot::Boots, Slot::Ring, Slot::Totem] {
            let c = paint(slot, Quality::Rare);
            assert_eq!((c.w, c.h), (16, 16));
            assert!(c.data.chunks(4).filter(|p| p[3] > 0).count() > 30, "{slot:?}");
        }
    }

    #[test]
    fn every_material_icon_is_painted() {
        for m in Material::ALL {
            let c = paint_material(m);
            assert!(c.data.chunks(4).filter(|p| p[3] > 0).count() > 30, "{m:?}");
        }
    }

    #[test]
    fn every_supply_icon_is_painted() {
        for s in Supply::ALL {
            let c = paint_supply(s);
            assert!(c.data.chunks(4).filter(|p| p[3] > 0).count() > 30, "{s:?}");
        }
    }
}
