//! Top-left status panel: 生命 / 体力 bars, 理智 tier, 污染 tier + 觉醒 stage, gold.

use bevy::prelude::*;
use tides_core::mind::{CorruptionTier, SanityTier, vessel_stage_name};

use crate::combat::Vitals;
use crate::player::{Motion, Player};
use crate::run_state::RunRes;
use crate::ui::{Fonts, PANEL_BG, PANEL_BORDER, TEXT, TEXT_MUTED, label};

#[derive(Component)]
enum Bar {
    Health,
    Stamina,
}

#[derive(Component)]
enum Readout {
    Health,
    Stamina,
    Sanity,
    Mind,
    Gold,
}

const BAR_W: f32 = 150.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud).add_systems(Update, (update_bars, update_readouts));
}

fn spawn_hud(mut commands: Commands, fonts: Res<Fonts>) {
    let f = &fonts.body;
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            padding: px(10).all(),
            row_gap: px(3),
            flex_direction: FlexDirection::Column,
            border: px(1).all(),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor::all(PANEL_BORDER),
        children![
            (label(f, "生命", 12.0, TEXT), Readout::Health),
            bar(Bar::Health, Color::srgb(0.72, 0.2, 0.22)),
            (label(f, "体力", 12.0, TEXT), Readout::Stamina),
            bar(Bar::Stamina, Color::srgb(0.36, 0.62, 0.34)),
            (label(f, "理智", 12.0, TEXT), Readout::Sanity),
            (label(f, "污染", 12.0, TEXT_MUTED), Readout::Mind),
            (label(f, "0 金币", 12.0, Color::srgb(0.95, 0.83, 0.45)), Readout::Gold),
        ],
    ));
}

fn bar(kind: Bar, color: Color) -> impl Bundle {
    (
        Node { width: px(BAR_W), height: px(6), border_radius: BorderRadius::all(px(3)), ..default() },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        children![(kind, Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(color))],
    )
}

fn update_bars(player: Single<(&Vitals, &Motion), With<Player>>, mut bars: Query<(&Bar, &mut Node)>) {
    let (v, m) = *player;
    for (bar, mut node) in &mut bars {
        let frac = match bar {
            Bar::Health => v.hp / v.max_hp,
            Bar::Stamina => m.stamina / m.max_stamina,
        };
        node.width = percent(frac.clamp(0.0, 1.0) * 100.0);
    }
}

fn sanity_color(tier: SanityTier) -> Color {
    match tier {
        SanityTier::Stable => Color::srgb(0.78, 0.84, 0.86),
        SanityTier::Shaken => Color::srgb(0.88, 0.78, 0.5),
        SanityTier::Fractured => Color::srgb(0.92, 0.55, 0.36),
        SanityTier::Collapsing => Color::srgb(0.95, 0.35, 0.35),
    }
}

fn update_readouts(
    run: Res<RunRes>,
    player: Single<(&Vitals, &Motion), With<Player>>,
    mut q: Query<(&Readout, &mut Text, &mut TextColor)>,
) {
    let (v, m) = *player;
    let w = &run.world;
    for (readout, mut text, mut color) in &mut q {
        let s = match readout {
            Readout::Health => format!("生命 {:.0}/{:.0}", v.hp, v.max_hp),
            Readout::Stamina => format!("体力 {:.0}/{:.0}", m.stamina, m.max_stamina),
            Readout::Sanity => {
                let tier = SanityTier::of(w.sanity);
                color.0 = sanity_color(tier);
                format!("理智 {} · {}", w.sanity, tier.name())
            }
            Readout::Mind => format!(
                "污染 {} · {} · {}",
                w.corruption,
                CorruptionTier::of(w.corruption).name(),
                vessel_stage_name(w.vessel_awakening)
            ),
            Readout::Gold => format!("{} 金币", w.gold),
        };
        if text.0 != s {
            text.0 = s;
        }
    }
}
