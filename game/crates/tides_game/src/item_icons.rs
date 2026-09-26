//! 16px item icons: one silhouette per equipment slot, metal/cloth tinted by
//! quality so a glance at the bag tells rare from broken (Stardew-style).

use std::collections::HashMap;

use bevy::prelude::*;
use tides_core::item::{Quality, Slot};

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
pub struct ItemIcons(HashMap<(Slot, Quality), Handle<Image>>);

impl ItemIcons {
    pub fn get(&mut self, slot: Slot, q: Quality, images: &mut Assets<Image>) -> Handle<Image> {
        self.0.entry((slot, q)).or_insert_with(|| images.add(paint(slot, q).into_image())).clone()
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ItemIcons>();
}

/// Draw a shape given as rows of chars: '#' outline, 'm' main, 'h' highlight,
/// 'w' wood/leather, 'g' gem. '.' is transparent.
fn stamp(rows: &[&str], q: Quality) -> Canvas {
    let (m, h) = palette(q);
    let mut c = Canvas::new(16, 16);
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let col = match ch {
                '#' => OUT,
                'm' => m,
                'h' => h,
                'w' => [120, 76, 40],
                'g' => [230, 60, 90],
                _ => continue,
            };
            c.px(x as u32, y as u32, col);
        }
    }
    c
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
}
