# 《潮蚀之环》Phaser → Godot 4 迁移计划

## 0. 阅读本文档之前

**这个项目从 Phaser + React + TS 切到 Godot 4。** 本文档是迁移指南。`godot-migration/scripts/` 下的 GDScript 文件是核心逻辑翻译，准备好直接拖进新 Godot 项目里。

**关键决定**：

- 引擎：**Godot 4.3+**（不要用 3.x，4.x 是当前主线）
- 脚本语言：**GDScript**（不用 C#，原因：Steam 上 C# 还需要 .NET runtime，GDScript 直接打包进 exe）
- 渲染：纯 2D，Top-down 视角（视设计可后续切 isometric）
- 部署：Steam exe via GodotSteam 插件
- 美术：AI 生成（PixelLab）+ itch.io CC0 资源

---

## 1. 安装 Godot

1. 去 https://godotengine.org/download 下载 **Godot 4.3** 或更新的 standard 版本（不要 .NET 版）
2. 解压到 `C:\Tools\Godot\` 或类似（绿色软件，不需要安装）
3. 双击 `Godot_v4.3-stable_win64.exe` 启动

## 2. 先做完官方 2D 教程（不可跳过）

在写一行项目代码前，**先花 2 小时跑官方 "Your first 2D game"**：

- https://docs.godotengine.org/en/stable/getting_started/first_2d_game/

跑完你会理解：
- Node、Scene、Script 三件套
- Signal（替代 React 的 props/事件）
- AnimatedSprite2D（替代 Phaser 的 anims）
- CollisionShape2D（替代 Arcade Physics body）
- Autoload 单例（替代 Zustand store）

**不会这些就开始改项目 = 浪费时间。**

## 3. 创建项目

打开 Godot → New Project：

- Project name: **TidesOfKhah**（潮蚀之环英文名）
- Project Path: `C:\project719A1\TidesOfKhah-Godot\`（**新文件夹**，不要覆盖 RPG-Game）
- Renderer: **Mobile**（2D 项目最佳选择，性能好、Web export 友好）
- Version Control Metadata: Git

创建后用 Godot 自带编辑器打开。

## 4. 项目目录结构（按 TECH_ARCHITECTURE.md 对齐）

在新 Godot 项目里建以下文件夹（Godot 用 `res://` 表示项目根）：

```
res://
├─ scenes/
│  ├─ scenes_main/
│  │  ├─ MainGame.tscn          # 整个游戏的入口（替代 React App.tsx）
│  │  └─ CharacterCreator.tscn  # 角色创建（替代 React CharacterCreator）
│  ├─ scenes_world/
│  │  ├─ MineIntro.tscn         # 黑潮矿区（替代 MineIntroScene.ts）
│  │  └─ TownAshlight.tscn      # 灰灯镇（后续）
│  ├─ actors/
│  │  ├─ Player.tscn            # 玩家角色
│  │  ├─ Enemy.tscn             # 通用敌人
│  │  ├─ Npc.tscn               # 通用 NPC
│  │  └─ Totem.tscn             # 图腾残片
│  └─ ui/
│     ├─ Hud.tscn               # 血/体/专注/污染/觉醒/金币
│     ├─ DialoguePanel.tscn     # 对话框
│     ├─ ChoicePanel.tscn       # 永久选择
│     ├─ Inventory.tscn         # 背包
│     ├─ QuestLog.tscn          # 任务
│     ├─ EventLog.tscn          # 日志
│     └─ VisionOverlay.tscn     # 幻象 overlay
├─ scripts/
│  ├─ core/
│  │  ├─ game_state.gd          # 全局状态单例（替代 useGameStore）
│  │  ├─ aidlc_rules.gd         # 规则审批
│  │  ├─ loot_generator.gd      # 装备掉落
│  │  ├─ save_system.gd         # 存档读写
│  │  └─ types.gd               # 常量与枚举
│  └─ actors/
│     ├─ player.gd
│     ├─ enemy.gd
│     └─ npc.gd
├─ data/
│  ├─ npcs.json                 # NPC 定义
│  ├─ items.json                # 装备模板
│  ├─ affixes.json              # 词条
│  └─ dungeons.json             # 地牢配置
├─ assets/
│  ├─ characters/               # 把 RPG-Game/public/assets/characters/*.jpg 全拷过来
│  │  ├─ disi_stage1.jpg
│  │  ├─ disi_stage2.jpg
│  │  ├─ disi_stage3.jpg
│  │  ├─ khah.jpg
│  │  ├─ dwarf_injured.jpg
│  │  ├─ dwarf_corrupted.jpg
│  │  └─ dwarf_captain_vision.jpg
│  ├─ sprites/                  # PixelLab 出的 4 方向 sprite
│  │  ├─ disi/
│  │  │  ├─ disi_south.png
│  │  │  ├─ disi_east.png
│  │  │  ├─ disi_west.png
│  │  │  └─ disi_north.png
│  │  └─ corrupted_dwarf/
│  ├─ tilesets/                 # itch.io 下载的瓦片包放这里
│  └─ ui/                       # UI 边框、按钮等
└─ project.godot                # Godot 自动生成
```

