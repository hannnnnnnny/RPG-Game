extends TestSuite

func test_sanity_tier_boundaries() -> void:
	assert_eq(MindState.sanity_tier(100).id, "stable")
	assert_eq(MindState.sanity_tier(80).id, "stable")
	assert_eq(MindState.sanity_tier(79).id, "shaken")
	assert_eq(MindState.sanity_tier(20).id, "fractured")
	assert_eq(MindState.sanity_tier(19).id, "collapsing")
	assert_eq(MindState.sanity_tier(0).id, "collapsing")

func test_corruption_tier_boundaries() -> void:
	assert_eq(MindState.corruption_tier(25).name, "清醒之身")
	assert_eq(MindState.corruption_tier(26).name, "染潮")
	assert_eq(MindState.corruption_tier(56).name, "黑脉")
	assert_eq(MindState.corruption_tier(81).name, "近神之壳")

func test_vessel_stage_names_clamp() -> void:
	assert_eq(MindState.vessel_stage_name(1), "低语入梦")
	assert_eq(MindState.vessel_stage_name(99), "终局容器")

func test_guard_softens_losses_only() -> void:
	assert_eq(MindState.soften_loss(-10, 0), -10, "no guard")
	assert_eq(MindState.soften_loss(-10, 100), -5, "halved")
	assert_eq(MindState.soften_loss(8, 100), 8, "gains untouched")
	assert_eq(MindState.soften_loss(-1, 1000), -1, "a loss is never erased")
