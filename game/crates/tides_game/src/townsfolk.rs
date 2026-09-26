//! 灰灯镇 on entry: townsfolk (LPC rigs built from their look), their slow
//! wandering, the gray lamps, readable props and 克哈's arrival whisper.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tides_core::areas::{Pt, Town};
use tides_core::story::town::{self, Head, Npc, Robe};

use crate::area::Area;
use crate::beats::PlayBeat;
use crate::coords::to_world;
use crate::interact::{Interactable, Target};
use crate::lighting::Light;
use crate::lpc::{Facing, LpcAnim, LpcLayouts, Pose, attach_layers};
use crate::physics::{Body, Velocity, YSort};
use crate::props_art::{self, PROP_SCALE, PropArt};
use crate::run_state::RunRes;

const WANDER_SPEED: f32 = 24.0;
const SPRITES: &str = "sprites/";

#[derive(Component)]
struct Wander {
    home: Vec2,
    target: Vec2,
    timer: f32,
}

#[derive(Resource)]
struct ArrivalWhisper;

pub fn plugin(app: &mut App) {
    app.init_resource::<PropImages>()
        .add_systems(OnEnter(Area::Town), (spawn_folk, spawn_lamps, spawn_props, |mut c: Commands| c.insert_resource(ArrivalWhisper)))
        .add_systems(Update, (wander, whisper));
}

fn robe_sheet(r: Robe) -> &'static str {
    match r {
        Robe::Black => "townsfolk/robe_black.png",
        Robe::Brown => "townsfolk/robe_brown.png",
        Robe::Blue => "townsfolk/robe_blue.png",
        Robe::Red => "townsfolk/robe_red.png",
        Robe::ForestGreen => "townsfolk/robe_forest_green.png",
        Robe::DarkGray => "townsfolk/robe_dark_gray.png",
    }
}

/// Bottom-to-top layer sheets for an NPC's look and pose.
fn sheets(npc: &Npc) -> Vec<String> {
    let layers: Vec<&str> = if npc.seated {
        let hair = if npc.head == Head::Long { "townsfolk/hair_long_sit.png" } else { "townsfolk/hair_plain_sit.png" };
        vec!["townsfolk/body_sit.png", "townsfolk/shirt_sit.png", hair]
    } else {
        let head = match npc.head {
            Head::Hood => "disi/hood_walk.png",
            Head::Plain => "townsfolk/hair_plain_walk.png",
            Head::Long => "townsfolk/hair_long_walk.png",
            Head::BangsShort => "townsfolk/hair_bangsshort_walk.png",
        };
        vec!["disi/body_walk.png", robe_sheet(npc.robe), head]
    };
    layers.iter().map(|l| format!("{SPRITES}{l}")).collect()
}

fn spawn_folk(mut commands: Commands, assets: Res<AssetServer>, layouts: Res<LpcLayouts>) {
    for (i, npc) in town::NPCS.iter().enumerate() {
        let at = to_world(npc.pos);
        let pose = if npc.seated { Pose::Sit } else { Pose::Walk };
        let e = commands
            .spawn((
                DespawnOnExit(Area::Town),
                Interactable::new(Target::Townsfolk(i), 70.0),
                LpcAnim::new(pose),
                Velocity::default(),
                Body { half: Vec2::new(9.0, 5.0) },
                YSort,
                Transform::from_translation(at.extend(10.0)),
                Visibility::default(),
            ))
            .id();
        if npc.wanders {
            commands.entity(e).insert(Wander { home: at, target: at, timer: 0.0 });
        }
        let handles: Vec<_> = sheets(npc).into_iter().map(|p| assets.load(p)).collect();
        attach_layers(&mut commands, e, pose, &handles, &layouts);
    }
}

/// Amble around home: loiter half the time, otherwise pick a nearby spot.
fn wander(time: Res<Time>, mut q: Query<(&mut Wander, &mut Velocity, &mut LpcAnim, &Transform)>) {
    for (mut w, mut vel, mut anim, tf) in &mut q {
        let pos = tf.translation.truncate();
        w.timer -= time.delta_secs();
        if w.timer <= 0.0 || pos.distance(w.target) < 8.0 {
            w.timer = 2.4 + fastrand::f32() * 3.1;
            w.target = if fastrand::bool() {
                pos
            } else {
                w.home + Vec2::new(fastrand::f32() * 180.0 - 90.0, fastrand::f32() * 140.0 - 70.0)
            };
        }
        let to = w.target - pos;
        vel.0 = if to.length() > 8.0 { to.normalize() * WANDER_SPEED } else { Vec2::ZERO };
        anim.moving = vel.0 != Vec2::ZERO;
        if anim.moving {
            anim.facing = Facing::from_vec(vel.0);
        }
    }
}

