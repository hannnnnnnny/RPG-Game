## 装备掉落 —— 等价 src/core/lootGenerator.ts
extends Node

const SLOTS := ["mainHand", "chest", "hands", "boots", "ring", "totem"]

const BASE_NAMES := {
	"mainHand": ["矿工断刃", "灰灯短剑", "黑潮凿斧", "锈蚀战镐", "残铁长矛"],
	"offHand": ["破裂法器", "矿灯提盏", "裂纹符盘"],
	"head": ["污灰兜帽", "锈铁矿盔", "破檐斗笠"],
	"chest": ["矿井皮甲", "盐石胸甲", "黑潮锁链衣", "残布外披"],
	"hands": ["裂岩手套", "旧皮手套", "锈钉护手"],
	"boots": ["逃亡者旧靴", "灰泥长靴", "钉底矿靴", "破布裹足"],
	"amulet": ["残响项链", "矿骨吊坠"],
	"ring": ["黑腕戒指", "灰灯铜戒", "锈铁指环"],
	"totem": ["图腾碎屑", "矿井护符", "封印残片"]
}

# 按品质给名字加前缀，让重复底材也能一眼区分。
const QUALITY_PREFIX := {
	"broken": "残破·",
	"common": "",
	"rare": "精制·",
	"corrupted": "黑潮·",
	"relic": "【遗世】",
	"mythic": "【神话】"
}

# 按主导词条类别给稀有以上的物品加一个风味词。
const CATEGORY_FLAVOR := {
	"attack": "嗜血",
	"defense": "坚岩",
	"mobility": "疾风",
	"forbidden": "低语",
	"vessel": "容器",
	"economy": "贪婪"
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
		"name": _build_name(slot, quality, affixes),
		"slot": slot,
		"quality": quality,
		"item_power": item_power,
		"upgrade_level": 0,
		"reroll_count": 0,
		"affixes": affixes,
		"core_effect": core_effect
	}

# 组装区分度高的名字：品质前缀 + (稀有以上)风味词 + 底材。
func _build_name(slot: String, quality: String, affixes: Array) -> String:
	var base: String = _pick(BASE_NAMES[slot])
	var prefix: String = QUALITY_PREFIX.get(quality, "")
	var flavor := ""
	# rare and above get a flavor word from their strongest affix's category.
	if quality != Types.QUALITY_BROKEN and quality != Types.QUALITY_COMMON and affixes.size() > 0:
		var best: Dictionary = affixes[0]
		for a in affixes:
			if int(a.value) > int(best.value):
				best = a
		flavor = CATEGORY_FLAVOR.get(best.category, "") + "之"
	return prefix + flavor + base

func gold_for_kill(source: String, world_tier: int) -> int:
	var base: int = 70 if source == "boss" else 24 if source == "elite" else 8
	return base * world_tier + (randi() % base)
