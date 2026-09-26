extends TestSuite

const PanelScene := preload("res://scenes/ui/ForgePanel.tscn")

var panel: Control

func before_each() -> void:
	GameState._reset_to_defaults()
	GameState.inventory = [{
		"id": "p1", "name": "测试锤", "slot": "mainHand", "quality": "rare",
		"item_power": 20, "upgrade_level": 0, "reroll_count": 0,
		"affixes": [{"id": "a", "label": "近战伤害", "category": "attack", "value": 10}],
	}]
	GameState.world_state.gold = 5000
	if panel:
		panel.queue_free()
	panel = PanelScene.instantiate()
	add_child(panel)

func test_open_selects_first_item() -> void:
	panel.open()
	assert_true(panel.visible, "visible")
	assert_eq(panel._selected_id, "p1", "auto-selected")

func test_upgrade_button_flow() -> void:
	panel.open()
	panel._upgrade()
	assert_eq(GameState.find_item("p1").upgrade_level, 1, "upgraded")

func test_pending_reroll_blocks_close_until_resolved() -> void:
	panel.open()
	panel._reroll(0)
	panel.close()
	assert_true(panel.visible, "cannot close with a paid offer pending")
	panel._resolve(true)
	panel.close()
	assert_false(panel.visible, "closes once resolved")

func test_salvage_requires_confirmation() -> void:
	panel.open()
	panel._ask_salvage()
	assert_false(GameState.find_item("p1").is_empty(), "not yet salvaged")
	panel._salvage()
	assert_true(GameState.find_item("p1").is_empty(), "salvaged after confirm")
