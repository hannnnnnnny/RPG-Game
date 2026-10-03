## Shared UI colours/labels so every panel shows item quality identically.
class_name UiPalette
extends RefCounted

const QUALITY_COLOR := {
	"broken": Color(0.55, 0.52, 0.46),
	"common": Color(0.88, 0.84, 0.74),
	"rare": Color(0.45, 0.60, 0.84),
	"corrupted": Color(0.61, 0.34, 0.65),
	"relic": Color(0.85, 0.71, 0.38),
	"mythic": Color(0.95, 0.83, 0.45),
}

const QUALITY_LABEL := {
	"broken": "破损",
	"common": "普通",
	"rare": "稀有",
	"corrupted": "污染",
	"relic": "遗物",
	"mythic": "神话",
}

const TEXT := Color(0.95, 0.92, 0.82)
const TEXT_MUTED := Color(0.66, 0.62, 0.55)
const GOLD := Color(0.95, 0.83, 0.45)
const DANGER := Color(0.86, 0.42, 0.36)

static func quality_color(item: Dictionary) -> Color:
	return QUALITY_COLOR.get(item.get("quality", ""), Color.WHITE)

static func quality_label(item: Dictionary) -> String:
	return QUALITY_LABEL.get(item.get("quality", ""), str(item.get("quality", "")))

## "黑潮·灰灯短剑 +3" — upgrade level shown wherever an item is named.
static func item_title(item: Dictionary) -> String:
	var lvl := int(item.get("upgrade_level", 0))
	return item.get("name", "?") + (" +%d" % lvl if lvl > 0 else "")
