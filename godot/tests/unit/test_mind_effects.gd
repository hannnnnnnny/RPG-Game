extends TestSuite

var crossed: Array = []

func before_each() -> void:
	crossed = []
	if not GameState.mind_tier_changed.is_connected(_on_crossed):
		GameState.mind_tier_changed.connect(_on_crossed)

func _on_crossed(meter: String, tier: Dictionary) -> void:
	crossed.push_back([meter, tier.id])

func test_delta_effect_is_relative_and_clamped() -> void:
	GameState.world_state.corruption = 96
	GameState._apply_effect({"path": "corruption", "delta": 10})
	assert_eq(GameState.world_state.corruption, 100, "clamped")
	GameState._apply_effect({"path": "corruption", "delta": -200})
	assert_eq(GameState.world_state.corruption, 0, "floored")

func test_absolute_effect_still_works() -> void:
	GameState._apply_effect({"path": "vessel_awakening", "value": 2})
	assert_eq(GameState.world_state.vessel_awakening, 2)

func test_sanity_loss_softened_by_guard() -> void:
	GameState.inventory = [{"id": "g", "slot": "amulet",
		"affixes": [{"label": "理智稳定", "category": "vessel", "value": 100}]}]
	GameState.equipped = {"amulet": "g"}
	GameState.world_state.sanity = 70
	GameState.change_meter("sanity", -10)
	assert_eq(GameState.world_state.sanity, 65, "halved loss")

func test_tier_crossing_emits_once() -> void:
	GameState.world_state.sanity = 82
	GameState.change_meter("sanity", -1)
	assert_eq(crossed, [], "still stable")
	GameState.change_meter("sanity", -5)
	assert_eq(crossed, [["sanity", "shaken"]], "crossed into 动摇")
