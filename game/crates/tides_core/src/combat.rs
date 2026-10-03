//! Hit resolution and combat economy. RNG is injected so tests are exact.

use crate::stats::Stats;

pub const CRIT_MULTIPLIER: f32 = 1.75;
/// 翻滚后伤害 applies to hits landed this many seconds after a roll ends.
pub const ROLL_BONUS_WINDOW: f32 = 1.0;
/// Design §12: death is light — lose a slice of gold, never equipment.
pub const DEATH_GOLD_LOSS: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub amount: i32,
    pub crit: bool,
}

pub fn roll_hit(stats: &Stats, rng: &mut fastrand::Rng, since_roll: f32) -> Hit {
    let mut amount = stats.attack as f32;
    if since_roll <= ROLL_BONUS_WINDOW {
        amount *= 1.0 + stats.roll_bonus;
    }
    let crit = rng.f32() < stats.crit_chance;
    if crit {
        amount *= CRIT_MULTIPLIER;
    }
    Hit { amount: (amount.round() as i32).max(1), crit }
}

/// Damage the player actually takes after 黑潮抗性.
pub fn mitigate(incoming: f32, stats: &Stats) -> f32 {
    incoming * (1.0 - stats.damage_reduction)
}

pub fn apply_gold_find(base_gold: u32, stats: &Stats) -> u32 {
    (base_gold as f32 * (1.0 + stats.gold_find)).round() as u32
}

pub fn death_gold_loss(gold: u32) -> u32 {
    (gold as f32 * DEATH_GOLD_LOSS).floor() as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillSource {
    Enemy,
    Elite,
    Boss,
}

/// Base gold for a kill before gold find; scales with world tier.
pub fn gold_for_kill(source: KillSource, world_tier: u8, rng: &mut fastrand::Rng) -> u32 {
    let base = match source {
        KillSource::Boss => 70,
        KillSource::Elite => 24,
        KillSource::Enemy => 8,
    };
    base * u32::from(world_tier) + rng.u32(0..base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(attack: i32, crit: f32, roll_bonus: f32) -> Stats {
        Stats { attack, crit_chance: crit, roll_bonus, ..Stats::default() }
    }

    #[test]
    fn plain_hit_equals_attack() {
        let mut rng = fastrand::Rng::with_seed(1);
        assert_eq!(roll_hit(&stats(10, 0.0, 0.0), &mut rng, 99.0), Hit { amount: 10, crit: false });
    }

    #[test]
    fn guaranteed_crit_multiplies() {
        let mut rng = fastrand::Rng::with_seed(1);
        let hit = roll_hit(&stats(10, 1.0, 0.0), &mut rng, 99.0);
        assert!(hit.crit);
        assert_eq!(hit.amount, 18);
    }

    #[test]
    fn roll_bonus_only_inside_window() {
        let mut rng = fastrand::Rng::with_seed(1);
        let s = stats(10, 0.0, 0.5);
        assert_eq!(roll_hit(&s, &mut rng, 0.2).amount, 15);
        assert_eq!(roll_hit(&s, &mut rng, 3.0).amount, 10);
    }

    #[test]
    fn crit_rate_matches_chance() {
        let mut rng = fastrand::Rng::with_seed(7);
        let s = stats(10, 0.25, 0.0);
        let crits = (0..4000).filter(|_| roll_hit(&s, &mut rng, 99.0).crit).count();
        let rate = crits as f32 / 4000.0;
        assert!((0.22..0.28).contains(&rate), "rate {rate}");
    }

    #[test]
    fn economy_rules() {
        let s = Stats { gold_find: 0.3, ..Stats::default() };
        assert_eq!(apply_gold_find(100, &s), 130);
        assert_eq!(death_gold_loss(250), 25);
        assert_eq!(death_gold_loss(5), 0);
    }

    #[test]
    fn resist_mitigates() {
        let s = Stats { damage_reduction: 0.25, ..Stats::default() };
        assert_eq!(mitigate(20.0, &s), 15.0);
    }

    #[test]
    fn boss_pays_more_than_enemy() {
        let mut rng = fastrand::Rng::with_seed(3);
        let enemy = gold_for_kill(KillSource::Enemy, 1, &mut rng);
        let boss = gold_for_kill(KillSource::Boss, 1, &mut rng);
        assert!(boss > enemy);
    }
}
