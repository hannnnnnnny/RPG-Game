## Affix definitions and rolling. Shared by loot drops and the forge's
## reroll so both draw from exactly the same pool and value curve.
class_name AffixTable
extends RefCounted

const POOL := [
	{"label": "近战伤害", "category": "attack"},
	{"label": "暴击率", "category": "attack"},
	{"label": "对感染者伤害", "category": "attack"},
	{"label": "最大生命", "category": "defense"},
	{"label": "黑潮抗性", "category": "defense"},
	{"label": "翻滚后伤害", "category": "mobility"},
	{"label": "体力回复", "category": "mobility"},
	{"label": "禁忌法术伤害", "category": "forbidden"},
	{"label": "理智稳定", "category": "vessel"},
	{"label": "金币掉落", "category": "economy"},
]

static func random_id(prefix: String, rng: RandomNumberGenerator) -> String:
	return "%s_%x_%x" % [prefix, Time.get_ticks_usec(), rng.randi()]

## One affix scaled to item_power (40%–110% of it, never below 2).
static func roll(item_power: int, rng: RandomNumberGenerator) -> Dictionary:
	var template: Dictionary = POOL[rng.randi() % POOL.size()]
	var value: int = maxi(2, int(item_power * (0.4 + rng.randf() * 0.7)))
	return {
		"id": random_id("affix", rng),
		"label": template.label,
		"category": template.category,
		"value": value,
	}
