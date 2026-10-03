//! 灰灯镇 tile art: dusky Stardew-flavoured grass, dirt paths, a plank
//! plaza, slate roofs, timber fronts with warm windows, stone town wall.

use tides_core::map::Tile;

use crate::paint::{Canvas, Cell};

pub fn paint(c: &mut Canvas, cell: &Cell, tile: Tile) {
    match tile {
        Tile::Wall => stone_wall(c, cell),
        Tile::Roof => roof(c, cell),
        Tile::Facade => facade(c, cell),
        Tile::Plank => plank(c, cell),
        Tile::Ground => dirt(c, cell),
        _ => grass(c, cell),
    }
}

fn grass(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    let base = if cell.noise(2) > 0.7 { [70, 94, 56] } else { [76, 100, 60] };
    c.fill(x, y, 16, 16, base);
    // Blade tufts: little "v" shapes.
    for i in 0..3 {
        let (tx, ty) = (cell.jitter(10 + i, 1, 13), cell.jitter(20 + i, 2, 12));
        c.px(x + tx, y + ty, [98, 126, 72]);
        c.px(x + tx + 2, y + ty, [98, 126, 72]);
        c.px(x + tx + 1, y + ty + 1, [60, 82, 48]);
    }
    if cell.noise(5) < 0.07 {
        let (fx, fy) = (x + cell.jitter(6, 3, 10), y + cell.jitter(7, 3, 10));
        let petal = if cell.noise(8) < 0.5 { [226, 214, 160] } else { [196, 170, 214] };
        c.px(fx, fy, petal);
        c.px(fx + 1, fy + 1, [240, 200, 80]);
    }
}

fn dirt(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [126, 98, 66]);
    for i in 0..6 {
        let col = if i % 2 == 0 { [140, 110, 76] } else { [108, 82, 54] };
        c.px(x + cell.jitter(i + 1, 0, 16), y + cell.jitter(i + 9, 0, 16), col);
    }
}

fn plank(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [156, 112, 66]);
    for seam in [0, 5, 10, 15] {
        c.fill(x, y + seam, 16, 1, [112, 76, 42]);
    }
    // Staggered board ends.
    let off = if cell.row.is_multiple_of(2) { 4 } else { 11 };
    c.fill(x + off, y + 1, 1, 4, [112, 76, 42]);
    c.fill(x + (off + 7) % 16, y + 6, 1, 4, [112, 76, 42]);
    if cell.noise(1) < 0.3 {
        c.px(x + cell.jitter(2, 1, 14), y + cell.jitter(3, 1, 14), [178, 132, 80]);
    }
}

/// Slate shingles in offset rows with a dark lower edge per row.
fn roof(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [84, 86, 108]);
    for (i, row) in [0u32, 4, 8, 12].iter().enumerate() {
        c.fill(x, y + row + 3, 16, 1, [54, 54, 74]);
        let off = if (cell.row + i as u32).is_multiple_of(2) { 0 } else { 4 };
        for sx in (off..16).step_by(8) {
            c.fill(x + sx, y + row, 1, 3, [60, 60, 80]);
        }
        c.fill(x, y + row, 16, 1, [104, 106, 130]);
    }
}

/// Vertical timber boards; some cells carry a warm lit window.
fn facade(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [146, 102, 60]);
    for bx in [0, 4, 8, 12] {
        c.fill(x + bx, y, 1, 16, [116, 78, 44]);
    }
    c.fill(x, y, 16, 2, [92, 60, 34]); // beam under the eaves
    if cell.col % 3 == 1 && cell.noise(3) < 0.7 {
        c.fill(x + 4, y + 5, 8, 7, [70, 44, 24]);
        c.fill(x + 5, y + 6, 6, 5, [238, 196, 104]);
        c.fill(x + 7, y + 6, 1, 5, [70, 44, 24]);
        c.fill(x + 5, y + 8, 6, 1, [70, 44, 24]);
    }
}

fn stone_wall(c: &mut Canvas, cell: &Cell) {
    let (x, y) = (cell.ox, cell.oy);
    c.fill(x, y, 16, 16, [96, 92, 88]);
    let mortar = [64, 60, 58];
    c.fill(x, y + 7, 16, 1, mortar);
    c.fill(x, y + 15, 16, 1, mortar);
    let off = if cell.row.is_multiple_of(2) { 6 } else { 11 };
    c.fill(x + off, y, 1, 7, mortar);
    c.fill(x + (off + 8) % 16, y + 8, 1, 7, mortar);
    c.fill(x, y, 16, 1, [122, 118, 112]);
    if cell.noise(4) < 0.3 {
        c.px(x + cell.jitter(5, 2, 12), y + cell.jitter(6, 2, 12), [80, 104, 70]); // moss
    }
}
