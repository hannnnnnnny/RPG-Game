## 通用城镇 NPC —— 站立、可对话。
## 在 town.gd 里按 E 触发 talk()。对话按污染值可分支（自治会门卫会起疑）。
class_name TownNpc
extends Area2D

@export_enum("warden", "merchant", "townsfolk") var kind: String = "townsfolk"
@export var npc_name: String = "镇民"
@export var lines: PackedStringArray = []
@export var body_color: Color = Color(0.5, 0.42, 0.3)

var anim_t: float = 0.0
var anim_frame: int = 0
var _line_idx: int = 0

func _ready() -> void:
	add_to_group("town_npc")
	add_to_group("interactable")
	set_process(true)

# Polymorphic entry point used by town._check_interaction.
func interact() -> void:
	talk()

func _process(delta: float) -> void:
	anim_t += delta
	if anim_t > 0.6:
		anim_t = 0.0
		anim_frame = 1 - anim_frame
		queue_redraw()

func talk() -> void:
	var text := ""
	# Warden reacts to how corrupted the player looks.
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
	GameState.set_dialogue({
		"speaker": npc_name,
		"text": text,
		"tone": Types.TONE_MEMORY
	})

func _player_name() -> String:
	return GameState.profile.get("name", "无名者")

# ---------- Procedural pixel art: standing townsperson ----------
func _draw() -> void:
	var bob := -1 if anim_frame == 1 else 0
	var skin := Color8(214, 176, 140)
	var skin_sh := Color8(176, 130, 92)
	var hair := Color8(60, 44, 32)
	var cloth := body_color
	var cloth_sh := body_color.darkened(0.35)
	var boot := Color8(40, 30, 20)
	var eye := Color8(28, 16, 10)
	var sc := 2.0
	var f := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		draw_rect(Rect2(x * sc, (y + bob) * sc, w * sc, h * sc), c, true)
	var fg := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		draw_rect(Rect2(x * sc, y * sc, w * sc, h * sc), c, true)
	# Hair / head
	f.call(-4, -22, 8, 3, hair)
	f.call(-4, -19, 8, 6, skin)
	f.call(3, -18, 1, 4, skin_sh)
	f.call(-2, -17, 1, 1, eye)
	f.call(1, -17, 1, 1, eye)
	# Cloak / tunic
	f.call(-5, -13, 10, 10, cloth)
	f.call(3, -12, 1, 8, cloth_sh)
	f.call(-5, -13, 1, 10, cloth_sh)
	# Belt
	f.call(-5, -5, 10, 1, boot)
	# Legs
	fg.call(-5, -3, 4, 5, boot)
	fg.call(1, -3, 4, 5, boot)
