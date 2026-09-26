extends TestSuite

func before_each() -> void:
	GameState._reset_to_defaults()

func _weapon(attack: int) -> Dictionary:
	return {
		"id": "w1", "name": "测试剑", "slot": Types.SLOT_MAIN_HAND,
		"quality": Types.QUALITY_COMMON, "item_power": 10,
		"affixes": [{"id": "a1", "label": "近战伤害", "category": "attack", "value": attack}]
	}

func test_unarmed_attack_is_base() -> void:
	assert_eq(GameState.get_attack_power(), GameState.BASE_ATTACK)

func test_equipped_weapon_adds_attack_affixes() -> void:
	GameState.inventory = [_weapon(5)]
	GameState.equipped = {Types.SLOT_MAIN_HAND: "w1"}
	assert_eq(GameState.get_attack_power(), GameState.BASE_ATTACK + 5)

func test_kill_gold_is_boosted_by_gold_find() -> void:
	seed(7)
	GameState.add_kill_gold("enemy")
	var plain: int = GameState.world_state.gold
	GameState._reset_to_defaults()
	GameState.inventory = [{
		"id": "r1", "slot": Types.SLOT_RING,
		"affixes": [{"label": "金币掉落", "category": "economy", "value": 100}],
	}]
	GameState.equipped = {Types.SLOT_RING: "r1"}
	seed(7)
	GameState.add_kill_gold("enemy")
	assert_eq(GameState.world_state.gold, plain * 2, "100% gold find doubles gold")
