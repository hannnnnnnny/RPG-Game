//! Drop generation and item pricing. Shared affix rolling means drops and
//! the forge's reroll draw from exactly the same pool and value curve.

use crate::item::{Affix, AffixId, AffixKind, Item, ItemId, Quality, Slot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropSource {
    Enemy,
    Elite,
    Totem,
}

fn base_names(slot: Slot) -> &'static [&'static str] {
    match slot {
        Slot::MainHand => &["矿工断刃", "灰灯短剑", "黑潮凿斧", "锈蚀战镐", "残铁长矛"],
        Slot::OffHand => &["破裂法器", "矿灯提盏", "裂纹符盘"],
        Slot::Head => &["污灰兜帽", "锈铁矿盔", "破檐斗笠"],
        Slot::Chest => &["矿井皮甲", "盐石胸甲", "黑潮锁链衣", "残布外披"],
        Slot::Hands => &["裂岩手套", "旧皮手套", "锈钉护手"],
        Slot::Boots => &["逃亡者旧靴", "灰泥长靴", "钉底矿靴", "破布裹足"],
        Slot::Amulet => &["残响项链", "矿骨吊坠"],
        Slot::Ring => &["黑腕戒指", "灰灯铜戒", "锈铁指环"],
        Slot::Totem => &["图腾碎屑", "矿井护符", "封印残片"],
    }
}

fn pick<T: Copy>(rng: &mut fastrand::Rng, items: &[T]) -> T {
    items[rng.usize(..items.len())]
}

/// One affix scaled to item power: 40%–110% of it, never below 2.
pub fn roll_affix(item_power: i32, rng: &mut fastrand::Rng) -> Affix {
    let value = ((item_power as f32 * (0.4 + rng.f32() * 0.7)) as i32).max(2);
    Affix { id: AffixId(rng.u64(..)), kind: pick(rng, &AffixKind::ALL), value }
}

fn roll_quality(source: DropSource, rng: &mut fastrand::Rng) -> Quality {
    let roll = rng.f32();
    match source {
        DropSource::Totem if roll > 0.55 => Quality::Rare,
        DropSource::Elite if roll > 0.45 => Quality::Rare,
        DropSource::Totem | DropSource::Elite => Quality::Common,
        DropSource::Enemy if roll > 0.86 => Quality::Rare,
        DropSource::Enemy if roll > 0.45 => Quality::Common,
        DropSource::Enemy => Quality::Broken,
    }
}

/// Quality prefix + (rare+) flavour word from the strongest affix + base.
fn build_name(slot: Slot, quality: Quality, affixes: &[Affix], rng: &mut fastrand::Rng) -> String {
    let base = pick(rng, base_names(slot));
    let flavor = match affixes.iter().max_by_key(|a| a.value) {
        Some(best) if quality >= Quality::Rare => format!("{}之", best.kind.category().flavor()),
        _ => String::new(),
    };
    format!("{}{}{}", quality.name_prefix(), flavor, base)
}

pub fn generate(source: DropSource, world_tier: u8, rng: &mut fastrand::Rng) -> Item {
    let slot = pick(rng, &Slot::DROPPABLE);
    let quality = roll_quality(source, rng);
    let totem_bonus = if source == DropSource::Totem { 5 } else { 0 };
    let item_power = i32::from(world_tier) * 10 + rng.i32(0..8) + totem_bonus;
    let affixes: Vec<Affix> =
        (0..quality.affix_count()).map(|_| roll_affix(item_power, rng)).collect();
    Item {
        id: ItemId(rng.u64(..)),
        name: build_name(slot, quality, &affixes, rng),
        slot,
        quality,
        item_power,
        upgrade_level: 0,
        reroll_count: 0,
        affixes,
        locked_affix: None,
        core_effect: (source == DropSource::Totem)
            .then(|| "触碰图腾后，容器觉醒经验小幅提高。".to_string()),
    }
}

fn quality_price_mult(q: Quality) -> f32 {
    match q {
        Quality::Broken => 0.6,
        Quality::Common => 1.4,
        Quality::Rare => 3.0,
        Quality::Corrupted => 4.0,
        Quality::Relic => 7.0,
        Quality::Mythic => 12.0,
    }
}

/// Shop buy price; selling returns 40% of it.
pub fn item_value(item: &Item) -> u32 {
    let affix_value: f32 = item.affixes.iter().map(|a| a.value as f32 * 1.5).sum();
    let v = item.item_power as f32 * quality_price_mult(item.quality) + affix_value;
    (v.round() as u32).max(1)
}

pub fn sell_value(item: &Item) -> u32 {
    ((item_value(item) as f32 * 0.4) as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affix_count_matches_quality() {
        let mut rng = fastrand::Rng::with_seed(11);
        for _ in 0..200 {
            let item = generate(DropSource::Elite, 2, &mut rng);
            assert_eq!(item.affixes.len(), item.quality.affix_count());
        }
    }

    #[test]
    fn affix_values_stay_in_curve() {
        let mut rng = fastrand::Rng::with_seed(5);
        for _ in 0..500 {
            let a = roll_affix(20, &mut rng);
            assert!((2..=22).contains(&a.value), "{}", a.value);
        }
    }

    #[test]
    fn totem_drops_have_core_effect_and_no_broken() {
        let mut rng = fastrand::Rng::with_seed(9);
        for _ in 0..100 {
            let item = generate(DropSource::Totem, 1, &mut rng);
            assert!(item.core_effect.is_some());
            assert!(item.quality >= Quality::Common);
        }
    }

    #[test]
    fn sell_below_buy() {
        let mut rng = fastrand::Rng::with_seed(2);
        let item = generate(DropSource::Enemy, 1, &mut rng);
        assert!(sell_value(&item) < item_value(&item));
    }

    #[test]
    fn same_seed_same_drop() {
        let a = generate(DropSource::Enemy, 1, &mut fastrand::Rng::with_seed(42));
        let b = generate(DropSource::Enemy, 1, &mut fastrand::Rng::with_seed(42));
        assert_eq!(a, b);
    }
}
