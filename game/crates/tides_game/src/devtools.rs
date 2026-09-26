//! Dev-only hooks driven by env vars, used to verify changes headlessly:
//!
//! - `TIDES_CAPTURE=path.png` — screenshot after a few seconds, then quit.
//! - `TIDES_STAGE=<name>` — put the world in a known state first
//!   (see `stage`), e.g. `forge` or `combat`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

const CAPTURE_AT: f32 = 2.5;
const QUIT_AT: f32 = 3.5;

#[derive(Resource)]
struct Capture {
    path: String,
    taken: bool,
}

pub fn plugin(app: &mut App) {
    if let Ok(path) = std::env::var("TIDES_CAPTURE") {
        app.insert_resource(Capture { path, taken: false }).add_systems(Update, capture);
    }
}

pub fn stage_name() -> Option<String> {
    std::env::var("TIDES_STAGE").ok()
}

fn capture(mut commands: Commands, time: Res<Time>, mut cap: ResMut<Capture>, mut exit: MessageWriter<AppExit>) {
    let t = time.elapsed_secs();
    if !cap.taken && t > CAPTURE_AT {
        cap.taken = true;
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(cap.path.clone()));
    }
    if t > QUIT_AT {
        exit.write(AppExit::Success);
    }
}
