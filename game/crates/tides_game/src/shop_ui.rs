//! 铜婶's general store, Stardew-style: the shelf of wares on the right
//! (click to buy), the bag below it (click twice to sell), and 铜婶's
//! running commentary on the left. Esc or × closes.

use bevy::prelude::*;
use tides_core::item::ItemId;
use tides_core::loot;
use tides_core::run::Run;
use tides_core::shop::{STOCK, Ware};

use crate::beats::Modal;
use crate::item_icons::ItemIcons;
use crate::run_state::RunRes;
use crate::ui::{Fonts, Frame, INK, INK_RED, INK_SOFT, UiKit, overlay, plank_button, text_cell, text_font};

const CELL: f32 = 52.0;
const SELL_COLS: usize = 9;
const SELL_SLOTS: usize = 18;
const GREETING: &str = "「要点什么？灯油卖完了，别问。」";

#[derive(Resource, Default, PartialEq)]
struct ShopState {
    /// Item clicked once for sale; the next click on it sells.
    armed: Option<ItemId>,
    /// 铜婶's last line and whether it was a refusal.
    status: Option<(String, bool)>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum ShopButton {
    Buy(Ware),
    Sell(ItemId),
    Close,
}

#[derive(Component)]
struct ShopRoot;

pub fn plugin(app: &mut App) {
    app.init_resource::<ShopState>().add_systems(Update, (on_open, close, click, rebuild).chain());
}

fn on_open(modal: Res<Modal>, mut st: ResMut<ShopState>) {
    if modal.is_changed() && *modal == Modal::Shop {
        *st = ShopState::default();
    }
}

fn close(keys: Res<ButtonInput<KeyCode>>, mut modal: ResMut<Modal>) {
    if *modal == Modal::Shop && keys.just_pressed(KeyCode::Escape) {
        *modal = Modal::None;
    }
}

fn click(q: Query<(&Interaction, &ShopButton), Changed<Interaction>>, mut modal: ResMut<Modal>, mut st: ResMut<ShopState>, mut run: ResMut<RunRes>) {
    for (i, btn) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        if *btn == ShopButton::Close {
            *modal = Modal::None;
            return;
        }
        apply(*btn, &mut st, &mut run);
    }
}

/// One press against the rules; 铜婶 answers in the status line.
fn apply(btn: ShopButton, st: &mut ShopState, run: &mut Run) {
    let armed = st.armed.take();
    st.status = match btn {
        ShopButton::Buy(ware) => Some(match run.buy(ware) {
            Ok(()) => (format!("「{}，拿好。」", ware.label()), false),
            Err(e) => (e.label().to_string(), true),
        }),
        ShopButton::Sell(id) if armed != Some(id) => {
            st.armed = Some(id);
            run.find_item(id).map(|item| (format!("「{}？给你 {} 金。再点一下就成交。」", item.title(), loot::sell_value(item)), false))
        }
        ShopButton::Sell(id) => Some(match run.sell(id) {
            Ok(gold) => (format!("「成交，{gold} 金。」"), false),
            Err(e) => (e.label().to_string(), true),
        }),
        ShopButton::Close => None,
    };
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
    st: Res<ShopState>,
    roots: Query<Entity, With<ShopRoot>>,
    kit: Res<UiKit>,
    fonts: Res<Fonts>,
    mut icons: ResMut<ItemIcons>,
    mut images: ResMut<Assets<Image>>,
) {
    let open = *modal == Modal::Shop;
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
    commands.spawn(overlay(ShopRoot)).with_children(|p| {
        p.spawn(Node { width: px(900), height: px(540), padding: UiRect::axes(px(36), px(30)), column_gap: px(24), ..default() }).with_children(|p| {
            p.spawn(ctx.kit.backdrop(Frame::Parchment));
            keeper(p, &ctx, &st);
            p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(10), flex_grow: 1.0, ..default() }).with_children(|p| {
                header(p, &ctx, &run);
                for (ware, price) in STOCK {
                    ware_row(p, &ctx, &run, ware, price, &mut icons, &mut images);
                }
                sell_grid(p, &ctx, &run, &st, &mut icons, &mut images);
            });
        });
    });
}

/// 铜婶's corner: name plate and what she just said.
fn keeper(p: &mut ChildSpawnerCommands, ctx: &Ctx, st: &ShopState) {
    let (line, err) = st.status.clone().unwrap_or_else(|| (GREETING.into(), false));
    p.spawn(Node { width: px(200), flex_shrink: 0.0, flex_direction: FlexDirection::Column, row_gap: px(12), padding: UiRect::axes(px(18), px(16)), ..default() })
        .with_children(|p| {
            p.spawn(ctx.kit.backdrop(Frame::Slot));
            p.spawn((Text::new("铜婶"), text_font(ctx.font, 20.0), TextColor(INK)));
            p.spawn((Text::new(line), text_font(ctx.font, 15.0), TextColor(if err { INK_RED } else { INK_SOFT })));
        });
}

