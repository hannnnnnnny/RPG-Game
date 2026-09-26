## 全局游戏状态单例 —— 合并 src/store/useGameStore.ts + src/core/worldState.ts
extends Node

signal profile_changed(profile: Dictionary)
signal world_state_changed(path: String, value: Variant)
signal combat_changed(combat: Dictionary)
signal inventory_changed(inventory: Array)
signal equipped_changed(equipped: Dictionary)
signal stats_changed(stats: Dictionary)
signal dialogue_opened(speaker: String, text: String, tone: String)
signal dialogue_closed()
signal choice_opened(choice: Dictionary)
signal choice_closed()
signal vision_opened(image_path: String, caption: String)
signal vision_closed()
signal log_appended(entry: String)
signal materials_changed(materials: Dictionary)

var profile: Dictionary = {}
var world_state: Dictionary = {}
var combat: Dictionary = {}
var inventory: Array = []
var equipped: Dictionary = {}
# Crafting materials, material id -> count (ids in Forge.MATERIAL_NAMES).
var materials: Dictionary = {}
var dialogue: Dictionary = {}
var active_choice: Dictionary = {}
var vision: Dictionary = {}
var log: Array[String] = []

# Transient (not saved): where the player should appear after a scene change,
# e.g. back at a building's door after leaving its interior.
var spawn_override: Vector2 = Vector2.ZERO
var has_spawn_override: bool = false

func set_spawn_override(pos: Vector2) -> void:
	spawn_override = pos
	has_spawn_override = true

func consume_spawn_override(default_pos: Vector2) -> Vector2:
	if has_spawn_override:
		has_spawn_override = false
		return spawn_override
	return default_pos

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
	materials = {}
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
	Audio.play_pickup()
	_schedule_save()

func equip_item(item_id: String) -> void:
	for item in inventory:
		if item.id == item_id:
			equipped[item.slot] = item.id
			emit_equipment_changed()
			_log("装备:%s" % item.name)
			_schedule_save()
			return

func add_gold(amount: int) -> void:
	world_state.gold += amount
	emit_signal("world_state_changed", "gold", world_state.gold)
	_log("获得 %d 金币。" % amount)
	_schedule_save()

## Kill reward: base gold for the source/tier, boosted by 金币掉落.
func add_kill_gold(source: String) -> void:
	var base := LootGenerator.gold_for_kill(source, int(world_state.world_tier))
	add_gold(CombatMath.apply_gold_find(base, get_stats()))

func add_materials(gained: Dictionary) -> void:
	for id in gained:
		materials[id] = int(materials.get(id, 0)) + int(gained[id])
	emit_signal("materials_changed", materials)
	_schedule_save()

func has_materials(cost: Dictionary) -> bool:
	for id in cost:
		if int(materials.get(id, 0)) < int(cost[id]):
			return false
	return true

## Deducts materials only if every one is affordable; returns success.
func spend_materials(cost: Dictionary) -> bool:
	if not has_materials(cost):
		return false
	for id in cost:
		materials[id] = int(materials[id]) - int(cost[id])
	emit_signal("materials_changed", materials)
	_schedule_save()
	return true

## Applies the death penalty and returns the gold lost.
func on_player_down() -> int:
	var lost := CombatMath.death_gold_loss(int(world_state.gold))
	if lost > 0:
		spend_gold(lost)
	_log("倒下了。遗失 %d 金币。" % lost)
	set_dialogue({
		"speaker": "克哈低语",
		"text": "死亡在这里没有耐心。%s 枚金币留在了黑暗里——站起来，再走一次。" % lost,
		"tone": Types.TONE_WHISPER
	})
	return lost

# Spend gold if affordable; returns success.
func spend_gold(amount: int) -> bool:
	if world_state.gold < amount:
		return false
	world_state.gold -= amount
	emit_signal("world_state_changed", "gold", world_state.gold)
	_schedule_save()
	return true

# Remove an item by id (also unequips it if equipped).
func remove_item(item_id: String) -> Dictionary:
	for i in range(inventory.size()):
		if inventory[i].id == item_id:
			var item: Dictionary = inventory[i]
			inventory.remove_at(i)
			if equipped.get(item.slot, "") == item_id:
				equipped.erase(item.slot)
				emit_equipment_changed()
			emit_signal("inventory_changed", inventory)
			_schedule_save()
			return item
	return {}

# ============ Forge actions (rules live in Forge; this pays + stores) ============

var _forge_rng := RandomNumberGenerator.new()

func find_item(item_id: String) -> Dictionary:
	for item in inventory:
		if item.id == item_id:
			return item
	return {}

func _replace_item(updated: Dictionary) -> void:
	for i in range(inventory.size()):
		if inventory[i].id == updated.id:
			inventory[i] = updated
			break
	emit_signal("inventory_changed", inventory)
	if equipped.get(updated.slot, "") == updated.id:
		emit_equipment_changed()
	_schedule_save()

func forge_upgrade(item_id: String) -> bool:
	var item := find_item(item_id)
	if item.is_empty() or not Forge.can_upgrade(item):
		return false
	if not spend_gold(Forge.upgrade_cost(item)):
		return false
	var out := Forge.upgraded(item)
	_replace_item(out)
	_log("强化 %s → +%d" % [item.name, out.upgrade_level])
	return true

## Pays for a reroll and returns the offered affix ({} if not allowed).
## Must be followed by forge_resolve_reroll to keep or take the offer.
func forge_reroll_offer(item_id: String, affix_index: int) -> Dictionary:
	var item := find_item(item_id)
	if item.is_empty() or not Forge.can_reroll(item, affix_index):
		return {}
	if not spend_gold(Forge.reroll_cost(item)):
		return {}
	return Forge.reroll_offer(item, _forge_rng)

func forge_resolve_reroll(item_id: String, affix_index: int, offer: Dictionary, accept: bool) -> void:
	var item := find_item(item_id)
	if item.is_empty():
		return
	_replace_item(Forge.resolve_reroll(item, affix_index, offer, accept))
	_log("重铸 %s：%s" % [item.name, "接受新词条" if accept else "保留旧词条"])

func forge_lock(item_id: String, affix_index: int) -> bool:
	var item := find_item(item_id)
	if item.is_empty() or not spend_materials(Forge.LOCK_MATERIAL):
		return false
	_replace_item(Forge.locked(item, affix_index))
	_log("锁定 %s 的一条词条。" % item.name)
	return true

## Breaks an item into materials; returns what was gained.
func salvage_item(item_id: String) -> Dictionary:
	var item := remove_item(item_id)
	if item.is_empty():
		return {}
	var gained := Forge.salvage_yield(item)
	add_materials(gained)
	_log("分解 %s。" % item.name)
	return gained

# Kept as an alias of StatCalc.BASE_ATTACK for older callers.
const BASE_ATTACK := StatCalc.BASE_ATTACK

## Combat stats derived from the whole loadout (see StatCalc).
func get_stats() -> Dictionary:
	return StatCalc.derive(inventory, equipped)

## Equipment and the stats derived from it always change together.
func emit_equipment_changed() -> void:
	emit_signal("equipped_changed", equipped)
	emit_signal("stats_changed", get_stats())

func get_attack_power() -> int:
	return int(get_stats().attack)

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
	emit_equipment_changed()
	emit_signal("materials_changed", materials)

func _log(entry: String) -> void:
	log.push_front(entry)
	if log.size() > 16:
		log.resize(16)
	emit_signal("log_appended", entry)
