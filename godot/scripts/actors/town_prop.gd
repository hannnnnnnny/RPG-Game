## 城镇可互动物 —— 门(进店)/告示牌/水井/木箱。按 E 触发。
## 加入 "interactable" 组，由 town._check_interaction 多态调用 interact()。
class_name TownProp
extends Area2D

@export_enum("door_store", "notice", "well", "crate", "sign") var kind: String = "sign"
@export var label_text: String = ""
@export var lines: PackedStringArray = []

var _idx: int = 0

func _ready() -> void:
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
		_:
			# plain sign post
			f.call(-2, -28, 4, 28, Color8(70, 50, 32))
			f.call(-10, -30, 20, 8, Color8(96, 68, 42))
			f.call(-10, -30, 20, 1, Color8(132, 96, 58))
