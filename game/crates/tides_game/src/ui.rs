//! Stardew-Valley-style UI kit: chunky wooden frames around cream parchment,
//! dark-brown pixel text. Frames are painted procedurally at 16px, upscaled
//! ×3 (nearest) and drawn as 9-slices so every panel shares one look.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use crate::paint::Canvas;

/// Pixel scale of UI art (1 art pixel = 3 screen pixels, like the world).
pub const UI_PX: u32 = 3;
/// Frame border thickness in art pixels (before scaling).
const BORDER: u32 = 6;

// Parchment palette.
pub const INK: Color = Color::srgb(0.24, 0.12, 0.05);
pub const INK_SOFT: Color = Color::srgb(0.47, 0.28, 0.16);
pub const INK_RED: Color = Color::srgb(0.66, 0.16, 0.12);
pub const INK_PURPLE: Color = Color::srgb(0.42, 0.18, 0.5);
pub const CREAM: Color = Color::srgb(0.99, 0.93, 0.78);
pub const GOLD_TEXT: Color = Color::srgb(0.98, 0.84, 0.36);

#[derive(Resource)]
pub struct Fonts {
    pub body: Handle<Font>,
}

#[derive(Clone, Copy)]
pub enum Frame {
    /// Wood border + parchment: dialogue, menus, info boxes.
    Parchment,
    /// Recessed tan square: inventory/toolbar slots.
    Slot,
    /// Wood border + wood plank fill: toolbar strip, bar casings.
    Plank,
}

#[derive(Resource)]
pub struct UiKit {
    parchment: Handle<Image>,
    slot: Handle<Image>,
    plank: Handle<Image>,
}

impl UiKit {
    /// The frame as an absolutely-positioned background child that fills
    /// its parent. (An `ImageNode` on the container itself would make it a
    /// measured leaf that ignores its padding and children.)
    pub fn backdrop(&self, frame: Frame) -> impl Bundle {
        (
            self.image(frame),
            Node { position_type: PositionType::Absolute, left: px(0), right: px(0), top: px(0), bottom: px(0), ..default() },
            ZIndex(-1),
        )
    }

    fn image(&self, frame: Frame) -> ImageNode {
        let image = match frame {
            Frame::Parchment => self.parchment.clone(),
            Frame::Slot => self.slot.clone(),
            Frame::Plank => self.plank.clone(),
        };
        ImageNode {
            image,
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all((BORDER * UI_PX) as f32),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
            ..default()
        }
    }
}

pub fn plugin(app: &mut App) {
    // PreStartup so every Startup system can already build UI.
    app.add_systems(PreStartup, setup_kit);
}

fn setup_kit(mut commands: Commands, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(Fonts { body: assets.load("fonts/zpix.ttf") });
    commands.insert_resource(UiKit {
        parchment: images.add(upscale(&paint_frame(PARCHMENT_FILL, true)).into_image()),
        slot: images.add(upscale(&paint_slot()).into_image()),
        plank: images.add(upscale(&paint_frame(PLANK_FILL, false)).into_image()),
    });
}

pub fn text_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() }
}


/// Full-screen dimmed layer that centres a window (menu, forge, shop).
pub fn overlay(marker: impl Bundle) -> impl Bundle {
    (
        marker,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        GlobalZIndex(20),
    )
}

