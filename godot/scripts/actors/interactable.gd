## 通用可交互点 —— 受伤矮人 / 图腾残片 / 矿井出口共用
##
## 用法：场景里建 Area2D + 这个脚本，配置 interact_type
class_name Interactable
extends Area2D

@export_enum("injured_dwarf", "totem_fragment", "mine_exit") var interact_type: String = "injured_dwarf"
@export var label: String = ""
@export var visual_color: Color = Color(0.4, 0.3, 0.2)

func _ready() -> void:
	add_to_group("interactable")
	queue_redraw()

func can_interact() -> bool:
	match interact_type:
		"injured_dwarf":
			return GameState.world_state.flags.first_dwarf_choice == ""
		"totem_fragment":
			return not GameState.world_state.flags.touched_totem_fragment
		"mine_exit":
			return true
	return false

func trigger() -> void:
	match interact_type:
		"injured_dwarf":
			_handle_injured_dwarf()
		"totem_fragment":
			_handle_totem()
		"mine_exit":
			_handle_exit()

func _handle_injured_dwarf() -> void:
	GameState.request_state_change({
		"type": "khah_whisper",
		"requested_by": "克哈低语",
		"target_id": "injured_dwarf",
		"reason": "玩家靠近第一个永久选择",
		"effects": [{"path": "flags.met_injured_dwarf", "value": true}]
	})
	GameState.set_active_choice({
		"id": "first_dwarf_choice",
		"title": "第一个永久选择",
		"body": "一个矮人倒在铁轨旁，手腕布条下有正在扩散的刺青。低语催你继续走。",
		"options": [
			{"id": "save", "label": "救他", "description": "理智稳定，但小镇可能承受感染风险。"},
			{"id": "abandon", "label": "抛下他", "description": "听从低语，安全离开。"},
			{"id": "kill", "label": "终结他", "description": "污染上升，猎人会认可这种冷酷。"}
		]
	})

func _handle_totem() -> void:
	var approved: bool = GameState.request_state_change({
		"type": "touch_totem_fragment",
		"requested_by": "图腾残片",
		"target_id": "mine_totem_fragment",
		"reason": "玩家触碰第一块封印残片",
		"effects": [
			{"path": "flags.touched_totem_fragment", "value": true},
			{"path": "vessel_awakening", "value": 2},
			{"path": "corruption", "value": min(100, GameState.world_state.corruption + 4)}
		]
	})
	if approved:
		var loot: Dictionary = LootGenerator.generate_loot("totem", GameState.world_state.world_tier)
		GameState.add_item(loot)
		GameState.set_vision(
			"res://assets/characters/dwarf_captain_vision.jpg",
			"矮人队长打开图腾的一瞬间，你看见紫色皮肤、黑色眼睛，以及一只虫子钻入王冠。"
		)
		GameState.set_dialogue({
			"speaker": "残响",
			"text": "矮人队长打开图腾的一瞬间，你看见紫色皮肤、黑色眼睛，以及一只虫子钻入王冠。",
			"tone": Types.TONE_MEMORY
		})

func _handle_exit() -> void:
	if GameState.world_state.flags.escaped_mine:
		GameState.set_dialogue({
			"speaker": "矿井出口",
			"text": "出口已经打开。下一版会从这里进入灰灯镇。",
			"tone": Types.TONE_MEMORY
		})
		return

	var approved: bool = GameState.request_state_change({
		"type": "escape_mine",
		"requested_by": "矿井出口",
		"target_id": "mine_exit",
		"reason": "玩家试图离开黑潮矿区",
		"effects": [{"path": "flags.escaped_mine", "value": true}]
	})
	if approved:
		GameState.set_dialogue({
			"speaker": "克哈低语",
			"text": "很好。现在去找那些还以为灯能挡住海的人。",
			"tone": Types.TONE_WHISPER
		})

func _draw() -> void:
	# Visual placeholder per type
	match interact_type:
		"injured_dwarf":
			# Lying dwarf
			draw_rect(Rect2(-20, -10, 40, 14), Color8(94, 52, 36), true)
			draw_rect(Rect2(-20, -4, 40, 4), Color8(31, 18, 10), true)
			draw_rect(Rect2(8, -16, 12, 10), Color8(219, 179, 138), true)
			draw_rect(Rect2(-10, 2, 14, 2), Color8(122, 29, 36), true)
		"totem_fragment":
			# Purple crystal shard
			draw_rect(Rect2(-2, -36, 4, 34), Color8(61, 31, 85), true)
			draw_rect(Rect2(-6, -28, 12, 20), Color8(88, 42, 120), true)
			draw_rect(Rect2(0, -36, 2, 30), Color8(164, 95, 209), true)
			draw_rect(Rect2(-8, -2, 16, 4), Color8(42, 21, 53), true)
		"mine_exit":
			# Cave arch
			draw_rect(Rect2(-30, -50, 60, 50), Color8(28, 20, 16), true)
			draw_rect(Rect2(-26, -46, 52, 46), Color8(8, 6, 8), true)
			draw_rect(Rect2(-6, -14, 12, 12), Color8(58, 111, 110), true)
