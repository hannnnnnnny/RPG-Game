## 角色创建界面 —— 输入名字 → 进入矿区
extends Control

@onready var name_input: LineEdit = $Panel/VBox/NameInput
@onready var enter_button: Button = $Panel/VBox/EnterButton

func _ready() -> void:
	enter_button.pressed.connect(_on_enter_pressed)
	name_input.text_submitted.connect(_on_name_submitted)

func _on_name_submitted(_text: String) -> void:
	_on_enter_pressed()

func _on_enter_pressed() -> void:
	var name := name_input.text.strip_edges()
	if name == "":
		name = "无名者"
	GameState.create_profile({
		"name": name,
		"gender": Types.GENDER_UNKNOWN,
		"appearance": Types.APPEARANCE_ASHEN
	})
	get_tree().change_scene_to_file("res://scenes/world/MineIntro.tscn")