fn header(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run) {
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|p| {
        p.spawn((Text::new("灰灯镇杂货铺"), text_font(ctx.font, 24.0), TextColor(INK)));
        p.spawn(Node { column_gap: px(16), align_items: AlignItems::Center, ..default() }).with_children(|p| {
            p.spawn((Text::new(format!("{} 金", run.world.gold)), text_font(ctx.font, 16.0), TextColor(INK)));
            plank_button(p, ctx.kit, ctx.font, "×", ShopButton::Close, true);
        });
    });
}

fn owned(run: &Run, ware: Ware) -> u32 {
    match ware {
        Ware::Supply(s) => run.supply_count(s),
        Ware::Material(m) => run.materials.get(&m).copied().unwrap_or(0),
    }
}

fn ware_icon(ware: Ware, icons: &mut ItemIcons, images: &mut Assets<Image>) -> Handle<Image> {
    match ware {
        Ware::Supply(s) => icons.supply(s, images),
        Ware::Material(m) => icons.material(m, images),
    }
}

/// A shelf row: icon, name + blurb, how many you hold, and the buy button.
#[allow(clippy::too_many_arguments)]
fn ware_row(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run, ware: Ware, price: u32, icons: &mut ItemIcons, images: &mut Assets<Image>) {
    let icon = ware_icon(ware, icons, images);
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(12), padding: UiRect::axes(px(12), px(6)), ..default() }).with_children(|p| {
        p.spawn(ctx.kit.backdrop(Frame::Slot));
        p.spawn((ImageNode::new(icon), Node { width: px(40), height: px(40), flex_shrink: 0.0, ..default() }));
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(2), flex_grow: 1.0, ..default() }).with_children(|p| {
            p.spawn((Text::new(ware.label()), text_font(ctx.font, 16.0), TextColor(INK)));
            p.spawn((Text::new(ware.blurb()), text_font(ctx.font, 12.0), TextColor(INK_SOFT)));
        });
        text_cell(p, ctx.font, format!("持有 {}", owned(run, ware)), 13.0, INK_SOFT, 70.0);
        plank_button(p, ctx.kit, ctx.font, &format!("{price} 金"), ShopButton::Buy(ware), run.world.gold >= price);
    });
}

/// Unworn gear for sale; an armed item sits on parchment until confirmed.
fn sell_grid(p: &mut ChildSpawnerCommands, ctx: &Ctx, run: &Run, st: &ShopState, icons: &mut ItemIcons, images: &mut Assets<Image>) {
    let bag: Vec<_> = run.inventory.iter().filter(|i| !run.is_equipped(i)).collect();
    let title = if bag.is_empty() { "出售 · 背包里没有能卖的（穿着的要先卸下）" } else { "出售 · 点两下卖掉" };
    p.spawn((Text::new(title), text_font(ctx.font, 15.0), TextColor(INK), Node { margin: UiRect::top(px(6)), ..default() }));
    let width = (CELL + 6.0) * SELL_COLS as f32;
    p.spawn(Node { width: px(width), flex_wrap: FlexWrap::Wrap, column_gap: px(6), row_gap: px(6), ..default() }).with_children(|p| {
        for i in 0..SELL_SLOTS.max(bag.len()).min(SELL_COLS * 3) {
            let Some(item) = bag.get(i) else {
                p.spawn((Node { width: px(CELL), height: px(CELL), ..default() }, children![ctx.kit.backdrop(Frame::Slot)]));
                continue;
            };
            let frame = if st.armed == Some(item.id) { Frame::Parchment } else { Frame::Slot };
            p.spawn((
                ShopButton::Sell(item.id),
                Button,
                Node { width: px(CELL), height: px(CELL), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                children![ctx.kit.backdrop(frame), (ImageNode::new(icons.get(item.slot, item.quality, images)), Node { width: px(36), height: px(36), ..default() })],
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tides_core::loot::DropSource;
    use tides_core::run::ActionError;
    use tides_core::shop::Supply;

    fn run_with_item() -> (Run, ItemId) {
        let mut run = Run::new("t");
        run.add_item(loot::generate(DropSource::Enemy, 1, &mut fastrand::Rng::with_seed(5)));
        let id = run.inventory[0].id;
        (run, id)
    }

    #[test]
    fn selling_takes_two_clicks_on_the_same_item() {
        let (mut run, id) = run_with_item();
        let mut st = ShopState::default();
        apply(ShopButton::Sell(id), &mut st, &mut run);
        assert!(run.find_item(id).is_some());
        apply(ShopButton::Sell(id), &mut st, &mut run);
        assert!(run.find_item(id).is_none());
        assert!(run.world.gold > 0);
    }

    #[test]
    fn any_other_click_disarms_a_sale() {
        let (mut run, id) = run_with_item();
        let mut st = ShopState::default();
        apply(ShopButton::Sell(id), &mut st, &mut run);
        apply(ShopButton::Buy(Ware::Supply(Supply::HealingDraught)), &mut st, &mut run);
        apply(ShopButton::Sell(id), &mut st, &mut run);
        assert!(run.find_item(id).is_some());
    }

    #[test]
    fn broke_buyers_hear_why() {
        let (mut run, _) = run_with_item();
        let mut st = ShopState::default();
        apply(ShopButton::Buy(Ware::Supply(Supply::CalmingTea)), &mut st, &mut run);
        assert_eq!(st.status, Some((ActionError::NotEnoughGold.label().to_string(), true)));
    }
}
