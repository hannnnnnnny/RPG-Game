//! Shared UI look: the Zpix CJK pixel font and the dark-glass palette.

use bevy::prelude::*;

pub const PANEL_BG: Color = Color::srgba(0.07, 0.06, 0.05, 0.88);
pub const PANEL_BORDER: Color = Color::srgba(0.6, 0.45, 0.25, 0.45);
pub const TEXT: Color = Color::srgb(0.95, 0.92, 0.82);
pub const TEXT_MUTED: Color = Color::srgb(0.66, 0.62, 0.55);

/// Zpix covers the CJK glyphs the default font lacks (no tofu boxes).
#[derive(Resource)]
pub struct Fonts {
    pub body: Handle<Font>,
}

pub fn plugin(app: &mut App) {
    // PreStartup so every Startup system can already use the font.
    app.add_systems(PreStartup, |mut commands: Commands, assets: Res<AssetServer>| {
        commands.insert_resource(Fonts { body: assets.load("fonts/zpix.ttf") });
    });
}

pub fn text_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() }
}

pub fn label(font: &Handle<Font>, s: &str, size: f32, color: Color) -> impl Bundle {
    (Text::new(s), text_font(font, size), TextColor(color))
}
