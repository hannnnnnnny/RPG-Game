## Equipment crafting rules (design §13.5): upgrade, reroll, lock, salvage.
## Pure functions that return *new* item dictionaries; GameState owns
## paying costs and swapping the result into the inventory.
class_name Forge
extends RefCounted

const MAX_UPGRADE := 10

const QUALITY_COST := {
	"broken": 0.6, "common": 1.0, "rare": 1.8,
	"corrupted": 2.2, "relic": 3.5, "mythic": 5.0,
}

static func _quality_cost(item: Dictionary) -> float:
	return QUALITY_COST.get(item.get("quality", "common"), 1.0)

static func can_upgrade(item: Dictionary) -> bool:
	return int(item.get("upgrade_level", 0)) < MAX_UPGRADE

## Gold for the next +1. Grows super-linearly so +10 is a real sink.
static func upgrade_cost(item: Dictionary) -> int:
	var next_level := int(item.get("upgrade_level", 0)) + 1
	return int(round(15.0 * _quality_cost(item) * pow(next_level, 1.4)))

## Upgrading never fails and never touches affixes (StatCalc scales them).
static func upgraded(item: Dictionary) -> Dictionary:
	var out := item.duplicate(true)
	if can_upgrade(item):
		out["upgrade_level"] = int(item.get("upgrade_level", 0)) + 1
	return out
