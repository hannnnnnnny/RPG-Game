//! Stardew-style HUD: info box + gold counter (top right), vertical 血/力
//! bars (bottom right), and the equipment toolbar (bottom centre).

use bevy::prelude::*;
use tides_core::item::{Quality, Slot};
use tides_core::mind::{CorruptionTier, SanityTier, vessel_stage_name};

use crate::area::AreaTitle;
use crate::combat::Vitals;
use crate::player::{Motion, Player};
use crate::run_state::RunRes;
use crate::ui::{CREAM, Fonts, Frame, GOLD_TEXT, INK, INK_PURPLE, INK_RED, INK_SOFT, UiKit, text_font};

const BAR_H: f32 = 168.0;
const TOOLBAR: [(Slot, &str); 6] = [
    (Slot::MainHand, "武器"),
    (Slot::Chest, "胸甲"),
    (Slot::Hands, "手套"),
    (Slot::Boots, "靴子"),
    (Slot::Ring, "戒指"),
    (Slot::Totem, "图腾"),
];

#[derive(Component, Clone, Copy)]
enum Readout {
    Area,
    Sanity,
    Corruption,
    Vessel,
    Gold,
}

#[derive(Component, Clone, Copy)]
enum BarFill {
    Health,
    Stamina,
}

#[derive(Component)]
struct ToolSlot(Slot);
#[derive(Component)]
struct ToolGem(Slot);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (spawn_info, spawn_bars, spawn_toolbar))
        .add_systems(Update, (update_readouts, update_bars, update_toolbar));
}

fn anchored(right: f32, top: Option<f32>, bottom: Option<f32>) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: px(right),
        top: top.map(px).unwrap_or(Val::Auto),
        bottom: bottom.map(px).unwrap_or(Val::Auto),
        ..default()
    }
}

fn spawn_info(mut commands: Commands, kit: Res<UiKit>, fonts: Res<Fonts>) {
    let f = &fonts.body;
    commands.spawn((
        Node { flex_direction: FlexDirection::Column, align_items: AlignItems::End, row_gap: px(4), ..anchored(12.0, Some(12.0), None) },
        children![
            (
                Node { flex_direction: FlexDirection::Column, row_gap: px(2), padding: UiRect::axes(px(26), px(20)), min_width: px(230), ..default() },
                children![
                    kit.backdrop(Frame::Parchment),
                    (Readout::Area, Text::new(""), text_font(f, 18.0), TextColor(INK)),
                    (Readout::Sanity, Text::new(""), text_font(f, 13.0), TextColor(INK)),
                    (Readout::Corruption, Text::new(""), text_font(f, 13.0), TextColor(INK_SOFT)),
                    (Readout::Vessel, Text::new(""), text_font(f, 13.0), TextColor(INK_PURPLE)),
                ],
            ),
            (
                Node { padding: UiRect::axes(px(22), px(14)), min_width: px(150), justify_content: JustifyContent::End, ..default() },
                children![kit.backdrop(Frame::Plank), (Readout::Gold, Text::new("0"), text_font(f, 18.0), TextColor(GOLD_TEXT))],
            ),
        ],
    ));
}

fn bar(kit: &UiKit, font: &Handle<Font>, glyph: &str, fill: BarFill) -> impl Bundle {
    (
        Node { width: px(42), height: px(BAR_H), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::axes(px(12), px(10)), row_gap: px(4), ..default() },
        children![
            kit.backdrop(Frame::Plank),
            (Text::new(glyph), text_font(font, 13.0), TextColor(CREAM)),
            (
                Node { width: percent(100), flex_grow: 1.0, flex_direction: FlexDirection::ColumnReverse, ..default() },
                BackgroundColor(Color::srgb(0.23, 0.11, 0.04)),
                children![(fill, Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.4, 0.8, 0.2)))],
            ),
        ],
    )
}

fn spawn_bars(mut commands: Commands, kit: Res<UiKit>, fonts: Res<Fonts>) {
    commands.spawn((
        Node { column_gap: px(6), align_items: AlignItems::End, ..anchored(12.0, None, Some(12.0)) },
        children![bar(&kit, &fonts.body, "力", BarFill::Stamina), bar(&kit, &fonts.body, "血", BarFill::Health)],
    ));
}

