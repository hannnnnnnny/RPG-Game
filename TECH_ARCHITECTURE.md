# 《潮蚀之环》技术架构文档

## 1. 技术栈

推荐技术栈：

- Vite。
- React。
- TypeScript。
- Phaser 3。
- Tiled 地图编辑器。
- Zustand 状态管理。
- JSON 或 TypeScript 数据驱动配置。
- 本地存档先用 localStorage 或 IndexedDB。
- 后期接 Node/Express 或 serverless AI 后端。

定位：

- Phaser 负责游戏世界、地图、碰撞、战斗、动画、敌人、掉落。
- React 负责 UI，包括背包、装备、对话、任务、地图、角色面板和设置。
- core 层负责纯逻辑，包括装备、任务、世界状态、AIDLC 审批、存档、掉落和结局判断。

## 2. 总体结构

```text
React App
  ├─ React UI
  │  ├─ 装备栏
  │  ├─ 背包
  │  ├─ 对话框
  │  ├─ 任务日志
  │  ├─ 角色面板
  │  └─ 地图与传送
  ├─ Phaser Game
  │  ├─ 地图
  │  ├─ 玩家
  │  ├─ 敌人
  │  ├─ 战斗
  │  ├─ 掉落
  │  └─ 场景切换
  ├─ Game Core
  │  ├─ 世界状态
  │  ├─ 存档
  │  ├─ 装备系统
  │  ├─ 任务系统
  │  ├─ AIDLC 规则
  │  ├─ 地牢系统
  │  └─ 结局判断
  └─ AIDLC Client
     └─ 后期连接 AI 服务
```

## 3. 推荐目录结构

```text
src/
  app/
    App.tsx
    providers/
    routes/
  game/
    PhaserGame.ts
    eventBus.ts
    scenes/
      BootScene.ts
      PreloadScene.ts
      MineIntroScene.ts
      UIScene.ts
    actors/
      Player.ts
      Enemy.ts
      NpcActor.ts
    combat/
      damage.ts
      stamina.ts
      hitbox.ts
    loot/
      dropLoot.ts
      pickup.ts
    maps/
  core/
    worldState.ts
    saveSystem.ts
    questSystem.ts
    equipmentSystem.ts
    lootGenerator.ts
    affixSystem.ts
    aidlcRules.ts
    endingRules.ts
    economySystem.ts
    dungeonSystem.ts
  data/
    npcs/
    items/
    affixes/
    quests/
    dungeons/
    bosses/
    factions/
    maps/
  ui/
    hud/
    inventory/
    dialogue/
    quest-log/
    character/
    map/
  store/
    useGameStore.ts
    useUiStore.ts
```

## 4. Phaser 与 React 通信

使用事件总线解耦 Phaser 和 React。

示例流程：

```text
Phaser 击杀怪物
→ eventBus.emit("loot:drop", item)
→ Zustand 写入背包状态
→ React 背包更新
```

```text
React 点击装备
→ eventBus.emit("player:equip", itemId)
→ core equipmentSystem 校验
→ Phaser 更新玩家战斗属性
```

```text
Phaser 触发克哈低语
→ eventBus.emit("dialogue:open", { npcId: "khah_whisper" })
→ React DialoguePanel 打开
```

## 5. 数据驱动

装备、词条、任务、NPC、地牢、Boss、阵营都使用数据驱动。

第一版可以先用 TypeScript 配置，后续可迁移为 JSON。

### 5.1 NPC 数据

```ts
type NpcDefinition = {
  id: string;
  name: string;
  role: "god_anchor" | "main" | "side" | "common";
  factionId?: string;
  personality: string[];
  publicKnowledge: string[];
  hiddenKnowledge: string[];
  forbiddenRevealFlags: string[];
  questIds: string[];
  memoryPolicy: "full" | "summary" | "minimal";
};
```

### 5.2 世界状态

```ts
type WorldState = {
  worldTier: 1 | 2 | 3 | 4 | 5;
  sanity: number;
  corruption: number;
  vesselAwakening: 0 | 1 | 2 | 3 | 4 | 5;
  parasiteLoad: number;
  flags: Record<string, boolean | number | string>;
  factions: Record<string, FactionState>;
  npcs: Record<string, NpcRuntimeState>;
  dungeons: Record<string, DungeonState>;
};
```

### 5.3 装备数据

