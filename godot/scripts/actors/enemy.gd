## 腐化矮人敌人 —— 看到玩家追上去打
class_name Enemy
extends CharacterBody2D

@export var hp: float = 20.0          # hidden HP — no bar; multi-hit feedback conveys it
@export var detect_range: float = 280.0
@export var attack_range: float = 34.0
@export var attack_cooldown: float = 0.85
@export var move_speed: float = 62.0
@export var contact_damage: float = 8.0

const KNOCKBACK_FORCE := 190.0
const KNOCKBACK_DECAY := 620.0

var attack_timer: float = 0.0
var anim_time: float = 0.0
var anim_frame: int = 0
var player_ref: Player = null
var knockback: Vector2 = Vector2.ZERO
var _dying: bool = false
var aggro: bool = false
var wander_dir: Vector2 = Vector2.ZERO
var wander_timer: float = 0.0

func _ready() -> void:
	add_to_group("enemy")
	_find_player()

func _find_player() -> void:
	var players := get_tree().get_nodes_in_group("player")
	if players.size() > 0:
		player_ref = players[0] as Player

func _physics_process(delta: float) -> void:
	if player_ref == null:
		_find_player()
		return

	attack_timer = max(0.0, attack_timer - delta)
	anim_time += delta
	if anim_time > 0.5:
		anim_time = 0.0
		anim_frame = 1 - anim_frame
		queue_redraw()

	var to_player := player_ref.global_position - global_position
	var dist := to_player.length()
	var move := Vector2.ZERO
	if dist < detect_range:
		# Spotted the player — telegraph the first time, then pursue.
		if not aggro:
			aggro = true
			_telegraph()
		move = to_player.normalized() * move_speed
	else:
		# Out of range: drift idly so the room feels alive instead of frozen.
		if aggro:
			aggro = false
		_wander(delta)
		move = wander_dir * (move_speed * 0.35)
	# Knockback decays toward zero; it adds onto movement so a hit visibly
	# shoves the enemy back even while it's pursuing.
	knockback = knockback.move_toward(Vector2.ZERO, KNOCKBACK_DECAY * delta)
	velocity = move + knockback
	move_and_slide()

	if dist < attack_range and attack_timer <= 0.0:
		attack_timer = attack_cooldown
		player_ref.take_damage(contact_damage)

func _wander(delta: float) -> void:
	wander_timer -= delta
	if wander_timer <= 0.0:
		wander_timer = randf_range(1.2, 2.6)
		if randf() < 0.45:
			wander_dir = Vector2.ZERO  # pause and loiter
		else:
			wander_dir = Vector2(randf_range(-1.0, 1.0), randf_range(-1.0, 1.0)).normalized()

func _telegraph() -> void:
	# A quick brighten + pop the instant the dwarf notices you — readable "!".
	modulate = Color(2.0, 1.3, 2.4)
	scale = Vector2(1.18, 1.18)
	var tw := create_tween()
	tw.set_parallel(true)
	tw.tween_property(self, "modulate", Color.WHITE, 0.28)
	tw.tween_property(self, "scale", Vector2.ONE, 0.28).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)

func take_damage(amount: float, from_pos: Vector2 = global_position) -> void:
	if _dying:
		return
	hp -= amount
	# Shove away from the hit origin.
	var dir := (global_position - from_pos)
	if dir.length() < 0.01:
		dir = Vector2.RIGHT
	knockback = dir.normalized() * KNOCKBACK_FORCE
	Audio.play_hit()
	_flash_hit()
	if hp <= 0.0:
		_die()

func _flash_hit() -> void:
	# Overbright white flash + squash punch — reads clearly even at range
	# in the dark purple scene.
	modulate = Color(2.6, 2.6, 2.8)
	scale = Vector2(1.2, 0.84)
	var tw := create_tween().set_parallel(true)
	tw.tween_property(self, "modulate", Color.WHITE, 0.14)
	tw.tween_property(self, "scale", Vector2.ONE, 0.14).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)

func _die() -> void:
	if _dying:
		return
	_dying = true
	set_physics_process(false)
	var world_tier: int = GameState.world_state.world_tier
	GameState.add_gold(LootGenerator.gold_for_kill("enemy", world_tier))
	if randf() > 0.36:
		GameState.add_item(LootGenerator.generate_loot("enemy", world_tier))
	# Quick death pop before removal.
	var tw := create_tween().set_parallel(true)
	tw.tween_property(self, "modulate:a", 0.0, 0.18)
	tw.tween_property(self, "scale", Vector2(0.5, 0.5), 0.18)
	await tw.finished
	queue_free()

# === Placeholder pixel art: hooded corrupted dwarf ===
func _draw() -> void:
	var bob := -2 if anim_frame == 1 else 0
	var hood := Color8(24, 5, 32)
	var beard := Color8(61, 27, 80)
	var skin := Color8(42, 16, 55)
	var glow := Color8(201, 122, 255)
	var armor := Color8(36, 19, 51)
	var armor_lit := Color8(110, 45, 160)
	var leg := Color8(21, 8, 32)

	# Head + hood
	draw_rect(Rect2(-9, -50 + bob, 18, 4), hood, true)
	draw_rect(Rect2(-6, -54 + bob, 12, 4), hood, true)
	# Face (purple)
	draw_rect(Rect2(-9, -46 + bob, 18, 10), skin, true)
	# Beard
	draw_rect(Rect2(-7, -36 + bob, 14, 4), beard, true)
	draw_rect(Rect2(-5, -32 + bob, 10, 2), beard, true)
	# Glowing eyes
	draw_rect(Rect2(-5, -42 + bob, 2, 2), glow, true)
	draw_rect(Rect2(3, -42 + bob, 2, 2), glow, true)
	# Body / chestplate
	draw_rect(Rect2(-9, -30 + bob, 18, 14), armor, true)
	draw_rect(Rect2(-7, -28 + bob, 14, 2), armor_lit, true)
	draw_rect(Rect2(-1, -26 + bob, 2, 8), armor_lit, true)
	# Legs
	draw_rect(Rect2(-9, -16, 8, 16), leg, true)
	draw_rect(Rect2(1, -16, 8, 16), leg, true)
