//! One interaction rule for everything: E acts on the nearest *active*
//! `Interactable` in range, and an "E" bubble floats over it.

use bevy::prelude::*;
use tides_core::story::mine::{self, Spot};
use tides_core::story::town;

use crate::beats::PlayBeat;
use crate::coords::Z_OVERLAY;
use crate::dialogue::InteractPressed;
use crate::player::Player;
use crate::run_state::RunRes;
use crate::ui::{Fonts, text_font};

const RANGE: f32 = 84.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    MineSpot(Spot),
    /// Index into `town::NPCS`.
    Townsfolk(usize),
    /// Flavour text props (notice board, fountain…); cycles lines.
    Sign(&'static str, &'static [&'static str]),
    /// The smithy anvil: opens the forge.
    Forge,
    /// The general store's door.
    Shop,
}

#[derive(Component)]
pub struct Interactable {
    pub target: Target,
    /// How often this has been used (NPC/sign line cycling).
    pub uses: usize,
    /// Height of the prompt bubble above the entity's feet.
    pub prompt_y: f32,
}

impl Interactable {
    pub fn new(target: Target, prompt_y: f32) -> Self {
        Self { target, uses: 0, prompt_y }
    }
}

#[derive(Component)]
struct Prompt;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_prompt).add_systems(Update, (interact, update_prompt));
}

fn is_active(run: &RunRes, t: Target) -> bool {
    match t {
        Target::MineSpot(s) => mine::is_active(run, s),
        _ => true,
    }
}

fn nearest(player: Vec2, run: &RunRes, items: &[(Entity, Vec2, Target)]) -> Option<(Entity, Vec2)> {
    items
        .iter()
        .filter(|(_, p, t)| p.distance(player) < RANGE && is_active(run, *t))
        .min_by(|a, b| a.1.distance(player).total_cmp(&b.1.distance(player)))
        .map(|(e, p, _)| (*e, *p))
}

fn collect<'a>(items: impl Iterator<Item = (Entity, &'a GlobalTransform, &'a Interactable)>) -> Vec<(Entity, Vec2, Target)> {
    items.map(|(e, gt, i)| (e, gt.translation().truncate(), i.target)).collect()
}

fn run_target(run: &mut RunRes, it: &mut Interactable) -> PlayBeat {
    let beat = match it.target {
        Target::MineSpot(s) => mine::interact(run, s),
        Target::Townsfolk(i) => town::talk(run, &town::NPCS[i], it.uses),
        Target::Sign(title, lines) => town::sign(title, lines, it.uses),
        // Placeholder until the forge/shop menus land.
        Target::Forge => town::sign("铁砧", &["炉火还热着。老锤说，等你带东西回来再动手。"], it.uses),
        Target::Shop => town::sign("杂货铺", &["铜婶在柜台后打盹。门口挂着牌子：盘点中。"], it.uses),
    };
    it.uses += 1;
    PlayBeat(beat)
}

fn interact(
    mut pressed: MessageReader<InteractPressed>,
    player: Single<&Transform, With<Player>>,
    mut all: Query<(Entity, &GlobalTransform, &mut Interactable)>,
    mut run: ResMut<RunRes>,
    mut beats: MessageWriter<PlayBeat>,
) {
    if pressed.read().count() == 0 {
        return;
    }
    let found = nearest(player.translation.truncate(), &run, &collect(all.iter()));
    let Some((e, _)) = found else { return };
    if let Ok((_, _, mut it)) = all.get_mut(e) {
        beats.write(run_target(&mut run, &mut it));
    }
}

fn spawn_prompt(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
        Prompt,
        Text2d::new("E"),
        text_font(&fonts.body, 14.0),
        TextColor(Color::srgb(0.98, 0.93, 0.72)),
        Transform::from_xyz(0.0, 0.0, Z_OVERLAY),
        Visibility::Hidden,
    ));
}

fn update_prompt(
    player: Single<&Transform, (With<Player>, Without<Prompt>)>,
    all: Query<(Entity, &GlobalTransform, &Interactable)>,
    run: Res<RunRes>,
    mut prompt: Single<(&mut Transform, &mut Visibility), With<Prompt>>,
) {
    let found = nearest(player.translation.truncate(), &run, &collect(all.iter()));
    match found.and_then(|(e, at)| all.get(e).ok().map(|(_, _, i)| at + Vec2::Y * i.prompt_y)) {
        Some(at) => {
            prompt.0.translation = at.extend(Z_OVERLAY);
            *prompt.1 = Visibility::Inherited;
        }
        None => *prompt.1 = Visibility::Hidden,
    }
}
