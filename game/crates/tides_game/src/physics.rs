//! Minimal top-down collision: actors are feet-boxes that slide along static
//! wall rectangles. No physics engine needed for this game's movement.

use bevy::prelude::*;
use tides_core::map::Rect;

use crate::coords::{actor_z, to_world};

/// Static wall boxes of the current area, in world space (min corner, size).
#[derive(Resource, Default)]
pub struct Walls(pub Vec<(Vec2, Vec2)>);

impl Walls {
    pub fn from_map_rects(rects: &[Rect]) -> Self {
        // Map rects are y-down with (x, y) at the top-left corner; in world
        // space that corner becomes the *top*, so the min corner is y - h.
        Self(rects.iter().map(|r| (to_world((r.x, r.y + r.h)), Vec2::new(r.w, r.h))).collect())
    }

    fn overlaps(&self, center: Vec2, half: Vec2) -> bool {
        self.0.iter().any(|(min, size)| {
            let max = *min + *size;
            center.x + half.x > min.x
                && center.x - half.x < max.x
                && center.y + half.y > min.y
                && center.y - half.y < max.y
        })
    }
}

/// Collision box around the entity's feet (its Transform origin).
#[derive(Component, Clone, Copy)]
pub struct Body {
    pub half: Vec2,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// Drives z from y so actors overlap correctly (see `coords::actor_z`).
#[derive(Component)]
pub struct YSort;

pub fn plugin(app: &mut App) {
    app.init_resource::<Walls>()
        .add_systems(Update, (move_bodies, y_sort).chain());
}

/// Move on each axis separately so actors slide along walls instead of
/// sticking to them.
fn move_bodies(time: Res<Time>, walls: Res<Walls>, mut q: Query<(&mut Transform, &Velocity, &Body)>) {
    let dt = time.delta_secs();
    for (mut tf, vel, body) in &mut q {
        let mut pos = tf.translation.truncate();
        for step in [Vec2::new(vel.x * dt, 0.0), Vec2::new(0.0, vel.y * dt)] {
            if !walls.overlaps(pos + step, body.half) {
                pos += step;
            }
        }
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
    }
}

fn y_sort(mut q: Query<&mut Transform, With<YSort>>) {
    for mut tf in &mut q {
        tf.translation.z = actor_z(tf.translation.y);
    }
}