/// One image per prop kind, painted once and shared.
#[derive(Resource, Default)]
struct PropImages(HashMap<PropArt, Handle<Image>>);

impl PropImages {
    fn get(&mut self, kind: PropArt, images: &mut Assets<Image>) -> Handle<Image> {
        self.0.entry(kind).or_insert_with(|| images.add(props_art::image(kind))).clone()
    }
}

fn prop(img: Handle<Image>, at: Pt) -> impl Bundle {
    (
        DespawnOnExit(Area::Town),
        Sprite::from_image(img),
        Anchor::BOTTOM_CENTER,
        YSort,
        Transform::from_translation(to_world(at).extend(10.0)).with_scale(Vec3::splat(PROP_SCALE)),
    )
}

fn spawn_lamps(mut commands: Commands, mut cache: ResMut<PropImages>, mut images: ResMut<Assets<Image>>) {
    let img = cache.get(PropArt::LampPost, &mut images);
    for p in Town::LAMPS {
        commands.spawn((
            prop(img.clone(), p),
            // Child offsets are in art pixels (the prop is scaled x3).
            children![(Light::new(Color::srgb(1.0, 0.8, 0.5), 170.0, 0.9).pulsing(0.08), Transform::from_xyz(0.0, 19.0, 0.0))],
        ));
    }
}

const NOTICE_LINES: &[&str] = &[
    "【告示】凡入镇者，须验手腕。见刺青者，鸣钟。——自治会",
    "【悬赏】矿区方向有黑潮渗出，能封住裂口者，重谢。",
    "【寻人】我的儿子下矿三天没回。若你见过他……别骗我。",
];
const FOUNTAIN_LINES: &[&str] = &["泉水从石口里淌出来，居然是清的。镇民轮班守着它，像守着最后一盏灯。"];
const LOCKED_LINES: &[&str] = &["门闩着。屋里有人压低声音说话，一听见脚步就停了。"];
const CLUTTER: [(PropArt, Pt); 6] = [
    (PropArt::Barrel, (560.0, 300.0)),
    (PropArt::Barrel, (578.0, 304.0)),
    (PropArt::Crate, (820.0, 300.0)),
    (PropArt::Crate, (280.0, 360.0)),
    (PropArt::Barrel, (860.0, 560.0)),
    (PropArt::Crate, (240.0, 420.0)),
];

fn spawn_props(mut commands: Commands, mut cache: ResMut<PropImages>, mut images: ResMut<Assets<Image>>) {
    let mut img = |k| cache.get(k, &mut images);
    commands.spawn((prop(img(PropArt::Notice), Town::NOTICE), Interactable::new(Target::Sign("灰灯镇告示板", NOTICE_LINES), 72.0)));
    commands.spawn((prop(img(PropArt::Fountain), Town::FOUNTAIN), Interactable::new(Target::Sign("镇心喷泉", FOUNTAIN_LINES), 76.0)));
    commands.spawn((prop(img(PropArt::Anvil), Town::ANVIL), Interactable::new(Target::Forge, 58.0),
        children![(Light::new(Color::srgb(1.0, 0.55, 0.25), 120.0, 0.9).pulsing(0.2), Transform::from_xyz(-10.0, 6.0, 0.0))]));
    for b in Town::BUILDINGS {
        let foot = (b.rect.x + b.rect.w / 2.0, b.rect.y + b.rect.h);
        let target = if b.name == "杂货铺" { Target::Shop } else { Target::Sign(b.name, LOCKED_LINES) };
        commands.spawn((prop(img(PropArt::Door), foot), Interactable::new(target, 76.0)));
    }
    for (k, p) in CLUTTER {
        commands.spawn(prop(img(k), p));
    }
}

fn whisper(mut commands: Commands, pending: Option<Res<ArrivalWhisper>>, run: Res<RunRes>, mut beats: MessageWriter<PlayBeat>) {
    if pending.is_some() {
        commands.remove_resource::<ArrivalWhisper>();
        beats.write(PlayBeat(town::on_enter(&run)));
    }
}
