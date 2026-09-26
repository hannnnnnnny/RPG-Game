//! The player (迪丝): walk / sprint / dodge-roll, 4-direction LPC animation.
//! Body, robe and hood are separate LPC sheets drawn as stacked child
//! sprites sharing one atlas index — no image compositing needed.

use bevy::prelude::*;
use tides_core::areas::Mine;

use crate::coords::to_world;
use crate::physics::{Body, Velocity, YSort};

const WALK_SPEED: f32 = 130.0;
const SPRINT_SPEED: f32 = 210.0;
const ROLL_SPEED: f32 = 250.0;
const ROLL_DURATION: f32 = 0.26;
const ROLL_COST: f32 = 28.0;
const SPRINT_DRAIN: f32 = 24.0;
/// LPC sheets: 9 frames × 4 rows of 64px (rows: up, left, down, right).
const FRAME: u32 = 64;
const WALK_FPS: f32 = 10.0;
const LAYERS: [&str; 3] = ["sprites/disi/body_walk.png", "sprites/disi/robe_walk.png", "sprites/disi/hood_walk.png"];

#[derive(Component)]
pub struct Player;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Facing {
    Up,
    Left,
    #[default]
    Down,
    Right,
}

impl Facing {
    fn row(self) -> usize {
        self as usize
    }

    pub fn from_vec(v: Vec2) -> Self {
        if v.x.abs() > v.y.abs() {
            if v.x < 0.0 { Facing::Left } else { Facing::Right }
        } else if v.y > 0.0 {
            Facing::Up
        } else {
            Facing::Down
        }
    }

    pub fn vec(self) -> Vec2 {
        match self {
            Facing::Up => Vec2::Y,
            Facing::Left => Vec2::NEG_X,
            Facing::Down => Vec2::NEG_Y,
            Facing::Right => Vec2::X,
        }
    }
}

/// Movement state. Stamina lives here; `Vitals` (HP) comes with combat.
#[derive(Component)]
pub struct Motion {
    pub facing: Facing,
    pub moving: bool,
    pub sprinting: bool,
    pub roll_timer: f32,
    /// Seconds since the last roll ended (drives 翻滚后伤害).
    pub since_roll: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_regen: f32,
    anim_clock: f32,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            facing: Facing::Down,
            moving: false,
            sprinting: false,
            roll_timer: 0.0,
            since_roll: 99.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen: tides_core::stats::BASE_STAMINA_REGEN,
            anim_clock: 0.0,
        }
    }
}

/// One LPC layer sprite under the player.
#[derive(Component)]
struct LpcLayer;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_player)
        .add_systems(Update, (read_input, animate).chain());
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(FRAME), 9, 4, None, None));
    let start = to_world(Mine::PLAYER_START);
    commands
        .spawn((
            Player,
            Motion::default(),
            crate::combat::Vitals::default(),
            Velocity::default(),
            Body { half: Vec2::new(10.0, 5.0) },
            YSort,
            Transform::from_translation(start.extend(10.0)),
            Visibility::default(),
        ))
        .with_children(|p| {
            // Warm miner's lamp that follows the player.
            p.spawn((
                crate::lighting::Light::new(Color::srgb(1.0, 0.83, 0.58), 230.0, 1.35),
                Transform::from_xyz(0.0, 16.0, 0.0),
            ));
            for (i, path) in LAYERS.iter().enumerate() {
                p.spawn((
                    LpcLayer,
                    Sprite::from_atlas_image(
                        assets.load(*path),
                        TextureAtlas { layout: layout.clone(), index: Facing::Down.row() * 9 },
                    ),
                    // Feet at the origin: lift the 64px frame so its base sits there.
                    Transform::from_xyz(0.0, 22.0, i as f32 * 0.01).with_scale(Vec3::splat(0.8)),
                ));
            }
        });
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut q: Query<(&mut Motion, &mut Velocity), With<Player>>) {
    let dt = time.delta_secs();
    for (mut m, mut vel) in &mut q {
        let dir = input_dir(&keys);
        m.moving = dir != Vec2::ZERO;
        if m.moving {
            m.facing = Facing::from_vec(dir);
        }
        m.roll_timer = (m.roll_timer - dt).max(0.0);
        m.since_roll = if m.roll_timer > 0.0 { 0.0 } else { m.since_roll + dt };
        if keys.just_pressed(KeyCode::Space) && m.stamina >= ROLL_COST && m.roll_timer <= 0.0 {
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
        let heading = if m.moving { dir } else if m.roll_timer > 0.0 { m.facing.vec() } else { Vec2::ZERO };
        vel.0 = heading * speed;
        update_stamina(&mut m, dt);
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

/// Frame 0 of each row is the idle pose; 1–8 are the walk cycle.
fn animate(
    time: Res<Time>,
    mut players: Query<(&mut Motion, &Children), With<Player>>,
    mut layers: Query<&mut Sprite, With<LpcLayer>>,
) {
    for (mut m, children) in &mut players {
        let speed = if m.sprinting { 1.5 } else { 1.0 };
        m.anim_clock = if m.moving { m.anim_clock + time.delta_secs() * speed } else { 0.0 };
        let frame = if m.moving { 1 + (m.anim_clock * WALK_FPS) as usize % 8 } else { 0 };
        let index = m.facing.row() * 9 + frame;
        let tint = if m.roll_timer > 0.0 { Color::srgb(0.6, 0.85, 0.8) } else { Color::WHITE };
        for child in children.iter() {
            if let Ok(mut sprite) = layers.get_mut(child) {
                sprite.color = tint;
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = index;
                }
            }
        }
    }
}
