//! 布林 on screen: if you saved him he limps after you through the mine
//! (keeping a step back, catching up if left far behind) and later waits by
//! the forge in 灰灯镇. Talking to him goes through `interact`.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::story::survivor;
use tides_core::world::Flag;

use crate::area::Area;
use crate::coords::to_world;
use crate::interact::{Interactable, Target};
use crate::paint::Canvas;
use crate::physics::{Body, Velocity, YSort};
use crate::player::Player;
use crate::run_state::RunRes;

/// A little slower than the player's walk: he's limping.
const LIMP_SPEED: f32 = 112.0;
const KEEP_DISTANCE: f32 = 64.0;
/// Further than this (sprinted ahead, rolled away) and he catches up off-screen.
const CATCH_UP: f32 = 420.0;

#[derive(Component)]
struct Follower;

#[derive(Resource)]
struct BrinArt(Handle<Image>);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut c: Commands, mut images: ResMut<Assets<Image>>| c.insert_resource(BrinArt(images.add(brin_image()))))
        .add_systems(OnEnter(Area::Town), wait_in_town)
        .add_systems(Update, (join_in_mine, follow).chain().run_if(in_state(Area::Mine)));
}

fn brin(art: &BrinArt, at: Vec2, area: Area) -> impl Bundle {
    (
        DespawnOnExit(area),
        Interactable::new(Target::Survivor, 62.0),
        Sprite::from_image(art.0.clone()),
        Anchor::BOTTOM_CENTER,
        YSort,
        Transform::from_translation(at.extend(10.0)).with_scale(Vec3::splat(2.0)),
    )
}

/// Spawns him as soon as he's saved, or on (re-)entering the mine with him.
fn join_in_mine(mut commands: Commands, run: Res<RunRes>, art: Res<BrinArt>, existing: Query<(), With<Follower>>, player: Single<&Transform, With<Player>>) {
    let escaped = run.world.has(Flag::EscapedMine);
    if !survivor::is_with_you(&run) || escaped || !existing.is_empty() {
        return;
    }
    let at = player.translation.truncate() + Vec2::new(-40.0, 0.0);
    commands.spawn((brin(&art, at, Area::Mine), Follower, Velocity::default(), Body { half: Vec2::new(8.0, 5.0) }));
}

fn follow(player: Single<&Transform, (With<Player>, Without<Follower>)>, mut q: Query<(&mut Transform, &mut Velocity, &mut Sprite), With<Follower>>) {
    let target = player.translation.truncate();
    for (mut tf, mut vel, mut sprite) in &mut q {
        let to = target - tf.translation.truncate();
        if to.length() > CATCH_UP {
            let behind = target - to.normalize() * KEEP_DISTANCE;
            tf.translation.x = behind.x;
            tf.translation.y = behind.y;
            vel.0 = Vec2::ZERO;
            continue;
        }
        vel.0 = if to.length() > KEEP_DISTANCE { to.normalize() * LIMP_SPEED } else { Vec2::ZERO };
        if to.x.abs() > 4.0 {
            sprite.flip_x = to.x < 0.0;
        }
    }
}

fn wait_in_town(mut commands: Commands, run: Res<RunRes>, art: Res<BrinArt>) {
    if survivor::is_with_you(&run) {
        commands.spawn(brin(&art, to_world(survivor::TOWN_POS), Area::Town));
    }
}

/// Bandaged dwarf: brown beard, tan face, rust tunic, a sling on one arm.
fn brin_image() -> Image {
    let mut c = Canvas::new(18, 28);
    let mut r = |x: i32, y: i32, w: u32, h: u32, col: [u8; 3]| {
        c.fill((x + 9) as u32, (y + 28) as u32, w, h, col);
    };
    let (hair, beard, skin) = ([70, 42, 24], [120, 70, 36], [219, 170, 128]);
    let (tunic, belt, leg, bandage) = ([150, 70, 40], [60, 36, 20], [64, 48, 40], [236, 228, 210]);
    r(-4, -27, 8, 3, hair);
    r(-5, -24, 10, 5, skin);
    r(-3, -22, 1, 1, [40, 24, 16]);
    r(2, -22, 1, 1, [40, 24, 16]);
    r(-5, -25, 10, 1, bandage);
    r(-5, -19, 10, 3, beard);
    r(-3, -16, 6, 2, beard);
    r(-6, -15, 12, 7, tunic);
    r(-6, -10, 12, 1, belt);
    r(2, -14, 5, 3, bandage);
    r(-5, -8, 4, 8, leg);
    r(1, -8, 4, 7, leg);
    c.into_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brin_is_painted() {
        let img = brin_image();
        let opaque = img.data.as_ref().unwrap().chunks(4).filter(|p| p[3] > 0).count();
        assert!(opaque > 100);
    }
}
