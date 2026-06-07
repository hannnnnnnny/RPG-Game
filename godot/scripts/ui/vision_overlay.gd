## 全屏幻象 overlay —— 图腾触发后矮人队长画面
extends Control

@onready var image: TextureRect = $Panel/VBox/Image
@onready var caption: Label = $Panel/VBox/Caption
@onready var close_btn: Button = $Panel/VBox/CloseBtn

func _ready() -> void:
	visible = false
	GameState.vision_opened.connect(_on_vision_opened)
	GameState.vision_closed.connect(_on_vision_closed)
	close_btn.pressed.connect(GameState.close_vision)

func _on_vision_opened(image_path: String, cap: String) -> void:
	var tex: Texture2D = load(image_path)
	if tex:
		image.texture = tex
	caption.text = cap
	visible = true

func _on_vision_closed() -> void:
	visible = false
