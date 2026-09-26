extends TestSuite

var rng := RandomNumberGenerator.new()

func before_each() -> void:
	rng.seed = 42

func test_plain_hit_equals_attack() -> void:
	var hit := CombatMath.roll_hit({"attack": 10, "crit_chance": 0.0}, rng, 99.0)
	assert_eq(hit.amount, 10, "amount")
	assert_false(hit.crit, "no crit at 0%")

func test_guaranteed_crit_multiplies() -> void:
	var hit := CombatMath.roll_hit({"attack": 10, "crit_chance": 1.0}, rng, 99.0)
	assert_true(hit.crit, "crit at 100%")
	assert_eq(hit.amount, int(round(10 * CombatMath.CRIT_MULTIPLIER)), "amount")

func test_roll_bonus_only_inside_window() -> void:
	var stats := {"attack": 10, "roll_bonus": 0.5}
	assert_eq(CombatMath.roll_hit(stats, rng, 0.2).amount, 15, "inside window")
	assert_eq(CombatMath.roll_hit(stats, rng, 3.0).amount, 10, "outside window")

func test_crit_rate_roughly_matches_chance() -> void:
	var crits := 0
	for i in range(2000):
		if CombatMath.roll_hit({"attack": 10, "crit_chance": 0.25}, rng, 99.0).crit:
			crits += 1
	assert_between(crits / 2000.0, 0.21, 0.29, "crit rate")

func test_gold_find_scales_gold() -> void:
	assert_eq(CombatMath.apply_gold_find(100, {"gold_find": 0.3}), 130)
	assert_eq(CombatMath.apply_gold_find(100, {}), 100)
