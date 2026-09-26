//! 黑腕队长·格罗姆 in the world: drives the tides_core brain, moves the body,
//! resolves strikes, draws the slam shockwave and the top-of-screen HP bar.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::areas::Mine;
use tides_core::boss::{self, Grom, Move, State};
use tides_core::story::mine;
use tides_core::world::Flag;

use crate::beats::{PlayBeat, SpawnBoss};
use crate::combat::Respawn;
use crate::coords::{Z_OVERLAY, to_world};
use crate::enemy::{self, EnemyArt, PlayerStruck};
use crate::lighting::Light;
use crate::paint::Canvas;
use crate::physics::{Body, Velocity, YSort};
use crate::player::Player;
use crate::run_state::RunRes;
use crate::ui::{Fonts, text_font};

const SCALE: f32 = 2.2;

#[derive(Component)]
pub struct Boss {
    pub brain: Grom,
    lunge_dir: Vec2,
    pub flash: f32,
}

#[derive(Component)]
struct Shockwave {
    t: f32,
}

#[derive(Component)]
struct BossBar;
#[derive(Component)]
struct BossBarFill;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_bar)
        .add_systems(Update, (spawn_on_signal, spawn_on_load, think, flash, reap, expand_waves, update_bar));
}

fn spawn_on_signal(
    mut commands: Commands,
    mut signal: MessageReader<SpawnBoss>,
    mut images: ResMut<Assets<Image>>,
    mut respawn: ResMut<Respawn>,
    existing: Query<(), With<Boss>>,
) {
    if signal.read().count() == 0 || !existing.is_empty() {
        return;
    }
    // Checkpoint: dying to Grom sends you back to the totem, not the start.
    respawn.0 = to_world(Mine::TOTEM_CHECKPOINT);
    spawn(&mut commands, &mut images);
}

/// A save made after the totem but before the kill still has him waiting.
fn spawn_on_load(
    mut done: Local<bool>,
    run: Res<RunRes>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut respawn: ResMut<Respawn>,
) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    let w = &run.world;
    if w.has(Flag::TouchedTotemFragment) {
        respawn.0 = to_world(Mine::TOTEM_CHECKPOINT);
        if !w.has(Flag::DefeatedGrom) {
            spawn(&mut commands, &mut images);
        }
    }
}

fn spawn(commands: &mut Commands, images: &mut Assets<Image>) {
    commands.spawn((
        Boss { brain: Grom::default(), lunge_dir: Vec2::X, flash: 0.0 },
        crate::area::AreaEntity,
        Velocity::default(),
        Body { half: Vec2::new(14.0, 6.0) },
        YSort,
        Sprite::from_image(images.add(grom_image())),
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(to_world(Mine::BOSS).extend(10.0)).with_scale(Vec3::splat(SCALE)),
        children![(Light::new(Color::srgb(0.7, 0.25, 0.95), 170.0, 0.9).pulsing(0.2), Transform::from_xyz(0.0, 14.0, 0.0))],
    ));
}

#[allow(clippy::too_many_arguments)]
fn think(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Boss>)>,
    mut q: Query<(&mut Boss, &mut Velocity, &Transform)>,
    mut struck: MessageWriter<PlayerStruck>,
    art: Res<EnemyArt>,
) {
    let target = player.translation.truncate();
    for (mut b, mut vel, tf) in &mut q {
        let pos = tf.translation.truncate();
        let to = target - pos;
        let intent = b.brain.tick(time.delta_secs(), to.length());
        if intent.aim_lunge {
            b.lunge_dir = to.normalize_or_zero();
        }
        vel.0 = match intent.movement {
            Move::Stop => Vec2::ZERO,
            Move::Chase(s) => to.normalize_or_zero() * s,
            Move::Lunge(s) => b.lunge_dir * s,
        };
        if let Some(s) = intent.strike {
            if to.length() < s.radius {
                struck.write(PlayerStruck { damage: s.damage });
            }
            if s.shockwave {
                commands.spawn((Shockwave { t: 0.0 }, Transform::from_translation(pos.extend(Z_OVERLAY - 6.0)), Visibility::default()));
            }
        }
        if intent.summon {
            for off in [Vec2::new(-70.0, 40.0), Vec2::new(70.0, 40.0)] {
                enemy::spawn(&mut commands, &art.0, pos + off);
            }
        }
    }
}

