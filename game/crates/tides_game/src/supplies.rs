//! Drinking supplies: Q for 愈伤药水, R for 灰灯宁神茶. The run spends the
//! bottle and applies sanity; health is healed here on the player's Vitals.

use bevy::prelude::*;
use tides_core::shop::{Supply, SupplyEffect};

use crate::beats::no_modal;
use crate::combat::{Vitals, float_text};
use crate::player::Player;
use crate::run_state::RunRes;
use crate::ui::{Fonts, text_font};

const KEYS: [(KeyCode, Supply); 2] = [(KeyCode::KeyQ, Supply::HealingDraught), (KeyCode::KeyR, Supply::CalmingTea)];

pub fn plugin(app: &mut App) {
    app.add_systems(Update, drink.run_if(no_modal));
}

fn drink(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut run: ResMut<RunRes>,
    player: Single<(&Transform, &mut Vitals), With<Player>>,
    fonts: Res<Fonts>,
) {
    let Some(supply) = KEYS.iter().find(|(k, _)| keys.just_pressed(*k)).map(|(_, s)| *s) else { return };
    let (tf, mut vitals) = player.into_inner();
    let at = tf.translation.truncate() + Vec2::Y * 64.0;
    let (text, color) = match run.use_supply(supply) {
        Ok(SupplyEffect::Heal(hp)) => {
            vitals.hp = (vitals.hp + hp).min(vitals.max_hp);
            (format!("+{hp}"), Color::srgb(0.5, 0.95, 0.5))
        }
        Ok(SupplyEffect::Sanity(n)) => (format!("理智 +{n}"), Color::srgb(0.75, 0.7, 1.0)),
        Err(e) => (e.label().to_string(), Color::srgb(0.9, 0.8, 0.7)),
    };
    float_text(&mut commands, at, text, text_font(&fonts.body, 16.0), color);
}