/// A wood-plank button. Disabled ones are dimmed and never get `action`
/// (or `Button`), so they can't be clicked.
pub fn plank_button(p: &mut ChildSpawnerCommands, kit: &UiKit, font: &Handle<Font>, text: &str, action: impl Bundle, enabled: bool) {
    let node = Node { padding: UiRect::axes(px(16), px(9)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() };
    let ink = if enabled { CREAM } else { CREAM.with_alpha(0.4) };
    let mut b = p.spawn((node, children![kit.backdrop(Frame::Plank), (Text::new(text), text_font(font, 14.0), TextColor(ink))]));
    if enabled {
        b.insert((action, Button));
    }
}

/// Fixed-width text column. The width lives on a wrapper: a width on the
/// text node itself makes CJK text measure as one glyph per line.
pub fn text_cell(p: &mut ChildSpawnerCommands, font: &Handle<Font>, text: impl Into<String>, size: f32, color: Color, width: f32) {
    p.spawn(Node { width: px(width), flex_shrink: 0.0, ..default() }).with_children(|p| {
        p.spawn((Text::new(text), text_font(font, size), TextColor(color)));
    });
}

// ---------------- Frame painting ----------------

const OUTLINE: [u8; 3] = [59, 28, 10];
const WOOD_HI: [u8; 3] = [232, 150, 70];
const WOOD: [u8; 3] = [196, 110, 42];
const WOOD_GRAIN: [u8; 3] = [170, 90, 32];
const WOOD_LO: [u8; 3] = [140, 70, 22];
const INNER_LINE: [u8; 3] = [92, 44, 14];
const PARCHMENT_FILL: ([u8; 3], [u8; 3]) = ([250, 224, 164], [232, 196, 128]);
const PLANK_FILL: ([u8; 3], [u8; 3]) = ([176, 96, 38], [150, 78, 28]);

/// 16×16 frame: rounded outline, bevelled wood rim, dark inner line, then
/// the fill with a soft inner shadow on its top/left edge.
fn paint_frame(fill: ([u8; 3], [u8; 3]), grain: bool) -> Canvas {
    let n = 16;
    let mut c = Canvas::new(n, n);
    for y in 0..n {
        for x in 0..n {
            let d = x.min(y).min(n - 1 - x).min(n - 1 - y);
            let top_left = x.min(y) == d;
            let col = match d {
                0 => OUTLINE,
                1 => if top_left { WOOD_HI } else { WOOD_LO },
                2 | 3 => if grain && (x + y * 3) % 7 == 0 { WOOD_GRAIN } else { WOOD },
                4 => INNER_LINE,
                5 if top_left => fill.1,
                _ => fill.0,
            };
            c.px(x, y, col);
        }
    }
    round_corners(&mut c);
    c
}

/// Recessed slot: dark rim, tan well, shadow on top/left.
fn paint_slot() -> Canvas {
    let n = 16;
    let mut c = Canvas::new(n, n);
    for y in 0..n {
        for x in 0..n {
            let d = x.min(y).min(n - 1 - x).min(n - 1 - y);
            let top_left = x.min(y) == d;
            let col = match d {
                0 => OUTLINE,
                1 => if top_left { [150, 90, 40] } else { [236, 196, 128] },
                2 if top_left => [196, 142, 82],
                _ => [222, 170, 100],
            };
            c.px(x, y, col);
        }
    }
    round_corners(&mut c);
    c
}

fn round_corners(c: &mut Canvas) {
    let n = c.w - 1;
    for (x, y) in [(0, 0), (n, 0), (0, n), (n, n)] {
        let i = ((y * c.w + x) * 4) as usize;
        c.data[i + 3] = 0;
    }
}

fn upscale(src: &Canvas) -> Canvas {
    let mut out = Canvas::new(src.w * UI_PX, src.h * UI_PX);
    for y in 0..out.h {
        for x in 0..out.w {
            let i = (((y / UI_PX) * src.w + x / UI_PX) * 4) as usize;
            let o = ((y * out.w + x) * 4) as usize;
            out.data[o..o + 4].copy_from_slice(&src.data[i..i + 4]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_corners_are_transparent_and_centre_is_fill() {
        let c = paint_frame(PARCHMENT_FILL, true);
        assert_eq!(c.data[3], 0);
        let mid = ((8 * 16 + 8) * 4) as usize;
        assert_eq!(&c.data[mid..mid + 3], &PARCHMENT_FILL.0);
    }

    #[test]
    fn upscale_triples_size() {
        let c = upscale(&paint_slot());
        assert_eq!((c.w, c.h), (48, 48));
    }
}
