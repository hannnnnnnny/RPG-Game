## 玩家角色 disi —— 程序化烘焙的 chibi sprite sheet + AnimatedSprite2D 动画
##
## 渲染管线已是 AnimatedSprite2D。换成真实手绘/AI sprite 只需：
##   1. 把一张 sprite sheet PNG 拖进 res://assets/sprites/disi/
##   2. 把 _ready 里的 `_bake_player_sheet()` 换成 `load("res://assets/sprites/disi/xxx.png")`
##   3. 保证帧布局是 9 帧（down 0-2 / up 3-5 / side 6-8，每组 neutral/左踏/右踏）
##      —— 或调整 _build_frames 的索引
## 其它逻辑（移动/翻滚/攻击/动画选择）都不用动。

class_name Player
extends CharacterBody2D

const WALK_SPEED := 140.0
const ROLL_SPEED := 240.0
const ROLL_DURATION := 0.26
const ATTACK_COOLDOWN := 0.42

# Sprite sheet geometry
const PFW := 18  # frame width  (source px)
const PFH := 32  # frame height (source px)

enum Facing { DOWN, LEFT, UP, RIGHT }

# disi palette
const C_OUT := Color8(12, 7, 16)
const C_HAIR := Color8(34, 20, 12)
const C_HAIR_HI := Color8(64, 40, 24)
const C_SKIN := Color8(231, 196, 161)
const C_SKIN_SH := Color8(174, 132, 102)
const C_EYE := Color8(22, 12, 10)
const C_SHIRT := Color8(78, 42, 108)
const C_SHIRT_HI := Color8(110, 64, 146)
const C_SHIRT_SH := Color8(48, 24, 70)
const C_PANTS := Color8(32, 26, 40)
const C_PANTS_SH := Color8(18, 13, 24)
const C_BOOT := Color8(64, 40, 24)
const C_BOOT_SH := Color8(40, 24, 14)
const C_BELT := Color8(150, 110, 44)

@export var stamina: float = 100.0
@export var health: float = 100.0
@export var max_stamina: float = 100.0
@export var max_health: float = 100.0

var facing: Facing = Facing.DOWN
var roll_timer: float = 0.0
var attack_timer: float = 0.0
var moving: bool = false

@onready var sprite: AnimatedSprite2D = $Sprite

signal attack_performed(facing_dir: int, position: Vector2)

func _ready() -> void:
	add_to_group("player")
	var sheet := _bake_player_sheet()
	sprite.sprite_frames = _build_frames(sheet)
	sprite.play("idle_down")

func _physics_process(delta: float) -> void:
	roll_timer = max(0.0, roll_timer - delta)
	attack_timer = max(0.0, attack_timer - delta)

	var input := Vector2.ZERO
	input.x = Input.get_axis("move_left", "move_right")
	input.y = Input.get_axis("move_up", "move_down")
	moving = input.length_squared() > 0.0

	if moving:
		input = input.normalized()
		if abs(input.x) > abs(input.y):
			facing = Facing.LEFT if input.x < 0 else Facing.RIGHT
		else:
			facing = Facing.UP if input.y < 0 else Facing.DOWN

	var speed := ROLL_SPEED if roll_timer > 0.0 else WALK_SPEED
	velocity = input * speed
	move_and_slide()

	if Input.is_action_just_pressed("roll") and stamina >= 28.0 and roll_timer <= 0.0:
		stamina -= 28.0
		roll_timer = ROLL_DURATION
		modulate = Color(0.6, 0.85, 0.8)
		await get_tree().create_timer(ROLL_DURATION).timeout
		modulate = Color.WHITE
	if roll_timer <= 0.0:
		stamina = min(max_stamina, stamina + delta * 22.0)

	if Input.is_action_just_pressed("attack") and attack_timer <= 0.0:
		attack_timer = ATTACK_COOLDOWN
		emit_signal("attack_performed", facing, global_position)

	_update_animation()

	GameState.set_combat({
		"health": int(round(health)),
		"max_health": int(max_health),
		"stamina": int(round(stamina)),
		"max_stamina": int(max_stamina),
		"focus": 30,
		"max_focus": 30
	})

