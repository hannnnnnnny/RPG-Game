## 黑腕队长·格罗姆 —— 第一个 Boss。
## 触碰图腾后由矮人队长腐化形态现身，挡在出口前。
## 状态机：chase → 蓄力(冲锋/砸地) → 攻击 → 恢复。半血进入二阶段（更快 + 召唤）。
class_name Boss
extends CharacterBody2D

signal health_changed(current: float, maxhp: float)
signal died()
signal summon_requested(at: Vector2)

@export var max_hp: float = 160.0
@export var move_speed: float = 56.0
@export var lunge_damage: float = 14.0
@export var slam_damage: float = 18.0
@export var slam_radius: float = 130.0

const LUNGE_SPEED := 380.0

var hp: float
var player_ref: Player
var phase: int = 1
var state: String = "chase"
var state_t: float = 0.0
var attack_cd: float = 1.2
var lunge_dir: Vector2 = Vector2.ZERO
var anim_t: float = 0.0
var anim_frame: int = 0
var _dead: bool = false

func _ready() -> void:
	add_to_group("boss")
	hp = max_hp
	_find_player()
	emit_signal("health_changed", hp, max_hp)

func _find_player() -> void:
	var players := get_tree().get_nodes_in_group("player")
	if players.size() > 0:
		player_ref = players[0] as Player

func _physics_process(delta: float) -> void:
	if _dead:
		return
	if player_ref == null:
		_find_player()
		return

	anim_t += delta
	if anim_t > 0.35:
		anim_t = 0.0
		anim_frame = 1 - anim_frame
		queue_redraw()

	state_t -= delta
	match state:
		"chase": _state_chase(delta)
		"windup_lunge": _state_windup_lunge()
		"lunge": _state_lunge()
		"windup_slam": _state_windup_slam()
		"recover": _state_recover()
	move_and_slide()

func _enter(s: String, t: float) -> void:
	state = s
	state_t = t
	velocity = Vector2.ZERO

func _state_chase(delta: float) -> void:
	attack_cd -= delta
	var to_player := player_ref.global_position - global_position
	var dist := to_player.length()
	var spd := move_speed * (1.45 if phase == 2 else 1.0)
	velocity = to_player.normalized() * spd
	_maybe_phase2()
	if attack_cd <= 0.0:
		if dist < 80.0:
			_enter("windup_slam", 0.55)
		else:
			_enter("windup_lunge", 0.5)

func _state_windup_lunge() -> void:
	# Telegraph: stutter-flash red so the player can react.
	modulate = Color(2.4, 1.0, 1.0) if (int(state_t * 14) % 2 == 0) else Color.WHITE
	if state_t <= 0.0:
		modulate = Color.WHITE
		lunge_dir = (player_ref.global_position - global_position).normalized()
		_enter("lunge", 0.3)

func _state_lunge() -> void:
	velocity = lunge_dir * LUNGE_SPEED
	if global_position.distance_to(player_ref.global_position) < 44.0:
		player_ref.take_damage(lunge_damage)
	if state_t <= 0.0:
		attack_cd = 1.5 if phase == 2 else 1.9
		_enter("recover", 0.45)

func _state_windup_slam() -> void:
	modulate = Color(2.4, 1.4, 1.0) if (int(state_t * 14) % 2 == 0) else Color.WHITE
	if state_t <= 0.0:
		modulate = Color.WHITE
		_do_slam()
		attack_cd = 1.6 if phase == 2 else 2.1
		_enter("recover", 0.6)

func _do_slam() -> void:
	if global_position.distance_to(player_ref.global_position) < slam_radius:
		player_ref.take_damage(slam_damage)
	# Spawn an expanding shockwave ring at the boss.
	var wave := Node2D.new()
	wave.global_position = global_position
	wave.set_script(load("res://scripts/world/shockwave.gd"))
	wave.set("max_radius", slam_radius)
	get_parent().add_child(wave)

func _state_recover() -> void:
	velocity = Vector2.ZERO
	if state_t <= 0.0:
		_enter("chase", 0.0)

func _maybe_phase2() -> void:
	if phase == 1 and hp <= max_hp * 0.5:
		phase = 2
		# Brighten with corruption and call for reinforcements once.
		modulate = Color(1.6, 0.9, 1.8)
		var tw := create_tween()
		tw.tween_property(self, "modulate", Color.WHITE, 0.5)
		emit_signal("summon_requested", global_position)

func take_damage(amount: float, from_pos: Vector2 = global_position) -> void:
	if _dead:
		return
	hp = max(0.0, hp - amount)
	emit_signal("health_changed", hp, max_hp)
	_flash_hit()
	if hp <= 0.0:
		_die()

func _flash_hit() -> void:
	modulate = Color(2.6, 2.6, 2.8)
	var tw := create_tween()
	tw.tween_property(self, "modulate", Color.WHITE, 0.12)

func _die() -> void:
	if _dead:
		return
	_dead = true
	set_physics_process(false)
	emit_signal("died")
	var tw := create_tween()
	tw.set_parallel(true)
	tw.tween_property(self, "modulate:a", 0.0, 0.9)
	tw.tween_property(self, "scale", Vector2(1.5, 1.5), 0.9)
	await tw.finished
	queue_free()

# ---------- Procedural pixel art: crowned black-arm captain ----------
func _draw() -> void:
	var bob := -2 if anim_frame == 1 else 0
	var crown := Color8(150, 110, 44)
	var hood := Color8(28, 8, 38)
	var skin := Color8(58, 22, 74)
	var beard := Color8(74, 34, 96)
	var glow := Color8(216, 130, 255)
	var armor := Color8(44, 24, 62)
	var armor_lit := Color8(120, 52, 168)
	var black_arm := Color8(10, 5, 14)
	var leg := Color8(24, 10, 36)
	var sc := 2.2  # bigger than a regular dwarf

	var f := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		draw_rect(Rect2(x * sc, (y + bob) * sc, w * sc, h * sc), c, true)
	var fg := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		draw_rect(Rect2(x * sc, y * sc, w * sc, h * sc), c, true)

	# Crown
	f.call(-9, -34, 18, 3, crown)
	f.call(-9, -37, 2, 3, crown)
	f.call(-1, -38, 2, 4, crown)
	f.call(7, -37, 2, 3, crown)
	# Hood / head
	f.call(-9, -31, 18, 4, hood)
	f.call(-9, -27, 18, 10, skin)
	# Beard
	f.call(-7, -17, 14, 4, beard)
	f.call(-5, -13, 10, 2, beard)
	# Glowing eyes
	f.call(-5, -23, 3, 2, glow)
	f.call(2, -23, 3, 2, glow)
	# Chestplate
	f.call(-10, -11, 20, 11, armor)
	f.call(-8, -9, 16, 2, armor_lit)
	f.call(-1, -7, 2, 6, armor_lit)
	# Black corrupted arm (left)
	f.call(-13, -10, 3, 11, black_arm)
	f.call(-13, 1, 3, 4, black_arm)
	# Right arm
	f.call(10, -10, 3, 9, armor)
	# Legs (don't bob)
	fg.call(-9, -1, 8, 10, leg)
	fg.call(1, -1, 8, 10, leg)
