## 被救矮人同伴 —— 选择"救他"后起身跟随玩家。
## 简单跟随：离玩家太远就追上去（远时加速，能跟上奔跑），近了就停下待机。
class_name Follower
extends CharacterBody2D

@export var follow_gap: float = 58.0
@export var max_speed: float = 230.0

var player_ref: Player
var anim_t: float = 0.0
var anim_frame: int = 0
var face_left: bool = false

func _ready() -> void:
	add_to_group("follower")
	_find_player()

func _find_player() -> void:
	var players := get_tree().get_nodes_in_group("player")
	if players.size() > 0:
		player_ref = players[0] as Player

func _physics_process(delta: float) -> void:
	if player_ref == null:
		_find_player()
		return

	anim_t += delta
	if anim_t > 0.22:
		anim_t = 0.0
		anim_frame = 1 - anim_frame
		queue_redraw()

	var to_player := player_ref.global_position - global_position
	var dist := to_player.length()
	if dist > follow_gap:
		var spd: float = clampf((dist - follow_gap) * 5.0, 0.0, max_speed)
		velocity = to_player.normalized() * spd
		if abs(to_player.x) > 4.0:
			face_left = to_player.x < 0.0
	else:
		velocity = Vector2.ZERO
	move_and_slide()

# ---------- Procedural pixel art: upright rescued dwarf ----------
func _draw() -> void:
	var bob := -1 if anim_frame == 1 else 0
	var skin := Color8(219, 179, 138)
	var skin_sh := Color8(180, 131, 86)
	var beard := Color8(150, 96, 54)
	var tunic := Color8(94, 60, 40)
	var tunic_sh := Color8(64, 40, 26)
	var bandage := Color8(216, 201, 160)
	var boot := Color8(40, 26, 16)
	var eye := Color8(26, 14, 8)
	var sc := 2.0
	var flip := face_left

	var f := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		var px := (-(x + w) if flip else x)
		draw_rect(Rect2(px * sc, (y + bob) * sc, w * sc, h * sc), c, true)
	var fg := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		var px := (-(x + w) if flip else x)
		draw_rect(Rect2(px * sc, y * sc, w * sc, h * sc), c, true)

	# Head
	f.call(-4, -20, 8, 6, skin)
	f.call(3, -19, 1, 4, skin_sh)
	# Bandage across brow
	f.call(-4, -20, 8, 2, bandage)
	# Eyes
	f.call(-2, -16, 1, 1, eye)
	f.call(1, -16, 1, 1, eye)
	# Beard
	f.call(-4, -14, 8, 4, beard)
	f.call(-3, -10, 6, 2, beard)
	# Tunic body
	f.call(-5, -10, 10, 8, tunic)
	f.call(3, -9, 1, 6, tunic_sh)
	# Belt
	f.call(-5, -3, 10, 1, tunic_sh)
	# Legs / boots (don't bob)
	fg.call(-5, -2, 4, 4, boot)
	fg.call(1, -2, 4, 4, boot)
