extends TestSuite

func _cleanup() -> void:
	SaveSystem.delete_save()

func test_roundtrip_restores_run_state() -> void:
	GameState.profile = {"name": "测试者"}
	GameState.world_state.gold = 321
	GameState.add_materials({"rare_mat": 2})
	GameState.inventory = [{"id": "x", "name": "剑", "slot": "mainHand", "affixes": []}]
	GameState.equipped = {"mainHand": "x"}
	SaveSystem.save()
	GameState._reset_to_defaults()
	assert_true(SaveSystem.load_save(), "loaded")
	assert_eq(GameState.profile.name, "测试者", "profile")
	assert_eq(GameState.world_state.gold, 321, "gold")
	assert_eq(GameState.materials.rare_mat, 2, "materials")
	assert_eq(GameState.equipped.mainHand, "x", "equipped")
	_cleanup()

func test_runner_sandboxes_the_save_path() -> void:
	assert_true(SaveSystem.save_path != SaveSystem.SAVE_PATH, "never the real save")
