//! Bottom dialogue box. Shows the front of `DialogueQueue`; E / Enter
//! advances. Border colour follows the line's tone; 克哈 gets a portrait.

use bevy::prelude::*;
use tides_core::story::{KHAH, Tone};

use crate::beats::{DialogueQueue, Modal};
use crate::pixelated::Pixelated;
use crate::ui::{Fonts, TEXT, text_font};

#[derive(Component)]
struct DialogueRoot;
#[derive(Component)]
struct Speaker;
#[derive(Component)]
struct Body;
#[derive(Component)]
struct Portrait;

pub fn plugin(app: &mut App) {
    app.add_message::<InteractPressed>()
        .add_systems(Startup, spawn_box).add_systems(Update, (advance, show).chain());
}

fn tone_color(tone: Tone) -> Color {
    match tone {
        Tone::Whisper => Color::srgb_u8(144, 80, 163),
        Tone::Warning => Color::srgb_u8(185, 72, 58),
        Tone::Memory => Color::srgb_u8(217, 182, 98),
    }
}

fn spawn_box(
    mut commands: Commands,
    fonts: Res<Fonts>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut pixelated: ResMut<Pixelated>,
) {
    let portrait = pixelated.get("characters/khah.jpg", 64, &assets, &mut images);
    commands.spawn((
        DialogueRoot,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(24),
            width: px(760),
            margin: UiRect::left(px(-380)),
            padding: px(14).all(),
            column_gap: px(14),
            border: px(2).all(),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.92)),
        BorderColor::all(tone_color(Tone::Memory)),
        Visibility::Hidden,
        children![
            (Portrait, ImageNode::new(portrait), Node { width: px(72), height: px(72), ..default() }),
            (
                Node { flex_direction: FlexDirection::Column, row_gap: px(6), flex_grow: 1.0, ..default() },
                children![
                    (Speaker, Text::new(""), text_font(&fonts.body, 15.0), TextColor(tone_color(Tone::Whisper))),
                    (Body, Text::new(""), text_font(&fonts.body, 16.0), TextColor(TEXT)),
                    (
                        Text::new("E / Enter 继续"),
                        text_font(&fonts.body, 11.0),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
                    ),
                ]
            ),
        ],
    ));
}

/// E/Enter is one input with two meanings: advance the text if a line is
/// showing, otherwise interact with the world. Never both on one press.
#[derive(Message, Clone, Copy)]
pub struct InteractPressed;

fn advance(
    keys: Res<ButtonInput<KeyCode>>,
    modal: Res<Modal>,
    mut queue: ResMut<DialogueQueue>,
    mut interact: MessageWriter<InteractPressed>,
) {
    if !keys.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]) || modal.is_open() {
        return;
    }
    if queue.0.pop_front().is_none() && keys.just_pressed(KeyCode::KeyE) {
        interact.write(InteractPressed);
    }
}

#[allow(clippy::type_complexity)]
fn show(
    queue: Res<DialogueQueue>,
    mut root: Single<(&mut Visibility, &mut BorderColor), With<DialogueRoot>>,
    mut speaker: Single<(&mut Text, &mut TextColor), (With<Speaker>, Without<Body>)>,
    mut body: Single<&mut Text, (With<Body>, Without<Speaker>)>,
    mut portrait: Single<&mut Node, With<Portrait>>,
) {
    if !queue.is_changed() {
        return;
    }
    let Some(line) = queue.0.front() else {
        *root.0 = Visibility::Hidden;
        return;
    };
    *root.0 = Visibility::Inherited;
    *root.1 = BorderColor::all(tone_color(line.tone));
    speaker.0.0 = line.speaker.clone();
    speaker.1.0 = tone_color(line.tone);
    body.0 = line.text.clone();
    portrait.display = if line.speaker == KHAH { Display::Flex } else { Display::None };
}

