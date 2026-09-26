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

func test_materials_add_and_spend() -> void:
	GameState.add_materials({"common_mat": 3, "rare_mat": 1})
	assert_true(GameState.spend_materials({"common_mat": 2}), "affordable")
	assert_eq(GameState.materials.common_mat, 1, "deducted")

func test_spend_materials_is_all_or_nothing() -> void:
	GameState.add_materials({"common_mat": 5})
	assert_false(GameState.spend_materials({"common_mat": 1, "rare_mat": 1}), "missing rare")
	assert_eq(GameState.materials.common_mat, 5, "nothing deducted")

func test_player_down_loses_gold_but_keeps_gear() -> void:
	GameState.world_state.gold = 300
	GameState.inventory = [_weapon(5)]
	GameState.equipped = {Types.SLOT_MAIN_HAND: "w1"}
	assert_eq(GameState.on_player_down(), 30, "lost")
	assert_eq(GameState.world_state.gold, 270, "gold")
	assert_eq(GameState.inventory.size(), 1, "gear kept")
	assert_eq(GameState.equipped.get(Types.SLOT_MAIN_HAND), "w1", "still equipped")
