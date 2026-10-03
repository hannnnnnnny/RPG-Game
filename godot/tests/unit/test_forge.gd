extends TestSuite

var rng := RandomNumberGenerator.new()

func before_each() -> void:
	rng.seed = 1

func _item(level := 0, quality := "common") -> Dictionary:
	return {
		"id": "i1", "name": "测试", "slot": "mainHand", "quality": quality,
		"item_power": 20, "upgrade_level": level, "reroll_count": 0,
		"affixes": [
			{"id": "a", "label": "近战伤害", "category": "attack", "value": 10},
			{"id": "b", "label": "最大生命", "category": "defense", "value": 12},
		],
	}

func test_upgrade_increments_level_without_touching_affixes() -> void:
	var src := _item(3)
	var out := Forge.upgraded(src)
	assert_eq(out.upgrade_level, 4, "level")
	assert_eq(out.affixes, src.affixes, "affixes unchanged")
	assert_eq(src.upgrade_level, 3, "source not mutated")

func test_upgrade_stops_at_max() -> void:
	var maxed := _item(Forge.MAX_UPGRADE)
	assert_false(Forge.can_upgrade(maxed), "cannot upgrade past max")
	assert_eq(Forge.upgraded(maxed).upgrade_level, Forge.MAX_UPGRADE, "stays at max")

func test_upgrade_cost_rises_with_level_and_quality() -> void:
	assert_true(Forge.upgrade_cost(_item(5)) > Forge.upgrade_cost(_item(0)), "level")
	assert_true(Forge.upgrade_cost(_item(0, "relic")) > Forge.upgrade_cost(_item(0)), "quality")

func test_reroll_accept_replaces_only_that_affix() -> void:
	var src := _item()
	var offer := Forge.reroll_offer(src, rng)
	var out := Forge.resolve_reroll(src, 1, offer, true)
	assert_eq(out.affixes[1], offer, "replaced")
	assert_eq(out.affixes[0], src.affixes[0], "other affix kept")
	assert_eq(out.reroll_count, 1, "counted")

func test_reroll_decline_keeps_affix_but_still_counts() -> void:
	var src := _item()
	var out := Forge.resolve_reroll(src, 0, Forge.reroll_offer(src, rng), false)
	assert_eq(out.affixes, src.affixes, "unchanged")
	assert_eq(out.reroll_count, 1, "counted")

func test_locked_affix_cannot_be_rerolled() -> void:
	var src := Forge.locked(_item(), 0)
	assert_false(Forge.can_reroll(src, 0), "locked")
	assert_true(Forge.can_reroll(src, 1), "other still rerollable")
	var out := Forge.resolve_reroll(src, 0, Forge.reroll_offer(src, rng), true)
	assert_eq(out.affixes[0], src.affixes[0], "locked affix survives accept")

func test_reroll_cost_grows_and_lock_surcharges() -> void:
	var fresh := _item()
	var used := fresh.duplicate(true)
	used.reroll_count = 4
	assert_true(Forge.reroll_cost(used) > Forge.reroll_cost(fresh) * 5, "steep growth")
	assert_true(Forge.reroll_cost(Forge.locked(fresh, 0)) > Forge.reroll_cost(fresh), "lock surcharge")

func test_salvage_yield_by_quality() -> void:
	assert_eq(Forge.salvage_yield(_item(0, "broken")), {Forge.MAT_COMMON: 1})
	var rare := Forge.salvage_yield(_item(0, "rare"))
	assert_eq(rare.get(Forge.MAT_RARE, 0), 1, "rare mat")

func test_salvage_refunds_upgrades() -> void:
	var y := Forge.salvage_yield(_item(6, "common"))
	assert_eq(y[Forge.MAT_COMMON], 2 + 3, "base 2 + half of +6")

func test_salvage_table_is_not_mutated() -> void:
	Forge.salvage_yield(_item(8, "common"))
	assert_eq(Forge.SALVAGE_TABLE.common, {Forge.MAT_COMMON: 2})
