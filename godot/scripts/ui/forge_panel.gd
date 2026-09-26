## 铁匠铺面板 —— 强化 / 重铸 / 锁定 / 分解（规则见 Forge，扣费见 GameState）。
## Opened via the "forge_panel" group (e.g. the town anvil). While a reroll
## offer is pending, every other action is disabled until the player keeps
## the old affix or takes the new one — the gold is already spent.
extends Control

@onready var wallet_label: Label = $Panel/VBox/Header/Wallet
@onready var close_btn: Button = $Panel/VBox/Header/Close
@onready var item_list: VBoxContainer = $Panel/VBox/Cols/ItemCol/Scroll/List
@onready var detail: VBoxContainer = $Panel/VBox/Cols/Detail
@onready var status_label: Label = $Panel/VBox/Status

var _selected_id: String = ""
var _pending: Dictionary = {}  # {"index": int, "offer": Dictionary}
var _confirm_salvage: bool = false

func _ready() -> void:
	visible = false
	add_to_group("forge_panel")
	close_btn.pressed.connect(close)
	GameState.inventory_changed.connect(func(_i): _refresh_if_open())
	GameState.materials_changed.connect(func(_m): _refresh_if_open())
	GameState.world_state_changed.connect(func(_p, _v): _refresh_if_open())

func open() -> void:
	visible = true
	_confirm_salvage = false
	if GameState.find_item(_selected_id).is_empty():
		_selected_id = GameState.inventory[0].id if not GameState.inventory.is_empty() else ""
	_refresh()

func close() -> void:
	if not _pending.is_empty():
		_set_status("先决定保留还是接受新词条。", true)
		return
	visible = false

func _input(event: InputEvent) -> void:
	if visible and event.is_action_pressed("ui_cancel"):
		close()
		get_viewport().set_input_as_handled()

func _refresh_if_open() -> void:
	if visible:
		_refresh()

func _refresh() -> void:
	wallet_label.text = _wallet_text()
	_build_item_list()
	_build_detail()

func _wallet_text() -> String:
	var parts := ["金币 %d" % GameState.world_state.gold]
	for id in Forge.MATERIAL_NAMES:
		parts.push_back("%s %d" % [Forge.MATERIAL_NAMES[id], int(GameState.materials.get(id, 0))])
	return " · ".join(parts)

func _set_status(text: String, warn: bool = false) -> void:
	status_label.text = text
	status_label.add_theme_color_override("font_color", UiPalette.DANGER if warn else UiPalette.TEXT_MUTED)

func _clear(node: Node) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()

# ---------------- Left: item list ----------------

func _build_item_list() -> void:
	_clear(item_list)
	if GameState.inventory.is_empty():
		item_list.add_child(_label("背包是空的。去矿里带点东西回来。", UiPalette.TEXT_MUTED, 12))
		return
	for item in GameState.inventory:
		item_list.add_child(_item_button(item))

func _item_button(item: Dictionary) -> Button:
	var btn := Button.new()
	btn.text = UiPalette.item_title(item)
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.clip_text = true
	btn.toggle_mode = true
	btn.button_pressed = item.id == _selected_id
	btn.disabled = not _pending.is_empty()
	btn.add_theme_color_override("font_color", UiPalette.quality_color(item))
	btn.add_theme_font_size_override("font_size", 13)
	btn.pressed.connect(_select.bind(item.id))
	return btn

func _select(item_id: String) -> void:
	_selected_id = item_id
	_confirm_salvage = false
	_set_status("")
	_refresh()

# ---------------- Right: detail ----------------

func _build_detail() -> void:
	_clear(detail)
	var item := GameState.find_item(_selected_id)
	if item.is_empty():
		detail.add_child(_label("未选择装备。", UiPalette.TEXT_MUTED, 13))
		return
	detail.add_child(_label(UiPalette.item_title(item), UiPalette.quality_color(item), 17))
	detail.add_child(_label("%s · 强度 %d · 已重铸 %d 次" % [
		UiPalette.quality_label(item), int(item.item_power), int(item.get("reroll_count", 0))
	], UiPalette.TEXT_MUTED, 12))
	for i in range(item.affixes.size()):
		detail.add_child(_affix_row(item, i))
	if not _pending.is_empty():
		detail.add_child(_offer_box(item))
	detail.add_child(_action_row(item))

func _affix_value(item: Dictionary, affix: Dictionary) -> int:
	return int(round(float(affix.value) * StatCalc.upgrade_multiplier(item)))

