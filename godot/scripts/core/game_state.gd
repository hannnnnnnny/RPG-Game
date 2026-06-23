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

# Debounced autosave: any state-changing call resets the timer; when it fires,
# we flush to disk. Avoids hammering the filesystem mid-combat.
var _save_timer: Timer
const SAVE_DEBOUNCE_SECONDS := 1.2

func _ready() -> void:
	_reset_to_defaults()
	_save_timer = Timer.new()
	_save_timer.one_shot = true
	_save_timer.wait_time = SAVE_DEBOUNCE_SECONDS
	_save_timer.timeout.connect(_flush_save)
	add_child(_save_timer)

func _schedule_save() -> void:
	# Skip until a profile exists (avoids saving the boot-time blank state).
	if profile.is_empty():
		return
	_save_timer.stop()
	_save_timer.start()

func _flush_save() -> void:
	if profile.is_empty():
		return
	SaveSystem.save()

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
	_schedule_save()

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
	_schedule_save()

func equip_item(item_id: String) -> void:
	for item in inventory:
		if item.id == item_id:
			equipped[item.slot] = item.id
			emit_signal("equipped_changed", equipped)
			_log("装备:%s" % item.name)
			_schedule_save()
			return

func add_gold(amount: int) -> void:
	world_state.gold += amount
	emit_signal("world_state_changed", "gold", world_state.gold)
	_log("获得 %d 金币。" % amount)
	_schedule_save()

# 玩家近战攻击力：空手基础 8，装备主手武器时加上其 attack 类词条之和。
# 例：空手 8 → 打 20 血怪需 3 下，不秒杀。装好武器后更高。
const BASE_ATTACK := 8

func get_attack_power() -> int:
	var weapon_id: String = equipped.get(Types.SLOT_MAIN_HAND, "")
	if weapon_id == "":
		return BASE_ATTACK
	for item in inventory:
		if item.id == weapon_id:
			var bonus := 0
			for affix in item.affixes:
				if affix.category == Types.AFFIX_ATTACK:
					bonus += int(affix.value)
			return BASE_ATTACK + bonus
	return BASE_ATTACK

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
	_schedule_save()
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
	SaveSystem.delete_save()
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
