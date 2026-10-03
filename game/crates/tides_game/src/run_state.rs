//! The playthrough as ECS resources: the rules-owning `Run` plus a seeded RNG.

use bevy::prelude::*;
use tides_core::run::{Run, RunEvent};

#[derive(Resource, Deref, DerefMut)]
pub struct RunRes(pub Run);

#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(pub fastrand::Rng);

/// RunEvents re-published each frame for any system that wants them.
#[derive(Message, Clone, Debug)]
pub struct RunEventMsg(pub RunEvent);

pub fn plugin(app: &mut App) {
    app.insert_resource(RunRes(Run::new("迪丝")))
        .insert_resource(GameRng(fastrand::Rng::new()))
        .add_message::<RunEventMsg>()
        .add_systems(Last, publish_events);
}

/// Drain the Run's event queue once per frame into Bevy messages.
fn publish_events(mut run: ResMut<RunRes>, mut out: MessageWriter<RunEventMsg>) {
    if run.events.is_empty() {
        return;
    }
    for e in run.drain_events() {
        out.write(RunEventMsg(e));
    }
}