func _update_animation() -> void:
	var dir := "down"
	match facing:
		Facing.UP:
			dir = "up"
			sprite.flip_h = false
		Facing.LEFT:
			dir = "side"
			sprite.flip_h = true
		Facing.RIGHT:
			dir = "side"
			sprite.flip_h = false
		_:
			dir = "down"
			sprite.flip_h = false
	var anim := ("walk_" if moving else "idle_") + dir
	if sprite.animation != anim:
		sprite.play(anim)

func take_damage(amount: float) -> void:
	health = max(0.0, health - amount)
	modulate = Color(1.4, 0.6, 0.6)
	await get_tree().create_timer(0.08).timeout
	modulate = Color.WHITE
	if health <= 0.0:
		_on_player_down()

func _on_player_down() -> void:
	health = max_health
	global_position = Vector2(155, 165)
	GameState.set_dialogue({
		"speaker": "克哈低语",
		"text": "死亡在这里没有耐心。站起来，再走一次。",
		"tone": Types.TONE_WHISPER
	})

# ============ Procedural sprite sheet bake ============
# 9 frames: down(0-2), up(3-5), side(6-8). Each group = neutral / left-step /
# right-step. Left facing reuses the side frames mirrored via flip_h.

func _bake_player_sheet() -> ImageTexture:
	var img := Image.create(PFW * 9, PFH, false, Image.FORMAT_RGBA8)
	for i in range(3):
		_draw_down(img, i * PFW, i)
	for i in range(3):
		_draw_up(img, (3 + i) * PFW, i)
	for i in range(3):
		_draw_side(img, (6 + i) * PFW, i)
	return ImageTexture.create_from_image(img)

func _build_frames(sheet: Texture2D) -> SpriteFrames:
	var sf := SpriteFrames.new()
	if sf.has_animation("default"):
		sf.remove_animation("default")
	var groups := {"down": 0, "up": 3, "side": 6}
	for d in groups:
		var base: int = groups[d]
		var idle_name: String = "idle_" + str(d)
		sf.add_animation(idle_name)
		sf.set_animation_loop(idle_name, true)
		sf.set_animation_speed(idle_name, 1.0)
		sf.add_frame(idle_name, _atlas(sheet, base))
		var walk_name: String = "walk_" + str(d)
		sf.add_animation(walk_name)
		sf.set_animation_loop(walk_name, true)
		sf.set_animation_speed(walk_name, 8.0)
		sf.add_frame(walk_name, _atlas(sheet, base))      # neutral
		sf.add_frame(walk_name, _atlas(sheet, base + 1))  # left step
		sf.add_frame(walk_name, _atlas(sheet, base))      # neutral
		sf.add_frame(walk_name, _atlas(sheet, base + 2))  # right step
	return sf

func _atlas(sheet: Texture2D, frame_idx: int) -> AtlasTexture:
	var at := AtlasTexture.new()
	at.atlas = sheet
	at.region = Rect2(frame_idx * PFW, 0, PFW, PFH)
	return at

func _pf(img: Image, x: int, y: int, w: int, h: int, col: Color) -> void:
	img.fill_rect(Rect2i(x, y, w, h), col)

func _boots(img: Image, ox: int, step: int) -> void:
	var lo := 0
	var ro := 0
	if step == 1:
		lo = -1
	elif step == 2:
		ro = -1
	_pf(img, ox + 5, 27 + lo, 3, 3, C_BOOT)
	_pf(img, ox + 5, 29 + lo, 3, 1, C_BOOT_SH)
	_pf(img, ox + 10, 27 + ro, 3, 3, C_BOOT)
	_pf(img, ox + 10, 29 + ro, 3, 1, C_BOOT_SH)

func _torso_down_up(img: Image, ox: int, back: bool) -> void:
	_pf(img, ox + 4, 13, 10, 8, C_SHIRT)
	_pf(img, ox + 4, 13, 1, 8, C_OUT)
	_pf(img, ox + 13, 13, 1, 8, C_OUT)
	_pf(img, ox + 5, 14, 1, 6, C_SHIRT_HI)
	_pf(img, ox + 12, 14, 1, 6, C_SHIRT_SH)
	if back:
		_pf(img, ox + 8, 13, 1, 7, C_SHIRT_SH)  # spine seam
	else:
		_pf(img, ox + 8, 13, 2, 2, C_SHIRT_SH)  # V collar
	# belt
	_pf(img, ox + 4, 20, 10, 1, C_BELT)
	# pants
	_pf(img, ox + 5, 21, 8, 6, C_PANTS)
	_pf(img, ox + 8, 21, 2, 6, C_PANTS_SH)

