## 黑潮矿区主场景 —— 程序化瓦片地图 + 实体生成 + 交互逻辑
##
## 这是 src/game/scenes/MineIntroScene.ts 的 Godot 等价物。
## 后续要换成 Tiled 编辑器导出的 .tmx → Godot 4 TileMap 资源。

extends Node2D

const WORLD_W := 1450
const WORLD_H := 900
const TILE_SIZE := 48

# Room rects (x, y, w, h) matching Phaser version
const ROOMS := [
	Rect2(60, 80, 350, 180),
	Rect2(360, 185, 520, 170),
	Rect2(760, 310, 350, 210),
	Rect2(1040, 470, 240, 260),
	Rect2(420, 540, 520, 180)
]

const PUDDLES := [
	{"cx": 685, "cy": 260, "rx": 95, "ry": 35},
	{"cx": 980, "cy": 445, "rx": 110, "ry": 45},
	{"cx": 555, "cy": 640, "rx": 85, "ry": 29}
]

const PATH_POINTS := [
	Vector2(200, 170),
	Vector2(620, 270),
	Vector2(930, 415),
	Vector2(1160, 600),
	Vector2(680, 630)
]

const LIGHT_TEX := preload("res://assets/textures/light_soft.tres")

@onready var player: Player = $Player
@onready var camera: Camera2D = $Player/Camera2D
@onready var enemies_root: Node2D = $Enemies
@onready var interactables_root: Node2D = $Interactables
@onready var objective_label: Label = $UILayer/Objective
@onready var hud: Control = $UILayer/Hud
@onready var dialogue: Control = $UILayer/Dialogue
@onready var choice: Control = $UILayer/Choice
@onready var vision: Control = $UILayer/Vision

var wall_tiles: Array = []  # Array of Rect2 for collision
var world_tex: ImageTexture  # Baked pixel-art tile map (replaces flat color blocks)

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_bake_world_texture()
	_spawn_player_at(Vector2(155, 165))
	_spawn_enemies()
	_spawn_interactables()
	_classify_walls()
	_setup_static_walls()

	player.attack_performed.connect(_on_player_attack)

	GameState.set_dialogue({
		"speaker": "克哈低语",
		"text": "往前。那些矿灯已经死了，但你的血还记得路。",
		"tone": Types.TONE_WHISPER
	})

func _spawn_player_at(pos: Vector2) -> void:
	player.global_position = pos

func _spawn_enemies() -> void:
	const EnemyScene := preload("res://scenes/actors/Enemy.tscn")
	for pos in [Vector2(510, 245), Vector2(825, 395), Vector2(1075, 575)]:
		var e: Enemy = EnemyScene.instantiate()
		e.global_position = pos
		enemies_root.add_child(e)
		# Faint purple aura so corrupted dwarves loom out of the dark.
		var aura := _make_glow(Color(0.55, 0.2, 0.7), 1.0, 0.45)
		aura.position = Vector2(0, -20)
		e.add_child(aura)

func _spawn_interactables() -> void:
	const InteractScene := preload("res://scenes/actors/Interactable.tscn")
	var configs := [
		{
			"type": "injured_dwarf", "pos": Vector2(395, 230), "label": "受伤矮人",
			"glow": Color(1.0, 0.66, 0.45), "scale": 1.5, "energy": 0.55, "pulse": false
		},
		{
			"type": "totem_fragment", "pos": Vector2(910, 382), "label": "图腾残片",
			"glow": Color(0.7, 0.35, 1.0), "scale": 2.4, "energy": 1.1, "pulse": true
		},
		{
			"type": "mine_exit", "pos": Vector2(1220, 650), "label": "矿井出口",
			"glow": Color(0.4, 0.86, 0.78), "scale": 2.1, "energy": 0.9, "pulse": false
		}
	]
	for cfg in configs:
		var n: Interactable = InteractScene.instantiate()
		n.interact_type = cfg.type
		n.label = cfg.label
		n.global_position = cfg.pos
		interactables_root.add_child(n)
		var light := _make_glow(cfg.glow, cfg.scale, cfg.energy)
		n.add_child(light)
		if cfg.pulse:
			_pulse_light(light)

func _make_glow(color: Color, scale: float, energy: float) -> PointLight2D:
	var light := PointLight2D.new()
	light.texture = LIGHT_TEX
	light.color = color
	light.texture_scale = scale
	light.energy = energy
	return light

func _pulse_light(light: PointLight2D) -> void:
	var base: float = light.energy
	var tween := create_tween().set_loops()
	tween.tween_property(light, "energy", base * 1.6, 1.3).set_trans(Tween.TRANS_SINE)
	tween.tween_property(light, "energy", base, 1.3).set_trans(Tween.TRANS_SINE)