fn spawn_toolbar(mut commands: Commands, kit: Res<UiKit>, fonts: Res<Fonts>) {
    let strip = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(10),
                left: percent(50),
                margin: UiRect::left(px(-201)),
                padding: px(15).all(),
                column_gap: px(4),
                ..default()
            },
            children![kit.backdrop(Frame::Plank)],
        ))
        .id();
    for (slot, name) in TOOLBAR {
        let cell = commands
            .spawn((
                ToolSlot(slot),
                Node { width: px(58), height: px(58), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(3), ..default() },
                children![
                    kit.backdrop(Frame::Slot),
                    (ToolGem(slot), Node { width: px(16), height: px(16), ..default() }, BackgroundColor(Color::NONE)),
                    (Text::new(name), text_font(&fonts.body, 10.0), TextColor(INK_SOFT)),
                ],
            ))
            .id();
        commands.entity(strip).add_child(cell);
    }
}

fn sanity_color(tier: SanityTier) -> Color {
    match tier {
        SanityTier::Stable => INK,
        SanityTier::Shaken => Color::srgb(0.6, 0.42, 0.08),
        SanityTier::Fractured => Color::srgb(0.75, 0.3, 0.08),
        SanityTier::Collapsing => INK_RED,
    }
}

/// "12,345" — thousands separators like Stardew's money box.
fn group_digits(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn update_readouts(run: Res<RunRes>, title: Res<AreaTitle>, mut q: Query<(&Readout, &mut Text, &mut TextColor)>) {
    if !run.is_changed() && !title.is_changed() {
        return;
    }
    let w = &run.world;
    for (r, mut text, mut color) in &mut q {
        text.0 = match r {
            Readout::Area => title.0.to_string(),
            Readout::Sanity => {
                let tier = SanityTier::of(w.sanity);
                color.0 = sanity_color(tier);
                format!("理智 {} · {}", w.sanity, tier.name())
            }
            Readout::Corruption => format!("污染 {} · {}", w.corruption, CorruptionTier::of(w.corruption).name()),
            Readout::Vessel => format!("容器 · {}", vessel_stage_name(w.vessel_awakening)),
            Readout::Gold => format!("{} 金", group_digits(w.gold)),
        };
    }
}

/// Stardew's energy bar shifts green → yellow → red as it drains.
fn level_color(frac: f32) -> Color {
    if frac > 0.5 {
        Color::srgb(0.36 + (1.0 - frac) * 0.9, 0.78, 0.2)
    } else {
        Color::srgb(0.85, 0.3 + frac * 1.0, 0.15)
    }
}

fn update_bars(player: Single<(&Vitals, &Motion), With<Player>>, mut fills: Query<(&BarFill, &mut Node, &mut BackgroundColor)>) {
    let (v, m) = *player;
    for (kind, mut node, mut bg) in &mut fills {
        let frac = match kind {
            BarFill::Health => v.hp / v.max_hp,
            BarFill::Stamina => m.stamina / m.max_stamina,
        }
        .clamp(0.0, 1.0);
        node.height = percent(frac * 100.0);
        bg.0 = match kind {
            BarFill::Health => Color::srgb(0.82, 0.16, 0.14),
            BarFill::Stamina => level_color(frac),
        };
    }
}

pub fn quality_color(q: Quality) -> Color {
    match q {
        Quality::Broken => Color::srgb(0.55, 0.52, 0.46),
        Quality::Common => Color::srgb(0.93, 0.9, 0.82),
        Quality::Rare => Color::srgb(0.3, 0.5, 0.9),
        Quality::Corrupted => Color::srgb(0.58, 0.25, 0.7),
        Quality::Relic => Color::srgb(0.9, 0.62, 0.15),
        Quality::Mythic => Color::srgb(0.95, 0.3, 0.3),
    }
}

/// A gem in the quality colour marks what's equipped in each slot.
fn update_toolbar(run: Res<RunRes>, mut gems: Query<(&ToolGem, &mut BackgroundColor)>) {
    if !run.is_changed() {
        return;
    }
    for (gem, mut bg) in &mut gems {
        bg.0 = run
            .equipped_items()
            .find(|i| i.slot == gem.0)
            .map(|i| quality_color(i.quality))
            .unwrap_or(Color::NONE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(group_digits(7), "7");
        assert_eq!(group_digits(1234), "1,234");
        assert_eq!(group_digits(1234567), "1,234,567");
    }
}
