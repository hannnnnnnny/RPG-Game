//! Stardew-style game menu (Tab / I, Esc closes): parchment window with tabs
//! 背包 (equipment column + stats + bag grid), 任务 (journal), 日志 (log).
//! Click a bag item to equip it, an equipped slot to take it off; hovering
//! any item shows a parchment tooltip that follows the cursor.

use bevy::prelude::*;
use tides_core::item::{Item, ItemId, Slot};
use tides_core::loot;
use tides_core::story::journal;

use crate::beats::{MenuTab, Modal};
use crate::hud::quality_color;
use crate::item_icons::ItemIcons;
use crate::run_state::RunRes;
use crate::ui::{Fonts, Frame, INK, INK_SOFT, UiKit, text_font};

const CELL: f32 = 60.0;
const BAG_COLS: usize = 8;
const BAG_ROWS: usize = 4;
const EQUIP_SLOTS: [(Slot, &str); 6] = [
    (Slot::MainHand, "武器"),
    (Slot::Chest, "胸甲"),
    (Slot::Hands, "手套"),
    (Slot::Boots, "靴子"),
    (Slot::Ring, "戒指"),
    (Slot::Totem, "图腾"),
];

#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct TabButton(MenuTab);
#[derive(Component)]
struct BagCell(ItemId);
#[derive(Component)]
struct EquipCell(Slot);
#[derive(Component)]
struct Tooltip;

/// Item under the cursor, for the tooltip.
#[derive(Resource, Default, PartialEq)]
struct Hovered(Option<ItemId>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Hovered>().add_systems(
        Update,
        (toggle, rebuild, click_tabs, click_items, track_hover, tooltip).chain(),
    );
}