func _draw_down(img: Image, ox: int, step: int) -> void:
	# hair
	_pf(img, ox + 5, 2, 8, 1, C_OUT)
	_pf(img, ox + 4, 3, 1, 9, C_OUT)
	_pf(img, ox + 13, 3, 1, 9, C_OUT)
	_pf(img, ox + 5, 3, 8, 5, C_HAIR)
	_pf(img, ox + 6, 3, 4, 1, C_HAIR_HI)
	# face
	_pf(img, ox + 5, 8, 8, 4, C_SKIN)
	_pf(img, ox + 5, 8, 1, 2, C_HAIR)
	_pf(img, ox + 12, 8, 1, 2, C_HAIR)
	_pf(img, ox + 11, 8, 1, 4, C_SKIN_SH)
	_pf(img, ox + 7, 9, 1, 1, C_EYE)
	_pf(img, ox + 10, 9, 1, 1, C_EYE)
	_pf(img, ox + 8, 11, 2, 1, C_SKIN_SH)
	_pf(img, ox + 5, 12, 8, 1, C_OUT)
	# body
	_torso_down_up(img, ox, false)
	_boots(img, ox, step)

func _draw_up(img: Image, ox: int, step: int) -> void:
	# all hair, no face
	_pf(img, ox + 5, 2, 8, 1, C_OUT)
	_pf(img, ox + 4, 3, 1, 9, C_OUT)
	_pf(img, ox + 13, 3, 1, 9, C_OUT)
	_pf(img, ox + 5, 3, 8, 9, C_HAIR)
	_pf(img, ox + 6, 3, 4, 1, C_HAIR_HI)
	_pf(img, ox + 5, 11, 8, 1, C_OUT)
	# body (back)
	_torso_down_up(img, ox, true)
	_boots(img, ox, step)

func _draw_side(img: Image, ox: int, step: int) -> void:
	# Faces RIGHT (left facing = flip_h). Back of head at left, face at right.
	_pf(img, ox + 4, 2, 8, 1, C_OUT)
	_pf(img, ox + 3, 3, 1, 9, C_OUT)
	_pf(img, ox + 12, 3, 1, 5, C_OUT)
	_pf(img, ox + 4, 3, 8, 5, C_HAIR)
	_pf(img, ox + 4, 8, 3, 3, C_HAIR)         # hair down the back
	_pf(img, ox + 4, 3, 1, 4, C_HAIR_HI)
	# face (right half)
	_pf(img, ox + 7, 8, 5, 4, C_SKIN)
	_pf(img, ox + 10, 9, 1, 1, C_EYE)
	_pf(img, ox + 12, 9, 1, 1, C_SKIN)        # nose tip
	_pf(img, ox + 8, 11, 3, 1, C_SKIN_SH)
	_pf(img, ox + 4, 12, 9, 1, C_OUT)
	# torso (slim profile)
	_pf(img, ox + 5, 13, 8, 8, C_SHIRT)
	_pf(img, ox + 5, 13, 1, 8, C_OUT)
	_pf(img, ox + 12, 13, 1, 8, C_OUT)
	_pf(img, ox + 6, 14, 1, 6, C_SHIRT_HI)
	_pf(img, ox + 11, 14, 1, 6, C_SHIRT_SH)
	# swinging arm (front), offset by step
	var arm := 0
	if step == 1:
		arm = -1
	elif step == 2:
		arm = 1
	_pf(img, ox + 10, 15 + arm, 2, 4, C_SHIRT_SH)
	_pf(img, ox + 10, 19 + arm, 1, 1, C_SKIN)  # hand
	# belt + pants
	_pf(img, ox + 5, 20, 8, 1, C_BELT)
	_pf(img, ox + 5, 21, 8, 6, C_PANTS)
	# stride: front foot / back foot
	var fo := 0
	if step == 1:
		fo = 1
	elif step == 2:
		fo = -1
	_pf(img, ox + 8 + fo, 27, 4, 3, C_BOOT)
	_pf(img, ox + 8 + fo, 29, 4, 1, C_BOOT_SH)
	_pf(img, ox + 4 - fo, 27, 4, 3, C_BOOT_SH)
	_pf(img, ox + 4 - fo, 29, 4, 1, C_OUT)
