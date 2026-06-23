## 背包 UI —— 右侧栏，可滚动，点击物品装备
##
## 监听 GameState.inventory_changed 和 equipped_changed 自动刷新。
extends Control

@onready var list_root: VBoxContainer = $Panel/Margin/VBox/Scroll/ItemList
@onready var empty_label: Label = $Panel/Margin/VBox/Empty

const QUALITY_COLOR := {
	"broken": Color(0.55, 0.52, 0.46),
	"common": Color(0.88, 0.84, 0.74),
	"rare": Color(0.45, 0.60, 0.84),
	"corrupted": Color(0.61, 0.34, 0.65),
	"relic": Color(0.85, 0.71, 0.38),
	"mythic": Color(0.95, 0.83, 0.45)
}

const QUALITY_LABEL := {
	"broken": "破损",
	"common": "普通",
	"rare": "稀有",
	"corrupted": "污染",
	"relic": "遗物",
	"mythic": "神话"
}

func _ready() -> void:
	visible = false  # 默认隐藏，按 B 呼出
	GameState.inventory_changed.connect(_refresh)
	GameState.equipped_changed.connect(_on_equipped_changed)
	_refresh(GameState.inventory)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("toggle_inventory"):
		visible = not visible

func _on_equipped_changed(_eq: Dictionary) -> void:
	_refresh(GameState.inventory)

func _refresh(inventory: Array) -> void:
	for child in list_root.get_children():
		child.queue_free()

	empty_label.visible = inventory.is_empty()
	if inventory.is_empty():
		return

	for item in inventory.slice(0, 12):  # Cap UI to first 12; bag holds all
		var row := _build_row(item)
		list_root.add_child(row)

func _build_row(item: Dictionary) -> Control:
	var btn := Button.new()
	btn.custom_minimum_size = Vector2(0, 62)
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.clip_text = true

	var equipped_mark := ""
	if GameState.equipped.get(item.slot, "") == item.id:
		equipped_mark = " · 已装备"

	var top_line := "%s %s" % [item.name, equipped_mark]
	var affix_line := ""
	if item.affixes.size() > 0:
		var a: Dictionary = item.affixes[0]
		affix_line = "%s +%d" % [a.label, a.value]

	btn.text = "%s\n%s · 强度 %d  %s" % [
		top_line,
		QUALITY_LABEL.get(item.quality, item.quality),
		item.item_power,
		affix_line
	]

	# Border tint by quality
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.07, 0.08, 0.09, 0.92)
	style.border_color = QUALITY_COLOR.get(item.quality, Color.WHITE)
	style.border_width_left = 3
	style.content_margin_left = 12
	style.content_margin_right = 8
	style.content_margin_top = 6
	style.content_margin_bottom = 6
	style.corner_radius_top_left = 4
	style.corner_radius_top_right = 4
	style.corner_radius_bottom_left = 4
	style.corner_radius_bottom_right = 4
	btn.add_theme_stylebox_override("normal", style)
	btn.add_theme_stylebox_override("hover", style)
	btn.add_theme_color_override("font_color", Color(0.95, 0.92, 0.82))
	btn.add_theme_font_size_override("font_size", 13)

	btn.pressed.connect(_on_item_pressed.bind(item.id))
	return btn

func _on_item_pressed(item_id: String) -> void:
	GameState.equip_item(item_id)
