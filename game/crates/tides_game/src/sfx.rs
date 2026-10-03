//! Sound: one-shot effects requested through `PlaySfx` (pitch jittered so
//! repeats don't drone), footsteps paced by the player's gait, hits and
//! pickups derived from existing messages, and the mine's ambient loop.

use bevy::audio::Volume;
use bevy::prelude::*;
use tides_core::run::RunEvent;

use crate::area::Area;
use crate::enemy::PlayerStruck;
use crate::player::{Motion, Player};
use crate::run_state::RunEventMsg;

const STEP_WALK: f32 = 0.34;
const STEP_SPRINT: f32 = 0.22;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    Swing,
    Hit,
    Pickup,
    Step,
}

#[derive(Message, Clone, Copy)]
pub struct PlaySfx(pub Sound);

#[derive(Resource)]
struct Clips {
    swing: Handle<AudioSource>,
    hit: Handle<AudioSource>,
    pickup: Handle<AudioSource>,
    step: Handle<AudioSource>,
    ambient: Handle<AudioSource>,
}

impl Clips {
    fn get(&self, s: Sound) -> (Handle<AudioSource>, f32) {
        match s {
            Sound::Swing => (self.swing.clone(), 0.5),
            Sound::Hit => (self.hit.clone(), 0.6),
            Sound::Pickup => (self.pickup.clone(), 0.6),
            Sound::Step => (self.step.clone(), 0.25),
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_message::<PlaySfx>()
        .add_systems(Startup, load)
        .add_systems(OnEnter(Area::Mine), start_ambient)
        .add_systems(Update, (footsteps, derived, play).chain());
}

fn load(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Clips {
        swing: assets.load("audio/swing.wav"),
        hit: assets.load("audio/hit.wav"),
        pickup: assets.load("audio/pickup.wav"),
        step: assets.load("audio/step.wav"),
        ambient: assets.load("audio/ambient.wav"),
    });
}

fn start_ambient(mut commands: Commands, clips: Res<Clips>) {
    commands.spawn((
        DespawnOnExit(Area::Mine),
        AudioPlayer::new(clips.ambient.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.45)),
    ));
}

fn play(mut commands: Commands, mut sfx: MessageReader<PlaySfx>, clips: Res<Clips>) {
    for PlaySfx(s) in sfx.read() {
        let (clip, volume) = clips.get(*s);
        let speed = 0.92 + fastrand::f32() * 0.16;
        commands.spawn((AudioPlayer::new(clip), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)).with_speed(speed)));
    }
}

/// Getting hurt and finding loot already have messages; sound them here.
fn derived(mut struck: MessageReader<PlayerStruck>, mut events: MessageReader<RunEventMsg>, mut out: MessageWriter<PlaySfx>) {
    if struck.read().count() > 0 {
        out.write(PlaySfx(Sound::Hit));
    }
    if events.read().any(|e| matches!(e.0, RunEvent::ItemGained(_))) {
        out.write(PlaySfx(Sound::Pickup));
    }
}

fn footsteps(time: Res<Time>, player: Single<&Motion, With<Player>>, mut until: Local<f32>, mut out: MessageWriter<PlaySfx>) {
    if !player.moving {
        *until = 0.0;
        return;
    }
    *until -= time.delta_secs();
    if *until <= 0.0 {
        *until = if player.sprinting { STEP_SPRINT } else { STEP_WALK };
        out.write(PlaySfx(Sound::Step));
    }
}