func _classify_tile(col: int, row: int) -> String:
	var cx := col * TILE_SIZE + TILE_SIZE / 2
	var cy := row * TILE_SIZE + TILE_SIZE / 2
	var in_room := false
	for r in ROOMS:
		if cx >= r.position.x and cx <= r.position.x + r.size.x \
				and cy >= r.position.y and cy <= r.position.y + r.size.y:
			in_room = true
			break
	if not in_room:
		return "wall"
	for p in PUDDLES:
		var dx: float = (cx - p.cx) / float(p.rx)
		var dy: float = (cy - p.cy) / float(p.ry)
		if dx * dx + dy * dy <= 1.0:
			return "puddle"
	if _is_path_tile(cx, cy):
		return "path"
	return "floor"

func _is_path_tile(cx: int, cy: int) -> bool:
	for i in range(PATH_POINTS.size() - 1):
		var p1: Vector2 = PATH_POINTS[i]
		var p2: Vector2 = PATH_POINTS[i + 1]
		var c := Vector2(cx, cy)
		var t := c.distance_to(p1) + c.distance_to(p2)
		var seg_len := p1.distance_to(p2)
		if abs(t - seg_len) < 26:
			return true
	return false

func _classify_walls() -> void:
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	wall_tiles.clear()
	# Greedy horizontal merge per row
	for r in range(rows):
		var c := 0
		while c < cols:
			if _classify_tile(c, r) != "wall":
				c += 1
				continue
			var run_end := c
			while run_end < cols and _classify_tile(run_end, r) == "wall":
				run_end += 1
			var rect := Rect2(c * TILE_SIZE, r * TILE_SIZE, (run_end - c) * TILE_SIZE, TILE_SIZE)
			wall_tiles.append(rect)
			c = run_end

func _setup_static_walls() -> void:
	# Create StaticBody2D with collision shapes for each wall strip
	var walls := StaticBody2D.new()
	walls.collision_layer = 1
	walls.collision_mask = 0
	add_child(walls)
	for rect: Rect2 in wall_tiles:
		var shape := CollisionShape2D.new()
		var rect_shape := RectangleShape2D.new()
		rect_shape.size = rect.size
		shape.shape = rect_shape
		shape.position = rect.position + rect.size / 2
		walls.add_child(shape)

func _draw() -> void:
	if world_tex == null:
		return
	# One baked pixel-art texture, scaled up nearest-neighbour (16px src -> 48px on screen).
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	draw_texture_rect(world_tex, Rect2(0, 0, cols * TILE_SIZE, rows * TILE_SIZE), false)

# ---------- Pixel-art tile baking ----------
# Each tile is authored at 16px and the whole map is baked into one Image at
# _ready, then drawn scaled. Detailed per-pixel work (brick seams, cracks,
# pebbles, ripples) reads as real pixel art instead of flat blocks. This is the
# port of the Phaser drawTile pass. Drop a real tileset in later by replacing
# _paint_tile with atlas blits.
const SRC_TILE := 16

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
	# Native fill_rect (C++) — far faster than per-pixel for the bulk fills.
	img.fill_rect(Rect2i(x, y, w, h), col)

