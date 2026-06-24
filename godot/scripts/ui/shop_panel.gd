## 商店面板 —— 左"购买"(商店存货) / 右"出售"(玩家背包)。
## open(stock) 显示；ESC 或关闭按钮收起。买卖即时更新金币与列表。
extends Control

@onready var gold_label: Label = $Panel/VBox/Header/Gold
@onready var buy_list: VBoxContainer = $Panel/VBox/Cols/BuyCol/Scroll/List
@onready var sell_list: VBoxContainer = $Panel/VBox/Cols/SellCol/Scroll/List
@onready var close_btn: Button = $Panel/VBox/Header/Close

const QUALITY_COLOR := {
	"broken": Color(0.55, 0.52, 0.46), "common": Color(0.88, 0.84, 0.74),
	"rare": Color(0.45, 0.60, 0.84), "corrupted": Color(0.61, 0.34, 0.65),
	"relic": Color(0.85, 0.71, 0.38), "mythic": Color(0.95, 0.83, 0.45)
}

var _stock: Array = []

func _ready() -> void:
	visible = false
	close_btn.pressed.connect(close)
	GameState.world_state_changed.connect(_on_world_changed)
	GameState.inventory_changed.connect(func(_i): if visible: _refresh())

func open(stock: Array) -> void:
	_stock = stock
	visible = true
	_refresh()

func close() -> void:
	visible = false

func _input(event: InputEvent) -> void:
	if visible and event.is_action_pressed("ui_cancel"):
		close()
		get_viewport().set_input_as_handled()

func _on_world_changed(path: String, _v: Variant) -> void:
	if visible and path == "gold":
		_update_gold()

func _update_gold() -> void:
	gold_label.text = "金币 %d" % GameState.world_state.gold

func _refresh() -> void:
	_update_gold()
	for c in buy_list.get_children():
		c.queue_free()
	for c in sell_list.get_children():
		c.queue_free()
	for item in _stock:
		buy_list.add_child(_make_row(item, true))
	if GameState.inventory.is_empty():
		var empty := Label.new()
		empty.text = "背包是空的。"
		empty.add_theme_color_override("font_color", Color(0.7, 0.66, 0.59))
		sell_list.add_child(empty)
	else:
		for item in GameState.inventory:
			sell_list.add_child(_make_row(item, false))

func _make_row(item: Dictionary, is_buy: bool) -> Control:
	var price: int = LootGenerator.item_value(item) if is_buy else LootGenerator.sell_value(item)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)

	var info := VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info.add_theme_constant_override("separation", 1)
	var name_lbl := Label.new()
	name_lbl.text = item.name
	name_lbl.add_theme_color_override("font_color", QUALITY_COLOR.get(item.quality, Color.WHITE))
	name_lbl.add_theme_font_size_override("font_size", 13)
	info.add_child(name_lbl)
	var sub := Label.new()
	var affix_txt := ""
	if item.affixes.size() > 0:
		affix_txt = "  %s+%d" % [item.affixes[0].label, item.affixes[0].value]
	sub.text = "强度 %d%s" % [item.item_power, affix_txt]
	sub.add_theme_color_override("font_color", Color(0.66, 0.62, 0.55))
	sub.add_theme_font_size_override("font_size", 11)
	info.add_child(sub)
	row.add_child(info)

	var btn := Button.new()
	btn.custom_minimum_size = Vector2(86, 34)
	btn.text = ("买 %d" % price) if is_buy else ("卖 %d" % price)
	if is_buy and GameState.world_state.gold < price:
		btn.disabled = true
	if is_buy:
		btn.pressed.connect(_buy.bind(item))
	else:
		btn.pressed.connect(_sell.bind(item))
	row.add_child(btn)
	return row

func _buy(item: Dictionary) -> void:
	var price: int = LootGenerator.item_value(item)
	if GameState.spend_gold(price):
		# Give a fresh copy (new id) so buying twice doesn't collide.
		var copy: Dictionary = item.duplicate(true)
		copy.id = "%s_buy_%x" % [item.id, randi()]
		GameState.add_item(copy)
		Audio.play_pickup()
		_stock.erase(item)
		_refresh()

func _sell(item: Dictionary) -> void:
	var price: int = LootGenerator.sell_value(item)
	GameState.remove_item(item.id)
	GameState.add_gold(price)
	Audio.play_pickup()
	_refresh()
