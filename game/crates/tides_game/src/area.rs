//! Areas as a Bevy state. Entering one bakes its tiles into a sprite,
//! installs its walls, bounds and mood, and places the player; everything
//! tagged `DespawnOnExit(area)` is cleaned up automatically on leaving.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::areas::{Mine, Pt};
use tides_core::map::{Layout, Tile, wall_rects};
use tides_core::story::AreaId;

use crate::beats::GoTo;
use crate::coords::{Z_MAP, to_world};
use crate::lighting::Ambient;
use crate::paint::{self, Canvas, Cell, SRC};
use crate::physics::Walls;
use crate::player::Player;
use crate::tiles_mine;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Area {
    /// Startup builds the player/UI first; then we enter the first area.
    /// (The initial state's OnEnter runs before every Startup schedule.)
    #[default]
    Boot,
    Mine,
    Town,
}

impl From<AreaId> for Area {
    fn from(id: AreaId) -> Self {
        match id {
            AreaId::Mine => Area::Mine,
            AreaId::Town => Area::Town,
        }
    }
}

/// Display name of the current area (HUD info box).
#[derive(Resource, Clone, Copy)]
pub struct AreaTitle(pub &'static str);

/// Pixel size of the current area (map space), for camera clamping.
#[derive(Resource, Clone, Copy)]
pub struct AreaBounds(pub Vec2);

/// Where the player appears on the next area entry (e.g. a shop door),
/// overriding the area's default entrance.
#[derive(Resource, Default)]
pub struct Arrival(pub Option<Pt>);

pub fn plugin(app: &mut App) {
    app.init_state::<Area>()
        .insert_resource(AreaTitle("黑潮矿区"))
        .insert_resource(AreaBounds(Vec2::ONE))
        .init_resource::<Arrival>()
        .add_systems(PostStartup, |mut next: ResMut<NextState<Area>>| next.set(Area::Mine))
        .add_systems(OnEnter(Area::Mine), enter_mine)
        .add_systems(Update, follow_goto);
}

fn follow_goto(mut go: MessageReader<GoTo>, mut next: ResMut<NextState<Area>>) {
    if let Some(GoTo(id)) = go.read().last() {
        next.set((*id).into());
    }
}

/// Shared setup for any area: map sprite, walls, bounds, title, mood.
pub struct AreaSpec<'a, L: Layout> {
    pub area: Area,
    pub layout: &'a L,
    pub title: &'static str,
    pub entrance: Pt,
    pub ambient: Ambient,
}

pub fn build_area<L: Layout>(
    spec: AreaSpec<L>,
    painter: impl Fn(&mut Canvas, &Cell, Tile),
    commands: &mut Commands,
    images: &mut Assets<Image>,
    arrival: &mut Arrival,
    player: &mut Transform,
) {
    let image = images.add(paint::bake(spec.layout, painter));
    commands.spawn((
        DespawnOnExit(spec.area),
        Sprite::from_image(image),
        Anchor::TOP_LEFT,
        Transform::from_xyz(0.0, 0.0, Z_MAP).with_scale(Vec3::splat(48.0 / SRC as f32)),
    ));
    commands.insert_resource(Walls::from_map_rects(&wall_rects(spec.layout)));
    let (w, h) = spec.layout.size();
    commands.insert_resource(AreaBounds(Vec2::new(w, h)));
    commands.insert_resource(AreaTitle(spec.title));
    commands.insert_resource(spec.ambient);
    let at = to_world(arrival.0.take().unwrap_or(spec.entrance));
    player.translation.x = at.x;
    player.translation.y = at.y;
}

fn enter_mine(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut arrival: ResMut<Arrival>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let spec = AreaSpec {
        area: Area::Mine,
        layout: &Mine,
        title: "黑潮矿区",
        entrance: Mine::PLAYER_START,
        ambient: Ambient { color: Color::srgb(0.03, 0.02, 0.06), darkness: 0.74 },
    };
    build_area(spec, tiles_mine::paint, &mut commands, &mut images, &mut arrival, &mut player);
}
