## The three-axis mind system (design §10): 理智 sanity, 污染 corruption,
## 容器觉醒 vessel awakening. Named tiers + how losses are softened.
class_name MindState
extends RefCounted

# Ordered high → low; the first tier whose floor the value reaches wins.
const SANITY_TIERS := [
	{"id": "stable", "floor": 80, "name": "稳定", "hint": "NPC 更信任你，但有些真相藏在你看不见的地方。"},
	{"id": "shaken", "floor": 50, "name": "动摇", "hint": "你开始看见刺青的异样，和残歌里的碎片。"},
	{"id": "fractured", "floor": 20, "name": "破裂", "hint": "隐藏的门在墙上浮现。克哈的痕迹无处不在。"},
	{"id": "collapsing", "floor": 0, "name": "濒临崩溃", "hint": "真相涌进来——混着幻听和谎言。"},
]

const CORRUPTION_TIERS := [
	{"id": "near_god", "floor": 81, "name": "近神之壳"},
	{"id": "black_vein", "floor": 56, "name": "黑脉"},
	{"id": "tide_stained", "floor": 26, "name": "染潮"},
	{"id": "clear", "floor": 0, "name": "清醒之身"},
]

const VESSEL_STAGES := ["空壳未醒", "低语入梦", "印痕显现", "双容器共鸣", "承灾之器", "终局容器"]

static func _tier(tiers: Array, value: int) -> Dictionary:
	for t in tiers:
		if value >= int(t.floor):
			return t
	return tiers[tiers.size() - 1]

static func sanity_tier(sanity: int) -> Dictionary:
	return _tier(SANITY_TIERS, sanity)

static func corruption_tier(corruption: int) -> Dictionary:
	return _tier(CORRUPTION_TIERS, corruption)

static func vessel_stage_name(stage: int) -> String:
	return VESSEL_STAGES[clampi(stage, 0, VESSEL_STAGES.size() - 1)]

## 理智稳定 softens sanity *losses* with diminishing returns (never gains):
## guard 50 → losses x0.67, guard 100 → x0.5.
static func soften_loss(delta: int, sanity_guard: int) -> int:
	if delta >= 0:
		return delta
	var factor := 100.0 / (100.0 + maxf(sanity_guard, 0.0))
	return mini(-1, int(round(delta * factor)))

static func clamp_meter(value: int) -> int:
	return clampi(value, 0, 100)
