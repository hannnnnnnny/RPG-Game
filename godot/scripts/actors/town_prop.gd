## 城镇可互动物 —— 门(进店)/告示牌/水井/木箱。按 E 触发。
## 加入 "interactable" 组，由 town._check_interaction 多态调用 interact()。
class_name TownProp
extends Area2D

@export_enum("door_store", "notice", "well", "crate", "sign", "stall", "fountain", "lantern_post", "barrel", "planter", "fence") var kind: String = "sign"
@export var label_text: String = ""
@export var lines: PackedStringArray = []
@export var decorative: bool = false  # pure scenery — not in the interactable group

var _idx: int = 0

func _ready() -> void:
	if not decorative:
		add_to_group("interactable")
	add_to_group("town_prop")
	queue_redraw()

func interact() -> void:
	match kind:
		"door_store":
			get_tree().change_scene_to_file("res://scenes/world/StoreInterior.tscn")
		_:
			var text := lines[_idx % lines.size()] if lines.size() > 0 else "……"
			_idx += 1
			GameState.set_dialogue({
				"speaker": label_text if label_text != "" else "灰灯镇",
				"text": text,
				"tone": Types.TONE_MEMORY
			})

func _draw() -> void:
	var sc := 2.0
	var f := func(x: float, y: float, w: float, h: float, c: Color) -> void:
		draw_rect(Rect2(x * sc, y * sc, w * sc, h * sc), c, true)
	match kind:
		"door_store":
			# Lit doorway in the shop wall.
			f.call(-12, -34, 24, 34, Color8(40, 28, 20))
			f.call(-9, -30, 18, 30, Color8(96, 64, 36))   # warm interior glow
			f.call(-9, -30, 18, 2, Color8(150, 110, 60))
			f.call(-2, -16, 2, 3, Color8(214, 170, 90))   # handle
			# Hanging sign
			f.call(-10, -44, 20, 8, Color8(74, 52, 34))
			f.call(-10, -44, 20, 1, Color8(120, 88, 54))
		"notice":
			# Notice board on posts.
			f.call(-14, -34, 28, 22, Color8(86, 60, 38))
			f.call(-14, -34, 28, 2, Color8(120, 86, 54))
			f.call(-12, -31, 8, 6, Color8(206, 196, 170))  # pinned papers
			f.call(-1, -30, 7, 5, Color8(206, 196, 170))
			f.call(-12, 0, 3, 8, Color8(56, 40, 26))
			f.call(9, 0, 3, 8, Color8(56, 40, 26))
		"well":
			f.call(-16, -10, 32, 18, Color8(86, 84, 90))
			f.call(-14, -8, 28, 12, Color8(34, 30, 40))    # dark water
			f.call(-16, -10, 32, 2, Color8(120, 116, 120))
			f.call(-3, -30, 2, 20, Color8(70, 50, 32))     # post
			f.call(-12, -32, 26, 3, Color8(74, 52, 34))    # roof beam
		"crate":
			f.call(-10, -16, 20, 18, Color8(104, 72, 42))
			f.call(-10, -16, 20, 2, Color8(140, 100, 60))
			f.call(-10, -2, 20, 2, Color8(64, 44, 26))
			f.call(-1, -16, 2, 18, Color8(78, 54, 32))
			f.call(-10, -8, 20, 1, Color8(78, 54, 32))
		"stall":
			# Market stall with a striped awning + goods on the counter.
			f.call(-26, -8, 52, 10, Color8(108, 76, 46))     # counter
			f.call(-26, -8, 52, 2, Color8(140, 100, 60))
			f.call(-28, -40, 4, 34, Color8(70, 50, 32))      # posts
			f.call(24, -40, 4, 34, Color8(70, 50, 32))
			for i in range(7):                                # striped awning
				var col := Color8(150, 60, 56) if i % 2 == 0 else Color8(206, 196, 170)
				f.call(-28 + i * 8, -44, 8, 6, col)
			f.call(-18, -14, 6, 6, Color8(180, 140, 70))     # goods
			f.call(-4, -13, 5, 5, Color8(120, 150, 90))
			f.call(8, -14, 6, 6, Color8(160, 90, 70))
		"fountain":
			f.call(-26, -16, 52, 30, Color8(92, 90, 96))     # stone rim
			f.call(-22, -12, 44, 22, Color8(40, 54, 70))     # water
			f.call(-22, -12, 44, 2, Color8(96, 120, 150))
			f.call(-2, -30, 4, 18, Color8(108, 104, 110))    # spout
			f.call(-1, -12, 2, 8, Color8(150, 180, 210))     # falling water
			for i in range(5):
				f.call(-18 + i * 9, -10, 1, 1, Color8(150, 180, 210))
		"lantern_post":
			f.call(-2, -40, 4, 40, Color8(58, 44, 30))       # post
			f.call(-7, -50, 14, 10, Color8(74, 56, 36))      # lamp housing
			f.call(-5, -48, 10, 6, Color8(255, 214, 140))    # warm glow
			f.call(-8, -52, 16, 2, Color8(92, 70, 46))
		"barrel":
			f.call(-9, -18, 18, 20, Color8(96, 66, 40))
			f.call(-9, -18, 18, 2, Color8(132, 96, 58))
			f.call(-9, -12, 18, 1, Color8(60, 42, 26))
			f.call(-9, -4, 18, 1, Color8(60, 42, 26))
			f.call(-9, 0, 18, 2, Color8(64, 44, 26))
		"planter":
			f.call(-12, -6, 24, 10, Color8(86, 60, 38))      # box
			f.call(-12, -6, 24, 2, Color8(116, 82, 50))
			f.call(-10, -12, 5, 6, Color8(96, 130, 78))      # foliage
			f.call(-2, -14, 5, 8, Color8(108, 142, 86))
			f.call(6, -11, 5, 5, Color8(96, 130, 78))
			f.call(-1, -14, 2, 2, Color8(210, 180, 90))      # a bloom
		"fence":
			f.call(-24, -14, 48, 3, Color8(78, 56, 36))      # top rail
			f.call(-24, -6, 48, 3, Color8(78, 56, 36))
			for i in range(5):
				f.call(-22 + i * 11, -18, 3, 18, Color8(92, 66, 42))
		_:
			# plain sign post
			f.call(-2, -28, 4, 28, Color8(70, 50, 32))
			f.call(-10, -30, 20, 8, Color8(96, 68, 42))
			f.call(-10, -30, 20, 1, Color8(132, 96, 58))
