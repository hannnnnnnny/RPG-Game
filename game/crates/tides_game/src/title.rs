//! Title screen: 继续 (load the save) or 新的旅程 (name the vessel and
//! start over). Starting over when a save exists asks twice, since the first
//! autosave overwrites it. Typing goes into the name field (IME included, so
//! Chinese names work). Dev runs (`TIDES_STAGE` / `TIDES_CAPTURE`) skip
//! straight into the mine, except `TIDES_STAGE=title`.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::window::{Ime, PrimaryWindow};
use tides_core::run::{NAME_MAX_CHARS, Profile, Run};
use tides_core::story::{self, AreaId};

use crate::area::{Area, Arrival};
use crate::beats::Modal;
use crate::combat::Respawn;
use crate::coords::to_world;
use crate::persist::{Autosave, SaveSlot, dev_run};
use crate::pixelated::Pixelated;
use crate::run_state::RunRes;
use crate::ui::{CREAM, Fonts, Frame, GOLD_TEXT, INK, INK_RED, INK_SOFT, UiKit, plank_button, text_font};

const DEFAULT_NAME: &str = "迪丝";

/// What the save slot held when the title opened.
enum SaveInfo {
    None,
    Found(Box<Run>),
    Broken(String),
}

#[derive(Resource)]
struct TitleState {
    save: SaveInfo,
    name: String,
    /// 新的旅程 was clicked once over an existing save.
    confirm_new: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum TitleButton {
    Continue,
    NewGame,
}

#[derive(Component)]
struct TitleRoot;

pub fn plugin(app: &mut App) {
    app.add_systems(PostStartup, boot)
        .add_systems(Update, (type_name, click, rebuild).chain().run_if(resource_exists::<TitleState>));
}

fn boot(mut commands: Commands, slot: Res<SaveSlot>, mut modal: ResMut<Modal>, mut next: ResMut<NextState<Area>>, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    let staged_title = std::env::var("TIDES_STAGE").is_ok_and(|s| s == "title");
    if dev_run() && !staged_title {
        next.set(Area::Mine);
        return;
    }
    let save = match slot.read() {
        Ok(Some(run)) => SaveInfo::Found(Box::new(run)),
        Ok(None) => SaveInfo::None,
        Err(e) => SaveInfo::Broken(e.to_string()),
    };
    commands.insert_resource(TitleState { save, name: DEFAULT_NAME.into(), confirm_new: false });
    *modal = Modal::Title;
    window.ime_enabled = true;
}

/// Typed text and IME commits extend the name; Backspace trims it.
fn type_name(mut keys: MessageReader<KeyboardInput>, mut ime: MessageReader<Ime>, mut st: ResMut<TitleState>, modal: Res<Modal>) {
    if *modal != Modal::Title {
        return;
    }
    let mut typed = String::new();
    for k in keys.read().filter(|k| k.state == ButtonState::Pressed) {
        match &k.logical_key {
            Key::Backspace => {
                st.name.pop();
            }
            _ => typed.extend(k.text.iter().flat_map(|t| t.chars())),
        }
    }
    for e in ime.read() {
        if let Ime::Commit { value, .. } = e {
            typed.push_str(value);
        }
    }
    for c in typed.chars().filter(|c| !c.is_control()) {
        if st.name.chars().count() < NAME_MAX_CHARS {
            st.name.push(c);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn click(
    q: Query<(&Interaction, &TitleButton), Changed<Interaction>>,
    mut st: ResMut<TitleState>,
    mut run: ResMut<RunRes>,
    mut modal: ResMut<Modal>,
    mut next: ResMut<NextState<Area>>,
    mut arrival: ResMut<Arrival>,
    mut respawn: ResMut<Respawn>,
    mut auto: ResMut<Autosave>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let Some(btn) = q.iter().find(|(i, _)| **i == Interaction::Pressed).map(|(_, b)| *b) else { return };
    let chosen = match (btn, &st.save) {
        (TitleButton::Continue, SaveInfo::Found(saved)) => Some((**saved).clone()),
        (TitleButton::NewGame, SaveInfo::None) => new_run(&st.name),
        (TitleButton::NewGame, _) if st.confirm_new => new_run(&st.name),
        (TitleButton::NewGame, _) => {
            st.confirm_new = true;
            None
        }
        _ => None,
    };
    let Some(chosen) = chosen else { return };
    let (area, at) = story::resume_point(&chosen);
    arrival.0 = at;
    if let Some(p) = at {
        respawn.0 = to_world(p);
    }
    run.0 = chosen;
    auto.enabled = true;
    *modal = Modal::None;
    next.set(area.into());
    window.ime_enabled = false;
}

fn new_run(typed: &str) -> Option<Run> {
    Some(Run::new(&Profile::sanitize_name(typed).unwrap_or_else(|| DEFAULT_NAME.into())))
}

fn area_name(a: AreaId) -> &'static str {
    match a {
        AreaId::Mine => "黑潮矿区",
        AreaId::Town => "灰灯镇",
    }
}

// ---------------- Drawing ----------------

struct Ctx<'a> {
    kit: &'a UiKit,
    font: &'a Handle<Font>,
}

#[allow(clippy::too_many_arguments)]
fn rebuild(
    mut commands: Commands,
    st: Res<TitleState>,
    modal: Res<Modal>,
    roots: Query<Entity, With<TitleRoot>>,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
    assets: Res<AssetServer>,
    mut pixelated: ResMut<Pixelated>,
    mut images: ResMut<Assets<Image>>,
) {
    if !st.is_changed() && !modal.is_changed() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    if *modal != Modal::Title {
        return;
    }
    let ctx = Ctx { kit: &kit, font: &fonts.body };
    let backdrop = pixelated.get("characters/khah.jpg", 200, &assets, &mut images);
    commands.spawn(screen(backdrop)).with_children(|p| {
        logo(p, &ctx);
        p.spawn(Node { width: px(540), flex_direction: FlexDirection::Column, row_gap: px(14), padding: UiRect::axes(px(40), px(32)), ..default() })
            .with_children(|p| {
                p.spawn(ctx.kit.backdrop(Frame::Parchment));
                save_card(p, &ctx, &st);
                name_field(p, &ctx, &st);
            });
    });
}

/// Opaque full-screen layer: 克哈's portrait, pixelated and sunk in shadow.
fn screen(backdrop: Handle<Image>) -> impl Bundle {
    (
        TitleRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(26),
            ..default()
        },
        BackgroundColor(Color::srgb(0.03, 0.02, 0.06)),
        GlobalZIndex(40),
        children![(
            ImageNode { image: backdrop, color: Color::srgba(0.45, 0.4, 0.6, 0.55), ..default() },
            Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
            ZIndex(-1),
        )],
    )
}

fn logo(p: &mut ChildSpawnerCommands, ctx: &Ctx) {
    let shadow = TextShadow { offset: Vec2::splat(4.0), color: Color::srgba(0.1, 0.03, 0.12, 0.9) };
    p.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(6), ..default() }).with_children(|p| {
        p.spawn((Text::new("潮蚀之环"), text_font(ctx.font, 72.0), TextColor(GOLD_TEXT), shadow));
        p.spawn((Text::new("T I D E S   O F   K H A H"), text_font(ctx.font, 16.0), TextColor(CREAM), shadow));
    });
}

