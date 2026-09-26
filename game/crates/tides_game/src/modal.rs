//! Full-screen story modals: the permanent choice and the totem vision.
//! Rebuilt whenever `Modal` changes; player control is frozen meanwhile.

use bevy::prelude::*;
use tides_core::story::{Choice, Vision, mine};
use tides_core::world::DwarfChoice;

use crate::beats::{Modal, PlayBeat};
use crate::pixelated::Pixelated;
use crate::run_state::RunRes;
use crate::ui::{Fonts, Frame, INK, INK_RED, INK_SOFT, UiKit, text_font};

#[derive(Component)]
struct ModalRoot;

#[derive(Component)]
struct OptionButton(DwarfChoice);

#[derive(Component)]
struct CloseVision;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (rebuild, choose, close_vision, hover));
}

fn rebuild(
    mut commands: Commands,
    modal: Res<Modal>,
    roots: Query<Entity, With<ModalRoot>>,
    fonts: Res<Fonts>,
    kit: Res<UiKit>,
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
            commands.entity(root).with_children(|p| choice_panel(p, c, &kit, &fonts.body));
        }
        Modal::Vision(v) => {
            let img = pixelated.get(v.image, 160, &assets, &mut images);
            commands.entity(root).with_children(|p| vision_panel(p, v, img, &kit, &fonts.body));
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        GlobalZIndex(10),
    )
}

fn panel_node(width: f32) -> Node {
    Node {
        width: px(width),
        flex_direction: FlexDirection::Column,
        row_gap: px(10),
        padding: UiRect::axes(px(36), px(30)),
        ..default()
    }
}

/// Stardew's question box: prompt text, then answers; the hovered answer
/// gets a tan highlight and a pointer.
fn choice_panel(p: &mut ChildSpawnerCommands, c: &Choice, kit: &UiKit, font: &Handle<Font>) {
    p.spawn(panel_node(640.0)).with_children(|p| {
        p.spawn(kit.backdrop(Frame::Parchment));
        p.spawn((Text::new(c.title), text_font(font, 22.0), TextColor(INK_RED)));
        p.spawn((Text::new(c.body), text_font(font, 16.0), TextColor(INK)));
        for o in &c.options {
            p.spawn((
                OptionButton(o.choice),
                Button,
                Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(12), px(6)), ..default() },
                BackgroundColor(Color::NONE),
                children![
                    (Text::new(format!("  {}", o.label)), text_font(font, 18.0), TextColor(INK)),
                    (Text::new(format!("    {}", o.description)), text_font(font, 12.0), TextColor(INK_SOFT)),
                ],
            ));
        }
    });
}

fn vision_panel(p: &mut ChildSpawnerCommands, v: &Vision, img: Handle<Image>, kit: &UiKit, font: &Handle<Font>) {
    p.spawn(Node { align_items: AlignItems::Center, ..panel_node(600.0) }).with_children(|p| {
        p.spawn(kit.backdrop(Frame::Parchment));
        p.spawn(Node { padding: px(9).all(), ..default() }).with_children(|p| {
            p.spawn(kit.backdrop(Frame::Slot));
            p.spawn((ImageNode::new(img), Node { width: px(480), height: px(300), ..default() }));
        });
        p.spawn((Text::new(v.caption), text_font(font, 15.0), TextColor(INK)));
        p.spawn((CloseVision, Button, Node { padding: px(6).all(), align_self: AlignSelf::End, ..default() },
            children![(Text::new("醒来 (E)"), text_font(font, 14.0), TextColor(INK_SOFT))]));
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

/// Hovered answer: tan highlight and a ▶ pointer in front of its label.
fn hover(
    mut q: Query<(&Interaction, &Children, &mut BackgroundColor), (With<OptionButton>, Changed<Interaction>)>,
    mut texts: Query<&mut Text>,
) {
    for (i, children, mut bg) in &mut q {
        let hot = *i != Interaction::None;
        bg.0 = if hot { Color::srgba(0.85, 0.6, 0.3, 0.35) } else { Color::NONE };
        if let Some(mut label) = children.first().and_then(|c| texts.get_mut(*c).ok()) {
            let bare = label.0.trim_start_matches(['▶', ' ']).to_string();
            label.0 = if hot { format!("▶ {bare}") } else { format!("  {bare}") };
        }
    }
}
