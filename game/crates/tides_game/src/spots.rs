//! Mine interaction spots (受伤矮人 / 图腾残片 / 矿井出口). Pressing E near the
//! closest active one runs its story beat.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::areas::Mine;
use tides_core::story::mine::{self, Spot};
use tides_core::world::DwarfChoice;

use crate::area::Area;
use crate::beats::PlayBeat;
use crate::coords::{Z_OVERLAY, to_world};
use crate::dialogue::InteractPressed;
use crate::lighting::Light;
use crate::paint::Canvas;
use crate::physics::YSort;
use crate::player::Player;
use crate::run_state::RunRes;
use crate::ui::{Fonts, text_font};

const RANGE: f32 = 90.0;

#[derive(Component)]
pub struct SpotC(pub Spot);

#[derive(Component)]
struct Prompt;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_prompt)
        .add_systems(OnEnter(Area::Mine), (spawn_spots, queue_intro))
        .add_systems(Update, (intro, interact, update_prompt, retire_spent));
}

fn spawn_spots(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let defs = [
        (Spot::InjuredDwarf, Mine::INJURED_DWARF, Light::new(Color::srgb(1.0, 0.66, 0.45), 90.0, 0.55)),
        (Spot::TotemFragment, Mine::TOTEM, Light::new(Color::srgb(0.7, 0.35, 1.0), 150.0, 1.1).pulsing(0.35)),
        (Spot::Exit, Mine::EXIT, Light::new(Color::srgb(0.4, 0.86, 0.78), 130.0, 0.9)),
    ];
    for (spot, at, light) in defs {
        commands.spawn((
            SpotC(spot),
            DespawnOnExit(Area::Mine),
            Sprite::from_image(images.add(spot_image(spot))),
            Anchor::BOTTOM_CENTER,
            YSort,
            Transform::from_translation(to_world(at).extend(10.0)).with_scale(Vec3::splat(2.0)),
            children![(light, Transform::from_xyz(0.0, 10.0, 0.0))],
        ));
    }
}

/// Art ported from the Godot `_draw` rects (halved: those drew at 2x).
fn spot_image(spot: Spot) -> Image {
    let (w, h) = match spot {
        Spot::InjuredDwarf => (20, 10),
        Spot::TotemFragment => (8, 20),
        Spot::Exit => (30, 25),
    };
    let mut c = Canvas::new(w, h);
    let mut r = |x: i32, y: i32, rw: u32, rh: u32, col: [u8; 3]| {
        c.fill((x + w as i32 / 2) as u32, (y + h as i32) as u32, rw, rh, col);
    };
    match spot {
        Spot::InjuredDwarf => {
            r(-10, -8, 20, 7, [94, 52, 36]);
            r(-10, -5, 20, 2, [31, 18, 10]);
            r(4, -10, 6, 5, [219, 179, 138]);
            r(-5, -2, 7, 1, [122, 29, 36]);
        }
        Spot::TotemFragment => {
            r(-1, -20, 2, 17, [61, 31, 85]);
            r(-3, -16, 6, 10, [88, 42, 120]);
            r(0, -20, 1, 15, [164, 95, 209]);
            r(-4, -3, 8, 2, [42, 21, 53]);
        }
        Spot::Exit => {
            r(-15, -25, 30, 25, [28, 20, 16]);
            r(-13, -23, 26, 23, [8, 6, 8]);
            r(-3, -7, 6, 6, [58, 111, 110]);
        }
    }
    c.into_image()
}

fn nearest_active<'a>(
    player: Vec2,
    run: &RunRes,
    spots: impl Iterator<Item = (&'a SpotC, &'a Transform)>,
) -> Option<(Spot, Vec2)> {
    spots
        .filter(|(s, _)| mine::is_active(run, s.0))
        .map(|(s, tf)| (s.0, tf.translation.truncate()))
        .filter(|(_, p)| p.distance(player) < RANGE)
        .min_by(|a, b| a.1.distance(player).total_cmp(&b.1.distance(player)))
}

fn interact(
    mut pressed: MessageReader<InteractPressed>,
    player: Single<&Transform, With<Player>>,
    spots: Query<(&SpotC, &Transform)>,
    mut run: ResMut<RunRes>,
    mut beats: MessageWriter<PlayBeat>,
) {
    if pressed.read().count() == 0 {
        return;
    }
    if let Some((spot, _)) = nearest_active(player.translation.truncate(), &run, spots.iter()) {
        beats.write(PlayBeat(mine::interact(&mut run, spot)));
    }
}

/// Set on entering the mine; the whisper plays on the next Update so the
/// message isn't lost across the state transition.
#[derive(Resource)]
struct IntroPending;

fn queue_intro(mut commands: Commands) {
    commands.insert_resource(IntroPending);
}

/// 克哈's whisper each time you (re-)enter the mine.
fn intro(mut commands: Commands, pending: Option<Res<IntroPending>>, run: Res<RunRes>, mut beats: MessageWriter<PlayBeat>) {
    if pending.is_some() {
        commands.remove_resource::<IntroPending>();
        beats.write(PlayBeat(mine::on_enter(&run)));
    }
}

fn spawn_prompt(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
        Prompt,
        Text2d::new("E"),
        text_font(&fonts.body, 14.0),
        TextColor(Color::srgb(0.95, 0.9, 0.7)),
        Transform::from_xyz(0.0, 0.0, Z_OVERLAY),
        Visibility::Hidden,
    ));
}

fn update_prompt(
    player: Single<&Transform, (With<Player>, Without<Prompt>)>,
    spots: Query<(&SpotC, &Transform), Without<Prompt>>,
    run: Res<RunRes>,
    mut prompt: Single<(&mut Transform, &mut Visibility), With<Prompt>>,
) {
    match nearest_active(player.translation.truncate(), &run, spots.iter()) {
        Some((_, at)) => {
            prompt.0.translation = (at + Vec2::Y * 64.0).extend(Z_OVERLAY);
            *prompt.1 = Visibility::Inherited;
        }
        None => *prompt.1 = Visibility::Hidden,
    }
}

/// Saved (he gets up) or killed, the dwarf is gone; abandoned, he stays.
fn retire_spent(run: Res<RunRes>, mut spots: Query<(&SpotC, &mut Visibility)>) {
    if !run.is_changed() {
        return;
    }
    for (s, mut vis) in &mut spots {
        let gone = matches!(run.world.dwarf_choice, Some(DwarfChoice::Save | DwarfChoice::Kill));
        if s.0 == Spot::InjuredDwarf && gone {
            *vis = Visibility::Hidden;
        }
    }
}
