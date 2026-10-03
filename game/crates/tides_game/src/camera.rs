//! Camera that eases after the player and never shows outside the area.

use bevy::prelude::*;

use crate::area::AreaBounds;
use crate::player::Player;

const FOLLOW_SPEED: f32 = 8.0;

#[derive(Component)]
pub struct MainCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera).add_systems(PostUpdate, follow_player.before(TransformSystems::Propagate));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((MainCamera, Camera2d));
}

fn follow_player(
    time: Res<Time>,
    bounds: Option<Res<AreaBounds>>,
    window: Single<&Window>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    mut cam: Single<&mut Transform, With<MainCamera>>,
) {
    let target = player.translation.truncate();
    let t = 1.0 - (-FOLLOW_SPEED * time.delta_secs()).exp();
    let mut pos = cam.translation.truncate().lerp(target, t);
    if let Some(bounds) = bounds {
        pos = clamp_to_area(pos, window.size() / 2.0, bounds.0);
    }
    cam.translation.x = pos.x;
    cam.translation.y = pos.y;
}

/// Keep the view inside [0, w] × [-h, 0]; centre on axes the area can't fill.
fn clamp_to_area(pos: Vec2, half_view: Vec2, size: Vec2) -> Vec2 {
    let axis = |p: f32, half: f32, lo: f32, hi: f32| {
        if hi - lo <= half * 2.0 { (lo + hi) / 2.0 } else { p.clamp(lo + half, hi - half) }
    };
    Vec2::new(axis(pos.x, half_view.x, 0.0, size.x), axis(pos.y, half_view.y, -size.y, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_inside_area() {
        let p = clamp_to_area(Vec2::new(-500.0, 500.0), Vec2::new(640.0, 360.0), Vec2::new(1450.0, 900.0));
        assert_eq!(p, Vec2::new(640.0, -360.0));
    }

    #[test]
    fn centres_when_area_smaller_than_view() {
        let p = clamp_to_area(Vec2::ZERO, Vec2::new(640.0, 360.0), Vec2::new(720.0, 480.0));
        assert_eq!(p, Vec2::new(360.0, -240.0));
    }
}
