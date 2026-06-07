## 全局常量 + 默认状态模板。Autoload 名 "Types"。
extends Node

const GENDER_FEMALE := "female"
const GENDER_MALE := "male"
const GENDER_UNKNOWN := "unknown"

const APPEARANCE_ASHEN := "ashen"
const APPEARANCE_WANDERER := "wanderer"
const APPEARANCE_MINER := "miner"
const APPEARANCE_NOBLE := "noble"

const SLOT_MAIN_HAND := "mainHand"
const SLOT_OFF_HAND := "offHand"
const SLOT_HEAD := "head"
const SLOT_CHEST := "chest"
const SLOT_HANDS := "hands"
const SLOT_BOOTS := "boots"
const SLOT_AMULET := "amulet"
const SLOT_RING := "ring"
const SLOT_TOTEM := "totem"

const QUALITY_BROKEN := "broken"
const QUALITY_COMMON := "common"
const QUALITY_RARE := "rare"
const QUALITY_CORRUPTED := "corrupted"
const QUALITY_RELIC := "relic"
const QUALITY_MYTHIC := "mythic"

const TONE_WHISPER := "whisper"
const TONE_WARNING := "warning"
const TONE_MEMORY := "memory"

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