/// Wind-ups stutter red (readable warning); hits flash white.
fn flash(time: Res<Time>, mut q: Query<(&mut Boss, &mut Sprite)>) {
    let stutter_on = (time.elapsed_secs() * 14.0) as i32 % 2 == 0;
    for (mut b, mut sprite) in &mut q {
        b.flash = (b.flash - time.delta_secs()).max(0.0);
        let winding = matches!(b.brain.state, State::WindupLunge | State::WindupSlam);
        sprite.color = if winding && stutter_on {
            Color::srgb(2.4, 1.0, 1.0)
        } else if b.flash > 0.0 && !winding {
            Color::srgb(2.6, 2.6, 2.8)
        } else {
            Color::WHITE
        };
    }
}

fn reap(mut commands: Commands, q: Query<(Entity, &Boss)>, mut run: ResMut<RunRes>, mut beats: MessageWriter<PlayBeat>) {
    for (e, b) in &q {
        if b.brain.is_dead() {
            commands.entity(e).despawn();
            beats.write(PlayBeat(mine::grom_defeated(&mut run)));
        }
    }
}

/// Expanding ring of pixel dashes for the ground slam.
fn expand_waves(mut commands: Commands, time: Res<Time>, mut gizmos: Gizmos, mut q: Query<(Entity, &mut Shockwave, &Transform)>) {
    const DURATION: f32 = 0.35;
    for (e, mut w, tf) in &mut q {
        w.t += time.delta_secs();
        let k = (w.t / DURATION).min(1.0);
        let r = boss::SLAM_RADIUS * k;
        let c = Color::srgba(0.85, 0.45, 1.0, (1.0 - k) * 0.8);
        let mut a = 0.0_f32;
        while a < std::f32::consts::TAU {
            let p = tf.translation.truncate() + Vec2::from_angle(a) * r;
            gizmos.rect_2d(Isometry2d::from_translation(p), Vec2::splat(4.0), c);
            a += 0.18;
        }
        if w.t >= DURATION {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_bar(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
        BossBar,
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: percent(50),
            width: px(420),
            margin: UiRect::left(px(-210)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        children![
            (Text::new("黑腕队长·格罗姆"), text_font(&fonts.body, 16.0), TextColor(Color::srgb(0.85, 0.6, 0.95))),
            (
                Node { width: percent(100), height: px(8), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                children![(BossBarFill, Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.6, 0.2, 0.75)))],
            ),
        ],
    ));
}

fn update_bar(boss: Query<&Boss>, mut bar: Single<&mut Visibility, With<BossBar>>, mut fill: Single<&mut Node, With<BossBarFill>>) {
    match boss.single() {
        Ok(b) => {
            **bar = Visibility::Inherited;
            fill.width = percent(b.brain.hp / boss::MAX_HP * 100.0);
        }
        Err(_) => **bar = Visibility::Hidden,
    }
}

/// Crowned, black-armed corrupted captain (port of the Godot `_draw`).
fn grom_image() -> Image {
    let mut c = Canvas::new(28, 48);
    let mut r = |x: i32, y: i32, w: u32, h: u32, col: [u8; 3]| c.fill((x + 14) as u32, (y + 39) as u32, w, h, col);
    let (crown, hood, skin, beard) = ([150, 110, 44], [28, 8, 38], [58, 22, 74], [74, 34, 96]);
    let (glow, armor, lit, arm, leg) = ([216, 130, 255], [44, 24, 62], [120, 52, 168], [10, 5, 14], [24, 10, 36]);
    r(-9, -34, 18, 3, crown);
    r(-9, -37, 2, 3, crown);
    r(-1, -38, 2, 4, crown);
    r(7, -37, 2, 3, crown);
    r(-9, -31, 18, 4, hood);
    r(-9, -27, 18, 10, skin);
    r(-7, -17, 14, 4, beard);
    r(-5, -13, 10, 2, beard);
    r(-5, -23, 3, 2, glow);
    r(2, -23, 3, 2, glow);
    r(-10, -11, 20, 11, armor);
    r(-8, -9, 16, 2, lit);
    r(-1, -7, 2, 6, lit);
    r(-13, -10, 3, 15, arm);
    r(10, -10, 3, 9, armor);
    r(-9, -1, 8, 10, leg);
    r(1, -1, 8, 10, leg);
    c.into_image()
}
