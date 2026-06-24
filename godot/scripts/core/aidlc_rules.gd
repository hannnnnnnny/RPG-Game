## AIDLC 状态变更审批 —— 等价 src/core/aidlcRules.ts
extends Node

const ALLOWED_FIRST_SCENE_CHANGES := [
	"record_first_choice",
	"touch_totem_fragment",
	"defeat_grom",
	"escape_mine",
	"khah_whisper"
]

func approve_state_change(request: Dictionary, world_state: Dictionary) -> Dictionary:
	if not request.type in ALLOWED_FIRST_SCENE_CHANGES:
		return {
			"approved": false,
			"reason": "当前第一版只允许黑潮矿区开局相关的世界状态变更。"
		}

	if request.type == "escape_mine" and not world_state.flags.touched_totem_fragment:
		return {
			"approved": false,
			"reason": "玩家还没有触碰图腾残片，矿井出口的黑潮不会退让。"
		}

	if request.type == "escape_mine" and not world_state.flags.defeated_grom:
		return {
			"approved": false,
			"reason": "黑腕队长·格罗姆挡在出口前。先击败他。"
		}

	if request.type == "escape_mine" and world_state.flags.escaped_mine:
		return {
			"approved": false,
			"reason": "矿井出口已经打开。灰灯镇的路在前方。"
		}

	if request.type == "record_first_choice" and world_state.flags.first_dwarf_choice != "":
		return {
			"approved": false,
			"reason": "第一个永久选择已经写入世界。"
		}

	return {
		"approved": true,
		"reason": "状态变更符合当前剧情和权限。"
	}
