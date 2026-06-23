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

func _ready() -> void:
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
		var dx := (cx - p.cx) / float(p.rx)
		var dy := (cy - p.cy) / float(p.ry)
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
	# Draw tile map procedurally
	var cols := int(ceil(WORLD_W / float(TILE_SIZE)))
	var rows := int(ceil(WORLD_H / float(TILE_SIZE)))
	for r in range(rows):
		for c in range(cols):
			var t := _classify_tile(c, r)
			var pos := Vector2(c * TILE_SIZE, r * TILE_SIZE)
			var color: Color
			match t:
				"wall":
					color = Color8(27, 20, 24)
				"puddle":
					color = Color8(43, 22, 56)
				"path":
					color = Color8(74, 58, 38)
				_:
					color = Color8(38, 32, 36)
			draw_rect(Rect2(pos, Vector2(TILE_SIZE, TILE_SIZE)), color, true)
			# Edge highlight on floor for subtle 3D feel
			if t == "floor":
				draw_rect(Rect2(pos, Vector2(TILE_SIZE, 2)), Color8(53, 42, 50), true)

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

	const REACH := 96.0
	const HALF_ARC := PI / 2.4
	for e in enemies_root.get_children():
		if not (e is Enemy): continue
		var to_enemy := e.global_position - pos
		if to_enemy.length() > REACH: continue
		var ang := to_enemy.angle()
		var diff := wrapf(ang - facing_angle, -PI, PI)
		if abs(diff) > HALF_ARC: continue
		e.take_damage(18)

func _spawn_slash(pos: Vector2, facing_angle: float) -> void:
	var slash := Node2D.new()
	slash.position = pos
	slash.rotation = facing_angle
	add_child(slash)
	slash.set_script(SlashScript)
	slash.set_process(true)

# Inline slash effect script
const SlashScript := preload("res://scripts/world/slash.gd")
