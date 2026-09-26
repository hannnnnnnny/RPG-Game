extends TestSuite

func _give_item(quality := "common") -> Dictionary:
	var item := {
		"id": "f1", "name": "锻件", "slot": "mainHand", "quality": quality,
		"item_power": 20, "upgrade_level": 0, "reroll_count": 0,
		"affixes": [{"id": "a", "label": "近战伤害", "category": "attack", "value": 10}],
	}
	GameState.inventory = [item]
	return item

func test_upgrade_spends_gold_and_raises_level() -> void:
	var item := _give_item()
	var cost := Forge.upgrade_cost(item)
	GameState.world_state.gold = cost + 5
	assert_true(GameState.forge_upgrade("f1"), "upgraded")
	assert_eq(GameState.find_item("f1").upgrade_level, 1, "level")
	assert_eq(GameState.world_state.gold, 5, "gold spent")

func test_upgrade_refused_when_broke() -> void:
	_give_item()
	GameState.world_state.gold = 0
	assert_false(GameState.forge_upgrade("f1"), "refused")
	assert_eq(GameState.find_item("f1").upgrade_level, 0, "unchanged")

func test_upgrading_equipped_item_updates_attack() -> void:
	_give_item()
	GameState.equipped = {"mainHand": "f1"}
	GameState.world_state.gold = 9999
	var before := GameState.get_attack_power()
	for i in range(5):
		GameState.forge_upgrade("f1")
	assert_eq(GameState.get_attack_power(), before + 5, "+50% of a 10 affix")

func test_reroll_offer_then_accept() -> void:
	_give_item()
	GameState.world_state.gold = 9999
	var offer := GameState.forge_reroll_offer("f1", 0)
	assert_false(offer.is_empty(), "offer made")
	GameState.forge_resolve_reroll("f1", 0, offer, true)
	assert_eq(GameState.find_item("f1").affixes[0], offer, "taken")

func test_lock_needs_rare_material() -> void:
	_give_item()
	assert_false(GameState.forge_lock("f1", 0), "no material")
	GameState.add_materials({Forge.MAT_RARE: 1})
	assert_true(GameState.forge_lock("f1", 0), "locked")
	assert_eq(GameState.materials[Forge.MAT_RARE], 0, "consumed")

func test_salvage_removes_item_and_grants_materials() -> void:
	_give_item("rare")
	GameState.salvage_item("f1")
	assert_true(GameState.find_item("f1").is_empty(), "removed")
	assert_eq(GameState.materials.get(Forge.MAT_RARE, 0), 1, "materials")