func _paint_tile(img: Image, ox: int, oy: int, type: String, c: int, r: int) -> void:
	var s := (c * 73856093) ^ (r * 19349663)

	if type == "wall":
		_fillr(img, ox, oy, 16, 16, Color8(27, 20, 30))
		# Bevel: lit top-left, shadowed bottom-right -> chunky rock blocks
		_fillr(img, ox, oy, 16, 2, Color8(44, 31, 54))
		_fillr(img, ox, oy, 2, 16, Color8(44, 31, 54))
		_fillr(img, ox, oy + 14, 16, 2, Color8(14, 9, 18))
		_fillr(img, ox + 14, oy, 2, 16, Color8(14, 9, 18))
		# Brick seams, offset row to row
		_fillr(img, ox, oy + 7, 16, 1, Color8(16, 10, 20))
		if (r % 2) == 0:
			_fillr(img, ox + 8, oy, 1, 7, Color8(16, 10, 20))
		else:
			_fillr(img, ox + 4, oy + 8, 1, 6, Color8(16, 10, 20))
		# Mineral speckle
		if _trand(s, 1) < 0.22:
			_px(img, ox + 4 + int(_trand(s, 2) * 8), oy + 4 + int(_trand(s, 3) * 8), Color8(96, 52, 130))
		return

	if type == "puddle":
		_fillr(img, ox, oy, 16, 16, Color8(43, 22, 56))
		_fillr(img, ox + 2, oy + 3, 5, 1, Color8(61, 29, 78))
		_fillr(img, ox + 8, oy + 7, 5, 1, Color8(61, 29, 78))
		_fillr(img, ox + 3, oy + 11, 6, 1, Color8(61, 29, 78))
		_px(img, ox + 5, oy + 5, Color8(120, 64, 180))
		_px(img, ox + 11, oy + 10, Color8(120, 64, 180))
		return

	if type == "path":
		_fillr(img, ox, oy, 16, 16, Color8(74, 58, 38))
		for i in range(7):
			_px(img, ox + int(_trand(s, i + 1) * 16), oy + int(_trand(s, i + 8) * 16), Color8(94, 74, 48))
		_px(img, ox + 4, oy + 9, Color8(53, 39, 22))
		_px(img, ox + 11, oy + 3, Color8(53, 39, 22))
		return

	# floor — cave stone with variation + cracks + pebbles
	var v := _trand(s, 2)
	var base := Color8(38, 32, 40)
	if v >= 0.6 and v < 0.85:
		base = Color8(45, 37, 47)
	elif v >= 0.85:
		base = Color8(33, 27, 36)
	_fillr(img, ox, oy, 16, 16, base)
	# Subtle bevel for tiled depth
	_fillr(img, ox, oy, 16, 1, Color8(53, 43, 57))
	_fillr(img, ox, oy, 1, 16, Color8(53, 43, 57))
	_fillr(img, ox, oy + 15, 16, 1, Color8(24, 19, 27))
	_fillr(img, ox + 15, oy, 1, 16, Color8(24, 19, 27))
	# Crack
	if _trand(s, 4) < 0.22:
		_px(img, ox + 5, oy + 9, Color8(22, 17, 25))
		_px(img, ox + 6, oy + 10, Color8(22, 17, 25))
		_px(img, ox + 7, oy + 10, Color8(22, 17, 25))
	# Pebbles
	if _trand(s, 3) < 0.2:
		_px(img, ox + 3 + int(_trand(s, 4) * 9), oy + 3 + int(_trand(s, 5) * 9), Color8(64, 52, 68))
	if _trand(s, 6) < 0.08:
		_px(img, ox + 5 + int(_trand(s, 7) * 6), oy + 5 + int(_trand(s, 8) * 6), Color8(122, 90, 132))

func _process(_delta: float) -> void:
	# Player interaction (E key)
	if Input.is_action_just_pressed("interact"):
		_check_interaction()

func _check_interaction() -> void:
	var nearest: Interactable = null
	var nearest_dist := 999999.0
	for n in interactables_root.get_children():
		if n is Interactable:
			var d := player.global_position.distance_to(n.global_position)
			if d < 90 and d < nearest_dist:
				nearest = n
				nearest_dist = d
	if nearest and nearest.can_interact():
		nearest.trigger()

func _on_player_attack(facing: int, pos: Vector2) -> void:
	# Slash effect — directional cone hit
	var facing_angle: float
	match facing:
		Player.Facing.DOWN: facing_angle = PI / 2
		Player.Facing.UP: facing_angle = -PI / 2
		Player.Facing.LEFT: facing_angle = PI
		Player.Facing.RIGHT: facing_angle = 0
		_: facing_angle = PI / 2

	_spawn_slash(pos, facing_angle)

	# Reach matches the visible blade arc; cone slightly wider than the
	# visual so hits feel forgiving. Damage comes from the equipped weapon.
	const REACH := 70.0
	const HALF_ARC := PI / 2.4
	var dmg: int = GameState.get_attack_power()
	var hit_any := false
	for e in enemies_root.get_children():
		if not (e is Enemy): continue
		var enemy := e as Enemy
		var to_enemy: Vector2 = enemy.global_position - pos
		if to_enemy.length() > REACH: continue
		var ang: float = to_enemy.angle()
		var diff: float = wrapf(ang - facing_angle, -PI, PI)
		if abs(diff) > HALF_ARC: continue
		enemy.take_damage(dmg, pos)
		hit_any = true
	# A connected hit gives a tiny camera kick so the swing has weight.
	if hit_any:
		var cam := player.get_node_or_null("Camera2D")
		if cam:
			cam.offset = Vector2(randf_range(-3, 3), randf_range(-3, 3))
			var tw := create_tween()
			tw.tween_property(cam, "offset", Vector2.ZERO, 0.12)

func _spawn_slash(pos: Vector2, facing_angle: float) -> void:
	var slash := Node2D.new()
	slash.position = pos
	slash.rotation = facing_angle
	add_child(slash)
	slash.set_script(SlashScript)
	slash.set_process(true)

# Inline slash effect script
const SlashScript := preload("res://scripts/world/slash.gd")
