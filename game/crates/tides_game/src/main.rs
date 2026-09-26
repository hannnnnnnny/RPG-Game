//! 《潮蚀之环》— Bevy front end. Rules live in `tides_core`; this crate is
//! presentation + input, organised as one plugin per domain.

use bevy::prelude::*;

mod area;
mod spots;
mod pixelated;
mod modal;
mod dialogue;
mod beats;
mod boss;
mod camera;
mod combat;
mod coords;
mod devtools;
mod enemy;
mod hud;
mod interact;
mod lighting;
mod lpc;
mod paint;
mod physics;
mod player;
mod run_state;
mod tiles_mine;
mod tiles_town;
mod townsfolk;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest()) // crisp pixel art
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "潮蚀之环 · Tides of Khah".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.05, 0.04, 0.06)))
        // Engine-level services, then world/gameplay, then presentation.
        .add_plugins((ui::plugin, run_state::plugin, pixelated::plugin, lpc::plugin, devtools::plugin))
        .add_plugins((area::plugin, physics::plugin, lighting::plugin, camera::plugin))
        .add_plugins((player::plugin, enemy::plugin, boss::plugin, combat::plugin, spots::plugin, townsfolk::plugin))
        .add_plugins((beats::plugin, dialogue::plugin, modal::plugin, hud::plugin, interact::plugin))
        .run();
}
