## 灰灯镇 —— 第一个安全镇。逃出矿井后到达。
## 程序化木石瓦片 + 建筑碰撞 + 暖色灯柱 + 可对话 NPC。无战斗。
extends Node2D

const WORLD_W := 1300
const WORLD_H := 820
const TILE_SIZE := 48
const SRC_TILE := 16

const LIGHT_TEX := preload("res://assets/textures/light_soft.tres")

# Buildings (impassable). Border is added procedurally with a south gate gap.
const BUILDINGS := [
	Rect2(120, 110, 300, 170),   # 自治会会堂
	Rect2(520, 90, 240, 150),    # 杂货铺
	Rect2(880, 130, 280, 180),   # 民居
	Rect2(150, 520, 260, 160),   # 民居
	Rect2(900, 520, 260, 170)    # 仓库
]
# Central plaza (wooden boards).
const PLAZA := Rect2(470, 350, 420, 230)
# Warm lamp posts — the "gray lamps" the town clings to.
const LAMPS := [
	Vector2(470, 350), Vector2(890, 350), Vector2(470, 580), Vector2(890, 580),
	Vector2(680, 300), Vector2(680, 700)
]

@onready var player: Player = $Player
@onready var npcs_root: Node2D = $Npcs
@onready var objective_label: Label = $UILayer/Objective

var wall_tiles: Array = []
var world_tex: ImageTexture
var follower: Follower = null

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_bake_world_texture()
	# Arrive from the south gate by default, or at a building door when
	# returning from an interior.
	player.global_position = GameState.consume_spawn_override(Vector2(650, 760))
	var cam := player.get_node_or_null("Camera2D")
	if cam:
		cam.limit_right = WORLD_W
		cam.limit_bottom = WORLD_H
	_classify_walls()
	_setup_static_walls()
	_setup_lamps()
	_spawn_npcs()
	_spawn_props()

	if GameState.world_state.flags.get("first_dwarf_choice", "") == "save":
		_spawn_follower()

	Audio.start_ambient()
	objective_label.text = "目标：灰灯镇——和镇民交谈。下一版会从这里继续。"
	GameState.set_dialogue({
		"speaker": "克哈低语",
		"text": "灯还亮着。可惜灯不知道，它照的人里有一个已经属于海。",
		"tone": Types.TONE_WHISPER
	})

func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("interact"):
		_check_interaction()

func _check_interaction() -> void:
	var nearest: Node2D = null
	var best := 999999.0
	for n in get_tree().get_nodes_in_group("interactable"):
		if n is Node2D:
			var d := player.global_position.distance_to((n as Node2D).global_position)
			if d < 84.0 and d < best:
				nearest = n
				best = d
	if nearest and nearest.has_method("interact"):
		nearest.interact()

func _setup_lamps() -> void:
	for pos in LAMPS:
		var light := PointLight2D.new()
		light.texture = LIGHT_TEX
		light.color = Color(1.0, 0.82, 0.5)
		light.texture_scale = 2.6
		light.energy = 1.1
		light.global_position = pos
		add_child(light)
		var base := _pulse_light(light)
		base = base  # keep flicker subtle

func _pulse_light(light: PointLight2D) -> float:
	var e: float = light.energy
	var tw := create_tween().set_loops()
	tw.tween_property(light, "energy", e * 0.85, 1.6).set_trans(Tween.TRANS_SINE)
	tw.tween_property(light, "energy", e, 1.6).set_trans(Tween.TRANS_SINE)
	return e

func _spawn_npcs() -> void:
	const NpcScene := preload("res://scenes/actors/TownNpc.tscn")
	var defs := [
		{
			"kind": "warden", "name": "灰灯门卫", "pos": Vector2(650, 660),
			"color": Color(0.32, 0.36, 0.42), "lines": []
		},
		{
			"kind": "townsfolk", "name": "守夜人", "pos": Vector2(1000, 640),
			"color": Color(0.4, 0.34, 0.3),
			"lines": ["夜里别靠近镇墙。灯一灭，墙外的东西就贴上来。",
				"你是从矿里出来的？那下面……还有活人吗？"]
		}
	]
	for d in defs:
		var n: TownNpc = NpcScene.instantiate()
		n.kind = d.kind
		n.npc_name = d.name
		n.body_color = d.color
		n.lines = PackedStringArray(d.lines)
		n.global_position = d.pos
		npcs_root.add_child(n)
		var glow := PointLight2D.new()
		glow.texture = LIGHT_TEX
		glow.color = Color(1.0, 0.85, 0.6)
		glow.texture_scale = 1.0
		glow.energy = 0.35
		glow.position = Vector2(0, -16)
		n.add_child(glow)

func _spawn_props() -> void:
	const PropScene := preload("res://scenes/actors/TownProp.tscn")
	var defs := [
		{
			"kind": "door_store", "pos": Vector2(640, 248), "label": "杂货铺", "lines": []
		},
		{
			"kind": "notice", "pos": Vector2(540, 600), "label": "灰灯镇告示板",
			"lines": ["【告示】凡入镇者，须验手腕。见刺青者，鸣钟。——自治会",
				"【悬赏】矿区方向有黑潮渗出，能封住裂口者，重谢。",
				"【寻人】我的儿子下矿三天没回。若你见过他……别骗我。"]
		},
		{
			"kind": "well", "pos": Vector2(680, 470), "label": "镇中水井",
			"lines": ["井水还算清。镇民说，等井水发黑那天，就该弃镇了。"]
		},
		{
			"kind": "crate", "pos": Vector2(820, 300), "label": "货箱",
			"lines": ["几只钉死的木箱。铜婶的货，没付钱别动。"]
		},
		{
			"kind": "crate", "pos": Vector2(280, 360), "label": "木箱",
			"lines": ["空的。里面只有潮湿的稻草和一只死掉的矿灯。"]
		}
	]
	for d in defs:
		var p: TownProp = PropScene.instantiate()
		p.kind = d.kind
		p.label_text = d.label
		p.lines = PackedStringArray(d.lines)
		p.global_position = d.pos
		npcs_root.add_child(p)

