//! Presents story `Beat`s from tides_core: queue dialogue, open the choice
//! or vision, hand out loot/gold, and signal boss spawns / area changes.

use std::collections::VecDeque;

use bevy::prelude::*;
use tides_core::loot;
use tides_core::story::{AreaId, Beat, Choice, Line, Vision};

use crate::run_state::{GameRng, RunRes};

#[derive(Message, Clone)]
pub struct PlayBeat(pub Beat);

#[derive(Message, Clone, Copy)]
pub struct SpawnBoss;

#[derive(Message, Clone, Copy)]
pub struct GoTo(pub AreaId);

/// Lines waiting to be read, front = on screen.
#[derive(Resource, Default)]
pub struct DialogueQueue(pub VecDeque<Line>);

/// The open modal, if any. Modals freeze player control.
#[derive(Resource, Default, PartialEq)]
pub enum Modal {
    #[default]
    None,
    Choice(Choice),
    Vision(Vision),
    /// The Stardew-style game menu (Tab / I).
    Menu(MenuTab),
    /// 老锤's anvil.
    Forge,
    /// 铜婶's general store.
    Shop,
    /// The title screen (before any area is entered).
    Title,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuTab {
    #[default]
    Bag,
    Journal,
    Log,
}

impl Modal {
    pub fn is_open(&self) -> bool {
        *self != Modal::None
    }
}

pub fn no_modal(modal: Res<Modal>) -> bool {
    !modal.is_open()
}

pub fn plugin(app: &mut App) {
    app.add_message::<PlayBeat>()
        .add_message::<SpawnBoss>()
        .add_message::<GoTo>()
        .init_resource::<DialogueQueue>()
        .init_resource::<Modal>()
        .add_systems(Update, play_beats);
}

fn play_beats(
    mut beats: MessageReader<PlayBeat>,
    mut queue: ResMut<DialogueQueue>,
    mut modal: ResMut<Modal>,
    mut run: ResMut<RunRes>,
    mut rng: ResMut<GameRng>,
    mut boss: MessageWriter<SpawnBoss>,
    mut go: MessageWriter<GoTo>,
) {
    for PlayBeat(beat) in beats.read() {
        queue.0.extend(beat.lines.iter().cloned());
        // A vision outranks a choice; both are one-shot story moments.
        if let Some(v) = &beat.vision {
            *modal = Modal::Vision(v.clone());
        } else if let Some(c) = &beat.choice {
            *modal = Modal::Choice(c.clone());
        }
        if let Some(src) = beat.drop {
            let tier = run.world.world_tier;
            run.add_item(loot::generate(src, tier, &mut rng));
        }
        if let Some(src) = beat.gold {
            run.add_kill_gold(src, &mut rng);
        }
        if beat.spawn_boss {
            boss.write(SpawnBoss);
        }
        if let Some(area) = beat.go_to {
            go.write(GoTo(area));
        }
    }
}
