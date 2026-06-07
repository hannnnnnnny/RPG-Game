## 常量和枚举，等价于 src/core/types.ts
## 因为 GDScript 没有严格的 union 类型，这里用 String 常量替代
## 所有需要约束的字段在使用处用 assert 验证

extends Node

# --- Gender ---
const GENDER_FEMALE := "female"
const GENDER_MALE := "male"
const GENDER_UNKNOWN := "unknown"
const ALL_GENDERS := [GENDER_FEMALE, GENDER_MALE, GENDER_UNKNOWN]

# --- Appearance ---
const APPEARANCE_ASHEN := "ashen"
const APPEARANCE_WANDERER := "wanderer"
const APPEARANCE_MINER := "miner"
const APPEARANCE_NOBLE := "noble"
const APPEARANCE_DWARF := "dwarf"

# --- Equipment slots ---
const SLOT_MAIN_HAND := "mainHand"
const SLOT_OFF_HAND := "offHand"
const SLOT_HEAD := "head"
const SLOT_CHEST := "chest"
const SLOT_HANDS := "hands"
const SLOT_BOOTS := "boots"
const SLOT_AMULET := "amulet"
const SLOT_RING := "ring"
const SLOT_TOTEM := "totem"
const ALL_SLOTS := [
	SLOT_MAIN_HAND, SLOT_OFF_HAND, SLOT_HEAD, SLOT_CHEST,
	SLOT_HANDS, SLOT_BOOTS, SLOT_AMULET, SLOT_RING, SLOT_TOTEM
]

# --- Item quality ---
const QUALITY_BROKEN := "broken"
const QUALITY_COMMON := "common"
const QUALITY_RARE := "rare"
const QUALITY_CORRUPTED := "corrupted"
const QUALITY_RELIC := "relic"
const QUALITY_MYTHIC := "mythic"
const ALL_QUALITIES := [
	QUALITY_BROKEN, QUALITY_COMMON, QUALITY_RARE,
	QUALITY_CORRUPTED, QUALITY_RELIC, QUALITY_MYTHIC
]

# --- Affix categories ---
const AFFIX_ATTACK := "attack"
const AFFIX_DEFENSE := "defense"
const AFFIX_MOBILITY := "mobility"
const AFFIX_FORBIDDEN := "forbidden"
const AFFIX_VESSEL := "vessel"
const AFFIX_ECONOMY := "economy"

# --- Dialogue tones ---
const TONE_WHISPER := "whisper"
const TONE_WARNING := "warning"
const TONE_MEMORY := "memory"

# --- Default profile / world / combat shapes ---
# Use these as starting templates when creating a new save.

static func make_default_profile() -> Dictionary:
	return {
		"name": "无名者",
		"gender": GENDER_UNKNOWN,
		"appearance": APPEARANCE_ASHEN
	}

static func make_default_world_state() -> Dictionary:
	return {
		"world_tier": 1,
		"sanity": 78,
		"corruption": 5,
		"vessel_awakening": 0,
		"parasite_load": 0,
		"gold": 0,
		"flags": {
			"awakened_by_khah": false,
			"met_injured_dwarf": false,
			"first_dwarf_choice": "",
			"touched_totem_fragment": false,
			"escaped_mine": false
		}
	}

static func make_default_combat() -> Dictionary:
	return {
		"health": 100,
		"max_health": 100,
		"stamina": 100,
		"max_stamina": 100,
		"focus": 30,
		"max_focus": 30
	}
