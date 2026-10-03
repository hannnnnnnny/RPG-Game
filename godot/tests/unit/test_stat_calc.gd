extends TestSuite

func _item(id: String, slot: String, label: String, value: int, upgrade := 0) -> Dictionary:
	return {
		"id": id, "slot": slot, "upgrade_level": upgrade,
		"affixes": [{"label": label, "category": "x", "value": value}],
	}

func test_empty_loadout_is_base_stats() -> void:
	var s := StatCalc.derive([], {})
	assert_eq(s.attack, StatCalc.BASE_ATTACK, "attack")
	assert_eq(s.max_health, StatCalc.BASE_MAX_HEALTH, "hp")
	assert_eq(s.damage_reduction, 0.0, "dr")

func test_every_equipped_slot_contributes() -> void:
	var inv := [
		_item("w", "mainHand", "近战伤害", 5),
		_item("r", "ring", "近战伤害", 3),
		_item("c", "chest", "最大生命", 20),
	]
	var s := StatCalc.derive(inv, {"mainHand": "w", "ring": "r", "chest": "c"})
	assert_eq(s.attack, StatCalc.BASE_ATTACK + 8, "attack")
	assert_eq(s.max_health, StatCalc.BASE_MAX_HEALTH + 20, "hp")

func test_unequipped_items_are_ignored() -> void:
	var inv := [_item("w", "mainHand", "近战伤害", 50)]
	assert_eq(StatCalc.derive(inv, {}).attack, StatCalc.BASE_ATTACK)

func test_upgrade_level_scales_affixes() -> void:
	var inv := [_item("w", "mainHand", "近战伤害", 10, 5)]
	assert_eq(StatCalc.derive(inv, {"mainHand": "w"}).attack, StatCalc.BASE_ATTACK + 15)

func test_damage_reduction_is_capped() -> void:
	var inv := [_item("c", "chest", "黑潮抗性", 500)]
	assert_eq(StatCalc.derive(inv, {"chest": "c"}).damage_reduction, StatCalc.DAMAGE_REDUCTION_CAP)
