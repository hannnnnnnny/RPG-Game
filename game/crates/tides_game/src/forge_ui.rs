//! 老锤's anvil, Stardew-style: pick a piece of gear on the left, then on the
//! right upgrade it, reroll or lock one affix, or salvage it for materials.
//! A paid reroll must be settled (take new / keep old) before anything else,
//! salvage asks twice, and Esc closes (an unsettled offer keeps the old affix).

use bevy::prelude::*;
use tides_core::forge::{self, Material};
use tides_core::item::{Affix, Item, ItemId};
use tides_core::run::{ActionError, Run};
use tides_core::stats::upgrade_multiplier;

use crate::beats::Modal;
use crate::item_icons::ItemIcons;
use crate::menu::quality_ink;
use crate::run_state::{GameRng, RunRes};
use crate::ui::{Fonts, Frame, INK, INK_RED, INK_SOFT, UiKit, overlay, plank_button, text_cell, text_font};

const CELL: f32 = 56.0;
const LIST_COLS: usize = 4;
const LIST_SLOTS: usize = 20;

/// A paid-for reroll waiting for the player's verdict.
#[derive(Clone, Debug, PartialEq)]
struct Offer {
    item: ItemId,
    index: usize,
    affix: Affix,
}

#[derive(Resource, Default, PartialEq)]
struct ForgeState {
    selected: Option<ItemId>,
    offer: Option<Offer>,
    /// Salvage was clicked once; the next click breaks the item.
    confirm_salvage: bool,
    /// Last result line and whether it was a refusal.
    status: Option<(String, bool)>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum ForgeButton {
    Pick(ItemId),
    Upgrade,
    Reroll(usize),
    Lock(usize),
    Salvage,
    TakeOffer,
    KeepOffer,
    Close,
}

#[derive(Component)]
struct ForgeRoot;

pub fn plugin(app: &mut App) {
    app.init_resource::<ForgeState>().add_systems(Update, (on_open, close, click, rebuild).chain());
}

/// Fresh state each visit, with the first piece of gear preselected.
fn on_open(modal: Res<Modal>, run: Res<RunRes>, mut st: ResMut<ForgeState>) {
    if modal.is_changed() && *modal == Modal::Forge {
        *st = ForgeState { selected: run.inventory.first().map(|i| i.id), ..default() };
    }
}

fn close(keys: Res<ButtonInput<KeyCode>>, mut modal: ResMut<Modal>, mut st: ResMut<ForgeState>, mut run: ResMut<RunRes>) {
    if *modal == Modal::Forge && keys.just_pressed(KeyCode::Escape) {
        leave(&mut modal, &mut st, &mut run);
    }
}

fn leave(modal: &mut Modal, st: &mut ForgeState, run: &mut Run) {
    if let Some(o) = st.offer.take() {
        run.forge_resolve_reroll(o.item, o.index, o.affix, false);
    }
    *modal = Modal::None;
}

fn click(
    q: Query<(&Interaction, &ForgeButton), Changed<Interaction>>,
    mut modal: ResMut<Modal>,
    mut st: ResMut<ForgeState>,
    mut run: ResMut<RunRes>,
    mut rng: ResMut<GameRng>,
) {
    for (i, btn) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        if *btn == ForgeButton::Close {
            leave(&mut modal, &mut st, &mut run);
            return;
        }
        apply(*btn, &mut st, &mut run, &mut rng);
    }
}

/// Run one button press against the rules and record what 老锤 says back.
fn apply(btn: ForgeButton, st: &mut ForgeState, run: &mut Run, rng: &mut fastrand::Rng) {
    if btn != ForgeButton::Salvage {
        st.confirm_salvage = false;
    }
    let result = match btn {
        ForgeButton::Pick(id) => {
            st.selected = Some(id);
            Ok(None)
        }
        ForgeButton::TakeOffer | ForgeButton::KeepOffer => Ok(settle(st, run, btn == ForgeButton::TakeOffer)),
        _ => st.selected.ok_or(ActionError::NoSuchItem).and_then(|id| work(btn, id, st, run, rng)),
    };
    st.status = match result {
        Ok(msg) => msg.map(|m| (m, false)),
        Err(e) => Some((e.label().to_string(), true)),
    };
}

