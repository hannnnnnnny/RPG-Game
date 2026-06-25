## 城镇 NPC —— LPC 合成角色（体型 + 随机色长袍 + 兜帽），4 方向行走动画，
## 平时在自己附近闲逛，按 E 对话。门卫按污染值分支。
class_name TownNpc
extends CharacterBody2D

const LPC_FRAME := 64
const BODY_PATH := "res://assets/sprites/disi/body_walk.png"
const HOOD_PATH := "res://assets/sprites/disi/hood_walk.png"
const ROBE_PATHS := {
	"purple": "res://assets/sprites/disi/robe_walk.png",
	"black": "res://assets/sprites/townsfolk/robe_black.png",
	"brown": "res://assets/sprites/townsfolk/robe_brown.png",
	"blue": "res://assets/sprites/townsfolk/robe_blue.png",
	"red": "res://assets/sprites/townsfolk/robe_red.png",
	"forest_green": "res://assets/sprites/townsfolk/robe_forest_green.png",
	"dark_gray": "res://assets/sprites/townsfolk/robe_dark_gray.png"
}

@export_enum("warden", "merchant", "townsfolk") var kind: String = "townsfolk"
@export var npc_name: String = "镇民"
@export var lines: PackedStringArray = []
@export var robe_color: String = "brown"
@export var hooded: bool = true
@export var wander: bool = true
@export var body_color: Color = Color.WHITE  # legacy, unused by LPC art

@onready var sprite: AnimatedSprite2D = $Sprite

var home: Vector2
var wander_target: Vector2
var wander_timer: float = 0.0
var facing: String = "down"
var moving: bool = false
var _line_idx: int = 0

func _ready() -> void:
	add_to_group("town_npc")
	add_to_group("interactable")
	home = global_position
	wander_target = home
	sprite.sprite_frames = _build_frames(_composite())
	sprite.play("idle_down")

func _physics_process(delta: float) -> void:
	if wander:
		wander_timer -= delta
		if wander_timer <= 0.0 or global_position.distance_to(wander_target) < 8.0:
			wander_timer = randf_range(1.6, 4.0)
			if randf() < 0.4:
				wander_target = global_position  # loiter
			else:
				wander_target = home + Vector2(randf_range(-120, 120), randf_range(-90, 90))
		var to_t := wander_target - global_position
		if to_t.length() > 8.0:
			velocity = to_t.normalized() * 42.0
		else:
			velocity = Vector2.ZERO
		move_and_slide()
	moving = velocity.length() > 4.0
	if moving:
		if abs(velocity.x) > abs(velocity.y):
			facing = "left" if velocity.x < 0 else "right"
		else:
			facing = "up" if velocity.y < 0 else "down"
	var anim := ("walk_" if moving else "idle_") + facing
	if sprite.animation != anim:
		sprite.play(anim)

func interact() -> void:
	talk()

func talk() -> void:
	var text := ""
	if kind == "warden":
		var c: int = GameState.world_state.corruption
		if c >= 56:
			text = "站住。你身上有黑潮的味道……灯火认得它。再往里走一步，我就敲钟。"
		elif c >= 26:
			text = "手腕给我看看。没有刺青就放你进——但别和镇民走太近。"
		else:
			text = "%s，活着从矿里出来的不多。进来吧，灰灯还亮着。" % _player_name()
	elif lines.size() > 0:
		text = lines[_line_idx % lines.size()]
		_line_idx += 1
	else:
		text = "……"
	GameState.set_dialogue({"speaker": npc_name, "text": text, "tone": Types.TONE_MEMORY})

func _player_name() -> String:
	return GameState.profile.get("name", "无名者")

# ---------- LPC composite (body + robe + optional hood) ----------
func _composite() -> ImageTexture:
	var body := _img(BODY_PATH)
	if body == null:
		return null
	var robe := _img(ROBE_PATHS.get(robe_color, ROBE_PATHS["brown"]))
	if robe != null:
		body.blend_rect(robe, Rect2i(0, 0, robe.get_width(), robe.get_height()), Vector2i.ZERO)
	if hooded:
		var hood := _img(HOOD_PATH)
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
		sf.set_animation_speed(walk, 9.0)
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
