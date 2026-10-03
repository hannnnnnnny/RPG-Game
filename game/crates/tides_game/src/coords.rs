//! Map space (design docs, y down) → Bevy world space (y up), plus draw order.

use bevy::prelude::*;
use tides_core::areas::Pt;

pub const Z_MAP: f32 = 0.0;
pub const Z_OVERLAY: f32 = 50.0;

pub fn to_world(p: Pt) -> Vec2 {
    Vec2::new(p.0, -p.1)
}

/// Y-sort for a top-down view: things lower on screen draw in front.
/// World y is ≤ 0 inside a map, so this stays within [10, 11).
pub fn actor_z(world_y: f32) -> f32 {
    10.0 - world_y * 0.0005
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_y_points_down_world_y_points_up() {
        assert_eq!(to_world((3.0, 4.0)), Vec2::new(3.0, -4.0));
    }

    #[test]
    fn lower_draws_in_front() {
        assert!(actor_z(to_world((0.0, 500.0)).y) > actor_z(to_world((0.0, 100.0)).y));
    }
}
