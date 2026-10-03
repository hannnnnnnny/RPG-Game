//! Corrupted dwarves (感染矮人): wander until they spot the player, then
//! chase and strike on contact. Hits shove them back and flash them white.

use bevy::prelude::*;
use tides_core::areas::Mine;

use crate::area::Area;
use crate::coords::to_world;
use crate::paint::Canvas;
use crate::physics::{Body, Velocity, YSort};
use crate::player::Player;

const DETECT_RANGE: f32 = 280.0;
const ATTACK_RANGE: f32 = 34.0;
const ATTACK_COOLDOWN: f32 = 0.85;
const MOVE_SPEED: f32 = 62.0;
pub const CONTACT_DAMAGE: f32 = 8.0;
const KNOCKBACK_FORCE: f32 = 190.0;
const KNOCKBACK_DECAY: f32 = 620.0;
const FLASH_TIME: f32 = 0.14;

#[derive(Component)]
pub struct Enemy {
    pub hp: f32,
    attack_timer: f32,
    knockback: Vec2,
    wander_dir: Vec2,
    wander_timer: f32,
    pub flash: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { hp: 20.0, attack_timer: 0.0, knockback: Vec2::ZERO, wander_dir: Vec2::ZERO, wander_timer: 0.0, flash: 0.0 }
    }
}

impl Enemy {
    /// Apply a hit: lose HP and get shoved away from `from`.
    pub fn hit(&mut self, amount: f32, self_pos: Vec2, from: Vec2) {
        self.hp -= amount;
        let dir = (self_pos - from).try_normalize().unwrap_or(Vec2::X);
        self.knockback = dir * KNOCKBACK_FORCE;
        self.flash = FLASH_TIME;
    }
}

/// An enemy's melee landed on the player this frame.
#[derive(Message)]
pub struct PlayerStruck {
    pub damage: f32,
}

#[derive(Resource)]
pub struct EnemyArt(pub Handle<Image>);

pub fn plugin(app: &mut App) {
    app.add_message::<PlayerStruck>()
        .add_systems(Startup, load_art)
        .add_systems(OnEnter(Area::Mine), spawn_mine_enemies)
        .add_systems(Update, (think, flash));
}

fn load_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(EnemyArt(images.add(dwarf_image())));
}

fn spawn_mine_enemies(mut commands: Commands, art: Res<EnemyArt>) {
    for p in Mine::ENEMIES {
        spawn(&mut commands, &art.0, to_world(p), Area::Mine);
    }
}

pub fn spawn(commands: &mut Commands, image: &Handle<Image>, at: Vec2, area: Area) {
    commands.spawn((
        Enemy::default(),
        DespawnOnExit(area),
        Velocity::default(),
        Body { half: Vec2::new(9.0, 5.0) },
        YSort,
        Sprite::from_image(image.clone()),
        bevy::sprite::Anchor::BOTTOM_CENTER,
        Transform::from_translation(at.extend(10.0)).with_scale(Vec3::splat(2.0)),
        // Faint violet aura so they loom out of the dark.
        crate::lighting::Light::new(Color::srgb(0.55, 0.2, 0.7), 70.0, 0.45),
    ));
}

fn think(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut enemies: Query<(&mut Enemy, &mut Velocity, &Transform), Without<Player>>,
    mut struck: MessageWriter<PlayerStruck>,
) {
    let dt = time.delta_secs();
    let target = player.translation.truncate();
    for (mut e, mut vel, tf) in &mut enemies {
        e.attack_timer = (e.attack_timer - dt).max(0.0);
        let to_player = target - tf.translation.truncate();
        let dist = to_player.length();
        let intent = if dist < DETECT_RANGE {
            to_player.normalize_or_zero() * MOVE_SPEED
        } else {
            wander(&mut e, dt) * MOVE_SPEED * 0.35
        };
        e.knockback = e.knockback.move_towards(Vec2::ZERO, KNOCKBACK_DECAY * dt);
        vel.0 = intent + e.knockback;
        if dist < ATTACK_RANGE && e.attack_timer <= 0.0 {
            e.attack_timer = ATTACK_COOLDOWN;
            struck.write(PlayerStruck { damage: CONTACT_DAMAGE });
        }
    }
}

/// Idle drift so rooms feel alive: pause or pick a new heading every 1–2.6s.
fn wander(e: &mut Enemy, dt: f32) -> Vec2 {
    e.wander_timer -= dt;
    if e.wander_timer <= 0.0 {
        e.wander_timer = 1.2 + fastrand::f32() * 1.4;
        e.wander_dir = if fastrand::f32() < 0.45 {
            Vec2::ZERO
        } else {
            Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0).normalize_or_zero()
        };
    }
    e.wander_dir
}

/// Over-bright flash + squash when hit, easing back over FLASH_TIME.
fn flash(time: Res<Time>, mut q: Query<(&mut Enemy, &mut Sprite, &mut Transform)>) {
    for (mut e, mut sprite, mut tf) in &mut q {
        e.flash = (e.flash - time.delta_secs()).max(0.0);
        let k = e.flash / FLASH_TIME;
        sprite.color = Color::srgb(1.0 + 1.6 * k, 1.0 + 1.6 * k, 1.0 + 1.8 * k);
        tf.scale = Vec3::new(2.0 * (1.0 + 0.2 * k), 2.0 * (1.0 - 0.16 * k), 1.0);
    }
}

/// Hooded corrupted dwarf: purple skin, beard, glowing eyes (27×28 px).
fn dwarf_image() -> Image {
    let mut c = Canvas::new(18, 28);
    let mut r = |x: i32, y: i32, w: u32, h: u32, col: [u8; 3]| {
        c.fill((x + 9) as u32, (y + 28) as u32, w, h, col);
    };
    let (hood, beard, skin) = ([24, 5, 32], [61, 27, 80], [42, 16, 55]);
    let (glow, armor, lit, leg) = ([201, 122, 255], [36, 19, 51], [110, 45, 160], [21, 8, 32]);
    // Coordinates are the Godot _draw ones halved (it drew at 2x).
    r(-3, -27, 6, 2, hood);
    r(-5, -25, 10, 2, hood);
    r(-5, -23, 10, 5, skin);
    r(-3, -21, 1, 1, glow);
    r(2, -21, 1, 1, glow);
    r(-4, -18, 8, 2, beard);
    r(-3, -16, 6, 1, beard);
    r(-5, -15, 10, 7, armor);
    r(-4, -14, 8, 1, lit);
    r(-1, -13, 1, 4, lit);
    r(-5, -8, 4, 8, leg);
    r(1, -8, 4, 8, leg);
    c.into_image()
}
