## HUD —— 左上 6 个状态条 + 右上 disi 头像
extends Control

@onready var health_bar: ProgressBar = $Panel/VBox/HealthBar
@onready var stamina_bar: ProgressBar = $Panel/VBox/StaminaBar
@onready var focus_bar: ProgressBar = $Panel/VBox/FocusBar
@onready var health_label: Label = $Panel/VBox/HealthLabel
@onready var stamina_label: Label = $Panel/VBox/StaminaLabel
@onready var focus_label: Label = $Panel/VBox/FocusLabel
@onready var corruption_label: Label = $Panel/VBox/Stats/CorruptionLabel
@onready var awakening_label: Label = $Panel/VBox/Stats/AwakeningLabel
@onready var gold_label: Label = $Panel/VBox/GoldLabel
@onready var disi_portrait: TextureRect = $DisiAvatar/HBox/Portrait
@onready var disi_name: Label = $DisiAvatar/HBox/VBox/Name
@onready var disi_status: Label = $DisiAvatar/HBox/VBox/Status

const DISI_STAGE1 := preload("res://assets/characters/disi_stage1.jpg")
const DISI_STAGE2 := preload("res://assets/characters/disi_stage2.jpg")
const DISI_STAGE3 := preload("res://assets/characters/disi_stage3.jpg")

# Pixelated versions, built once, so the avatar matches the pixel world.
var _px_stage1: Texture2D
var _px_stage2: Texture2D
var _px_stage3: Texture2D

func _ready() -> void:
	_px_stage1 = Types.pixelate(DISI_STAGE1, 48)
	_px_stage2 = Types.pixelate(DISI_STAGE2, 48)
	_px_stage3 = Types.pixelate(DISI_STAGE3, 48)
	GameState.combat_changed.connect(_on_combat_changed)
	GameState.world_state_changed.connect(_on_world_changed)
	GameState.profile_changed.connect(_on_profile_changed)
	_refresh_all()

func _refresh_all() -> void:
	_on_combat_changed(GameState.combat)
	_on_world_changed("*", null)
	_on_profile_changed(GameState.profile)

func _on_combat_changed(c: Dictionary) -> void:
	health_bar.max_value = c.max_health
	health_bar.value = c.health
	stamina_bar.max_value = c.max_stamina
	stamina_bar.value = c.stamina
	focus_bar.max_value = c.max_focus
	focus_bar.value = c.focus
	# Numbers right in the labels so HP is readable at a glance.
	health_label.text = "生命 %d/%d" % [int(c.health), int(c.max_health)]
	stamina_label.text = "体力 %d/%d" % [int(c.stamina), int(c.max_stamina)]
	focus_label.text = "专注 %d/%d" % [int(c.focus), int(c.max_focus)]

func _on_world_changed(_path: String, _value: Variant) -> void:
	corruption_label.text = "污染 %d" % GameState.world_state.corruption
	awakening_label.text = "觉醒 %d" % GameState.world_state.vessel_awakening
	gold_label.text = "%d 金币" % GameState.world_state.gold
	_update_disi_stage()

func _on_profile_changed(profile: Dictionary) -> void:
	if profile.has("name"):
		disi_name.text = profile.name
	_update_disi_stage()

func _update_disi_stage() -> void:
	var c: int = GameState.world_state.corruption
	var stage_label := ""
	if c <= 25:
		disi_portrait.texture = _px_stage1
		stage_label = "神志清明"
	elif c <= 55:
		disi_portrait.texture = _px_stage2
		stage_label = "渗透中"
	else:
		disi_portrait.texture = _px_stage3
		stage_label = "意志崩碎"
	disi_status.text = "污染 %d · %s" % [c, stage_label]
