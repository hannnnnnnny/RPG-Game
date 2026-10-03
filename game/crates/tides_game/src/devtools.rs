//! Dev-only hooks driven by env vars, used to verify changes headlessly:
//!
//! - `TIDES_CAPTURE=path.png` — screenshot after a few seconds, then quit.
//! - `TIDES_STAGE=<name>` — put the world in a known state first
//!   (see `stage`), e.g. `forge` or `combat`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use tides_core::areas::Mine;
use tides_core::forge::Material;
use tides_core::shop::Supply;
use tides_core::loot::{self, DropSource};
use tides_core::story::mine::{self, Spot};

use crate::area::Area;
use crate::beats::{DialogueQueue, MenuTab, Modal, PlayBeat, SpawnBoss};
use crate::coords::to_world;
use crate::player::Player;
use crate::run_state::RunRes;

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
    app.add_systems(Update, stage);
}

const STAGE_AT: f32 = 1.0;

/// Drive the story into a named state, once, shortly after start.
#[allow(clippy::too_many_arguments)] // a Bevy system: each param is one resource or query
fn stage(
    time: Res<Time>,
    mut done: Local<bool>,
    mut run: ResMut<RunRes>,
    mut queue: ResMut<DialogueQueue>,
    mut beats: MessageWriter<PlayBeat>,
    mut boss: MessageWriter<SpawnBoss>,
    mut player: Single<&mut Transform, With<Player>>,
    mut next_area: ResMut<NextState<Area>>,
    mut modal: ResMut<Modal>,
) {
    if *done || time.elapsed_secs() < STAGE_AT {
        return;
    }
    *done = true;
    let Ok(name) = std::env::var("TIDES_STAGE") else { return };
    queue.0.clear();
    if matches!(name.as_str(), "menu" | "journal" | "forge" | "shop") {
        let mut rng = fastrand::Rng::with_seed(7);
        for src in [DropSource::Elite, DropSource::Totem, DropSource::Enemy, DropSource::Enemy, DropSource::Elite] {
            let item = loot::generate(src, 1, &mut rng);
            run.add_item(item);
        }
        let first = run.inventory[0].id;
        let _ = run.equip(first);
        run.world.gold = 1284;
        run.materials.insert(Material::Rare, 2);
        run.supplies.insert(Supply::HealingDraught, 3);
        *modal = match name.as_str() {
            "menu" => Modal::Menu(MenuTab::Bag),
            "journal" => Modal::Menu(MenuTab::Journal),
            "shop" => Modal::Shop,
            _ => Modal::Forge,
        };
        return;
    }
    if name == "town" {
        next_area.set(Area::Town);
        return;
    }
    if name == "boss" {
        // Totem already touched, vision dismissed, standing in Grom's room.
        mine::interact(&mut run, Spot::TotemFragment);
        boss.write(SpawnBoss);
        let p = to_world((960.0, 470.0));
        player.translation.x = p.x;
        player.translation.y = p.y;
        return;
    }
    let (spot, at) = match name.as_str() {
        "choice" => (Spot::InjuredDwarf, Mine::INJURED_DWARF),
        "vision" => (Spot::TotemFragment, Mine::TOTEM),
        "exit" => (Spot::Exit, Mine::EXIT),
        _ => return,
    };
    let p = to_world((at.0 - 40.0, at.1 + 10.0));
    player.translation.x = p.x;
    player.translation.y = p.y;
    beats.write(PlayBeat(mine::interact(&mut run, spot)));
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
