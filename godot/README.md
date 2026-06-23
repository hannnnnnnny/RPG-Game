# 《潮蚀之环》Godot 4 项目

从 Phaser/React/TS 完整移植过来的 Godot 4 项目。**直接可以在 Godot 4.3+ 里打开运行。**

## 怎么打开

1. **下载 Godot 4.3 standard 版**：https://godotengine.org/download
   - 选 Standard，**不要** .NET 版本
   - 解压到 `C:\Tools\Godot\` 或类似（绿色软件，无需安装）
2. **运行 Godot**：双击 `Godot_v4.3-stable_win64.exe`
3. **导入项目**：在 Project Manager 点 `Import` → 浏览到 `C:\project719A1\TidesOfKhah-Godot\` → 选 `project.godot` → 点 `Import & Edit`
4. **第一次打开** Godot 会扫描所有资源，给 7 张 JPG 自动生成 `.import` 元数据。等 1-2 分钟，看右下角进度条
5. **按 F5 运行**

## 第一次运行你会看到什么

按 F5 后：

1. **角色创建界面**：紫黑色背景，"潮蚀之环"标题，名字输入框默认"无名者"，"进入矿井"按钮
2. **点进入矿井** → 切到 `MineIntro` 场景
3. **黑潮矿区**：程序化绘制的瓦片地图，5 个房间用土黄色 path 连起来
4. **玩家 disi**：紫衣黑发占位像素小人，键盘 WASD 走动
5. **HUD**：左上 6 个数据条（生命/体力/专注/污染/觉醒/金币），右上 disi 头像（用 `disi_stage1.jpg`）
6. **克哈低语**：底部对话框弹出 "往前。那些矿灯已经死了..."
7. **三个敌人**：紫色腐化矮人，看到玩家追上来打
8. **三个交互点**：受伤矮人（紫色刺青）/ 图腾残片（紫水晶柱）/ 矿井出口（拱门）

按键映射：
- **WASD / ↑↓←→**：移动
- **J**：挥剑
- **SPACE**：翻滚（消耗体力）
- **E**：与最近的交互点互动

## 项目结构

```
TidesOfKhah-Godot/
├── project.godot            # Godot 项目配置（含 autoload + 输入映射）
├── README.md                # 本文件
├── assets/
│   ├── characters/          # AI 立绘 .jpg（已从 RPG-Game 复制）
│   │   ├── disi_stage1.jpg  # 污染 0-25
│   │   ├── disi_stage2.jpg  # 污染 26-55
│   │   ├── disi_stage3.jpg  # 污染 56+
│   │   ├── khah.jpg         # 克哈对话头像
│   │   ├── dwarf_injured.jpg
│   │   ├── dwarf_corrupted.jpg
│   │   └── dwarf_captain_vision.jpg
│   ├── sprites/             # PixelLab 出的 sprite 放这里（目前空）
│   └── ui/                  # UI 边框、按钮等（目前空）
├── scenes/
│   ├── main/
│   │   ├── MainGame.tscn         # 入口
│   │   └── CharacterCreator.tscn
│   ├── world/
│   │   └── MineIntro.tscn        # 黑潮矿区
│   ├── actors/
│   │   ├── Player.tscn
│   │   ├── Enemy.tscn
│   │   └── Interactable.tscn
│   └── ui/
│       ├── Hud.tscn
│       ├── Dialogue.tscn
│       ├── Choice.tscn
│       └── Vision.tscn
└── scripts/
    ├── core/
    │   ├── types.gd          # 常量 + 默认状态模板
    │   ├── aidlc_rules.gd    # 审批
    │   ├── loot_generator.gd # 装备掉落
    │   ├── game_state.gd     # 全局单例（替代 useGameStore + worldState）
    │   └── save_system.gd    # 存档
    ├── actors/
    │   ├── player.gd
    │   ├── enemy.gd
    │   └── interactable.gd
    ├── world/
    │   ├── mine_intro.gd
    │   └── slash.gd          # 剑光月牙动画
    ├── ui/
    │   ├── hud.gd
    │   ├── dialogue_panel.gd
    │   ├── choice_panel.gd
    │   ├── vision_overlay.gd
    │   ├── inventory.gd        # 背包面板
    │   ├── quest_panel.gd      # 任务面板
    │   └── event_log.gd        # 日志面板
    └── main/
        └── character_creator.gd  # 含 continue/new flow
