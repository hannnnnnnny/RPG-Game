extends TestSuite

func test_generated_item_has_required_fields() -> void:
	var item: Dictionary = LootGenerator.generate_loot("enemy", 1)
	for key in ["id", "name", "slot", "quality", "item_power", "affixes"]:
		assert_true(item.has(key), "missing field %s" % key)

func test_affix_count_matches_quality() -> void:
	for i in range(30):
		var item: Dictionary = LootGenerator.generate_loot("elite", 2)
		var expected: int = LootGenerator._affix_count(item.quality)
		assert_eq(item.affixes.size(), expected, "affix count")

func test_sell_value_below_buy_value() -> void:
	var item: Dictionary = LootGenerator.generate_loot("enemy", 1)
	assert_true(LootGenerator.sell_value(item) < LootGenerator.item_value(item))
