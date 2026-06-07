## 全局游戏状态单例 —— 合并 src/store/useGameStore.ts + src/core/worldState.ts
extends Node

signal profile_changed(profile: Dictionary)
signal world_state_changed(path: String, value: Variant)
signal combat_changed(combat: Dictionary)
signal inventory_changed(inventory: Array)
signal equipped_changed(equipped: Dictionary)
signal dialogue_opened(speaker: String, text: String, tone: String)
signal dialogue_closed()
signal choice_opened(choice: Dictionary)
signal choice_closed()
signal vision_opened(image_path: String, caption: String)
signal vision_closed()
signal log_appended(entry: String)

var profile: Dictionary = {}
var world_state: Dictionary = {}
var combat: Dictionary = {}
var inventory: Array = []
var equipped: Dictionary = {}
var dialogue: Dictionary = {}
var active_choice: Dictionary = {}
var vision: Dictionary = {}
var log: Array[String] = []

func _ready() -> void:
	_reset_to_defaults()

func _reset_to_defaults() -> void:
	profile = {}
	world_state = Types.make_default_world_state()
	combat = Types.make_default_combat()
	inventory = []
	equipped = {}
	dialogue = {}
	active_choice = {}
	vision = {}
	log = ["存档初始化。"]

func create_profile(new_profile: Dictionary) -> void:
	profile = new_profile.duplicate(true)
	world_state.flags.awakened_by_khah = true
	world_state.vessel_awakening = 1
	emit_signal("profile_changed", profile)
	emit_signal("world_state_changed", "vessel_awakening", 1)
	set_dialogue({
		"speaker": "克哈低语",
		"text": "%s，醒来。石头正在合拢，而你不该死在这里。" % profile.name,
		"tone": Types.TONE_WHISPER
	})
	_log("%s 在黑潮矿区苏醒。" % profile.name)

func set_combat(new_combat: Dictionary) -> void:
	combat = new_combat.duplicate(true)
	emit_signal("combat_changed", combat)

func set_dialogue(new_dialogue: Dictionary) -> void:
	dialogue = new_dialogue.duplicate(true)
	emit_signal("dialogue_opened", dialogue.speaker, dialogue.text, dialogue.get("tone", ""))

func close_dialogue() -> void:
	dialogue = {}
	emit_signal("dialogue_closed")

func set_active_choice(choice: Dictionary) -> void:
	active_choice = choice.duplicate(true)
	emit_signal("choice_opened", active_choice)

func close_choice() -> void:
	active_choice = {}
	emit_signal("choice_closed")

func set_vision(image_path: String, caption: String) -> void:
	vision = {"image": image_path, "caption": caption}
	emit_signal("vision_opened", image_path, caption)

func close_vision() -> void:
	vision = {}
	emit_signal("vision_closed")

func add_item(item: Dictionary) -> void:
	inventory.push_front(item)
	emit_signal("inventory_changed", inventory)
	_log("获得装备：%s" % item.name)

func equip_item(item_id: String) -> void:
	for item in inventory:
		if item.id == item_id:
			equipped[item.slot] = item.id
			emit_signal("equipped_changed", equipped)
			_log("装备：%s" % item.name)
			return

func add_gold(amount: int) -> void:
	world_state.gold += amount
	emit_signal("world_state_changed", "gold", world_state.gold)
	_log("获得 %d 金币。" % amount)

func request_state_change(request: Dictionary) -> bool:
	var decision: Dictionary = AidlcRules.approve_state_change(request, world_state)
	if not decision.approved:
		set_dialogue({
			"speaker": request.requested_by,
			"text": decision.reason,
			"tone": Types.TONE_WARNING
		})
		return false

	for effect in request.effects:
		_set_path(effect.path, effect.value)
	_log("世界状态变更：%s" % request.type)
	return true

func _set_path(path: String, value: Variant) -> void:
	if path.begins_with("flags."):
		var flag_name = path.substr(6)
		world_state.flags[flag_name] = value
		emit_signal("world_state_changed", path, value)
		return
	if world_state.has(path):
		world_state[path] = value
		emit_signal("world_state_changed", path, value)
		return
	push_warning("Unknown state path: %s" % path)

func reset_run() -> void:
	_reset_to_defaults()
	emit_signal("profile_changed", profile)
	emit_signal("world_state_changed", "*", null)
	emit_signal("combat_changed", combat)
	emit_signal("inventory_changed", inventory)
	emit_signal("equipped_changed", equipped)

func _log(entry: String) -> void:
	log.push_front(entry)
	if log.size() > 16:
		log.resize(16)
	emit_signal("log_appended", entry)
