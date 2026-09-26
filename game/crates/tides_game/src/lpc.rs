//! LPC (Liberated Pixel Cup) characters built from stacked layer sheets.
//! Walk sheets: 9 frames × 4 rows of 64px (up, left, down, right; frame 0
//! idle). Sit sheets: 3 frames × 4 rows. Every layer shares one atlas index,
//! so body + clothes + hair animate in lockstep with no image compositing.

use bevy::prelude::*;

pub const FRAME: u32 = 64;
const WALK_FPS: f32 = 10.0;
const SIT_FPS: f32 = 1.6;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pose {
    Walk,
    Sit,
}

/// Animation state; gameplay code sets facing/moving, `animate` does the rest.
#[derive(Component)]
pub struct LpcAnim {
    pub pose: Pose,
    pub facing: Facing,
    pub moving: bool,
    pub speed: f32,
    pub tint: Color,
    clock: f32,
}

impl LpcAnim {
    pub fn new(pose: Pose) -> Self {
        Self { pose, facing: Facing::Down, moving: false, speed: 1.0, tint: Color::WHITE, clock: 0.0 }
    }

    fn index(&self) -> usize {
        match self.pose {
            Pose::Walk => {
                let frame = if self.moving { 1 + (self.clock * WALK_FPS) as usize % 8 } else { 0 };
                self.facing.row() * 9 + frame
            }
            // Seated folk face the viewer and breathe through 3 frames.
            Pose::Sit => Facing::Down.row() * 3 + (self.clock * SIT_FPS) as usize % 3,
        }
    }
}

#[derive(Component)]
struct LpcLayer;

#[derive(Resource)]
pub struct LpcLayouts {
    walk: Handle<TextureAtlasLayout>,
    sit: Handle<TextureAtlasLayout>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, make_layouts).add_systems(Update, animate);
}

fn make_layouts(mut commands: Commands, mut layouts: ResMut<Assets<TextureAtlasLayout>>) {
    commands.insert_resource(LpcLayouts {
        walk: layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(FRAME), 9, 4, None, None)),
        sit: layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(FRAME), 3, 4, None, None)),
    });
}

/// Spawn the layer sprites (bottom to top) as children of `parent`.
pub fn attach_layers(
    commands: &mut Commands,
    parent: Entity,
    pose: Pose,
    sheets: &[Handle<Image>],
    layouts: &LpcLayouts,
) {
    let layout = match pose {
        Pose::Walk => layouts.walk.clone(),
        Pose::Sit => layouts.sit.clone(),
    };
    commands.entity(parent).with_children(|p| {
        for (i, sheet) in sheets.iter().enumerate() {
            p.spawn((
                LpcLayer,
                Sprite::from_atlas_image(sheet.clone(), TextureAtlas { layout: layout.clone(), index: 0 }),
                // Feet at the parent origin: lift the 64px frame so its base sits there.
                Transform::from_xyz(0.0, 22.0, i as f32 * 0.01).with_scale(Vec3::splat(0.8)),
            ));
        }
    });
}

fn animate(time: Res<Time>, mut rigs: Query<(&mut LpcAnim, &Children)>, mut layers: Query<&mut Sprite, With<LpcLayer>>) {
    for (mut a, children) in &mut rigs {
        let running = a.moving || a.pose == Pose::Sit;
        a.clock = if running { a.clock + time.delta_secs() * a.speed } else { 0.0 };
        let index = a.index();
        for child in children.iter() {
            if let Ok(mut sprite) = layers.get_mut(child) {
                sprite.color = a.tint;
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = index;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_uses_frame_zero_of_the_facing_row() {
        let mut a = LpcAnim::new(Pose::Walk);
        a.facing = Facing::Right;
        assert_eq!(a.index(), 27);
    }

    #[test]
    fn walking_cycles_frames_one_to_eight() {
        let mut a = LpcAnim::new(Pose::Walk);
        a.moving = true;
        a.clock = 0.35;
        assert!((19..=26).contains(&a.index()));
    }

    #[test]
    fn facing_from_vector() {
        assert_eq!(Facing::from_vec(Vec2::new(0.1, 1.0)), Facing::Up);
        assert_eq!(Facing::from_vec(Vec2::new(-2.0, 1.0)), Facing::Left);
    }
}
