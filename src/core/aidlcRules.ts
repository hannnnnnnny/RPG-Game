import type { StateChangeDecision, StateChangeRequest, WorldState } from "./types";

const allowedFirstSceneChanges = new Set([
  "record_first_choice",
  "touch_totem_fragment",
  "escape_mine",
  "khah_whisper"
]);

export function approveStateChange(
  request: StateChangeRequest,
  worldState: WorldState
): StateChangeDecision {
  if (!allowedFirstSceneChanges.has(request.type)) {
    return {
      approved: false,
      reason: "当前第一版只允许黑潮矿区开局相关的世界状态变更。"
    };
  }

  if (request.type === "escape_mine" && !worldState.flags.touchedTotemFragment) {
    return {
      approved: false,
      reason: "玩家还没有触碰图腾残片，矿井出口的黑潮不会退让。"
    };
  }

  if (request.type === "record_first_choice" && worldState.flags.firstDwarfChoice) {
    return {
      approved: false,
      reason: "第一个永久选择已经写入世界。"
    };
  }

  return {
    approved: true,
    reason: "状态变更符合当前剧情和权限。"
  };
}

