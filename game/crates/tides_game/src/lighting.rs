//! 2D darkness + point lights (the "dark Stardew" mood). Any entity with a
//! `Light` punches a soft pool in the area-wide darkness quad each frame.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

use crate::area::AreaBounds;
use crate::coords::Z_OVERLAY;
use crate::run_state::RunRes;

const MAX_LIGHTS: usize = 32;

/// Requires `Visibility` so lights on hidden entities switch off with them.
#[derive(Component, Clone, Copy)]
#[require(Transform, Visibility)]
pub struct Light {
    pub color: Color,
    pub radius: f32,
    pub energy: f32,
    /// Breathing amplitude (0 = steady), e.g. the totem's pulse.
    pub pulse: f32,
}

impl Light {
    pub const fn new(color: Color, radius: f32, energy: f32) -> Self {
        Self { color, radius, energy, pulse: 0.0 }
    }

    pub const fn pulsing(mut self, amount: f32) -> Self {
        self.pulse = amount;
        self
    }
}

/// How dark an area is; set when the area loads.
#[derive(Resource, Clone, Copy)]
pub struct Ambient {
    pub color: Color,
    pub darkness: f32,
}

#[derive(ShaderType, Clone, Debug)]
struct LightingUniform {
    lights: [Vec4; MAX_LIGHTS],
    colors: [Vec4; MAX_LIGHTS],
    ambient: Vec4,
    screen: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct LightingMaterial {
    #[uniform(0)]
    u: LightingUniform,
}

impl Material2d for LightingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lighting.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct Darkness(Handle<LightingMaterial>);

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<LightingMaterial>::default())
        .insert_resource(Ambient { color: Color::srgb(0.03, 0.02, 0.06), darkness: 0.74 })
        .add_systems(Update, (spawn_darkness.run_if(resource_changed::<AreaBounds>), update_lights).chain());
}

/// One darkness quad sized to the current area; replaced on area change.
fn spawn_darkness(
    mut commands: Commands,
    old: Query<Entity, With<Darkness>>,
    bounds: Res<AreaBounds>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LightingMaterial>>,
) {
    for e in &old {
        commands.entity(e).despawn();
    }
    let size = bounds.0;
    let mat = materials.add(LightingMaterial {
        u: LightingUniform {
            lights: [Vec4::ZERO; MAX_LIGHTS],
            colors: [Vec4::ZERO; MAX_LIGHTS],
            ambient: Vec4::ZERO,
            screen: Vec4::ZERO,
        },
    });
    commands.spawn((
        Darkness(mat.clone()),
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(mat),
        Transform::from_xyz(size.x / 2.0, -size.y / 2.0, Z_OVERLAY - 5.0),
    ));
}

fn update_lights(
    time: Res<Time>,
    ambient: Res<Ambient>,
    run: Res<RunRes>,
    window: Single<&Window>,
    darkness: Query<&Darkness>,
    lights: Query<(&Light, &GlobalTransform, &InheritedVisibility)>,
    mut materials: ResMut<Assets<LightingMaterial>>,
) {
    let Ok(dark) = darkness.single() else { return };
    let Some(mut mat) = materials.get_mut(&dark.0) else { return };
    let t = time.elapsed_secs();
    let mut n = 0;
    for (light, gt, vis) in &lights {
        if n == MAX_LIGHTS || !vis.get() {
            continue;
        }
        let breathe = 1.0 + light.pulse * (t * 2.4).sin();
        let p = gt.translation().truncate();
        mat.u.lights[n] = Vec4::new(p.x, p.y, light.radius, light.energy * breathe);
        mat.u.colors[n] = light.color.to_linear().to_vec4();
        n += 1;
    }
    let a = ambient.color.to_linear();
    mat.u.ambient = Vec4::new(a.red, a.green, a.blue, ambient.darkness);
    let corruption = f32::from(run.world.corruption) / 100.0;
    let size = window.physical_size().as_vec2();
    mat.u.screen = Vec4::new(size.x, size.y, corruption, n as f32);
}
