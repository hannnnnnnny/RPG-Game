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

# ---------------- Reroll + lock ----------------

const LOCK_COST_MULT := 1.5

static func is_locked(item: Dictionary, affix_index: int) -> bool:
	var affixes: Array = item.get("affixes", [])
	if affix_index < 0 or affix_index >= affixes.size():
		return false
	return affixes[affix_index].get("id") == item.get("locked_affix_id", "")

static func can_reroll(item: Dictionary, affix_index: int) -> bool:
	var count: int = item.get("affixes", []).size()
	return affix_index >= 0 and affix_index < count and not is_locked(item, affix_index)

## Each reroll on the same item costs more; a lock makes every reroll pricier.
static func reroll_cost(item: Dictionary) -> int:
	var n := int(item.get("reroll_count", 0))
	var cost := 25.0 * _quality_cost(item) * pow(1.0 + n, 1.6)
	if item.get("locked_affix_id", "") != "":
		cost *= LOCK_COST_MULT
	return int(round(cost))

## A fresh affix offer for one slot. The player then keeps or accepts it.
static func reroll_offer(item: Dictionary, rng: RandomNumberGenerator) -> Dictionary:
	return AffixTable.roll(int(item.get("item_power", 10)), rng)

## Resolves an offer. The reroll counts (and was paid) either way.
static func resolve_reroll(item: Dictionary, affix_index: int, offer: Dictionary, accept: bool) -> Dictionary:
	var out := item.duplicate(true)
	out["reroll_count"] = int(item.get("reroll_count", 0)) + 1
	if accept and can_reroll(item, affix_index):
		out.affixes[affix_index] = offer.duplicate(true)
	return out

static func locked(item: Dictionary, affix_index: int) -> Dictionary:
	var out := item.duplicate(true)
	var affixes: Array = item.get("affixes", [])
	if affix_index >= 0 and affix_index < affixes.size():
		out["locked_affix_id"] = affixes[affix_index].get("id", "")
	return out

# ---------------- Salvage ----------------

const MAT_COMMON := "common_mat"
const MAT_RARE := "rare_mat"
const MAT_RESIDUE := "corrupt_residue"

const MATERIAL_NAMES := {
	MAT_COMMON: "普通材料",
	MAT_RARE: "稀有材料",
	MAT_RESIDUE: "污染残渣",
}

## Locking an affix consumes this (design: 锁定需要高级材料).
const LOCK_MATERIAL := {MAT_RARE: 1}

const SALVAGE_TABLE := {
	"broken": {MAT_COMMON: 1},
	"common": {MAT_COMMON: 2},
	"rare": {MAT_COMMON: 3, MAT_RARE: 1},
	"corrupted": {MAT_COMMON: 2, MAT_RARE: 1, MAT_RESIDUE: 2},
	"relic": {MAT_COMMON: 4, MAT_RARE: 3},
	"mythic": {MAT_COMMON: 6, MAT_RARE: 5},
}

## Materials returned for breaking an item down. Half of its upgrade
## levels come back as common materials so upgrading isn't a dead end.
static func salvage_yield(item: Dictionary) -> Dictionary:
	var out: Dictionary = SALVAGE_TABLE.get(item.get("quality", "common"), {}).duplicate()
	var refund := int(item.get("upgrade_level", 0)) / 2
	if refund > 0:
		out[MAT_COMMON] = int(out.get(MAT_COMMON, 0)) + refund
	return out
