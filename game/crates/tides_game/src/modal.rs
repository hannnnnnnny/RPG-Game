//! Full-screen story modals: the permanent choice and the totem vision.
//! Rebuilt whenever `Modal` changes; player control is frozen meanwhile.

use bevy::prelude::*;
use tides_core::story::{Choice, Vision, mine};
use tides_core::world::DwarfChoice;

use crate::beats::{Modal, PlayBeat};
use crate::pixelated::Pixelated;
use crate::run_state::RunRes;
use crate::ui::{Fonts, TEXT, TEXT_MUTED, text_font};

#[derive(Component)]
struct ModalRoot;

#[derive(Component)]
struct OptionButton(DwarfChoice);

#[derive(Component)]
struct CloseVision;

const GOLD: Color = Color::srgb(0.85, 0.71, 0.38);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (rebuild, choose, close_vision, hover));
}

fn rebuild(
    mut commands: Commands,
    modal: Res<Modal>,
    roots: Query<Entity, With<ModalRoot>>,
    fonts: Res<Fonts>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut pixelated: ResMut<Pixelated>,
) {
    if !modal.is_changed() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let root = commands.spawn(backdrop()).id();
    match &*modal {
        Modal::None => {
            commands.entity(root).despawn();
        }
        Modal::Choice(c) => {
            commands.entity(root).with_children(|p| choice_panel(p, c, &fonts.body));
        }
        Modal::Vision(v) => {
            let img = pixelated.get(v.image, 160, &assets, &mut images);
            commands.entity(root).with_children(|p| vision_panel(p, v, img, &fonts.body));
        }
    }
}

fn backdrop() -> impl Bundle {
    (
        ModalRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.66)),
        GlobalZIndex(10),
    )
}

fn panel_node(width: f32) -> Node {
    Node {
        width: px(width),
        flex_direction: FlexDirection::Column,
        row_gap: px(12),
        padding: px(24).all(),
        border: px(1).all(),
        border_radius: BorderRadius::all(px(10)),
        ..default()
    }
}

fn choice_panel(p: &mut ChildSpawnerCommands, c: &Choice, font: &Handle<Font>) {
    p.spawn((panel_node(620.0), BackgroundColor(Color::srgba(0.06, 0.05, 0.07, 0.97)), BorderColor::all(GOLD)))
        .with_children(|p| {
            p.spawn((Text::new(c.title), text_font(font, 24.0), TextColor(GOLD)));
            p.spawn((Text::new(c.body), text_font(font, 15.0), TextColor(TEXT)));
            for o in &c.options {
                p.spawn((
                    OptionButton(o.choice),
                    Button,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: px(10).all(),
                        border: px(1).all(),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)),
                    BorderColor::all(Color::srgba(0.85, 0.71, 0.38, 0.35)),
                    children![
                        (Text::new(o.label), text_font(font, 17.0), TextColor(TEXT)),
                        (Text::new(o.description), text_font(font, 12.0), TextColor(TEXT_MUTED)),
                    ],
                ));
            }
        });
}

fn vision_panel(p: &mut ChildSpawnerCommands, v: &Vision, img: Handle<Image>, font: &Handle<Font>) {
    p.spawn((panel_node(560.0), BackgroundColor(Color::srgba(0.03, 0.02, 0.05, 0.97)), BorderColor::all(Color::srgb(0.56, 0.31, 0.64))))
        .with_children(|p| {
            p.spawn((ImageNode::new(img), Node { width: px(512), height: px(320), ..default() }));
            p.spawn((Text::new(v.caption), text_font(font, 15.0), TextColor(TEXT)));
            p.spawn((CloseVision, Button, Node { padding: px(8).all(), align_self: AlignSelf::End, ..default() },
                children![(Text::new("醒来  (E)"), text_font(font, 14.0), TextColor(TEXT_MUTED))]));
        });
}

fn choose(
    q: Query<(&Interaction, &OptionButton), Changed<Interaction>>,
    mut modal: ResMut<Modal>,
    mut run: ResMut<RunRes>,
    mut beats: MessageWriter<PlayBeat>,
) {
    for (i, opt) in &q {
        if *i == Interaction::Pressed {
            *modal = Modal::None;
            beats.write(PlayBeat(mine::choose(&mut run, opt.0)));
        }
    }
}

fn close_vision(keys: Res<ButtonInput<KeyCode>>, q: Query<&Interaction, With<CloseVision>>, mut modal: ResMut<Modal>) {
    let clicked = q.iter().any(|i| *i == Interaction::Pressed);
    if matches!(*modal, Modal::Vision(_)) && (clicked || keys.just_pressed(KeyCode::KeyE)) {
        *modal = Modal::None;
    }
}

fn hover(mut q: Query<(&Interaction, &mut BackgroundColor), (With<OptionButton>, Changed<Interaction>)>) {
    for (i, mut bg) in &mut q {
        bg.0 = match i {
            Interaction::Hovered => Color::srgba(0.85, 0.71, 0.38, 0.14),
            _ => Color::srgba(1.0, 1.0, 1.0, 0.04),
        };
    }
}
