## 腐化矮人敌人 —— 看到玩家追上去打
class_name Enemy
extends CharacterBody2D

@export var hp: float = 32.0
@export var detect_range: float = 280.0
@export var attack_range: float = 34.0
@export var attack_cooldown: float = 0.85
@export var move_speed: float = 62.0
@export var contact_damage: float = 8.0

var attack_timer: float = 0.0
var anim_time: float = 0.0
var anim_frame: int = 0
var player_ref: Player = null

func _ready() -> void:
	add_to_group("enemy")
	_find_player()

func _find_player() -> void:
	var players := get_tree().get_nodes_in_group("player")
	if players.size() > 0:
		player_ref = players[0]

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
	if dist < detect_range:
		velocity = to_player.normalized() * move_speed
	else:
		velocity = Vector2.ZERO
	move_and_slide()

	if dist < attack_range and attack_timer <= 0.0:
		attack_timer = attack_cooldown
		player_ref.take_damage(contact_damage)

func take_damage(amount: float) -> void:
	hp -= amount
	modulate = Color(0.9, 0.7, 0.9)
	await get_tree().create_timer(0.08).timeout
	modulate = Color.WHITE
	if hp <= 0.0:
		_die()

func _die() -> void:
	var world_tier: int = GameState.world_state.world_tier
	GameState.add_gold(LootGenerator.gold_for_kill("enemy", world_tier))
	if randf() > 0.36:
		GameState.add_item(LootGenerator.generate_loot("enemy", world_tier))
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
