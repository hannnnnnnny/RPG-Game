## Outgoing-damage rolls. Pure + RNG-injected so tests are deterministic.
class_name CombatMath
extends RefCounted

const CRIT_MULTIPLIER := 1.75
# 翻滚后伤害 applies to hits landed this soon after a dodge roll ends.
const ROLL_BONUS_WINDOW := 1.0

## Returns {"amount": int, "crit": bool} for one hit on one target.
static func roll_hit(stats: Dictionary, rng: RandomNumberGenerator, since_roll: float) -> Dictionary:
	var amount := float(stats.get("attack", StatCalc.BASE_ATTACK))
	if since_roll <= ROLL_BONUS_WINDOW:
		amount *= 1.0 + float(stats.get("roll_bonus", 0.0))
	var crit := rng.randf() < float(stats.get("crit_chance", 0.0))
	if crit:
		amount *= CRIT_MULTIPLIER
	return {"amount": maxi(1, int(round(amount))), "crit": crit}

## Gold after the loadout's 金币掉落 bonus.
static func apply_gold_find(base_gold: int, stats: Dictionary) -> int:
	return int(round(base_gold * (1.0 + float(stats.get("gold_find", 0.0)))))
