//! Player melee (mouse-aimed cone), incoming damage and light death.
//! Numbers come from `tides_core::combat`; this is feel and feedback.

use bevy::prelude::*;
use tides_core::areas::Mine;
use tides_core::combat::{self, KillSource};
use tides_core::loot::{self, DropSource};

use crate::camera::MainCamera;
use crate::coords::{Z_OVERLAY, to_world};
use crate::enemy::{Enemy, PlayerStruck};
use crate::player::{Facing, Motion, Player};
use crate::run_state::{GameRng, RunRes};

const ATTACK_COOLDOWN: f32 = 0.34;
/// Reach matches the visible blade arc; the cone is a bit wider than it
/// looks so hits feel forgiving.
const REACH: f32 = 70.0;
const HALF_ARC: f32 = std::f32::consts::PI / 2.4;
const ENEMY_DROP_CHANCE: f32 = 0.64;

#[derive(Component)]
pub struct Vitals {
    pub hp: f32,
    pub max_hp: f32,
    attack_timer: f32,
    hurt_flash: f32,
}

impl Default for Vitals {
    fn default() -> Self {
        Self { hp: 100.0, max_hp: 100.0, attack_timer: 0.0, hurt_flash: 0.0 }
    }
}

/// Where the player comes back after falling; levels move it forward.
#[derive(Resource)]
pub struct Respawn(pub Vec2);

/// Floating damage number rising and fading out.
#[derive(Component)]
struct Floater {
    life: f32,
}

/// Short-lived crescent showing the swing.
#[derive(Component)]
struct Slash {
    life: f32,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(Respawn(to_world(Mine::PLAYER_START)))
        .add_systems(Update, (sync_stats, swing, take_hits, reap_enemies, float_up, fade_slash));
}

/// Loadout stats → max HP and stamina regen, only when the run changed.
fn sync_stats(run: Res<RunRes>, mut q: Query<(&mut Vitals, &mut Motion), With<Player>>) {
    if !run.is_changed() {
        return;
    }
    let stats = run.stats();
    for (mut v, mut m) in &mut q {
        v.max_hp = stats.max_health as f32;
        v.hp = v.hp.min(v.max_hp);
        m.stamina_regen = stats.stamina_regen;
    }
}

fn cursor_world(window: &Window, cam: (&Camera, &GlobalTransform)) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    cam.0.viewport_to_world_2d(cam.1, cursor).ok()
}

#[allow(clippy::too_many_arguments)]
fn swing(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    cam: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut player: Single<(&Transform, &mut Vitals, &mut Motion), With<Player>>,
    mut enemies: Query<(&Transform, &mut Enemy), Without<Player>>,
    run: Res<RunRes>,
    mut rng: ResMut<GameRng>,
) {
    let (ptf, ref mut vitals, ref mut motion) = *player;
    vitals.attack_timer = (vitals.attack_timer - time.delta_secs()).max(0.0);
    let pressed = mouse.just_pressed(MouseButton::Left) || keys.just_pressed(KeyCode::KeyJ);
    if !pressed || vitals.attack_timer > 0.0 {
        return;
    }
    vitals.attack_timer = ATTACK_COOLDOWN;
    let origin = ptf.translation.truncate();
    let aim = cursor_world(&window, *cam)
        .map(|c| c - origin)
        .filter(|v| v.length() > 1.0)
        .unwrap_or_else(|| motion.facing.vec());
    motion.facing = Facing::from_vec(aim);
    spawn_slash(&mut commands, origin + Vec2::Y * 16.0, aim.to_angle());
    let stats = run.stats();
    for (etf, mut enemy) in &mut enemies {
        let epos = etf.translation.truncate();
        let to = epos - origin;
        if to.length() > REACH || to.angle_to(aim).abs() > HALF_ARC {
            continue;
        }
        let hit = combat::roll_hit(&stats, &mut rng, motion.since_roll);
        enemy.hit(hit.amount as f32, epos, origin);
        spawn_number(&mut commands, epos + Vec2::Y * 40.0, hit.amount, hit.crit);
    }
}

fn spawn_slash(commands: &mut Commands, at: Vec2, angle: f32) {
    commands.spawn((
        Slash { life: 0.14 },
        Sprite::from_color(Color::srgba(1.0, 0.95, 0.8, 0.85), Vec2::new(46.0, 6.0)),
        Transform::from_translation((at + Vec2::from_angle(angle) * 34.0).extend(Z_OVERLAY))
            .with_rotation(Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2)),
    ));
}

fn spawn_number(commands: &mut Commands, at: Vec2, amount: i32, crit: bool) {
    let (text, size, color) = if crit {
        (format!("{amount}!"), 24.0, Color::srgb(1.0, 0.62, 0.3))
    } else {
        (amount.to_string(), 18.0, Color::srgb(1.0, 0.95, 0.72))
    };
    commands.spawn((
        Floater { life: 0.6 },
        Text2d::new(text),
        TextFont { font_size: FontSize::Px(size), ..default() },
        TextColor(color),
        Transform::from_translation(at.extend(Z_OVERLAY + 1.0)),
    ));
}

fn float_up(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Floater, &mut Transform, &mut TextColor)>) {
    let dt = time.delta_secs();
    for (e, mut f, mut tf, mut color) in &mut q {
        f.life -= dt;
        tf.translation.y += 48.0 * dt;
        color.0.set_alpha((f.life / 0.4).clamp(0.0, 1.0));
        if f.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

fn fade_slash(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Slash, &mut Sprite)>) {
    for (e, mut s, mut sprite) in &mut q {
        s.life -= time.delta_secs();
        sprite.color.set_alpha((s.life / 0.14).clamp(0.0, 1.0) * 0.85);
        if s.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

/// Dead enemies pay gold and maybe drop loot.
fn reap_enemies(mut commands: Commands, q: Query<(Entity, &Enemy)>, mut run: ResMut<RunRes>, mut rng: ResMut<GameRng>) {
    for (e, enemy) in &q {
        if enemy.hp > 0.0 {
            continue;
        }
        commands.entity(e).despawn();
        run.add_kill_gold(KillSource::Enemy, &mut rng);
        if rng.f32() < ENEMY_DROP_CHANCE {
            let tier = run.world.world_tier;
            run.add_item(loot::generate(DropSource::Enemy, tier, &mut rng));
        }
    }
}

/// Incoming hits after 黑潮抗性; at 0 HP: lose 10% gold, respawn, full HP.
fn take_hits(
    time: Res<Time>,
    mut struck: MessageReader<PlayerStruck>,
    mut player: Single<(&mut Transform, &mut Vitals), With<Player>>,
    mut run: ResMut<RunRes>,
    respawn: Res<Respawn>,
) {
    let (ref mut tf, ref mut v) = *player;
    v.hurt_flash = (v.hurt_flash - time.delta_secs()).max(0.0);
    let stats = run.stats();
    for hit in struck.read() {
        v.hp = (v.hp - combat::mitigate(hit.damage, &stats)).max(0.0);
        v.hurt_flash = 0.08;
    }
    if v.hp <= 0.0 {
        v.hp = v.max_hp;
        tf.translation.x = respawn.0.x;
        tf.translation.y = respawn.0.y;
        run.on_player_down();
    }
}
