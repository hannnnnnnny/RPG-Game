//! The player (迪丝): walk / sprint / dodge-roll, drawn as an LPC rig
//! (body + purple robe + hood).

use bevy::prelude::*;
use tides_core::areas::Mine;

use crate::beats::Modal;
use crate::coords::to_world;
use crate::lighting::Light;
use crate::lpc::{Facing, LpcAnim, LpcLayouts, Pose, attach_layers};
use crate::physics::{Body, Velocity, YSort};

const WALK_SPEED: f32 = 130.0;
const SPRINT_SPEED: f32 = 210.0;
const ROLL_SPEED: f32 = 250.0;
const ROLL_DURATION: f32 = 0.26;
const ROLL_COST: f32 = 28.0;
const SPRINT_DRAIN: f32 = 24.0;
const LAYERS: [&str; 3] = ["sprites/disi/body_walk.png", "sprites/disi/robe_walk.png", "sprites/disi/hood_walk.png"];

#[derive(Component)]
pub struct Player;

/// Movement state. HP lives in `combat::Vitals`; facing in `LpcAnim`.
#[derive(Component)]
pub struct Motion {
    pub moving: bool,
    pub sprinting: bool,
    pub roll_timer: f32,
    /// Seconds since the last roll ended (drives 翻滚后伤害).
    pub since_roll: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_regen: f32,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            moving: false,
            sprinting: false,
            roll_timer: 0.0,
            since_roll: 99.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen: tides_core::stats::BASE_STAMINA_REGEN,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_player).add_systems(Update, read_input);
}

fn spawn_player(mut commands: Commands, assets: Res<AssetServer>, layouts: Res<LpcLayouts>) {
    let player = commands
        .spawn((
            Player,
            Motion::default(),
            crate::combat::Vitals::default(),
            LpcAnim::new(Pose::Walk),
            Velocity::default(),
            Body { half: Vec2::new(10.0, 5.0) },
            YSort,
            Transform::from_translation(to_world(Mine::PLAYER_START).extend(10.0)),
            Visibility::default(),
            // Warm miner's lamp that follows the player.
            children![(Light::new(Color::srgb(1.0, 0.83, 0.58), 230.0, 1.35), Transform::from_xyz(0.0, 16.0, 0.0))],
        ))
        .id();
    let sheets: Vec<_> = LAYERS.iter().map(|p| assets.load(*p)).collect();
    attach_layers(&mut commands, player, Pose::Walk, &sheets, &layouts);
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    modal: Res<Modal>,
    mut q: Query<(&mut Motion, &mut LpcAnim, &mut Velocity), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut m, mut anim, mut vel) in &mut q {
        // Story modals freeze the player in place.
        let dir = if modal.is_open() { Vec2::ZERO } else { input_dir(&keys) };
        m.moving = dir != Vec2::ZERO;
        if m.moving {
            anim.facing = Facing::from_vec(dir);
        }
        m.roll_timer = (m.roll_timer - dt).max(0.0);
        m.since_roll = if m.roll_timer > 0.0 { 0.0 } else { m.since_roll + dt };
        if keys.just_pressed(KeyCode::Space) && m.stamina >= ROLL_COST && m.roll_timer <= 0.0 && !modal.is_open() {
            m.stamina -= ROLL_COST;
            m.roll_timer = ROLL_DURATION;
        }
        m.sprinting = keys.pressed(KeyCode::ShiftLeft) && m.moving && m.stamina > 1.0;
        let speed = if m.roll_timer > 0.0 {
            ROLL_SPEED
        } else if m.sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        // A roll with no input dodges the way you're facing.
        let heading = if m.moving { dir } else if m.roll_timer > 0.0 { anim.facing.vec() } else { Vec2::ZERO };
        vel.0 = heading * speed;
        update_stamina(&mut m, dt);
        anim.moving = m.moving;
        anim.speed = if m.sprinting { 1.5 } else { 1.0 };
        anim.tint = if m.roll_timer > 0.0 { Color::srgb(0.6, 0.85, 0.8) } else { Color::WHITE };
    }
}

fn input_dir(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let axis = |neg: [KeyCode; 2], pos: [KeyCode; 2]| {
        f32::from(u8::from(keys.any_pressed(pos))) - f32::from(u8::from(keys.any_pressed(neg)))
    };
    let x = axis([KeyCode::KeyA, KeyCode::ArrowLeft], [KeyCode::KeyD, KeyCode::ArrowRight]);
    let y = axis([KeyCode::KeyS, KeyCode::ArrowDown], [KeyCode::KeyW, KeyCode::ArrowUp]);
    Vec2::new(x, y).normalize_or_zero()
}

fn update_stamina(m: &mut Motion, dt: f32) {
    if m.sprinting {
        m.stamina = (m.stamina - SPRINT_DRAIN * dt).max(0.0);
    } else if m.roll_timer <= 0.0 {
        m.stamina = (m.stamina + m.stamina_regen * dt).min(m.max_stamina);
    }
}