fn settle(st: &mut ForgeState, run: &mut Run, take: bool) -> Option<String> {
    let o = st.offer.take()?;
    run.forge_resolve_reroll(o.item, o.index, o.affix, take);
    Some(if take { "新词条烙进去了。" } else { "旧词条留下了，钱是不退的。" }.to_string())
}

fn work(btn: ForgeButton, id: ItemId, st: &mut ForgeState, run: &mut Run, rng: &mut fastrand::Rng) -> Result<Option<String>, ActionError> {
    match btn {
        ForgeButton::Upgrade => run.forge_upgrade(id).map(|_| Some("锤声落下，火星四溅。".into())),
        ForgeButton::Reroll(index) => {
            let affix = run.forge_reroll_offer(id, index, rng)?;
            st.offer = Some(Offer { item: id, index, affix });
            Ok(Some("炉火翻出一条新词条。要，还是不要？".into()))
        }
        ForgeButton::Lock(index) => run.forge_lock(id, index).map(|_| Some("这条词条被钉死了。".into())),
        ForgeButton::Salvage if !st.confirm_salvage => {
            st.confirm_salvage = true;
            Ok(Some("真要砸了它？再点一次「分解」。".into()))
        }
        ForgeButton::Salvage => {
            let gained = run.salvage(id)?;
            st.confirm_salvage = false;
            st.selected = run.inventory.first().map(|i| i.id);
            Ok(Some(format!("砸成了 {}。", materials_line(&gained))))
        }
        _ => Ok(None),
    }
}

fn materials_line(m: &forge::Materials) -> String {
    let parts: Vec<String> = m.iter().map(|(k, n)| format!("{} ×{n}", k.label())).collect();
    if parts.is_empty() { "一堆废铁".into() } else { parts.join("、") }
}

// ---------------- Drawing ----------------

struct Ctx<'a> {
    kit: &'a UiKit,
    font: &'a Handle<Font>,
}

#[allow(clippy::too_many_arguments)]
fn rebuild(
    mut commands: Commands,
    modal: Res<Modal>,
    run: Res<RunRes>,
    st: Res<ForgeState>,
    roots: Query<Entity, With<ForgeRoot>>,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
    mut icons: ResMut<ItemIcons>,
    mut images: ResMut<Assets<Image>>,
) {
    let open = *modal == Modal::Forge;
    if !modal.is_changed() && !(open && (run.is_changed() || st.is_changed())) {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !open {
        return;
    }
    let ctx = Ctx { kit: &kit, font: &fonts.body };
    commands.spawn(overlay(ForgeRoot)).with_children(|p| {
        p.spawn(window_body()).with_children(|p| {
            p.spawn(ctx.kit.backdrop(Frame::Parchment));
            header(p, &ctx, &run);
            p.spawn(Node { column_gap: px(28), flex_grow: 1.0, ..default() }).with_children(|p| {
                item_list(p, &ctx, &run, &st, &mut icons, &mut images);
                detail(p, &ctx, &run, &st);
            });
            status_line(p, &ctx, &st);
        });
    });
}

fn window_body() -> Node {
    Node {
        width: px(900),
        height: px(520),
        flex_direction: FlexDirection::Column,
        row_gap: px(14),
        padding: UiRect::axes(px(36), px(30)),
        ..default()
    }
}

fn header(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run) {
    let mats: Vec<String> = Material::ALL.iter().map(|m| format!("{} {}", m.label(), run.materials.get(m).copied().unwrap_or(0))).collect();
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|p| {
        p.spawn((Text::new("老锤的铁砧"), text_font(ctx.font, 24.0), TextColor(INK)));
        p.spawn(Node { column_gap: px(18), align_items: AlignItems::Center, ..default() }).with_children(|p| {
            p.spawn((Text::new(mats.join("  ·  ")), text_font(ctx.font, 14.0), TextColor(INK_SOFT)));
            p.spawn((Text::new(format!("{} 金", run.world.gold)), text_font(ctx.font, 16.0), TextColor(INK)));
            button(p, ctx, "×", ForgeButton::Close, true);
        });
    });
}

