extends TestSuite

const ChoiceScene := preload("res://scenes/ui/Choice.tscn")

func _pick(option_id: String) -> void:
	var panel := ChoiceScene.instantiate()
	add_child(panel)
	panel._on_option_picked(option_id)
	panel.queue_free()

func test_choice_after_totem_does_not_lower_corruption() -> void:
	GameState.world_state.corruption = 9  # totem already touched (+4)
	GameState.world_state.sanity = 74
	_pick("save")
	assert_true(GameState.world_state.corruption >= 9, "corruption never drops from a choice")

func test_harsher_choices_cost_more() -> void:
	_pick("save")
	var save_c: int = GameState.world_state.corruption
	var save_s: int = GameState.world_state.sanity
	GameState._reset_to_defaults()
	_pick("kill")
	assert_true(GameState.world_state.corruption > save_c, "kill corrupts more")
	assert_true(GameState.world_state.sanity < save_s, "kill costs more sanity")