## 5. 配置 Autoload（关键步骤）

打开 Project → Project Settings → Autoload，添加 4 个单例（按顺序）：

| 名称 | 路径 | 作用 |
|---|---|---|
| GameState | `res://scripts/core/game_state.gd` | 玩家档案 + 世界状态 + 背包 |
| AidlcRules | `res://scripts/core/aidlc_rules.gd` | 状态变更审批 |
| LootGenerator | `res://scripts/core/loot_generator.gd` | 装备生成 |
| SaveSystem | `res://scripts/core/save_system.gd` | 存档 |

**Enable** 全部勾上。这样任何脚本里都能直接调 `GameState.world_state.corruption` 等。

## 6. 迁移顺序（按周）

### Week 1：地基

目标：能创建角色 → 进入空场景 → 看到玩家小人能走动

1. 把 `godot-migration/scripts/core/*.gd` 全部拷进新项目的 `res://scripts/core/`
2. 配置 Autoload（见 §5）
3. 建 `Player.tscn`：CharacterBody2D + AnimatedSprite2D + CollisionShape2D
4. 用 PixelLab 出的 4 张 PNG 做 AnimatedSprite2D 的 frames（walk_down/up/left/right + idle_*）
5. 建 `MineIntro.tscn`：占位 TileMap + 一个 Player 实例
6. 建 `CharacterCreator.tscn` → 玩家输入名字 → 切到 MineIntro

**验收**：能进矿区，键盘 WASD 走动，sprite 朝对应方向播 walk anim

### Week 2：UI + 对话 + 选择

目标：克哈低语、第一个永久选择、HUD 数字能更新

1. 建 `Hud.tscn` —— 6 个 ProgressBar/Label，连 GameState 的 signal
2. 建 `DialoguePanel.tscn` —— 听 GameState.dialogue_opened signal
3. 建 `ChoicePanel.tscn` —— 听 GameState.choice_opened signal
4. 矿区里加 NPC（受伤矮人 Area2D），玩家走近按 E 触发选择
5. 用 AidlcRules 做选择审批

**验收**：完整跑通"开局醒来 → 触碰图腾 → 救/抛/杀矮人 → 数字更新"

### Week 3：战斗 + 装备掉落

目标：能砍死敌人，掉装备，背包能装备

1. 给 Player 加挥剑动作 —— AnimationPlayer 时间轴
2. 加 Slash Hitbox（Area2D 短时间出现）
3. 敌人 Enemy.tscn —— 简单 AI 追玩家
4. 敌人死亡调 LootGenerator.generate_loot 加进背包
5. 建 Inventory UI

**验收**：能完整跑通 Phaser 版的所有矿区流程

### Week 4：存档 + Steam 接入