fn item_list(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run, st: &ForgeState, icons: &mut ItemIcons, images: &mut Assets<Image>) {
    let width = (CELL + 6.0) * LIST_COLS as f32;
    p.spawn(Node { width: px(width), flex_wrap: FlexWrap::Wrap, align_content: AlignContent::FlexStart, column_gap: px(6), row_gap: px(6), ..default() })
        .with_children(|p| {
            for i in 0..LIST_SLOTS {
                let Some(item) = run.inventory.get(i) else {
                    p.spawn((Node { width: px(CELL), height: px(CELL), ..default() }, children![ctx.kit.backdrop(Frame::Slot)]));
                    continue;
                };
                // The chosen piece sits on parchment instead of a sunken slot.
                let frame = if st.selected == Some(item.id) { Frame::Parchment } else { Frame::Slot };
                p.spawn((
                    ForgeButton::Pick(item.id),
                    Button,
                    Node { width: px(CELL), height: px(CELL), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    children![ctx.kit.backdrop(frame), (ImageNode::new(icons.get(item.slot, item.quality, images)), Node { width: px(40), height: px(40), ..default() })],
                ));
            }
        });
}

fn detail(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run, st: &ForgeState) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(10), flex_grow: 1.0, ..default() }).with_children(|p| {
        let Some(item) = st.selected.and_then(|id| run.find_item(id)) else {
            let hint = if run.inventory.is_empty() { "老锤：「空着手来的？去矿里带点东西回来。」" } else { "挑一件放上铁砧。" };
            p.spawn((Text::new(hint), text_font(ctx.font, 17.0), TextColor(INK_SOFT)));
            return;
        };
        let worn = if run.is_equipped(item) { "  · 装备中" } else { "" };
        p.spawn((Text::new(format!("{}{worn}", item.title())), text_font(ctx.font, 20.0), TextColor(quality_ink(item))));
        let busy = st.offer.is_some();
        upgrade_row(p, ctx, item, run, busy);
        for (index, affix) in item.affixes.iter().enumerate() {
            affix_row(p, ctx, item, index, affix, run, busy);
        }
        if let Some(o) = &st.offer {
            offer_panel(p, ctx, item, o);
        }
        salvage_row(p, ctx, item, run, st);
    });
}

fn row(p: &mut ChildSpawnerCommands, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(10), ..default() }).with_children(build);
}

fn upgrade_row(p: &mut ChildSpawnerCommands, ctx: &Ctx, item: &Item, run: &Run, busy: bool) {
    row(p, |p| {
        let lvl = format!("强化 +{}/{}", item.upgrade_level, forge::MAX_UPGRADE);
        cell(p, ctx, lvl, 15.0, INK, 200.0);
        if forge::can_upgrade(item) {
            let cost = forge::upgrade_cost(item);
            button(p, ctx, &format!("强化  {cost} 金"), ForgeButton::Upgrade, !busy && run.world.gold >= cost);
        } else {
            p.spawn((Text::new("已到极限"), text_font(ctx.font, 14.0), TextColor(INK_SOFT)));
        }
    });
}

fn affix_text(item: &Item, a: &Affix) -> String {
    format!("{} +{}", a.kind.label(), (a.value as f32 * upgrade_multiplier(item)).round())
}

fn affix_row(p: &mut ChildSpawnerCommands, ctx: &Ctx, item: &Item, index: usize, a: &Affix, run: &Run, busy: bool) {
    let locked = forge::is_locked(item, index);
    row(p, |p| {
        let mark = if locked { "[锁] " } else { "· " };
        cell(p, ctx, format!("{mark}{}", affix_text(item, a)), 15.0, INK, 200.0);
        if locked {
            p.spawn((Text::new("已锁定，重铸会绕开它"), text_font(ctx.font, 13.0), TextColor(INK_SOFT)));
            return;
        }
        let cost = forge::reroll_cost(item);
        button(p, ctx, &format!("重铸  {cost} 金"), ForgeButton::Reroll(index), !busy && run.world.gold >= cost);
        button(p, ctx, "锁定  1 稀有材料", ForgeButton::Lock(index), !busy && run.has_materials(&forge::lock_cost()));
    });
}

fn offer_panel(p: &mut ChildSpawnerCommands, ctx: &Ctx, item: &Item, o: &Offer) {
    let old = item.affixes.get(o.index).map(|a| affix_text(item, a)).unwrap_or_default();
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(8), padding: UiRect::axes(px(20), px(16)), ..default() }).with_children(|p| {
        p.spawn(ctx.kit.backdrop(Frame::Slot));
        p.spawn((Text::new(format!("旧：{old}    →    新：{}", affix_text(item, &o.affix))), text_font(ctx.font, 15.0), TextColor(INK)));
        row(p, |p| {
            button(p, ctx, "要新的", ForgeButton::TakeOffer, true);
            button(p, ctx, "留旧的", ForgeButton::KeepOffer, true);
        });
    });
}

