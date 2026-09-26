## Derives the player's combat stats from everything equipped.
## Pure static functions (no scene/autoload access) so it is unit-testable
## and can be reused by UI previews ("what if I equip this?").
class_name StatCalc
extends RefCounted

const BASE_ATTACK := 8
const BASE_MAX_HEALTH := 100
const BASE_STAMINA_REGEN := 20.0
const CRIT_CAP := 50.0
const DAMAGE_REDUCTION_CAP := 0.6
# Each +1 upgrade adds 10% to every affix on the item (design §13.5:
# upgrades raise base power but never change which affixes exist).
const UPGRADE_STEP := 0.1

# Affix label -> stat key. Labels are what older saves stored, so mapping
# from them keeps existing items working without a save migration.
const LABEL_TO_STAT := {
	"近战伤害": "attack",
	"对感染者伤害": "attack",  # every current enemy is infected
	"暴击率": "crit",
	"最大生命": "max_health",
	"黑潮抗性": "resist",
	"翻滚后伤害": "roll_bonus",
	"体力回复": "stamina_regen",
	"禁忌法术伤害": "spell",
	"理智稳定": "sanity_guard",
	"金币掉落": "gold_find",
}

static func equipped_items(inventory: Array, equipped: Dictionary) -> Array:
	var ids: Array = equipped.values()
	return inventory.filter(func(it: Dictionary) -> bool: return it.get("id") in ids)

static func upgrade_multiplier(item: Dictionary) -> float:
	return 1.0 + UPGRADE_STEP * int(item.get("upgrade_level", 0))

## Raw summed affix values per stat key across the given items.
static func totals(items: Array) -> Dictionary:
	var out := {}
	for item in items:
		var mult := upgrade_multiplier(item)
		for affix in item.get("affixes", []):
			var key: String = LABEL_TO_STAT.get(affix.get("label", ""), "")
			if key == "":
				continue
			out[key] = out.get(key, 0.0) + float(affix.get("value", 0)) * mult
	return out

static func derive(inventory: Array, equipped: Dictionary) -> Dictionary:
	var t := totals(equipped_items(inventory, equipped))
	return {
		"attack": BASE_ATTACK + int(round(t.get("attack", 0.0))),
		"max_health": BASE_MAX_HEALTH + int(round(t.get("max_health", 0.0))),
		"crit_chance": minf(CRIT_CAP, t.get("crit", 0.0)) / 100.0,
		"damage_reduction": minf(DAMAGE_REDUCTION_CAP, t.get("resist", 0.0) / 100.0),
		"stamina_regen": BASE_STAMINA_REGEN + t.get("stamina_regen", 0.0) * 0.5,
		"roll_bonus": t.get("roll_bonus", 0.0) / 100.0,
		"spell_power": int(round(t.get("spell", 0.0))),
		"sanity_guard": int(round(t.get("sanity_guard", 0.0))),
		"gold_find": t.get("gold_find", 0.0) / 100.0,
	}
