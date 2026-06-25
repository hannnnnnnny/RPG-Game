## 杂货铺室内 —— 星露谷式进店：木地板小屋 + 柜台 + 铜婶 + 货架 + 门垫出口。
## 走到柜台按 E 开商店；走到门垫(或按 E)回灰灯镇。
extends Node2D

const ROOM_W := 720
const ROOM_H := 480
const TILE_SIZE := 48
const SRC_TILE := 16
const STORE_DOOR_IN_TOWN := Vector2(640, 270)  # where you reappear in town

const LIGHT_TEX := preload("res://assets/textures/light_soft.tres")

@onready var player: Player = $Player
@onready var shop: Control = $UILayer/ShopPanel
@onready var objective_label: Label = $UILayer/Objective

var world_tex: ImageTexture
var counter_pos := Vector2(360, 196)
var exit_pos := Vector2(360, 430)
var stock: Array = []

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_bake_room()
	player.global_position = Vector2(360, 400)
	var cam := player.get_node_or_null("Camera2D")
	if cam:
		cam.limit_left = 0
		cam.limit_top = 0
		cam.limit_right = ROOM_W
		cam.limit_bottom = ROOM_H
	_setup_walls()
	_setup_decor()
	_generate_stock()
	Audio.start_ambient()
	objective_label.text = "杂货铺 —— 走到柜台按 E 买卖，走到门口离开。"

func _generate_stock() -> void:
	# A small curated stock: a couple weapons, armor, a totem, mixed quality.
	var tier: int = GameState.world_state.world_tier
	for src in ["enemy", "enemy", "elite", "enemy", "totem", "elite"]:
		stock.append(LootGenerator.generate_loot(src, tier))

func _process(_delta: float) -> void:
	if shop.visible:
		return
	if Input.is_action_just_pressed("interact"):
		if player.global_position.distance_to(counter_pos) < 84.0:
			shop.open(stock)
		elif player.global_position.distance_to(exit_pos) < 80.0:
			_leave()
	# Walking onto the door mat also leaves.
	if player.global_position.distance_to(exit_pos) < 34.0:
		_leave()

func _leave() -> void:
	GameState.set_spawn_override(STORE_DOOR_IN_TOWN)
	get_tree().change_scene_to_file("res://scenes/world/TownAshlight.tscn")

func _setup_decor() -> void:
	# 铜婶 behind the counter.
	const NpcScene := preload("res://scenes/actors/TownNpc.tscn")
	var copper: TownNpc = NpcScene.instantiate()
	copper.kind = "merchant"
	copper.npc_name = "杂货商·铜婶"
	copper.robe_color = "red"
	copper.hooded = false
	copper.wander = false  # stays behind the counter
	copper.lines = PackedStringArray(["走到柜台前，按 E，咱们做笔买卖。"])
	copper.global_position = Vector2(360, 150)
	add_child(copper)
	# Warm hanging lamp over the counter.
	var lamp := PointLight2D.new()
	lamp.texture = LIGHT_TEX
	lamp.color = Color(1.0, 0.84, 0.55)
	lamp.texture_scale = 3.0
	lamp.energy = 1.2
	lamp.global_position = Vector2(360, 210)
	add_child(lamp)
	# A second soft fill light by the door.
	var lamp2 := PointLight2D.new()
	lamp2.texture = LIGHT_TEX
	lamp2.color = Color(0.9, 0.8, 0.6)
	lamp2.texture_scale = 2.2
	lamp2.energy = 0.7
	lamp2.global_position = exit_pos
	add_child(lamp2)

# ---------- Room tile bake ----------
func _classify(col: int, row: int) -> String:
	var cx := col * TILE_SIZE + TILE_SIZE / 2
	var cy := row * TILE_SIZE + TILE_SIZE / 2
	if cx < 40 or cx > ROOM_W - 40 or cy < 40 or cy > ROOM_H - 40:
		# leave a door gap at the south-center
		if not (cy > ROOM_H - 56 and cx > 312 and cx < 408):
			return "wall"
	# Counter: a horizontal bar across the middle the player can't cross.
	if cy > 168 and cy < 224 and cx > 180 and cx < 540:
		return "counter"
	return "floor"

