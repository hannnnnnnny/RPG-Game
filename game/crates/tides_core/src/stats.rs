//! Combat stats derived from the whole loadout. Every equipped slot counts
//! and each upgrade level adds 10% to all of that item's affixes.

use crate::item::{AffixKind, Item};

pub const BASE_ATTACK: i32 = 8;
pub const BASE_MAX_HEALTH: i32 = 100;
pub const BASE_STAMINA_REGEN: f32 = 20.0;
pub const CRIT_CAP: f32 = 0.5;
pub const DAMAGE_REDUCTION_CAP: f32 = 0.6;
/// Design §13.5: upgrades raise power but never change which affixes exist.
pub const UPGRADE_STEP: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub attack: i32,
    pub max_health: i32,
    pub crit_chance: f32,
    pub damage_reduction: f32,
    pub stamina_regen: f32,
    /// Bonus multiplier for hits right after a dodge roll (0.5 = +50%).
    pub roll_bonus: f32,
    pub spell_power: i32,
    pub sanity_guard: i32,
    pub gold_find: f32,
}

impl Default for Stats {
    fn default() -> Self {
        derive([])
    }
}

pub fn upgrade_multiplier(item: &Item) -> f32 {
    1.0 + UPGRADE_STEP * f32::from(item.upgrade_level)
}

/// Summed, upgrade-scaled value of one affix kind across the given items.
fn total<'a>(items: impl IntoIterator<Item = &'a Item> + Clone, kind: AffixKind) -> f32 {
    items
        .into_iter()
        .flat_map(|it| {
            let mult = upgrade_multiplier(it);
            it.affixes.iter().filter(move |a| a.kind == kind).map(move |a| a.value as f32 * mult)
        })
        .sum()
}

/// Derive stats from the *equipped* items only.
pub fn derive<'a>(equipped: impl IntoIterator<Item = &'a Item> + Clone) -> Stats {
    let t = |k| total(equipped.clone(), k);
    // Every current enemy is infected, so 对感染者伤害 counts as attack.
    let attack = t(AffixKind::MeleeDamage) + t(AffixKind::InfectedDamage);
    Stats {
        attack: BASE_ATTACK + attack.round() as i32,
        max_health: BASE_MAX_HEALTH + t(AffixKind::MaxHealth).round() as i32,
        crit_chance: (t(AffixKind::CritRate) / 100.0).min(CRIT_CAP),
        damage_reduction: (t(AffixKind::TideResist) / 100.0).min(DAMAGE_REDUCTION_CAP),
        stamina_regen: BASE_STAMINA_REGEN + t(AffixKind::StaminaRegen) * 0.5,
        roll_bonus: t(AffixKind::PostRollDamage) / 100.0,
        spell_power: t(AffixKind::ForbiddenSpell).round() as i32,
        sanity_guard: t(AffixKind::SanityGuard).round() as i32,
        gold_find: t(AffixKind::GoldFind) / 100.0,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::item::{Affix, AffixId, ItemId, Quality, Slot};

    pub(crate) fn item(slot: Slot, kind: AffixKind, value: i32, upgrade: u8) -> Item {
        Item {
            id: ItemId(value as u64),
            name: "测试".into(),
            slot,
            quality: Quality::Common,
            item_power: 20,
            upgrade_level: upgrade,
            reroll_count: 0,
            affixes: vec![Affix { id: AffixId(1), kind, value }],
            locked_affix: None,
            core_effect: None,
        }
    }

    #[test]
    fn empty_loadout_is_base() {
        let s = Stats::default();
        assert_eq!(s.attack, BASE_ATTACK);
        assert_eq!(s.max_health, BASE_MAX_HEALTH);
        assert_eq!(s.damage_reduction, 0.0);
    }

    #[test]
    fn every_equipped_slot_contributes() {
        let gear = [
            item(Slot::MainHand, AffixKind::MeleeDamage, 5, 0),
            item(Slot::Ring, AffixKind::MeleeDamage, 3, 0),
            item(Slot::Chest, AffixKind::MaxHealth, 20, 0),
        ];
        let s = derive(&gear);
        assert_eq!(s.attack, BASE_ATTACK + 8);
        assert_eq!(s.max_health, BASE_MAX_HEALTH + 20);
    }

    #[test]
    fn upgrades_scale_affixes() {
        let gear = [item(Slot::MainHand, AffixKind::MeleeDamage, 10, 5)];
        assert_eq!(derive(&gear).attack, BASE_ATTACK + 15);
    }

    #[test]
    fn caps_apply() {
        let gear = [
            item(Slot::Chest, AffixKind::TideResist, 500, 0),
            item(Slot::Ring, AffixKind::CritRate, 500, 0),
        ];
        let s = derive(&gear);
        assert_eq!(s.damage_reduction, DAMAGE_REDUCTION_CAP);
        assert_eq!(s.crit_chance, CRIT_CAP);
    }
}