fn salvage_row(p: &mut ChildSpawnerCommands, ctx: &Ctx, item: &Item, run: &Run, st: &ForgeState) {
    row(p, |p| {
        let preview = format!("分解可得：{}", materials_line(&forge::salvage_yield(item)));
        cell(p, ctx, preview, 14.0, INK_SOFT, 300.0);
        if run.is_equipped(item) {
            p.spawn((Text::new("先卸下才能分解"), text_font(ctx.font, 13.0), TextColor(INK_SOFT)));
            return;
        }
        let label = if st.confirm_salvage { "确认分解！" } else { "分解" };
        button(p, ctx, label, ForgeButton::Salvage, st.offer.is_none());
    });
}

fn status_line(p: &mut ChildSpawnerCommands, ctx: &Ctx, st: &ForgeState) {
    let (text, err) = st.status.clone().unwrap_or_else(|| ("老锤叼着烟斗，等你开口。".into(), false));
    p.spawn((Text::new(text), text_font(ctx.font, 15.0), TextColor(if err { INK_RED } else { INK_SOFT })));
}

fn button(p: &mut ChildSpawnerCommands, ctx: &Ctx, label: &str, action: ForgeButton, enabled: bool) {
    plank_button(p, ctx.kit, ctx.font, label, action, enabled);
}

fn cell(p: &mut ChildSpawnerCommands, ctx: &Ctx, text: impl Into<String>, size: f32, color: Color, width: f32) {
    text_cell(p, ctx.font, text, size, color, width);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tides_core::loot::{self, DropSource};

    fn run_with_item(gold: u32) -> (Run, ItemId) {
        let mut run = Run::new("t");
        run.add_item(loot::generate(DropSource::Elite, 1, &mut fastrand::Rng::with_seed(3)));
        run.world.gold = gold;
        let id = run.inventory[0].id;
        (run, id)
    }

    #[test]
    fn salvage_needs_two_clicks() {
        let (mut run, id) = run_with_item(0);
        let mut st = ForgeState { selected: Some(id), ..default() };
        let mut rng = fastrand::Rng::with_seed(1);
        apply(ForgeButton::Salvage, &mut st, &mut run, &mut rng);
        assert!(run.find_item(id).is_some());
        apply(ForgeButton::Salvage, &mut st, &mut run, &mut rng);
        assert!(run.find_item(id).is_none());
    }

    #[test]
    fn another_click_disarms_salvage() {
        let (mut run, id) = run_with_item(0);
        let mut st = ForgeState { selected: Some(id), ..default() };
        let mut rng = fastrand::Rng::with_seed(1);
        apply(ForgeButton::Salvage, &mut st, &mut run, &mut rng);
        apply(ForgeButton::Pick(id), &mut st, &mut run, &mut rng);
        apply(ForgeButton::Salvage, &mut st, &mut run, &mut rng);
        assert!(run.find_item(id).is_some());
    }

    #[test]
    fn refused_actions_report_errors() {
        let (mut run, id) = run_with_item(0);
        let mut st = ForgeState { selected: Some(id), ..default() };
        apply(ForgeButton::Upgrade, &mut st, &mut run, &mut fastrand::Rng::with_seed(1));
        assert_eq!(st.status, Some((ActionError::NotEnoughGold.label().to_string(), true)));
    }

    #[test]
    fn leaving_with_an_open_offer_keeps_the_old_affix() {
        let (mut run, id) = run_with_item(10_000);
        let before = run.find_item(id).unwrap().affixes.clone();
        let mut st = ForgeState { selected: Some(id), ..default() };
        apply(ForgeButton::Reroll(0), &mut st, &mut run, &mut fastrand::Rng::with_seed(1));
        assert!(st.offer.is_some());
        let mut modal = Modal::Forge;
        leave(&mut modal, &mut st, &mut run);
        let item = run.find_item(id).unwrap();
        assert_eq!(item.affixes, before);
        assert_eq!(item.reroll_count, 1);
        assert!(modal == Modal::None);
    }
}