fn toggle(keys: Res<ButtonInput<KeyCode>>, mut modal: ResMut<Modal>) {
    let open_key = keys.any_just_pressed([KeyCode::Tab, KeyCode::KeyI, KeyCode::KeyB]);
    match &*modal {
        Modal::Menu(_) if open_key || keys.just_pressed(KeyCode::Escape) => *modal = Modal::None,
        Modal::None if open_key => *modal = Modal::Menu(MenuTab::Bag),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn rebuild(
    mut commands: Commands,
    modal: Res<Modal>,
    run: Res<RunRes>,
    roots: Query<Entity, With<MenuRoot>>,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
    mut icons: ResMut<ItemIcons>,
    mut images: ResMut<Assets<Image>>,
) {
    let open_tab = match &*modal {
        Modal::Menu(tab) => Some(*tab),
        _ => None,
    };
    if !modal.is_changed() && !(open_tab.is_some() && run.is_changed()) {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let Some(tab) = open_tab else { return };
    let ctx = Ctx { kit: &kit, font: &fonts.body };
    let root = commands.spawn(window_root()).id();
    commands.entity(root).with_children(|p| {
        tabs_row(p, &ctx, tab);
        p.spawn(window_body()).with_children(|p| {
            p.spawn(ctx.kit.backdrop(Frame::Parchment));
            match tab {
                MenuTab::Bag => bag_tab(p, &ctx, &run, &mut icons, &mut images),
                MenuTab::Journal => journal_tab(p, &ctx, &run),
                MenuTab::Log => log_tab(p, &ctx, &run),
            }
        });
    });
}

struct Ctx<'a> {
    kit: &'a UiKit,
    font: &'a Handle<Font>,
}

fn window_root() -> impl Bundle {
    (
        MenuRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        GlobalZIndex(20),
    )
}

fn window_body() -> Node {
    Node { width: px(880), height: px(470), padding: UiRect::axes(px(36), px(30)), column_gap: px(28), ..default() }
}

fn tabs_row(p: &mut ChildSpawnerCommands, ctx: &Ctx, active: MenuTab) {
    p.spawn(Node { width: px(880), column_gap: px(6), padding: UiRect::left(px(24)), ..default() }).with_children(|p| {
        for (tab, name) in [(MenuTab::Bag, "背包"), (MenuTab::Journal, "任务"), (MenuTab::Log, "日志")] {
            // The active tab sits lower, tucked into the window like Stardew's.
            let drop = if tab == active { 8.0 } else { 0.0 };
            p.spawn((
                TabButton(tab),
                Button,
                Node { width: px(96), height: px(48), top: px(drop), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                children![ctx.kit.backdrop(Frame::Parchment), (Text::new(name), text_font(ctx.font, 16.0), TextColor(INK))],
            ));
        }
    });
}

fn slot_cell(p: &mut ChildSpawnerCommands, ctx: &Ctx, icon: Option<Handle<Image>>, marker: impl Bundle, caption: Option<&str>) {
    p.spawn((
        marker,
        Button,
        Node { width: px(CELL), height: px(CELL), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, ..default() },
    ))
    .with_children(|p| {
        p.spawn(ctx.kit.backdrop(Frame::Slot));
        match icon {
            Some(img) => {
                p.spawn((ImageNode::new(img), Node { width: px(42), height: px(42), ..default() }));
            }
            None => {
                p.spawn((Text::new(caption.unwrap_or("")), text_font(ctx.font, 11.0), TextColor(INK_SOFT)));
            }
        }
    });
}

fn bag_tab(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &RunRes, icons: &mut ItemIcons, images: &mut Assets<Image>) {
    // Left: name, equipment grid, stats.
    p.spawn(Node { width: px(250), flex_direction: FlexDirection::Column, row_gap: px(10), ..default() }).with_children(|p| {
        p.spawn((Text::new(run.profile.name.clone()), text_font(ctx.font, 22.0), TextColor(INK)));
        p.spawn(Node { flex_wrap: FlexWrap::Wrap, width: px(CELL * 3.0 + 12.0), column_gap: px(6), row_gap: px(6), ..default() }).with_children(|p| {
            for (slot, name) in EQUIP_SLOTS {
                let icon = run.equipped_items().find(|i| i.slot == slot).map(|i| icons.get(i.slot, i.quality, images));
                slot_cell(p, ctx, icon, EquipCell(slot), Some(name));
            }
        });
        let s = run.stats();
        let lines = format!(
            "攻击 {}    生命 {}\n暴击 {:.0}%    抗性 {:.0}%\n金币掉落 +{:.0}%",
            s.attack, s.max_health, pct(s.crit_chance), pct(s.damage_reduction), pct(s.gold_find)
        );
        p.spawn((Text::new(lines), text_font(ctx.font, 14.0), TextColor(INK_SOFT)));
    });
    // Right: the bag (unequipped items only, like Stardew).
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(10), ..default() }).with_children(|p| {
        p.spawn((Text::new("背包"), text_font(ctx.font, 18.0), TextColor(INK)));
        p.spawn(Node { flex_wrap: FlexWrap::Wrap, width: px((CELL + 6.0) * BAG_COLS as f32), column_gap: px(6), row_gap: px(6), ..default() }).with_children(|p| {
            let bag: Vec<&Item> = run.inventory.iter().filter(|i| !run.is_equipped(i)).collect();
            for i in 0..BAG_COLS * BAG_ROWS {
                match bag.get(i) {
                    Some(item) => slot_cell(p, ctx, Some(icons.get(item.slot, item.quality, images)), BagCell(item.id), None),
                    None => {
                        p.spawn((Node { width: px(CELL), height: px(CELL), ..default() }, children![ctx.kit.backdrop(Frame::Slot)]));
                    }
                }
            }
        });
    });
}

/// Percent with no "-0" from negative-zero floats.
fn pct(v: f32) -> f32 {
    (v * 100.0).round() + 0.0
}

fn journal_tab(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &RunRes) {
    let (chapter, objectives) = journal::current(run);
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(12), ..default() }).with_children(|p| {
        p.spawn((Text::new(chapter), text_font(ctx.font, 22.0), TextColor(INK)));
        for o in objectives {
            let (mark, color) = if o.done { ("√", INK_SOFT) } else { ("□", INK) };
            p.spawn((Text::new(format!("{mark}  {}", o.text)), text_font(ctx.font, 17.0), TextColor(color)));
        }
    });
}