```ts
type ItemInstance = {
  id: string;
  baseItemId: string;
  itemPower: number;
  quality: "broken" | "common" | "rare" | "corrupted" | "relic" | "mythic";
  slot: EquipmentSlot;
  levelRequirement: number;
  upgradeLevel: number;
  affixes: AffixInstance[];
  lockedAffixId?: string;
  rerollCount: number;
  coreEffectId?: string;
};
```

### 5.4 任务数据

```ts
type QuestDefinition = {
  id: string;
  title: string;
  type: "main" | "fixed_side" | "npc_story" | "faction" | "aidlc_dynamic";
  giverNpcId?: string;
  prerequisites: RuleCondition[];
  objectives: QuestObjective[];
  rewards: QuestReward[];
  failureConditions: RuleCondition[];
  worldEffectsOnComplete: StateChange[];
  worldEffectsOnFail: StateChange[];
};
```

## 6. AIDLC 接入流程

第一版先做脚本化克哈低语和 AIDLC 数据结构。真实 AI 后端后续接入。

### 6.1 对话输入

发送给 AIDLC 的上下文包括：

- 玩家名字。
- 玩家当前状态。
- NPC 定义。
- NPC 记忆摘要。
- 当前世界状态摘要。
- 当前任务状态。
- 允许透露的信息。
- 禁止透露的信息。
- 可申请的状态变更类型。

### 6.2 AI 输出

AI 返回结构化结果：

```ts
type AidlcResponse = {
  dialogue: string;
  memoryUpdates?: MemoryPatch[];
  questOffer?: DynamicQuestDraft;
  stateChangeRequest?: StateChangeRequest;
  refusalReason?: string;
};
```

### 6.3 状态变更审批

AI 不能直接改世界。它只能提交请求。

```ts
type StateChangeRequest = {
  type: string;
  targetId: string;
  requestedByNpcId: string;
  reason: string;
  proposedEffects: StateChange[];
};
```

审批检查：

- NPC 是否有权限。
- 玩家条件是否满足。
- 当前剧情节点是否允许。
- 是否破坏主线。
- 是否提前泄露真相。
- 是否与已有 worldFlags 冲突。
- 奖励是否符合 NPC 资源。

若拒绝，NPC 在对话中合理拒绝玩家，并可提供低权限替代方案。

## 7. 存档结构

第一版本地存档保存：

- 玩家名字。
- 性别和外貌。
- 当前地图。
- 当前位置。
- 生命、体力、专注。
- 理智、污染、容器觉醒、寄生值。
- 装备和背包。
- 金币。
- 世界等级。
- worldFlags。
- NPC 记忆摘要。
- 任务状态。
- 地牢传送点。
- 已触发幻视。
- 第一个永久选择。

后续可迁移到 IndexedDB，以支持更大的 NPC 记忆和装备库。

## 8. 第一版技术目标

第一版只做黑潮矿区开局场景。

必须完成：

- Vite + React + TypeScript 项目。
- Phaser 嵌入 React。
- BootScene、PreloadScene、MineIntroScene。
- 玩家创建数据结构。
- 玩家移动。
- 普通攻击。
- 翻滚和体力。
- 基础敌人 AI。
- 基础掉落。
- 金币掉落。
- 随机词条雏形。
- 克哈低语 UI。
- 图腾残片互动。
- worldState 和 saveSystem。
- AIDLC 状态变更审批雏形。

暂不完成：

- 真实 AI API。
- 全量 NPC。
- 全量地牢。
- 完整装备重铸 UI。
- 完整阵营系统 UI。
- 完整结局系统 UI。

## 9. 后端 AI 预留

后期真实 AIDLC 建议通过后端服务接入，避免在前端暴露 API key。

推荐接口：

```text
POST /api/aidlc/dialogue
POST /api/aidlc/summarize-memory
POST /api/aidlc/generate-dynamic-quest
```

后端职责：

- 调用 AI 模型。
- 注入 NPC 和世界上下文。
- 过滤禁止透露的信息。
- 返回结构化响应。
- 不直接写入存档。

前端规则系统仍然负责最终审批。

## 10. 开发原则

- 主线真相固定。
- AI 输出必须结构化。
- AI 不能直接改世界。
- 重要系统全部数据驱动。
- 第一版少内容，但架构按完整游戏预留。
- 战斗优先保证爽快，不做复杂硬核门槛。
- UI 由 React 管，地图和战斗由 Phaser 管。