func _bake_room() -> void:
	var cols := int(ceil(ROOM_W / float(TILE_SIZE)))
	var rows := int(ceil(ROOM_H / float(TILE_SIZE)))
	var img := Image.create(cols * SRC_TILE, rows * SRC_TILE, false, Image.FORMAT_RGBA8)
	for r in range(rows):
		for c in range(cols):
			_paint(img, c * SRC_TILE, r * SRC_TILE, _classify(c, r), c, r)
	world_tex = ImageTexture.create_from_image(img)

func _fillr(img: Image, x: int, y: int, w: int, h: int, col: Color) -> void:
	img.fill_rect(Rect2i(x, y, w, h), col)

func _trand(s: int, n: int) -> float:
	return fmod(abs(sin(float(s) * float(n + 1))), 1.0)

func _paint(img: Image, ox: int, oy: int, type: String, c: int, r: int) -> void:
	var s := (c * 73856093) ^ (r * 19349663)
	if type == "wall":
		_fillr(img, ox, oy, 16, 16, Color8(70, 52, 38))
		_fillr(img, ox, oy, 16, 2, Color8(98, 74, 52))
		_fillr(img, ox, oy + 14, 16, 2, Color8(40, 28, 20))
		_fillr(img, ox, oy + 8, 16, 1, Color8(46, 32, 22))
		return
	if type == "counter":
		_fillr(img, ox, oy, 16, 16, Color8(126, 88, 52))
		_fillr(img, ox, oy, 16, 2, Color8(158, 116, 72))
		_fillr(img, ox, oy + 14, 16, 2, Color8(82, 56, 32))
		_fillr(img, ox + 7, oy, 1, 16, Color8(96, 66, 40))
		return
	# wood plank floor
	_fillr(img, ox, oy, 16, 16, Color8(110, 80, 50))
	_fillr(img, ox, oy + 5, 16, 1, Color8(86, 60, 36))
	_fillr(img, ox, oy + 11, 16, 1, Color8(86, 60, 36))
	if _trand(s, 1) < 0.25:
		_fillr(img, ox + int(_trand(s, 2) * 13) + 1, oy + int(_trand(s, 3) * 13) + 1, 1, 1, Color8(132, 98, 62))

func _draw() -> void:
	if world_tex == null:
		return
	var cols := int(ceil(ROOM_W / float(TILE_SIZE)))
	var rows := int(ceil(ROOM_H / float(TILE_SIZE)))
	draw_texture_rect(world_tex, Rect2(0, 0, cols * TILE_SIZE, rows * TILE_SIZE), false)

func _setup_walls() -> void:
	var cols := int(ceil(ROOM_W / float(TILE_SIZE)))
	var rows := int(ceil(ROOM_H / float(TILE_SIZE)))
	var walls := StaticBody2D.new()
	walls.collision_layer = 1
	walls.collision_mask = 0
	add_child(walls)
	for r in range(rows):
		var c := 0
		while c < cols:
			var t := _classify(c, r)
			if t != "wall" and t != "counter":
				c += 1
				continue
			var run_end := c
			while run_end < cols:
				var tt := _classify(run_end, r)
				if tt != "wall" and tt != "counter":
					break
				run_end += 1
			var shape := CollisionShape2D.new()
			var rs := RectangleShape2D.new()
			rs.size = Vector2((run_end - c) * TILE_SIZE, TILE_SIZE)
			shape.shape = rs
			shape.position = Vector2(c * TILE_SIZE + rs.size.x / 2, r * TILE_SIZE + TILE_SIZE / 2)
			walls.add_child(shape)
			c = run_end
