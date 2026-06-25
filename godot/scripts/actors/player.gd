## 玩家角色 disi —— LPC 开源角色（女性体型 + 紫袍 + 兜帽，运行时合成）
## 真 4 方向行走/站立动画 + 奔跑(Shift) + 鼠标瞄准攻击。
##
## 美术来源：Liberated Pixel Cup (LPC)，CC-BY-SA 3.0 / GPL 3.0 / OGA-BY 3.0。
## 见 godot/CREDITS.md。三层在 _ready 用 Image.blend_rect 合成。
class_name Player
extends CharacterBody2D

const WALK_SPEED := 130.0
const SPRINT_SPEED := 210.0
const ROLL_SPEED := 250.0
const ROLL_DURATION := 0.26
const ATTACK_COOLDOWN := 0.34
const STAMINA_SPRINT_DRAIN := 24.0
const STAMINA_REGEN := 20.0

const LPC_FRAME := 64  # source frame size

enum Facing { DOWN, LEFT, UP, RIGHT }

@export var stamina: float = 100.0
@export var health: float = 100.0
@export var max_stamina: float = 100.0
@export var max_health: float = 100.0

var facing: Facing = Facing.DOWN
var roll_timer: float = 0.0
var attack_timer: float = 0.0
var step_timer: float = 0.0
var moving: bool = false
var sprinting: bool = false

@onready var sprite: AnimatedSprite2D = $Sprite

signal attack_performed(aim_angle: float, position: Vector2)

func _ready() -> void:
	add_to_group("player")
	var sheet := _composite_lpc()
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

	# Sprint: hold Shift while moving, drains stamina.
	sprinting = Input.is_action_pressed("sprint") and moving and stamina > 1.0
	var speed := WALK_SPEED
	if roll_timer > 0.0:
		speed = ROLL_SPEED
	elif sprinting:
		speed = SPRINT_SPEED
	velocity = input * speed
	move_and_slide()

	# Stamina: drain while sprinting, otherwise regen (and not mid-roll).
	if sprinting:
		stamina = max(0.0, stamina - STAMINA_SPRINT_DRAIN * delta)
	elif roll_timer <= 0.0:
		stamina = min(max_stamina, stamina + STAMINA_REGEN * delta)

	# Footsteps (quicker cadence when sprinting).
	if moving:
		step_timer -= delta
		if step_timer <= 0.0:
			step_timer = 0.22 if sprinting else 0.34
			Audio.play_step()
	else:
		step_timer = 0.0

	if Input.is_action_just_pressed("roll") and stamina >= 28.0 and roll_timer <= 0.0:
		stamina -= 28.0
		roll_timer = ROLL_DURATION
		modulate = Color(0.6, 0.85, 0.8)
		await get_tree().create_timer(ROLL_DURATION).timeout
		modulate = Color.WHITE

	# Attack toward the mouse cursor — ARPG-style free aim.
	if Input.is_action_just_pressed("attack") and attack_timer <= 0.0:
		attack_timer = ATTACK_COOLDOWN
		var aim := get_global_mouse_position() - global_position
		if aim.length() < 1.0:
			aim = _facing_vector()
		_face_toward(aim)
		Audio.play_swing()
		emit_signal("attack_performed", aim.angle(), global_position)

	_update_animation()

	GameState.set_combat({
		"health": int(round(health)),
		"max_health": int(max_health),
		"stamina": int(round(stamina)),
		"max_stamina": int(max_stamina),
		"focus": 30,
		"max_focus": 30
	})

func _facing_vector() -> Vector2:
	match facing:
		Facing.UP: return Vector2.UP
		Facing.LEFT: return Vector2.LEFT
		Facing.RIGHT: return Vector2.RIGHT
		_: return Vector2.DOWN

func _face_toward(v: Vector2) -> void:
	if abs(v.x) > abs(v.y):
		facing = Facing.LEFT if v.x < 0 else Facing.RIGHT
	else:
		facing = Facing.UP if v.y < 0 else Facing.DOWN

func _update_animation() -> void:
	var dir := "down"
	match facing:
		Facing.UP: dir = "up"
		Facing.LEFT: dir = "left"
		Facing.RIGHT: dir = "right"
		_: dir = "down"
	sprite.speed_scale = 1.5 if sprinting else 1.0
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

# ============ LPC layer composite ============
# Stack body + purple robe + hood into one walk sheet (576x256, 9 frames x 4
# rows). LPC layers align perfectly by design, so a straight alpha blend works.

func _composite_lpc() -> ImageTexture:
	var body := _img("res://assets/sprites/disi/body_walk.png")
	var robe := _img("res://assets/sprites/disi/robe_walk.png")
	var hood := _img("res://assets/sprites/disi/hood_walk.png")
	if body == null:
		return null
	if robe != null:
		body.blend_rect(robe, Rect2i(0, 0, robe.get_width(), robe.get_height()), Vector2i.ZERO)
	if hood != null:
		body.blend_rect(hood, Rect2i(0, 0, hood.get_width(), hood.get_height()), Vector2i.ZERO)
	return ImageTexture.create_from_image(body)

func _img(path: String) -> Image:
	var tex: Texture2D = load(path)
	if tex == null:
		return null
	var im := tex.get_image()
	if im == null:
		return null
	im = im.duplicate()
	if im.get_format() != Image.FORMAT_RGBA8:
		im.convert(Image.FORMAT_RGBA8)
	return im

# Rows: 0=up, 1=left, 2=down, 3=right. Frame 0 = standing (idle); 1-8 = walk.
func _build_frames(sheet: Texture2D) -> SpriteFrames:
	var sf := SpriteFrames.new()
	if sf.has_animation("default"):
		sf.remove_animation("default")
	if sheet == null:
		return sf
	var rows := {"up": 0, "left": 1, "down": 2, "right": 3}
	for dir in rows:
		var row: int = rows[dir]
		var walk := "walk_" + str(dir)
		sf.add_animation(walk)
		sf.set_animation_loop(walk, true)
		sf.set_animation_speed(walk, 10.0)
		for col in range(1, 9):
			sf.add_frame(walk, _atlas(sheet, col, row))
		var idle := "idle_" + str(dir)
		sf.add_animation(idle)
		sf.set_animation_loop(idle, true)
		sf.set_animation_speed(idle, 1.0)
		sf.add_frame(idle, _atlas(sheet, 0, row))
	return sf

func _atlas(sheet: Texture2D, col: int, row: int) -> AtlasTexture:
	var at := AtlasTexture.new()
	at.atlas = sheet
	at.region = Rect2(col * LPC_FRAME, row * LPC_FRAME, LPC_FRAME, LPC_FRAME)
	return at
