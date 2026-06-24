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

	var escaped: bool = flags.get("escaped_mine", false)
	var touched: bool = flags.get("touched_totem_fragment", false)
	var beat_grom: bool = flags.get("defeated_grom", false)

	if not escaped:
		objectives.append("逃出黑潮矿区")
		if str(flags.get("first_dwarf_choice", "")) == "":
			objectives.append("处理受伤矮人的命运")
		if not touched:
			objectives.append("触碰图腾残片")
		if touched and not beat_grom:
			objectives.append("击败黑腕队长·格罗姆")
		if beat_grom and not escaped:
			objectives.append("前往矿井出口")
	else:
		objectives.append("灰灯镇：与镇民交谈")
		objectives.append("灰灯镇的秘密还在雾里（下一版）")

	for obj in objectives:
		var label := Label.new()
		label.text = "·  " + obj
		label.add_theme_color_override("font_color", Color(0.87, 0.83, 0.74))
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		list_root.add_child(label)
