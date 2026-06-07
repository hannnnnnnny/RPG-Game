## 装备掉落 —— 等价 src/core/lootGenerator.ts
extends Node

const SLOTS := ["mainHand", "chest", "hands", "boots", "ring", "totem"]

const BASE_NAMES := {
	"mainHand": ["矿工断刃", "灰灯短剑", "黑潮凿斧"],
	"offHand": ["破裂法器"],
	"head": ["污灰兜帽"],
	"chest": ["矿井皮甲", "盐石胸甲"],
	"hands": ["裂岩手套", "旧皮手套"],
	"boots": ["逃亡者旧靴", "灰泥长靴"],
	"amulet": ["残响项链"],
	"ring": ["黑腕戒指", "灰灯铜戒"],
	"totem": ["图腾碎屑", "矿井护符"]
}

const AFFIX_POOL := [
	{"label": "近战伤害", "category": "attack"},
	{"label": "暴击率", "category": "attack"},
	{"label": "对感染者伤害", "category": "attack"},
	{"label": "最大生命", "category": "defense"},
	{"label": "黑潮抗性", "category": "defense"},
	{"label": "翻滚后伤害", "category": "mobility"},
	{"label": "体力回复", "category": "mobility"},
	{"label": "禁忌法术伤害", "category": "forbidden"},
	{"label": "理智稳定", "category": "vessel"},
	{"label": "金币掉落", "category": "economy"}
]

func _random_id(prefix: String) -> String:
	return "%s_%x_%x" % [prefix, Time.get_ticks_msec(), randi()]

func _pick(arr: Array) -> Variant:
	return arr[randi() % arr.size()]

func _quality_for_source(source: String) -> String:
	var roll := randf()
	if source == "totem":
		return Types.QUALITY_RARE if roll > 0.55 else Types.QUALITY_COMMON
	if source == "elite":
		return Types.QUALITY_RARE if roll > 0.45 else Types.QUALITY_COMMON
	if roll > 0.86:
		return Types.QUALITY_RARE
	if roll > 0.45:
		return Types.QUALITY_COMMON
	return Types.QUALITY_BROKEN

func _affix_count(quality: String) -> int:
	match quality:
		Types.QUALITY_BROKEN: return 1
		Types.QUALITY_COMMON: return 2
		Types.QUALITY_RARE: return 3
		_: return 4

func generate_loot(source: String, world_tier: int) -> Dictionary:
	var slot: String = _pick(SLOTS)
	var quality := _quality_for_source(source)
	var item_power: int = world_tier * 10 + (randi() % 8) + (5 if source == "totem" else 0)
	var affixes: Array = []

	for i in range(_affix_count(quality)):
		var template: Dictionary = _pick(AFFIX_POOL)
		var value: int = max(2, int(item_power * (0.4 + randf() * 0.7)))
		affixes.push_back({
			"id": _random_id("affix_%d" % i),
			"label": template.label,
			"category": template.category,
			"value": value
		})

	var core_effect := ""
	if source == "totem":
		core_effect = "触碰图腾后，容器觉醒经验小幅提高。"

	return {
		"id": _random_id("item"),
		"name": _pick(BASE_NAMES[slot]),
		"slot": slot,
		"quality": quality,
		"item_power": item_power,
		"upgrade_level": 0,
		"reroll_count": 0,
		"affixes": affixes,
		"core_effect": core_effect
	}

func gold_for_kill(source: String, world_tier: int) -> int:
	var base: int = 70 if source == "boss" else 24 if source == "elite" else 8
	return base * world_tier + (randi() % base)
