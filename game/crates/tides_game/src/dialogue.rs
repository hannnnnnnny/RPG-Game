//! Stardew-style dialogue box: a wide parchment box with the text on the
//! left and, for characters with a face, a framed portrait plus name plate
//! on the right. E / Enter advances.

use bevy::prelude::*;
use tides_core::story::{KHAH, Tone};

use crate::beats::{DialogueQueue, Modal};
use crate::pixelated::Pixelated;
use crate::ui::{Fonts, Frame, INK, INK_PURPLE, INK_RED, INK_SOFT, UiKit, text_font};

#[derive(Component)]
struct DialogueRoot;
#[derive(Component)]
struct Speaker;
#[derive(Component)]
struct Body;
#[derive(Component)]
struct PortraitColumn;

/// E/Enter is one input with two meanings: advance the text if a line is
/// showing, otherwise interact with the world. Never both on one press.
#[derive(Message, Clone, Copy)]
pub struct InteractPressed;

pub fn plugin(app: &mut App) {
    app.add_message::<InteractPressed>()
        .add_systems(Startup, spawn_box)
        .add_systems(Update, (advance, show).chain());
}

/// Speaker name colour carries the tone (whisper / warning / memory).
fn tone_ink(tone: Tone) -> Color {
    match tone {
        Tone::Whisper => INK_PURPLE,
        Tone::Warning => INK_RED,
        Tone::Memory => INK_SOFT,
    }
}

fn spawn_box(
    mut commands: Commands,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut pixelated: ResMut<Pixelated>,
) {
    let f = &fonts.body;
    let portrait = pixelated.get("characters/khah.jpg", 64, &assets, &mut images);
    commands.spawn((
        DialogueRoot,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(92),
            width: px(900),
            height: px(210),
            margin: UiRect::left(px(-450)),
            column_gap: px(8),
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(5),
        children![
            (
                Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: UiRect::axes(px(34), px(28)), row_gap: px(8), ..default() },
                children![
                    kit.backdrop(Frame::Parchment),
                    (Speaker, Text::new(""), text_font(f, 14.0), TextColor(INK_PURPLE)),
                    (Body, Text::new(""), text_font(f, 20.0), TextColor(INK)),
                ],
            ),
            (
                PortraitColumn,
                Node { width: px(210), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(8), padding: px(20).all(), ..default() },
                children![
                    kit.backdrop(Frame::Parchment),
                    (
                        Node { width: px(132), height: px(132), padding: px(8).all(), ..default() },
                        children![kit.backdrop(Frame::Slot), (ImageNode::new(portrait), Node { width: percent(100), height: percent(100), ..default() })],
                    ),
                    (Text::new(KHAH), text_font(f, 15.0), TextColor(INK)),
                ],
            ),
        ],
    ));
}

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
    modal: Res<Modal>,
    mut root: Single<&mut Visibility, With<DialogueRoot>>,
    mut speaker: Single<(&mut Text, &mut TextColor), (With<Speaker>, Without<Body>)>,
    mut body: Single<&mut Text, (With<Body>, Without<Speaker>)>,
    mut portrait: Single<&mut Node, With<PortraitColumn>>,
) {
    if !queue.is_changed() && !modal.is_changed() {
        return;
    }
    // Lines wait behind a vision/choice and appear once it closes.
    let front = if modal.is_open() { None } else { queue.0.front() };
    let Some(line) = front else {
        **root = Visibility::Hidden;
        return;
    };
    **root = Visibility::Inherited;
    speaker.0.0 = line.speaker.clone();
    speaker.1.0 = tone_ink(line.tone);
    body.0 = line.text.clone();
    // Only 克哈 has a face for now; others speak from the text box alone.
    portrait.display = if line.speaker == KHAH { Display::Flex } else { Display::None };
}