1. SaveSystem 用 `ConfigFile` 或 `ResourceSaver` 写 user://save_v1.cfg
2. 装 GodotSteam 插件 https://github.com/CoaguCo-Industries/GodotSteam
3. 申请 Steam Direct 开发者账号（$100）
4. 配置 steam_appid.txt
5. 写第一个成就："醒于矿区"

**验收**：游戏退出再开能恢复，Steam Overlay 能显示成就

### Week 5+：内容铺量

按 GAME_DESIGN.md 推进：灰灯镇 → 红瀑沼泽 → 13 地牢之一……

## 7. AIDLC 后端（不是 Godot 的事）

GDScript 调 LLM 的方式：

```gdscript
# scripts/aidlc/client.gd
var http := HTTPRequest.new()
add_child(http)
http.request_completed.connect(_on_response)

func ask_npc(npc_id: String, message: String) -> void:
    var url = "https://your-cloudflare-worker.com/api/aidlc/dialogue"
    var body = JSON.stringify({
        "npc_id": npc_id,
        "world_state_summary": _summarize(),
        "message": message
    })
    http.request(url, ["Content-Type: application/json"], HTTPClient.METHOD_POST, body)
```

**LLM API key 不能放在 Godot 项目里**（玩家能反编译 pck 看到）。必须放后端。

后端用：
- **Cloudflare Workers**（免费额度够前期）
- 或 **Vercel Serverless**
- 或自己开 VPS 跑 Node/Express

后端把 key 注入 → 调 Claude/GPT → 过滤敏感字段（不能透露的真相）→ 返回 JSON。

## 8. 不要做的事

- ❌ 别用 Godot 内置的 LLM 插件（多数是玩具）
- ❌ 别用 C# 版 Godot（Steam 打包麻烦）
- ❌ 别上 3D 节点（CharacterBody3D 等）—— 你是 2D
- ❌ 别用 Web export 当主目标 —— Steam 是 desktop
- ❌ 别再换引擎 —— 这是承诺过的"最后一次切"

## 9. 复用清单

| 文件 / 资源 | 直接复用 | 翻译复用 | 丢弃 |
|---|---|---|---|
| GAME_DESIGN.md | ✅ | | |
| TECH_ARCHITECTURE.md | ✅ 作为对照 | | |
| public/assets/characters/*.jpg | ✅ 拷到 res://assets/characters/ | | |
| PixelLab 出的 sprite PNG | ✅ | | |
| core/types.ts | | ✅ → types.gd（在本目录已写） | |
| core/worldState.ts | | ✅ → game_state.gd | |
| core/aidlcRules.ts | | ✅ → aidlc_rules.gd | |
| core/lootGenerator.ts | | ✅ → loot_generator.gd | |
| core/saveSystem.ts | | ✅ → save_system.gd | |
| store/useGameStore.ts | | ✅ 合并进 game_state.gd | |
| game/scenes/MineIntroScene.ts | | | ❌ 重做成 Godot scene |
| game/PhaserGame.tsx | | | ❌ Godot 自带游戏循环 |
| app/App.tsx 全部 UI | | | ❌ 重做成 Control 节点 |
| styles.css | | | ❌ Godot 用 Theme 资源做样式 |
| package.json / vite.config.ts | | | ❌ 不需要 |

## 10. 起手第一件事

**今天就做**：
1. 下载 Godot 4.3 standard 版
2. 跑完官方 2D 教程
3. 创建 TidesOfKhah-Godot 项目
4. 把 `godot-migration/scripts/` 里 4 个 .gd 拷进新项目
5. 配置 Autoload
6. 建一个空场景，写 `print(GameState.world_state)` 测试单例是否生效

**做完发我 Godot 项目截图**，我接着帮你建第一个真正的场景。

---

**一句话总结**：你的设计不变、AI 资产不变、克哈/disi/伊芙剧情不变。变的只是**实现技术栈**。Phaser 项目留着当对照（别删），新项目独立另起，跑通后老项目可以归档。