fn log_tab(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &RunRes) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }).with_children(|p| {
        p.spawn((Text::new("日志"), text_font(ctx.font, 22.0), TextColor(INK)));
        for line in run.log.iter().take(14) {
            p.spawn((Text::new(format!("· {line}")), text_font(ctx.font, 14.0), TextColor(INK_SOFT)));
        }
    });
}

fn click_tabs(q: Query<(&Interaction, &TabButton), Changed<Interaction>>, mut modal: ResMut<Modal>) {
    for (i, tab) in &q {
        if *i == Interaction::Pressed {
            *modal = Modal::Menu(tab.0);
        }
    }
}

fn click_items(
    bag: Query<(&Interaction, &BagCell), Changed<Interaction>>,
    equip: Query<(&Interaction, &EquipCell), Changed<Interaction>>,
    mut run: ResMut<RunRes>,
) {
    for (i, cell) in &bag {
        if *i == Interaction::Pressed {
            let _ = run.equip(cell.0);
        }
    }
    for (i, cell) in &equip {
        if *i == Interaction::Pressed {
            run.unequip(cell.0);
        }
    }
}

fn track_hover(
    bag: Query<(&Interaction, &BagCell)>,
    equip: Query<(&Interaction, &EquipCell)>,
    run: Res<RunRes>,
    mut hovered: ResMut<Hovered>,
) {
    let from_bag = bag.iter().find(|(i, _)| **i != Interaction::None).map(|(_, c)| c.0);
    let from_equip = equip
        .iter()
        .find(|(i, _)| **i != Interaction::None)
        .and_then(|(_, c)| run.equipped.get(&c.0).copied());
    hovered.set_if_neq(Hovered(from_bag.or(from_equip)));
}

fn tooltip_text(item: &Item) -> String {
    let mut s = format!("{} · {} · 强度 {}\n", item.quality.label(), item.slot.label(), item.item_power);
    let mult = tides_core::stats::upgrade_multiplier(item);
    for a in &item.affixes {
        let locked = if Some(a.id) == item.locked_affix { "[锁] " } else { "" };
        s += &format!("{locked}{} +{}\n", a.kind.label(), (a.value as f32 * mult).round());
    }
    if let Some(core) = &item.core_effect {
        s += &format!("核心：{core}\n");
    }
    s + &format!("售价 {} 金", loot::sell_value(item))
}

#[allow(clippy::too_many_arguments)] // a Bevy system: each param is one resource or query
fn tooltip(
    mut commands: Commands,
    hovered: Res<Hovered>,
    run: Res<RunRes>,
    old: Query<Entity, With<Tooltip>>,
    mut placed: Query<&mut Node, With<Tooltip>>,
    window: Single<&Window>,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
) {
    if let Some(cursor) = window.cursor_position() {
        for mut n in &mut placed {
            n.left = px(cursor.x + 18.0);
            n.top = px(cursor.y + 18.0);
        }
    }
    if !hovered.is_changed() {
        return;
    }
    for e in &old {
        commands.entity(e).despawn();
    }
    let Some(item) = hovered.0.and_then(|id| run.find_item(id)) else { return };
    commands.spawn((
        Tooltip,
        Node { position_type: PositionType::Absolute, flex_direction: FlexDirection::Column, row_gap: px(4), padding: UiRect::axes(px(24), px(20)), max_width: px(320), ..default() },
        GlobalZIndex(30),
        children![
            kit.backdrop(Frame::Parchment),
            (Text::new(item.title()), text_font(&fonts.body, 17.0), TextColor(quality_ink(item))),
            (Text::new(tooltip_text(item)), text_font(&fonts.body, 13.0), TextColor(INK)),
        ],
    ));
}

/// Quality colours darkened so they read on parchment.
pub fn quality_ink(item: &Item) -> Color {
    let c = quality_color(item.quality).to_srgba();
    Color::srgb(c.red * 0.62, c.green * 0.55, c.blue * 0.55)
}
