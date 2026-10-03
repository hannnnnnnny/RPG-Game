//! Crafting rules (design §13.5): upgrade, single-affix reroll, lock, salvage.
//! Functions return new values; the caller pays and stores them.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::item::{Affix, Item, Quality};
use crate::loot::roll_affix;

pub const MAX_UPGRADE: u8 = 10;
pub const LOCK_COST_MULT: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Material {
    Common,
    Rare,
    CorruptResidue,
}

impl Material {
    pub const ALL: [Material; 3] = [Material::Common, Material::Rare, Material::CorruptResidue];

    pub fn label(self) -> &'static str {
        match self {
            Material::Common => "普通材料",
            Material::Rare => "稀有材料",
            Material::CorruptResidue => "污染残渣",
        }
    }
}

/// Material id → count. BTreeMap keeps UI and saves in a stable order.
pub type Materials = BTreeMap<Material, u32>;

/// Locking an affix consumes one rare material (锁定需要高级材料).
pub fn lock_cost() -> Materials {
    Materials::from([(Material::Rare, 1)])
}

fn quality_cost(q: Quality) -> f32 {
    match q {
        Quality::Broken => 0.6,
        Quality::Common => 1.0,
        Quality::Rare => 1.8,
        Quality::Corrupted => 2.2,
        Quality::Relic => 3.5,
        Quality::Mythic => 5.0,
    }
}

// ---------------- Upgrade ----------------

pub fn can_upgrade(item: &Item) -> bool {
    item.upgrade_level < MAX_UPGRADE
}

/// Gold for the next +1; super-linear so +10 is a real sink.
pub fn upgrade_cost(item: &Item) -> u32 {
    let next = f32::from(item.upgrade_level + 1);
    (15.0 * quality_cost(item.quality) * next.powf(1.4)).round() as u32
}

/// Never fails; never touches affixes (stats scale them per level).
pub fn upgraded(item: &Item) -> Item {
    let mut out = item.clone();
    if can_upgrade(item) {
        out.upgrade_level += 1;
    }
    out
}

// ---------------- Reroll + lock ----------------

pub fn is_locked(item: &Item, index: usize) -> bool {
    item.affixes.get(index).is_some_and(|a| Some(a.id) == item.locked_affix)
}

pub fn can_reroll(item: &Item, index: usize) -> bool {
    index < item.affixes.len() && !is_locked(item, index)
}

/// Each reroll on the same item costs more; a lock surcharges every reroll.
pub fn reroll_cost(item: &Item) -> u32 {
    let n = item.reroll_count as f32;
    let mut cost = 25.0 * quality_cost(item.quality) * (1.0 + n).powf(1.6);
    if item.locked_affix.is_some() {
        cost *= LOCK_COST_MULT;
    }
    cost.round() as u32
}

pub fn reroll_offer(item: &Item, rng: &mut fastrand::Rng) -> Affix {
    roll_affix(item.item_power, rng)
}

/// Resolve an offer: keep the old affix or take the new one. The reroll
/// counts (and was paid for) either way.
pub fn resolve_reroll(item: &Item, index: usize, offer: Affix, accept: bool) -> Item {
    let mut out = item.clone();
    out.reroll_count += 1;
    if accept && can_reroll(item, index) {
        out.affixes[index] = offer;
    }
    out
}

pub fn locked(item: &Item, index: usize) -> Item {
    let mut out = item.clone();
    if let Some(a) = item.affixes.get(index) {
        out.locked_affix = Some(a.id);
    }
    out
}

// ---------------- Salvage ----------------

/// Materials from breaking an item down; half its upgrade levels come back
/// as common materials so upgrading is never a dead end.
pub fn salvage_yield(item: &Item) -> Materials {
    use Material::*;
    let mut out: Materials = match item.quality {
        Quality::Broken => [(Common, 1)].into(),
        Quality::Common => [(Common, 2)].into(),
        Quality::Rare => [(Common, 3), (Rare, 1)].into(),
        Quality::Corrupted => [(Common, 2), (Rare, 1), (CorruptResidue, 2)].into(),
        Quality::Relic => [(Common, 4), (Rare, 3)].into(),
        Quality::Mythic => [(Common, 6), (Rare, 5)].into(),
    };
    let refund = u32::from(item.upgrade_level / 2);
    if refund > 0 {
        *out.entry(Common).or_default() += refund;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{AffixId, AffixKind, ItemId, Slot};

    fn item(level: u8, quality: Quality) -> Item {
        Item {
            id: ItemId(1),
            name: "锻件".into(),
            slot: Slot::MainHand,
            quality,
            item_power: 20,
            upgrade_level: level,
            reroll_count: 0,
            affixes: vec![
                Affix { id: AffixId(1), kind: AffixKind::MeleeDamage, value: 10 },
                Affix { id: AffixId(2), kind: AffixKind::MaxHealth, value: 12 },
            ],
            locked_affix: None,
            core_effect: None,
        }
    }

    #[test]
    fn upgrade_increments_and_caps() {
        let src = item(3, Quality::Common);
        let out = upgraded(&src);
        assert_eq!(out.upgrade_level, 4);
        assert_eq!(out.affixes, src.affixes);
        assert_eq!(upgraded(&item(MAX_UPGRADE, Quality::Common)).upgrade_level, MAX_UPGRADE);
    }

    #[test]
    fn upgrade_cost_rises_with_level_and_quality() {
        assert!(upgrade_cost(&item(5, Quality::Common)) > upgrade_cost(&item(0, Quality::Common)));
        assert!(upgrade_cost(&item(0, Quality::Relic)) > upgrade_cost(&item(0, Quality::Common)));
    }

    #[test]
    fn reroll_accept_and_decline() {
        let src = item(0, Quality::Common);
        let offer = reroll_offer(&src, &mut fastrand::Rng::with_seed(1));
        let taken = resolve_reroll(&src, 1, offer.clone(), true);
        assert_eq!(taken.affixes[1], offer);
        assert_eq!(taken.affixes[0], src.affixes[0]);
        let kept = resolve_reroll(&src, 1, offer, false);
        assert_eq!(kept.affixes, src.affixes);
        assert_eq!(kept.reroll_count, 1);
    }

    #[test]
    fn locked_affix_survives_reroll_and_surcharges() {
        let src = locked(&item(0, Quality::Common), 0);
        assert!(!can_reroll(&src, 0));
        assert!(can_reroll(&src, 1));
        let offer = reroll_offer(&src, &mut fastrand::Rng::with_seed(2));
        assert_eq!(resolve_reroll(&src, 0, offer, true).affixes[0], src.affixes[0]);
        assert!(reroll_cost(&src) > reroll_cost(&item(0, Quality::Common)));
    }

    #[test]
    fn reroll_cost_grows_steeply() {
        let fresh = item(0, Quality::Common);
        let mut used = fresh.clone();
        used.reroll_count = 4;
        assert!(reroll_cost(&used) > reroll_cost(&fresh) * 5);
    }

    #[test]
    fn salvage_by_quality_with_upgrade_refund() {
        assert_eq!(salvage_yield(&item(0, Quality::Broken)), Materials::from([(Material::Common, 1)]));
        assert_eq!(salvage_yield(&item(0, Quality::Rare))[&Material::Rare], 1);
        assert_eq!(salvage_yield(&item(6, Quality::Common))[&Material::Common], 5);
    }
}
