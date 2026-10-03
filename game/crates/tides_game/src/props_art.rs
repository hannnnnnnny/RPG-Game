//! Procedural pixel art for town props, painted at 1x and shown at the
//! world's ×3 scale. Coordinates are (x, y) from the sprite's top-left.

use bevy::prelude::*;

use crate::paint::Canvas;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropArt {
    Fountain,
    Notice,
    Door,
    Anvil,
    LampPost,
    Barrel,
    Crate,
}

/// World scale for props (matches the 16px→48px tiles).
pub const PROP_SCALE: f32 = 3.0;

pub fn image(kind: PropArt) -> Image {
    match kind {
        PropArt::Fountain => fountain(),
        PropArt::Notice => notice(),
        PropArt::Door => door(),
        PropArt::Anvil => anvil(),
        PropArt::LampPost => lamp_post(),
        PropArt::Barrel => barrel(),
        PropArt::Crate => crate_box(),
    }
    .into_image()
}

type Rgb = [u8; 3];
const OUTLINE: Rgb = [40, 26, 18];

fn fountain() -> Canvas {
    let mut c = Canvas::new(32, 22);
    let stone = [132, 130, 128];
    let stone_hi = [168, 166, 160];
    c.fill(1, 10, 30, 11, OUTLINE);
    c.fill(2, 11, 28, 9, stone);
    c.fill(2, 11, 28, 1, stone_hi);
    c.fill(4, 12, 24, 5, [70, 110, 150]); // water
    c.fill(5, 13, 6, 1, [150, 190, 220]);
    c.fill(18, 15, 5, 1, [150, 190, 220]);
    c.fill(13, 3, 6, 10, OUTLINE); // centre column
    c.fill(14, 4, 4, 9, stone);
    c.fill(14, 4, 1, 9, stone_hi);
    c.fill(11, 1, 10, 3, OUTLINE); // spout bowl
    c.fill(12, 2, 8, 1, stone_hi);
    c.fill(15, 0, 2, 2, [150, 190, 220]); // jet
    c
}

fn notice() -> Canvas {
    let mut c = Canvas::new(18, 20);
    let post = [96, 62, 36];
    c.fill(2, 10, 2, 10, post);
    c.fill(14, 10, 2, 10, post);
    c.fill(0, 0, 18, 13, OUTLINE);
    c.fill(1, 1, 16, 11, [150, 100, 58]);
    c.fill(1, 1, 16, 1, [182, 128, 76]);
    for (x, y, w, h) in [(3, 3, 5, 6), (9, 2, 6, 4), (10, 7, 5, 4)] {
        c.fill(x, y, w, h, [232, 222, 196]); // pinned papers
        c.fill(x + 1, y + 1, w - 2, 1, [150, 140, 120]);
    }
    c.px(5, 3, [180, 40, 30]);
    c.px(12, 2, [180, 40, 30]);
    c
}

fn door() -> Canvas {
    let mut c = Canvas::new(16, 22);
    c.fill(0, 0, 16, 22, OUTLINE);
    c.fill(1, 1, 14, 21, [110, 68, 36]);
    for x in [4, 8, 12] {
        c.fill(x, 2, 1, 20, [86, 50, 26]);
    }
    c.fill(1, 1, 14, 2, [70, 42, 22]); // lintel
    c.fill(11, 12, 2, 2, [220, 180, 90]); // handle
    c
}

fn anvil() -> Canvas {
    let mut c = Canvas::new(30, 16);
    // Coal forge on the left.
    c.fill(0, 4, 11, 12, OUTLINE);
    c.fill(1, 5, 9, 10, [70, 62, 62]);
    c.fill(2, 6, 7, 4, [236, 120, 48]);
    c.fill(3, 7, 3, 2, [255, 200, 100]);
    // Stump + anvil on the right.
    c.fill(15, 9, 12, 7, [92, 62, 38]);
    c.fill(15, 9, 12, 1, [128, 90, 56]);
    c.fill(12, 3, 18, 6, OUTLINE);
    c.fill(13, 4, 16, 4, [70, 72, 82]);
    c.fill(13, 4, 16, 1, [150, 154, 166]);
    c.fill(10, 4, 3, 3, [70, 72, 82]); // horn
    c
}

fn lamp_post() -> Canvas {
    let mut c = Canvas::new(8, 22);
    c.fill(3, 6, 2, 16, [52, 40, 32]);
    c.fill(1, 20, 6, 2, [52, 40, 32]);
    c.fill(0, 0, 8, 7, OUTLINE);
    c.fill(1, 1, 6, 5, [255, 214, 130]);
    c.fill(1, 1, 6, 1, [255, 240, 190]);
    c
}

fn barrel() -> Canvas {
    let mut c = Canvas::new(12, 14);
    c.fill(1, 0, 10, 14, OUTLINE);
    c.fill(2, 1, 8, 12, [150, 94, 48]);
    c.fill(2, 3, 8, 1, [80, 80, 86]);
    c.fill(2, 10, 8, 1, [80, 80, 86]);
    c.fill(3, 1, 1, 12, [176, 116, 64]);
    c
}

fn crate_box() -> Canvas {
    let mut c = Canvas::new(14, 13);
    c.fill(0, 0, 14, 13, OUTLINE);
    c.fill(1, 1, 12, 11, [170, 118, 64]);
    c.fill(1, 1, 12, 1, [200, 150, 90]);
    for i in 0..11 {
        c.px(1 + i, 1 + i, [120, 80, 42]); // diagonal brace
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_prop_paints_something() {
        for kind in [PropArt::Fountain, PropArt::Notice, PropArt::Door, PropArt::Anvil, PropArt::LampPost, PropArt::Barrel, PropArt::Crate] {
            let img = image(kind);
            let opaque = img.data.as_ref().unwrap().chunks(4).filter(|p| p[3] > 0).count();
            assert!(opaque > 20, "{kind:?} is empty");
        }
    }
}