func _affix_row(item: Dictionary, index: int) -> Control:
	var affix: Dictionary = item.affixes[index]
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var locked := Forge.is_locked(item, index)
	var text := "%s%s +%d" % ["[锁] " if locked else "", affix.label, _affix_value(item, affix)]
	var lbl := _label(text, UiPalette.TEXT, 13)
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(lbl)
	var reroll := _button("重铸 %d" % Forge.reroll_cost(item), _reroll.bind(index))
	reroll.disabled = locked or not _pending.is_empty() \
		or GameState.world_state.gold < Forge.reroll_cost(item)
	row.add_child(reroll)
	var lock := _button("锁定", _lock.bind(index))
	lock.tooltip_text = "消耗 1 稀有材料。锁定后此词条不会被重铸，但重铸更贵。"
	lock.disabled = locked or not _pending.is_empty() or not GameState.has_materials(Forge.LOCK_MATERIAL)
	row.add_child(lock)
	return row

func _offer_box(item: Dictionary) -> Control:
	var box := VBoxContainer.new()
	var old: Dictionary = item.affixes[_pending.index]
	var offer: Dictionary = _pending.offer
	box.add_child(_label("旧：%s +%d   →   新：%s +%d" % [
		old.label, _affix_value(item, old), offer.label, _affix_value(item, offer)
	], UiPalette.GOLD, 14))
	var row := HBoxContainer.new()
	row.add_child(_button("保留旧词条", _resolve.bind(false)))
	row.add_child(_button("接受新词条", _resolve.bind(true)))
	box.add_child(row)
	return box

func _action_row(item: Dictionary) -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	var busy := not _pending.is_empty()
	if Forge.can_upgrade(item):
		var cost := Forge.upgrade_cost(item)
		var up := _button("强化 → +%d（%d 金）" % [int(item.get("upgrade_level", 0)) + 1, cost], _upgrade)
		up.disabled = busy or GameState.world_state.gold < cost
		row.add_child(up)
	else:
		row.add_child(_label("已强化至上限 +%d" % Forge.MAX_UPGRADE, UiPalette.GOLD, 13))
	if _confirm_salvage:
		row.add_child(_button("确认分解", _salvage))
		row.add_child(_button("取消", _cancel_salvage))
	else:
		var s := _button("分解", _ask_salvage)
		s.disabled = busy
		row.add_child(s)
	return row

# ---------------- Actions ----------------

func _upgrade() -> void:
	if GameState.forge_upgrade(_selected_id):
		Audio.play_hit()
		_set_status("锤声落定。强化成功。")
	else:
		_set_status("金币不够。", true)

func _reroll(index: int) -> void:
	var offer := GameState.forge_reroll_offer(_selected_id, index)
	if offer.is_empty():
		_set_status("无法重铸这条词条。", true)
		return
	_pending = {"index": index, "offer": offer}
	_set_status("新词条已经出炉。保留旧的，还是换上新的？")
	_refresh()

func _resolve(accept: bool) -> void:
	var p := _pending
	_pending = {}
	GameState.forge_resolve_reroll(_selected_id, p.index, p.offer, accept)
	_set_status("接受了新词条。" if accept else "保留了旧词条。")

func _lock(index: int) -> void:
	if GameState.forge_lock(_selected_id, index):
		_set_status("词条已锁定。")
	else:
		_set_status("需要 1 稀有材料。", true)

func _ask_salvage() -> void:
	_confirm_salvage = true
	var item := GameState.find_item(_selected_id)
	_set_status("分解后无法恢复。将得到：%s" % _yield_text(Forge.salvage_yield(item)), true)
	_refresh()

func _cancel_salvage() -> void:
	_confirm_salvage = false
	_set_status("")
	_refresh()

func _salvage() -> void:
	_confirm_salvage = false
	var gained := GameState.salvage_item(_selected_id)
	_selected_id = GameState.inventory[0].id if not GameState.inventory.is_empty() else ""
	_set_status("分解完成：%s" % _yield_text(gained))
	_refresh()

func _yield_text(mats: Dictionary) -> String:
	var parts: Array[String] = []
	for id in mats:
		parts.push_back("%s ×%d" % [Forge.MATERIAL_NAMES.get(id, id), int(mats[id])])
	return "、".join(parts)

# ---------------- Widgets ----------------

func _label(text: String, color: Color, size: int) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_color_override("font_color", color)
	l.add_theme_font_size_override("font_size", size)
	return l

func _button(text: String, cb: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(0, 30)
	b.add_theme_font_size_override("font_size", 12)
	b.pressed.connect(cb)
	return b