func _spawn_follower() -> void:
	if follower != null and is_instance_valid(follower):
		return
	const FollowerScene := preload("res://scenes/actors/Follower.tscn")
	follower = FollowerScene.instantiate()
	follower.global_position = player.global_position + Vector2(-40, 10)
	add_child(follower)

# ---------- Tile bake (town theme) ----------

func _classify_tile(col: int, row: int) -> String:
	var cx := col * TILE_SIZE + TILE_SIZE / 2
	var cy := row * TILE_SIZE + TILE_SIZE / 2
	# Border wall (with a south gate gap so you arrive from the mine).
	if cx < 40 or cx > WORLD_W - 40 or cy < 40 or cy > WORLD_H - 40:
		var gate := cy > WORLD_H - 60 and cx > 580 and cx < 720
		if not gate:
			return "wall"
	for b in BUILDINGS:
		if b.has_point(Vector2(cx, cy)):
			return "wall"
	if PLAZA.has_point(Vector2(cx, cy)):
		return "plank"
	return "ground"

func _bake_world_texture() -> void:
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	var img := Image.create(cols * SRC_TILE, rows * SRC_TILE, false, Image.FORMAT_RGBA8)
	for r in range(rows):
		for c in range(cols):
			_paint_tile(img, c * SRC_TILE, r * SRC_TILE, _classify_tile(c, r), c, r)
	world_tex = ImageTexture.create_from_image(img)

func _trand(s: int, n: int) -> float:
	return fmod(abs(sin(float(s) * float(n + 1))), 1.0)

func _px(img: Image, x: int, y: int, col: Color) -> void:
	if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height():
		img.set_pixel(x, y, col)

func _fillr(img: Image, x: int, y: int, w: int, h: int, col: Color) -> void:
	img.fill_rect(Rect2i(x, y, w, h), col)

func _paint_tile(img: Image, ox: int, oy: int, type: String, c: int, r: int) -> void:
	var s := (c * 73856093) ^ (r * 19349663)
	if type == "wall":
		# Timber-and-stone building/wall, warmer than the mine.
		_fillr(img, ox, oy, 16, 16, Color8(74, 58, 44))
		_fillr(img, ox, oy, 16, 2, Color8(104, 82, 60))
		_fillr(img, ox, oy, 2, 16, Color8(92, 72, 54))
		_fillr(img, ox, oy + 14, 16, 2, Color8(40, 30, 22))
		_fillr(img, ox + 14, oy, 2, 16, Color8(46, 34, 26))
		_fillr(img, ox, oy + 7, 16, 1, Color8(44, 32, 24))
		if (r % 2) == 0:
			_fillr(img, ox + 8, oy, 1, 7, Color8(44, 32, 24))
		else:
			_fillr(img, ox + 4, oy + 8, 1, 6, Color8(44, 32, 24))
		return
	if type == "plank":
		# Wooden plaza boards.
		_fillr(img, ox, oy, 16, 16, Color8(120, 86, 52))
		_fillr(img, ox, oy + 5, 16, 1, Color8(92, 64, 38))
		_fillr(img, ox, oy + 11, 16, 1, Color8(92, 64, 38))
		if _trand(s, 1) < 0.3:
			_px(img, ox + int(_trand(s, 2) * 14) + 1, oy + int(_trand(s, 3) * 14) + 1, Color8(142, 104, 64))
		return
	# ground — packed dirt / cobble
	var v := _trand(s, 2)
	var base := Color8(78, 66, 50)
	if v >= 0.6 and v < 0.85:
		base = Color8(86, 72, 54)
	elif v >= 0.85:
		base = Color8(70, 58, 44)
	_fillr(img, ox, oy, 16, 16, base)
	_fillr(img, ox, oy, 16, 1, Color8(96, 80, 60))
	_fillr(img, ox, oy + 15, 16, 1, Color8(54, 44, 32))
	if _trand(s, 3) < 0.2:
		_px(img, ox + 3 + int(_trand(s, 4) * 9), oy + 3 + int(_trand(s, 5) * 9), Color8(108, 92, 70))

func _draw() -> void:
	if world_tex == null:
		return
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	draw_texture_rect(world_tex, Rect2(0, 0, cols * TILE_SIZE, rows * TILE_SIZE), false)

func _classify_walls() -> void:
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	wall_tiles.clear()
	for r in range(rows):
		var c := 0
		while c < cols:
			if _classify_tile(c, r) != "wall":
				c += 1
				continue
			var run_end := c
			while run_end < cols and _classify_tile(run_end, r) == "wall":
				run_end += 1
			wall_tiles.append(Rect2(c * TILE_SIZE, r * TILE_SIZE, (run_end - c) * TILE_SIZE, TILE_SIZE))
			c = run_end

func _setup_static_walls() -> void:
	var walls := StaticBody2D.new()
	walls.collision_layer = 1
	walls.collision_mask = 0
	add_child(walls)
	for rect: Rect2 in wall_tiles:
		var shape := CollisionShape2D.new()
		var rs := RectangleShape2D.new()
		rs.size = rect.size
		shape.shape = rs
		shape.position = rect.position + rect.size / 2
		walls.add_child(shape)
