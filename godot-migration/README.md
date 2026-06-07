# Godot 迁移工具包

这个文件夹是把现有 Phaser/React/TS 项目搬到 Godot 4 的"起手包"。

## 里面是什么

```
godot-migration/
├─ README.md                  # 本文件
├─ MIGRATION_PLAN.md          # ⭐ 必读，5 周路线图
├─ project.godot.template     # Godot 项目配置参考（不要直接覆盖）
└─ scripts/
   └─ core/
      ├─ types.gd             # 常量 + 默认状态模板
      ├─ aidlc_rules.gd       # 状态变更审批
      ├─ loot_generator.gd    # 装备掉落
      ├─ game_state.gd        # 全局状态单例
      └─ save_system.gd       # 存档读写
```

## 怎么用

1. 读 **MIGRATION_PLAN.md**，按 §1-3 装 Godot、跑教程、建新项目
2. 把 `scripts/core/*.gd` 五个文件拷进新 Godot 项目的 `res://scripts/core/`
3. 按 MIGRATION_PLAN.md §5 配 Autoload
4. 把 `public/assets/characters/*.jpg` 拷到新项目 `res://assets/characters/`
5. PixelLab 出的 sprite PNG 放 `res://assets/sprites/disi/`
6. 开始按 Week 1 节奏建场景

## 老项目怎么办

**别删！**`C:\project719A1\RPG-Game\` 保留作为：
- 设计文档参考（GAME_DESIGN.md / TECH_ARCHITECTURE.md 一字不改地继续用）
- 美术资产仓库（PixelLab 出图、AI 立绘存这里）
- 行为参考（不确定 Godot 该怎么实现某功能时回来看 Phaser 是怎么做的）

迁移完成、Godot 项目能正常跑后，老项目可以归档但不要 `rm -rf`。

## 已经翻译过的核心逻辑

下表对照 src/ 里的 TS 文件和本目录下的 GDScript 文件：

| TS 文件 | GDScript 翻译 | 状态 |
|---|---|---|
| `src/core/types.ts` | `scripts/core/types.gd` | ✅ 完整 |
| `src/core/worldState.ts` | `scripts/core/game_state.gd` 里的 `make_default_world_state()` | ✅ 完整 |
| `src/core/aidlcRules.ts` | `scripts/core/aidlc_rules.gd` | ✅ 完整 |
| `src/core/lootGenerator.ts` | `scripts/core/loot_generator.gd` | ✅ 完整 |
| `src/core/saveSystem.ts` | `scripts/core/save_system.gd` | ✅ 完整（用 ConfigFile） |
| `src/store/useGameStore.ts` | `scripts/core/game_state.gd` | ✅ 合并完成 |

**逻辑等价但语法不同**的要点：

- TS 的 `Record<string, ...>` → GDScript 的 `Dictionary`
- Zustand 的订阅 → Godot 的 signal + connect
- React 的 `useState` → 组件内 `@onready var x = ...`
- TS 的 union 类型（`"enemy" | "elite"`）→ GDScript 用 String 常量 + 调用处 assert

## 还没翻译的（属于场景层，Week 1-3 才做）

- `MineIntroScene.ts` → 整套要在 Godot 编辑器里用 TileMap + Node 拼出来
- `PhaserGame.tsx` → Godot 自带主循环，不需要
- `App.tsx` 里所有 React 组件 → 全部重做成 Control 节点场景

## 不要再换引擎了

签了 commitment：从今天起 3 个月专注 Godot，不再讨论 UE / Unity / RPG Maker / Tauri / NW.js / Web 部署。

如果发现 Godot 真有难处，先把问题描述清楚，看是不是能在 Godot 内解决，不要默认"是工具问题、要换工具"。
