## 底部对话框 —— 听 GameState.dialogue_opened
extends Control

@onready var speaker_label: Label = $Panel/HBox/Content/Speaker
@onready var text_label: Label = $Panel/HBox/Content/Text
@onready var portrait: TextureRect = $Panel/HBox/Portrait
@onready var close_button: Button = $Panel/HBox/Close

const KHAH_PORTRAIT := preload("res://assets/characters/khah.jpg")

func _ready() -> void:
	visible = false
	GameState.dialogue_opened.connect(_on_dialogue_opened)
	GameState.dialogue_closed.connect(_on_dialogue_closed)
	close_button.pressed.connect(GameState.close_dialogue)

func _on_dialogue_opened(speaker: String, text: String, tone: String) -> void:
	speaker_label.text = speaker
	text_label.text = text
	# Show Khah portrait when speaker is 克哈低语
	if speaker == "克哈低语":
		portrait.texture = KHAH_PORTRAIT
		portrait.visible = true
	else:
		portrait.visible = false
	# Tint border based on tone
	var panel: Panel = $Panel
	var style: StyleBoxFlat = panel.get_theme_stylebox("panel") as StyleBoxFlat
	if style == null:
		style = StyleBoxFlat.new()
	style.border_color = _color_for_tone(tone)
	style.border_width_left = 2
	style.border_width_right = 2
	style.border_width_top = 2
	style.border_width_bottom = 2
	style.bg_color = Color(0.05, 0.06, 0.07, 0.92)
	panel.add_theme_stylebox_override("panel", style)
	visible = true

func _on_dialogue_closed() -> void:
	visible = false

func _color_for_tone(tone: String) -> Color:
	match tone:
		"whisper": return Color8(144, 80, 163)
		"warning": return Color8(185, 72, 58)
		"memory": return Color8(217, 182, 98)
		_: return Color8(217, 182, 98)