```

## Autoload 单例

`project.godot` 已配置以下 5 个 autoload，任何脚本里都能直接调：

```gdscript
GameState.world_state.corruption       # 读
GameState.set_dialogue({...})          # 写
GameState.request_state_change({...})  # 走审批
AidlcRules.approve_state_change(...)
LootGenerator.generate_loot("enemy", 1)
SaveSystem.save()
Types.GENDER_FEMALE
```

## 已实现 vs Phaser 版的对照

| 功能 | Phaser 版 | Godot 版 | 状态 |
|---|---|---|---|
| 角色创建 | React `CharacterCreator` | `CharacterCreator.tscn` | ✅ 等价 |
| 全局状态 | Zustand `useGameStore` | autoload `GameState` | ✅ 等价 |
| AIDLC 审批 | `aidlcRules.ts` | `aidlc_rules.gd` | ✅ 等价 |
| 装备生成 | `lootGenerator.ts` | `loot_generator.gd` | ✅ 等价 |
| 玩家移动 + 4 方向 sprite | Phaser sprite + anims | CharacterBody2D + `_draw()` 占位 | ✅ 占位完成 |
| 程序化瓦片地图 | Phaser Graphics + canvas | Node2D `_draw()` | ✅ 等价 |
| 墙体碰撞 | Phaser StaticGroup | StaticBody2D + CollisionShape2D | ✅ 等价 |
| 挥剑动画 | Phaser AnimatedSprite | `slash.gd` 时序绘制 | ✅ 等价 |
| 敌人 AI | Phaser physics moveToObject | `_physics_process` 追玩家 | ✅ 等价 |
| 交互（图腾/矮人/出口） | distance check + key press | distance check + interact action | ✅ 等价 |
| HUD（6 条 + disi 头像） | React + lucide-react icons | Hud.tscn ProgressBar + TextureRect | ✅ 等价 |
| 对话框 + 克哈头像 | React `DialoguePanel` | Dialogue.tscn | ✅ 等价 |
| 永久选择 | React `ChoicePanel` | Choice.tscn | ✅ 等价 |
| 全屏幻象 overlay | React `VisionOverlay` | Vision.tscn | ✅ 等价 |
| 存档（localStorage） | Zustand persist | `SaveSystem` + `ConfigFile` + 防抖触发 | ✅ 等价 |
| 背包 UI | React `InventoryPanel` | Inventory.tscn 含品质染色边框 | ✅ 等价 |
| 任务面板 | React `QuestPanel` | Quest.tscn 由 flags 推导 | ✅ 等价 |
| 日志面板 | React `LogPanel` | EventLog.tscn 显示最近 6 条 | ✅ 等价 |
| 标题继续/新游戏/重置 | 无（Phaser 版只有创建） | CharacterCreator 检测存档自动切 | ✅ 升级 |

## 已知限制 / 占位

1. **玩家 sprite 是 GDScript 程序化绘制**（draw_rect 拼像素），效果跟原 Phaser 版的占位 sprite 相当。**这是临时方案**，建议尽快用 PixelLab 出 4 方向 PNG 替换：
   - 替换方法：把 4 张 PNG 拖进 `assets/sprites/disi/`，编辑器里改 Player.tscn 根节点 → 把脚本里的 `_draw()` 删掉，改成调 `$AnimatedSprite2D.play()`
2. **敌人 / 受伤矮人 / 图腾 / 出口** 也是程序化绘制占位，同样可替换
3. **没有真正的瓦片资产**，矿区背景是 `_draw` 画的色块。Week 3-4 切到 Tiled 编辑器 + Godot TileMap 资源
4. **没有音效**：第二版补
5. **存档自动触发未接** —— `SaveSystem.save()` 函数写好了但还没在关键事件后调。需要在 game_state.gd 里加 `_log()` 时同步触发，或者用 Timer 定时存
6. **没有背包/任务/日志面板**：第二版补

## 怎么扩展（下一步）

### 1. 替换玩家 sprite 为真实 PixelLab 资产

1. PixelLab 跑出 4 方向 PNG：`disi_south.png` / `disi_east.png` / `disi_west.png` / `disi_north.png`
2. 拖进 `assets/sprites/disi/`，编辑器自动 import
3. 打开 `Player.tscn`，添加子节点 `AnimatedSprite2D`
4. 给 AnimatedSprite2D 创建 SpriteFrames 资源，添加 4 个 animation：
   - `walk_down`、`walk_up`、`walk_left`、`walk_right`
   - 每个加 2-3 帧（PixelLab 出的几张组合）
5. 修改 `player.gd`：
   - 删掉整个 `_draw()` 函数
   - 在 `_physics_process` 结尾加：
     ```gdscript
     var anim_name = ("walk_" if moving else "idle_") + facing_str()
     $AnimatedSprite2D.play(anim_name)
     ```

### 2. 接入背包 UI

1. 建 `scenes/ui/Inventory.tscn`：Panel + GridContainer + 物品按钮模板
2. 写 `scripts/ui/inventory.gd` 监听 `GameState.inventory_changed`
3. 在 MineIntro.tscn UILayer 实例化

### 3. 接入 GodotSteam（Steam 上架准备）

1. 下载 GodotSteam 4.x：https://github.com/CoaguCo-Industries/GodotSteam
2. 替换 Godot 二进制为 GodotSteam 版本
3. 在 `scripts/core/` 加 `steam_client.gd`，初始化 + 成就 API
4. 申请 Steamworks 开发者账号（$100）

## 暗黑氛围系统（"暗黑星露谷"风格）

矿区不再是平铺的色块，而是一座有光影的暗黑洞穴。三层叠加：

**1. 全局压暗（CanvasModulate）**
- `MineIntro.tscn` 里的 `Ambient` 节点把整张地图乘到约 40% 亮度，并带一点冷紫
- 注意：CanvasModulate 只影响世界画布，不影响 UI（UI 在独立 CanvasLayer，保持清晰）

**2. 2D 动态光照（PointLight2D）**
- 玩家身上挂一盏暖色矿灯（`Player.tscn` 的 `TorchLight`），跟着走，照亮周围
- 图腾残片：紫色脉冲光（呼吸动画）
- 矿井出口：青绿微光
- 受伤矮人：暗淡暖光
- 腐化矮人敌人：幽紫光晕，在黑暗中浮现
- 所有光共用一张程序生成的径向渐变贴图 `assets/textures/light_soft.tres`（无需图片文件）

**3. 后处理 shader（`assets/shaders/dark_atmosphere.gdshader`）**
- 铺满屏幕的 `PostFX` 层，读屏做四件事：降饱和 / 暗部染紫 / 暗角 vignette / 污染渗透
- **污染渗透随 `world_state.corruption` 动态变化**：污染越高，屏幕边缘紫色侵蚀越浓
  （`post_process.gd` 监听 `world_state_changed` 实时更新 shader uniform）

调参位置：
- 太暗/太亮 → `MineIntro.tscn` 的 `Ambient.color`
- 矿灯范围 → `Player.tscn` 的 `TorchLight.texture_scale` / `energy`
- 褪色/暗角强度 → `MineIntro.tscn` 的 `PostMat` shader 参数

**这一层完全不依赖外部美术资产**。换上真瓦片 / sprite 后，光影系统自动作用在新素材上，效果会更好。

## Phaser 老项目去哪儿了

留在 `C:\project719A1\RPG-Game\`，**不要删**。作为：
- 设计文档参考（`GAME_DESIGN.md` / `TECH_ARCHITECTURE.md`）
- 美术资产仓库（PixelLab 出图、AI 立绘）
- 行为对照（不确定 Godot 该怎么实现某功能时回来看 Phaser）

老项目的 React UI 和 Phaser 场景代码作为参考保留，**不再维护**。

## 不要做的事

- ❌ 别在 Godot 里再装 Phaser 或 web 视图
- ❌ 别又想着换引擎（已经承诺过 Godot 是最后一站）
- ❌ 别把 `.import` 文件夹提交 git（自动生成，已在 `.gitignore` 排除）
- ❌ 别直接编辑 `project.godot` 的 input 段，用编辑器 Project Settings → Input Map

## 验证项目完整性

打开 Godot 后，在 Output 面板（底部）看：

- ✅ 应该看到 `Project loaded successfully`
- ✅ 5 个 autoload 名（Types、AidlcRules、LootGenerator、GameState、SaveSystem）在 Project Settings → Autoload 里
- ✅ 按 F5 能进入角色创建界面
- ✅ 输入名字、进入矿井，能看到玩家小人 + 房间 + 敌人
- ❌ 如果看到 "Cannot find script" 错误 → 路径写错了，告诉我具体报错
- ❌ 如果看到 "Parse error" → GDScript 语法错误，把行号发给我

## 5 个 autoload 注册顺序很重要

`project.godot` 已经按这个顺序写好（Types 最先，被其他引用）：

```
Types          → res://scripts/core/types.gd
AidlcRules     → res://scripts/core/aidlc_rules.gd
LootGenerator  → res://scripts/core/loot_generator.gd
GameState      → res://scripts/core/game_state.gd
SaveSystem    → res://scripts/core/save_system.gd
```

如果手动改了顺序，可能会出现 "Identifier 'Types' not declared" 报错。
