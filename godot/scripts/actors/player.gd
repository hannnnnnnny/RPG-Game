## 玩家角色 —— 占位像素小人 + 4 方向走路 + 翻滚 + 攻击
##
## 替换为 PixelLab 真实 sprite：
##   1. 把 4 张 PNG 拖进 res://assets/sprites/disi/
##   2. 编辑器里把 Player.tscn 根节点改成 AnimatedSprite2D
##   3. 创建 SpriteFrames 资源，加 walk_down/up/left/right + idle_*
##   4. 删掉本脚本的 _draw 块，改用 $AnimatedSprite2D.play("walk_" + facing_str)

class_name Player
extends CharacterBody2D

const WALK_SPEED := 140.0
const ROLL_SPEED := 240.0
const ROLL_DURATION := 0.26
const ATTACK_COOLDOWN := 0.42

enum Facing { DOWN, LEFT, UP, RIGHT }

@export var stamina: float = 100.0
@export var health: float = 100.0
@export var max_stamina: float = 100.0
@export var max_health: float = 100.0

var facing: Facing = Facing.DOWN
var roll_timer: float = 0.0
var attack_timer: float = 0.0
var anim_frame: int = 0
var anim_time: float = 0.0
var moving: bool = false

signal attack_performed(facing_dir: int, position: Vector2)

func _ready() -> void:
	add_to_group("player")

func facing_str() -> String:
	match facing:
		Facing.DOWN: return "down"
		Facing.UP: return "up"
		Facing.LEFT: return "left"
		Facing.RIGHT: return "right"
		_: return "down"

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

	if moving:
		anim_time += delta
		if anim_time > 0.12:
			anim_time = 0.0
			anim_frame = (anim_frame + 1) % 4
	else:
		anim_frame = 0
		anim_time = 0.0
	queue_redraw()

	GameState.set_combat({
		"health": int(round(health)),
		"max_health": int(max_health),
		"stamina": int(round(stamina)),
		"max_stamina": int(max_stamina),
		"focus": 30,
		"max_focus": 30
	})

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

# === Placeholder pixel art: simple chunky character ===
# Body sits above origin so feet anchor at (0, 0).
func _draw() -> void:
	var hair := Color8(42, 21, 5)
	var skin := Color8(235, 202, 167)
	var shirt := Color8(61, 31, 85)
	var pants := Color8(31, 24, 40)
	var boot := Color8(21, 8, 16)
	var outline := Color8(10, 5, 16)

	# Step variation
	var step_offset := 0.0
	if moving and (anim_frame == 1 or anim_frame == 3):
		step_offset = -2.0

	# Simple silhouette (origin at feet, body above)
	# All in screen pixels (no need to scale here since node has scale=2 in scene)
	# Head
	draw_rect(Rect2(-7, -56, 14, 12), outline, true)
	draw_rect(Rect2(-6, -55, 12, 10), hair, true)
	if facing == Facing.DOWN:
		draw_rect(Rect2(-6, -48, 12, 5), skin, true)
		# Eyes
		draw_rect(Rect2(-4, -46, 2, 2), outline, true)
		draw_rect(Rect2(2, -46, 2, 2), outline, true)
	elif facing == Facing.UP:
		pass  # all hair
	else:
		# Side: half face
		var face_x := -6 if facing == Facing.RIGHT else 0
		draw_rect(Rect2(face_x, -48, 6, 5), skin, true)
	# Body
	draw_rect(Rect2(-7, -44, 14, 18), outline, true)
	draw_rect(Rect2(-6, -43, 12, 16), shirt, true)
	# Belt
	draw_rect(Rect2(-7, -26, 14, 2), outline, true)
	# Pants
	draw_rect(Rect2(-7, -24, 14, 14), outline, true)
	draw_rect(Rect2(-6, -23, 12, 12), pants, true)
	# Boots (with step animation)
	draw_rect(Rect2(-7, -10 + step_offset, 6, 10 - step_offset), boot, true)
	draw_rect(Rect2(1, -10 - step_offset, 6, 10 + step_offset), boot, true)
