## 任务面板 —— 从 world_state.flags 推导当前目标
extends Control

@onready var list_root: VBoxContainer = $Panel/Margin/VBox/List

func _ready() -> void:
	visible = false  # 默认隐藏，按 M 呼出
	GameState.world_state_changed.connect(_on_world_changed)
	_refresh()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("toggle_quest"):
		visible = not visible

func _on_world_changed(_path: String, _value: Variant) -> void:
	_refresh()

func _refresh() -> void:
	for child in list_root.get_children():
		child.queue_free()

	var flags: Dictionary = GameState.world_state.flags
	var objectives: Array[String] = []

	objectives.append("逃出黑潮矿区")

	if str(flags.get("first_dwarf_choice", "")) == "":
		objectives.append("处理受伤矮人的命运")

	if not flags.get("touched_totem_fragment", false):
		objectives.append("触碰图腾残片")

	if flags.get("touched_totem_fragment", false) and not flags.get("escaped_mine", false):
		objectives.append("前往矿井出口")

	if flags.get("escaped_mine", false):
		objectives.append("灰灯镇的路还在雾里")

	for obj in objectives:
		var label := Label.new()
		label.text = "·  " + obj
		label.add_theme_color_override("font_color", Color(0.87, 0.83, 0.74))
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		list_root.add_child(label)
