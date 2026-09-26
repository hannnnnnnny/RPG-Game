//! 《潮蚀之环》— Bevy front end. Rules live in `tides_core`; this crate is
//! presentation + input, organised as one plugin per domain.

use bevy::prelude::*;

mod area;
mod camera;
mod combat;
mod coords;
mod devtools;
mod enemy;
mod hud;
mod lighting;
mod paint;
mod physics;
mod player;
mod run_state;
mod tiles_mine;
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
        .add_plugins((
            ui::plugin,
            run_state::plugin,
            area::plugin,
            physics::plugin,
            player::plugin,
            camera::plugin,
            enemy::plugin,
            combat::plugin,
            hud::plugin,
            lighting::plugin,
            devtools::plugin,
        ))
        .run();
}
