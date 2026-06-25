## 城镇 NPC —— LPC 合成角色。外观可选：兜帽 或 多种发型（不再光头）。
## 站立者四处缓慢闲逛；seated 者坐着不动。按 E 对话。门卫按污染分支。
class_name TownNpc
extends CharacterBody2D

const LPC := 64
const DIR := "res://assets/sprites/"
const BODY_WALK := DIR + "disi/body_walk.png"
const HOOD_WALK := DIR + "disi/hood_walk.png"
const BODY_SIT := DIR + "townsfolk/body_sit.png"
const SHIRT_SIT := DIR + "townsfolk/shirt_sit.png"

const ROBE := {
	"purple": DIR + "disi/robe_walk.png",
	"black": DIR + "townsfolk/robe_black.png",
	"brown": DIR + "townsfolk/robe_brown.png",
	"blue": DIR + "townsfolk/robe_blue.png",
	"red": DIR + "townsfolk/robe_red.png",
	"forest_green": DIR + "townsfolk/robe_forest_green.png",
	"dark_gray": DIR + "townsfolk/robe_dark_gray.png"
}
const HAIR_WALK := {
	"plain": DIR + "townsfolk/hair_plain_walk.png",
	"long": DIR + "townsfolk/hair_long_walk.png",
	"bangsshort": DIR + "townsfolk/hair_bangsshort_walk.png"
}
const HAIR_SIT := {
	"plain": DIR + "townsfolk/hair_plain_sit.png",
	"long": DIR + "townsfolk/hair_long_sit.png"
}

@export_enum("warden", "merchant", "townsfolk") var kind: String = "townsfolk"
@export var npc_name: String = "镇民"
@export var lines: PackedStringArray = []
@export var robe_color: String = "brown"
@export_enum("hood", "plain", "long", "bangsshort") var head: String = "hood"
@export var seated: bool = false
@export var wander: bool = true

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
	if seated:
		wander = false
		sprite.sprite_frames = _build_sit(_composite_sit())
		sprite.play("sit")
	else:
		sprite.sprite_frames = _build_walk(_composite_walk())
		sprite.play("idle_down")

func _physics_process(delta: float) -> void:
	if seated:
		return
	if wander:
		wander_timer -= delta
		if wander_timer <= 0.0 or global_position.distance_to(wander_target) < 8.0:
			wander_timer = randf_range(2.4, 5.5)
			if randf() < 0.5:
				wander_target = global_position  # loiter often
			else:
				wander_target = home + Vector2(randf_range(-90, 90), randf_range(-70, 70))
		var to_t := wander_target - global_position
		if to_t.length() > 8.0:
			velocity = to_t.normalized() * 24.0
		else:
			velocity = Vector2.ZERO
		move_and_slide()
	moving = velocity.length() > 3.0
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

# ---------- LPC composite ----------
func _composite_walk() -> ImageTexture:
	var img := _img(BODY_WALK)
	if img == null:
		return null
	_over(img, ROBE.get(robe_color, ROBE["brown"]))
	if head == "hood":
		_over(img, HOOD_WALK)
	elif HAIR_WALK.has(head):
		_over(img, HAIR_WALK[head])
	return ImageTexture.create_from_image(img)

func _composite_sit() -> ImageTexture:
	var img := _img(BODY_SIT)
	if img == null:
		return null
	_over(img, SHIRT_SIT)
	var hs: String = head if HAIR_SIT.has(head) else "plain"
	_over(img, HAIR_SIT[hs])
	return ImageTexture.create_from_image(img)

func _over(base: Image, path: String) -> void:
	var top := _img(path)
	if top != null:
		base.blend_rect(top, Rect2i(0, 0, top.get_width(), top.get_height()), Vector2i.ZERO)

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

func _build_walk(sheet: Texture2D) -> SpriteFrames:
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
		sf.set_animation_speed(walk, 8.0)
		for col in range(1, 9):
			sf.add_frame(walk, _atlas(sheet, col, row))
		var idle := "idle_" + str(dir)
		sf.add_animation(idle)
		sf.set_animation_loop(idle, true)
		sf.set_animation_speed(idle, 1.0)
		sf.add_frame(idle, _atlas(sheet, 0, row))
	return sf

# Sit sheet is 192x256 = 3 cols x 4 rows; use the "down" row as a gentle idle-sit.
func _build_sit(sheet: Texture2D) -> SpriteFrames:
	var sf := SpriteFrames.new()
	if sf.has_animation("default"):
		sf.remove_animation("default")
	if sheet == null:
		return sf
	sf.add_animation("sit")
	sf.set_animation_loop("sit", true)
	sf.set_animation_speed("sit", 1.6)
	for col in range(0, 3):
		sf.add_frame("sit", _atlas(sheet, col, 2))
	return sf

func _atlas(sheet: Texture2D, col: int, row: int) -> AtlasTexture:
	var at := AtlasTexture.new()
	at.atlas = sheet
	at.region = Rect2(col * LPC, row * LPC, LPC, LPC)
	return at
