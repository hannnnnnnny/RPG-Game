//! 黑潮矿区 tile art: violet brick walls, cave-stone floor, the old rail
//! path and black-tide puddles. Port of the Godot `_paint_tile`.

use tides_core::map::Tile;

use crate::paint::{Canvas, Cell};

const MORTAR: [u8; 3] = [28, 20, 34];

pub fn paint(c: &mut Canvas, cell: &Cell, tile: Tile) {
    match tile {
        Tile::Wall => wall(c, cell),
        Tile::Puddle => puddle(c, cell),
        Tile::Path => path(c, cell),
        _ => floor(c, cell),
    }
}

/// Two-block brickwork with a lit top lip so walls read as stone, not void.
fn wall(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [52, 42, 60]);
    c.fill(x, y, 16, 2, [78, 62, 92]);
    c.fill(x, y, 2, 16, [66, 52, 78]);
    c.fill(x, y + 14, 16, 2, [26, 18, 32]);
    c.fill(x + 14, y, 2, 16, [30, 22, 36]);
    c.fill(x, y + 7, 16, 1, MORTAR);
    // Offset seams row to row → brick bond.
    let (top, bottom) = if cell.row.is_multiple_of(2) { (8, 4) } else { (4, 11) };
    c.fill(x + top, y, 1, 7, MORTAR);
    c.fill(x + bottom, y + 8, 1, 6, MORTAR);
    if cell.noise(1) < 0.26 {
        c.px(x + cell.jitter(2, 4, 8), y + cell.jitter(3, 4, 8), [124, 70, 160]);
    }
    if cell.noise(4) < 0.12 {
        c.px(x + cell.jitter(5, 3, 9), y + cell.jitter(6, 3, 9), [92, 78, 104]);
    }
}

fn puddle(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [43, 22, 56]);
    c.fill(x + 2, y + 3, 5, 1, [61, 29, 78]);
    c.fill(x + 8, y + 7, 5, 1, [61, 29, 78]);
    c.fill(x + 3, y + 11, 6, 1, [61, 29, 78]);
    c.px(x + 5, y + 5, [120, 64, 180]);
    c.px(x + 11, y + 10, [120, 64, 180]);
}

fn path(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [74, 58, 38]);
    for i in 0..7 {
        c.px(x + cell.jitter(i + 1, 0, 16), y + cell.jitter(i + 8, 0, 16), [94, 74, 48]);
    }
    c.px(x + 4, y + 9, [53, 39, 22]);
    c.px(x + 11, y + 3, [53, 39, 22]);
}

/// Cave stone with tone variation, bevels, cracks and pebbles.
fn floor(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    let base = match cell.noise(2) {
        v if v >= 0.85 => [33, 27, 36],
        v if v >= 0.6 => [45, 37, 47],
        _ => [38, 32, 40],
    };
    c.fill(x, y, 16, 16, base);
    c.fill(x, y, 16, 1, [53, 43, 57]);
    c.fill(x, y, 1, 16, [53, 43, 57]);
    c.fill(x, y + 15, 16, 1, [24, 19, 27]);
    c.fill(x + 15, y, 1, 16, [24, 19, 27]);
    if cell.noise(4) < 0.22 {
        for (dx, dy) in [(5, 9), (6, 10), (7, 10)] {
            c.px(x + dx, y + dy, [22, 17, 25]);
        }
    }
    if cell.noise(3) < 0.2 {
        c.px(x + cell.jitter(4, 3, 9), y + cell.jitter(5, 3, 9), [64, 52, 68]);
    }
    if cell.noise(6) < 0.08 {
        c.px(x + cell.jitter(7, 5, 6), y + cell.jitter(8, 5, 6), [122, 90, 132]);
    }
}
