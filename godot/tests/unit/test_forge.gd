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
