## 事件日志面板 —— 最近 6 条
extends Control

@onready var list_root: VBoxContainer = $Panel/Margin/VBox/List

func _ready() -> void:
	GameState.log_appended.connect(_on_log_appended)
	_refresh()

func _on_log_appended(_entry: String) -> void:
	_refresh()

func _refresh() -> void:
	for child in list_root.get_children():
		child.queue_free()

	var entries: Array = GameState.log.slice(0, 6)
	var index := 1
	for entry in entries:
		var label := Label.new()
		label.text = "%d. %s" % [index, entry]
		label.add_theme_color_override("font_color", Color(0.85, 0.81, 0.73))
		label.add_theme_font_size_override("font_size", 13)
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		list_root.add_child(label)
		index += 1
