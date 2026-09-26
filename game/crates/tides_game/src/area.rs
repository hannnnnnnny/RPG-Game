//! Loading an area: bake its tiles into one sprite and install its walls.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::areas::Mine;
use tides_core::map::{Layout, wall_rects};

use crate::coords::Z_MAP;
use crate::paint::{self, SRC};
use crate::physics::Walls;
use crate::tiles_mine;

/// Everything spawned for the current area; despawned on area change.
#[derive(Component)]
pub struct AreaEntity;

/// Pixel size of the current area (map space), for camera clamping.
#[derive(Resource, Clone, Copy)]
pub struct AreaBounds(pub Vec2);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_mine);
}

fn spawn_mine(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(paint::bake(&Mine, tiles_mine::paint));
    let scale = 48.0 / SRC as f32;
    commands.spawn((
        AreaEntity,
        Sprite::from_image(image),
        Anchor::TOP_LEFT,
        Transform::from_xyz(0.0, 0.0, Z_MAP).with_scale(Vec3::splat(scale)),
    ));
    commands.insert_resource(Walls::from_map_rects(&wall_rects(&Mine)));
    let (w, h) = Mine.size();
    commands.insert_resource(AreaBounds(Vec2::new(w, h)));
}