fn save_card(p: &mut ChildSpawnerCommands, ctx: &Ctx, st: &TitleState) {
    match &st.save {
        SaveInfo::Found(run) => {
            let (area, _) = story::resume_point(run);
            let line = format!("{} · {} · {} 金", run.profile.name, area_name(area), run.world.gold);
            p.spawn((Text::new(line), text_font(ctx.font, 16.0), TextColor(INK)));
            plank_button(p, ctx.kit, ctx.font, "继续旅程", TitleButton::Continue, true);
        }
        SaveInfo::None => {
            p.spawn((Text::new("还没有存档。"), text_font(ctx.font, 15.0), TextColor(INK_SOFT)));
        }
        SaveInfo::Broken(why) => {
            p.spawn((Text::new(format!("存档读不出来了（{why}）。\n开始新的旅程会覆盖它。")), text_font(ctx.font, 13.0), TextColor(INK_RED)));
        }
    }
}

fn name_field(p: &mut ChildSpawnerCommands, ctx: &Ctx, st: &TitleState) {
    p.spawn((Text::new(format!("容器的名字（最多 {NAME_MAX_CHARS} 字，Backspace 删除）")), text_font(ctx.font, 13.0), TextColor(INK_SOFT)));
    p.spawn(Node { padding: UiRect::axes(px(18), px(12)), ..default() }).with_children(|p| {
        p.spawn(ctx.kit.backdrop(Frame::Slot));
        p.spawn((Text::new(format!("{}_", st.name)), text_font(ctx.font, 22.0), TextColor(INK)));
    });
    let overwriting = !matches!(st.save, SaveInfo::None);
    let label = if overwriting && st.confirm_new { "确认覆盖存档，重新开始" } else { "新的旅程" };
    plank_button(p, ctx.kit, ctx.font, label, TitleButton::NewGame, true);
    if overwriting && st.confirm_new {
        p.spawn((Text::new("旧存档会被覆盖，无法找回。"), text_font(ctx.font, 13.0), TextColor(INK_RED)));
    }
}
