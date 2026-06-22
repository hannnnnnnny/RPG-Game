## 入口界面 ——
##   有存档：显示 "继续旅程 / 重新开始" 两个按钮
##   无存档：显示名字输入 + "进入矿井" 按钮
extends Control

@onready var fresh_panel: Control = $FreshPanel
@onready var continue_panel: Control = $ContinuePanel

@onready var name_input: LineEdit = $FreshPanel/Margin/VBox/NameInput
@onready var enter_button: Button = $FreshPanel/Margin/VBox/EnterButton

@onready var continue_name_label: Label = $ContinuePanel/Margin/VBox/SavedName
@onready var continue_button: Button = $ContinuePanel/Margin/VBox/ContinueButton
@onready var reset_button: Button = $ContinuePanel/Margin/VBox/ResetButton

func _ready() -> void:
	enter_button.pressed.connect(_on_enter_pressed)
	name_input.text_submitted.connect(_on_name_submitted)
	continue_button.pressed.connect(_on_continue_pressed)
	reset_button.pressed.connect(_on_reset_pressed)

	if SaveSystem.has_save() and SaveSystem.load_save():
		_show_continue()
	else:
		_show_fresh()

func _on_name_submitted(_text: String) -> void:
	_on_enter_pressed()

func _show_fresh() -> void:
	fresh_panel.visible = true
	continue_panel.visible = false

func _show_continue() -> void:
	fresh_panel.visible = false
	continue_panel.visible = true
	var saved_name: String = GameState.profile.get("name", "无名者")
	var tier: int = GameState.world_state.get("world_tier", 1)
	var corruption: int = GameState.world_state.get("corruption", 0)
	continue_name_label.text = "%s · 世界等级 %d · 污染 %d" % [saved_name, tier, corruption]

func _on_enter_pressed() -> void:
	var player_name := name_input.text.strip_edges()
	if player_name == "":
		player_name = "无名者"
	GameState.create_profile({
		"name": player_name,
		"gender": Types.GENDER_UNKNOWN,
		"appearance": Types.APPEARANCE_ASHEN
	})
	get_tree().change_scene_to_file("res://scenes/world/MineIntro.tscn")

func _on_continue_pressed() -> void:
	get_tree().change_scene_to_file("res://scenes/world/MineIntro.tscn")

func _on_reset_pressed() -> void:
	GameState.reset_run()
	_show_fresh()
