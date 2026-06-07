---
name: 潮蚀之环
description: Web-first 2D 暗黑克苏鲁 ARPG 的容器觉醒视觉系统
colors:
  miners-lantern-gold: "#d9b662"
  lantern-wick: "#8c6530"
  dwarven-wound: "#b95b61"
  mine-lichen: "#88a96d"
  distant-lantern: "#7bb0c9"
  khah-whisper-violet: "#8b5a9b"
  lantern-cream: "#f2ead8"
  tunnel-mist: "#b3aa98"
  soot-gray: "#7f786b"
  mine-black: "#080a0b"
  tunnel-black: "#0d1011"
typography:
  display:
    fontFamily: "Georgia, 'Times New Roman', serif"
    fontSize: "clamp(34px, 6vw, 52px)"
    fontWeight: 600
    lineHeight: 0.96
    letterSpacing: "normal"
  headline:
    fontFamily: "Georgia, 'Times New Roman', serif"
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.15
    letterSpacing: "normal"
  title:
    fontFamily: "Aptos, 'Microsoft YaHei', 'PingFang SC', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 800
    lineHeight: 1.3
    letterSpacing: "normal"
  body:
    fontFamily: "Aptos, 'Microsoft YaHei', 'PingFang SC', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.65
    letterSpacing: "normal"
  label:
    fontFamily: "Aptos, 'Microsoft YaHei', 'PingFang SC', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 700
    lineHeight: 1.4
    letterSpacing: "0.08em"
  numeric:
    fontFamily: "Aptos, 'Microsoft YaHei', 'PingFang SC', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 700
    lineHeight: 1.4
    letterSpacing: "normal"
    fontFeature: "tabular-nums"
rounded:
  xs: "6px"
  sm: "7px"
  md: "8px"
  lg: "10px"
  xl: "12px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "18px"
  xl: "24px"
  xxl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.miners-lantern-gold}"
    textColor: "{colors.mine-black}"
    rounded: "{rounded.xs}"
    padding: "13px 20px"
    height: "50px"
  button-primary-hover:
    backgroundColor: "{colors.miners-lantern-gold}"
    textColor: "{colors.mine-black}"
  button-secondary:
    backgroundColor: "{colors.tunnel-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.xs}"
    padding: "12px 14px"
    height: "46px"
  button-secondary-selected:
    backgroundColor: "{colors.lantern-wick}"
    textColor: "{colors.lantern-cream}"
  button-quiet:
    backgroundColor: "{colors.tunnel-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.xs}"
    padding: "9px 12px"
    height: "38px"
  input-text:
    backgroundColor: "{colors.mine-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.xs}"
    padding: "13px 14px"
  card-panel:
    backgroundColor: "{colors.tunnel-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.md}"
    padding: "14px"
  hud-pill:
    backgroundColor: "{colors.mine-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.sm}"
    padding: "0 10px"
    height: "36px"
  dialogue-panel:
    backgroundColor: "{colors.mine-black}"
    textColor: "{colors.lantern-cream}"
    rounded: "{rounded.md}"
    padding: "16px"
---

# Design System: 潮蚀之环

## 1. Overview

**Creative North Star: "The Vessel's Threshold"**

界面是玩家容器觉醒的门槛。系统当下的语调是**冷峻矮人金属**：黑铁载体上点状金光、锐边、薄阴影里的重量感。但门槛是双向的 —— 随着污染（`worldState.corruption`）和容器觉醒（`worldState.vesselAwakening`）攀升，UI 的色相和能量应当能向克哈的紫和黑潮的血推进。`disi-avatar.stage-2/3` 的边框色变化是这一隐喻已经落地的雏形；未来动态主题演变（按 corruption 推进 surface 与 line 的色相）以这个 North Star 为合法理由。

整套系统拒绝四种东西，每一个都来自 PRODUCT.md 的 anti-references：迷你传三型手游面板、Roblox 卡通调、AAA 设置菜单的全灰冷青、Notion/Linear 的 SaaS 净色。代之以**实际重量**：1px gold-line 上叠加 22-90px 的厚阴影、衬线 H1 与方块像素立绘的并置、`backdrop-filter: blur(12px)` 让 HUD 像悬浮在矿井烟雾上的合金铭牌。

**Key Characteristics:**
- 黑铁载体（`#080a0b` 全屏底）+ 点状金光（`#d9b662` ≤15% 屏占）+ 偶发紫红（仅在污染相关上下文）
- 三轴并列的人格：潮湿腐烂 × 冷峻工业矮人机械 × 不可信梦境（PRODUCT.md）
- 衬线（Georgia）只在文学时刻：H1、Choice 标题、Vision caption。其余一律中文优先无衬线栈
- Material-dramatic elevation：modal 与 vision 走 22-90px 厚阴影；普通 panel 是 12-34px
- 中文是基线，英文是充气符；font-stack 头部永远是中文家族

## 2. Colors: The Lantern-Light Palette

调色板按"矿灯把一切照见的范围"分布：暖金作为光源、冷黑作为载体、四种状态色作为光晕颜色（生命的血、体力的苔、专注的远光、污染的低语）。

### Primary
- **Miner's Lantern Gold** (`#d9b662`): 单一品牌色。focus ring、所有选中态边框、`section-title` 图标、`primary-action` 渐变填充、brand-mark 字色、列表项目符号 marker。屏占 ≤15%；超过这条线就破坏稀缺感
- **Lantern Wick** (`#8c6530`): Gold 的深位变体。`primary-action` 渐变的暗端、`button-secondary-selected` 的填充。不作为独立强调使用

### Secondary (State Vocabulary)
状态色不参与装饰；只有当语义触发时才出现，且必须以条状能量或图标着色而非大面积填充：

- **Dwarven Wound** (`#b95b61`): 生命（`meter-fill.red` 渐变终点 `#8e3e45 → #b95b61`）、destructive 警示、感染矮人血迹
- **Mine Lichen** (`#88a96d`): 体力（`meter-fill.green`）、矮人友方氛围、矿区潮湿绿调
- **Distant Lantern** (`#7bb0c9`): 专注（`meter-fill.blue`）、稀有装备品质（`.item-row.rare`）、远处幻光
- **Khah's Whisper Violet** (`#8b5a9b`): 污染相关一切。`disi-avatar.stage-2` 边框、`dialogue-panel.whisper` 边框、`vision-frame` pixel art 镶边、克哈头像描边、`.item-row.corrupted` 标识

### Neutral (Mine-Black Ramp)
- **Mine Black** (`#080a0b`): 全屏底色。`body` background、`play-area` 内底、所有 modal scrim 的最深层
- **Tunnel Black** (`#0d1011`): 一阶提升。`panel-section` 渐变端、`input` 内容区
- **Surface Muted** (`rgba(26,30,30,0.82)`): 二阶提升。HUD pill / meter / disi-avatar 容器，配 `backdrop-filter: blur(12px)`
- **Lantern Cream** (`#f2ead8`): 正文颜色。8.6:1 on Mine Black。所有 body / dialogue / log 必须用它，不要用比它更暗的 ramp
- **Tunnel Mist** (`#b3aa98`): 次级正文 / muted text。约 7.5:1 on Mine Black。安全，但只用于辅助而非主信息
- **Soot Gray** (`#7f786b`): 仅用于装饰性标签（`stat-grid dt` 这类）。**约 3.6:1，未达 AA**，不允许承载任何 14px 以下正文或可交互文字
- **Gold-Line** (`rgba(206,181,116,0.18)` 与 `rgba(225,197,125,0.36)`): 卡片 / panel 1px 边框。永远是金色透明叠在 surface 上，不要用纯灰边

### Named Rules

**The 15% Lantern Rule.** Miner's Lantern Gold 在任何屏幕上的色块占比 ≤ 15%（含填充按钮、icon、边框、文本）。超过这条线，矿灯就不再是光源，变成装饰品。

**The Violet Permission Rule.** Khah's Whisper Violet 只能在污染相关上下文出现：克哈低语、第二/三阶段容器、感染矮人、堕落品质装备、幻象边缘。永远不要拿它当中性强调色用。

**The Soot Ceiling Rule.** `#7f786b` (Soot Gray) 是天花板，不是地板。它仅用于装饰性次级标签（`stat-grid dt`、`meter span` 单位）。一旦超过 14px 或承载可读信息，必须升级到 Tunnel Mist (`#b3aa98`) 或 Lantern Cream (`#f2ead8`)。

## 3. Typography

**Display Font:** Georgia, 'Times New Roman', serif（文学时刻，仅 H1、Choice 标题、Vision caption）
**Body Font:** Aptos, 'Microsoft YaHei', 'PingFang SC', 'Noto Sans SC', system-ui, sans-serif（中文优先全栈）
**Numeric Font:** 同 Body，启用 `font-variant-numeric: tabular-nums`（HUD 数值、stat-grid、meter b）

**Character:** 衬线与现代无衬线在同一屏并置 —— 衬线承载克苏鲁文学叙述（"血没有溅到你身上，它像认识你一样避开了"），无衬线承载战斗反馈和数据。两者不竞争：衬线只出现在玩家停下来读字的时刻。

### Hierarchy
- **Display** (Georgia, 600, `clamp(34px, 6vw, 52px)`, line-height 0.96): 仅 `creation-panel h1`（"黑潮矿区醒来的人"）和 `vision` 顶级 caption。`text-wrap: balance` 必加
- **Headline** (Georgia, 600, 28px, line-height 1.15): `choice-panel h2`（永久选择标题）。是 Display 之外唯一允许的衬线尺寸
- **Title** (无衬线, 800, 16px): `section-title h2`（背包/任务/日志/角色面板章节头）。配金色图标
- **Body** (无衬线, 400, 15px, line-height 1.65): 所有 paragraph、`dialogue-panel p`、`choice-list span`、`log-section li`。`text-wrap: pretty` 必加；最大行宽 65-75ch
- **Label** (无衬线, 700, 13px, letter-spacing 0.08em): `field-label`、`brand-mark`、`hud-pill` 内容、按钮文字。允许的唯一一处带 tracking 的微大写场景
- **Numeric** (无衬线, 700, 11-13px, `tabular-nums`): 所有数值（生命、体力、污染、金币、强度）。必须 tabular，否则数字跳动

### Named Rules

**The Serif-Sanctuary Rule.** Georgia 只允许在 H1 / Headline / Vision caption / Choice 标题。出现在 button、label、HUD、表格、任何 ≤16px 的位置都是错的。中文系统字体在小字号下永远赢衬线。

**The Chinese-First Stack Rule.** 所有 font-family 声明的首个真实条目必须是中文家族（"Microsoft YaHei" / "PingFang SC" / "Noto Sans SC"），即使其前面挂了 Aptos。引入任何新 web font 之前先确认中文字形完整或显式 fallback；缺中文字形的 web font 直接拒用。

**The Tabular Numeric Rule.** 任何在 HUD / stat-grid / meter / inventory 强度 / 金币 出现的数值必须启用 `font-variant-numeric: tabular-nums`。数字跳动 = 体验未完成。

## 4. Elevation: Material-Dramatic Depth

走 Material-dramatic 路线：阴影是有重量的，是矿灯下金属的反射状态。系统使用三个明确的高度层级，每一层都配套独立的阴影词汇。不接受 SaaS 风格的 0 1px 2px 0 微阴影 —— 在 `#080a0b` 底色上那种阴影根本看不见。

### Shadow Vocabulary

- **Resting** (`box-shadow: 0 12px 34px rgba(0,0,0,0.18), inset 0 1px 0 rgba(255,255,255,0.035)`): `panel-section`、`item-row` 默认。中等坠落 + 一道 inset 顶光模拟金属反光
- **Hovering** (`box-shadow: inset 0 0 0 1px rgba(255,255,255,0.018), 0 18px 60px rgba(0,0,0,0.34)`): `play-area`、`disi-avatar` resting。半悬浮的 HUD 容器
- **Imposing** (`box-shadow: 0 22px 70px rgba(0,0,0,0.42), inset 0 1px 0 rgba(255,255,255,0.04)`): `dialogue-panel`、`choice-panel`、`primary-action`。强制读者注意的关键界面，配合 `backdrop-filter: blur(12px)` 让背后矿灯模糊化
- **Dramatic** (`box-shadow: 0 30px 80px rgba(0,0,0,0.6)`): `vision-frame`、`creation-panel`（32-90px）。仪式时刻，玩家必须停下来
- **Stage-Pulse** (动画 `disi-pulse`, 1.6s ease-in-out infinite): 仅用于容器觉醒第 3 阶，传达"角色正在被吞噬"。必须配套 `@media (prefers-reduced-motion: reduce)` 替代方案（crossfade 静止边框，禁止脉冲）

### Named Rules

**The Lit-from-Above Rule.** 任何卡片/面板/按钮的 inset highlight 永远在顶部（`inset 0 1px 0 rgba(255,255,255,0.03~0.05)`）。它模拟矿灯从上方照下的反光。inset highlight 出现在其他方向是错的。

**The Backdrop-Blur Rule.** `backdrop-filter: blur()` 只允许在悬浮于 Phaser 游戏画面之上的 UI（HUD、disi-avatar、dialogue、choice、vision-overlay）。不要在 `panel-section` 或 `creation-panel` 这类全屏 UI 上用 backdrop-blur —— 那是 PRODUCT.md 明令拒绝的 SaaS 玻璃风。

**The No-Mid-Shadow Rule.** SaaS 风格的 `box-shadow: 0 1px 3px rgba(0,0,0,0.1)` 在矿黑底色上完全看不见，等于没写。要么用 Resting (12px+)，要么不要 shadow。中间挡位是浪费。

## 5. Components

### Buttons

- **Shape:** 锐边有重量的冷金属（PRODUCT.md 人格直接落地）。统一 6px 圆角（`{rounded.xs}`）。`button:active` 必须 `transform: translateY(1px) scale(0.99)` 给物理反馈
- **Primary** (`.primary-action`): Gold 渐变填充 (`linear-gradient(180deg, #d2aa51, #9c6e2f)`)、深色文字 `#141008`、800 字重、min-height 50px、padding 13px+。box-shadow 大坠落 (`0 14px 34px rgba(122,83,29,0.28)`)。专用于一屏最重要的不可逆动作
- **Secondary** (`.segmented button`, `.appearance-grid button`, `.choice-list button`): 黑底 + 金色 1px 描边（`rgba(225,211,168,0.18)`），hover 升至 `var(--line-strong)`。min-height 46px
- **Selected state** (`.selected`): Lantern Wick 渐变填充 + 金色厚边 (`rgba(217,182,98,0.86)`) + inset highlight。永远不只是改字色
- **Quiet** (`.quiet-button`): 同 Secondary，但只在内部辅助操作（重置旅程）使用。不允许放在外层 CTA 位置
- **Focus** (`:focus-visible`): 2px 金色 outline、2px offset。所有按钮都 inherit 这条 root 规则，禁止 outline: none 覆盖

### Inputs (`.creation-panel input`)

- **Style:** Mine Black 底、`rgba(220,202,150,0.3)` 1px 描边、6px 圆角、padding 13px 14px、inset 1px 10px 黑阴影（凿入感）
- **Focus:** 边框升至 Miner's Lantern Gold + `0 0 0 3px rgba(217,182,98,0.11)` 金色光晕
- **Disabled / Error:** 当前未定义。引入时遵循：disabled 用 opacity 0.5 + cursor not-allowed；error 用 Dwarven Wound 边框（不是 destructive 红，是 wound 沉郁红）

### Card / Panel (`.panel-section`, `.creation-panel`)

- **Corner Style:** 8px (`{rounded.md}`)。永远不要超过 12px；24px+ 是 PRODUCT.md 明令禁止的卡通调
- **Background:** 双层 —— linear-gradient 形成 Tunnel Black 顶 → Mine Black 底，叠加 `repeating-linear-gradient(135deg, rgba(217,182,98,0.025) ...)` 极淡金色斜纹。后者是合法的"金属凿刻"质感，不属于 codex 的 stripe ban（强度 < 3%，从内容侧根本不可读为装饰条）
- **Border:** 永远 1px Gold-Line（`rgba(206,181,116,0.18)`），强调时切 Gold-Line Strong
- **Shadow:** Resting tier。重要 modal-like panel 用 Imposing
- **Padding:** 14px 标准；`creation-panel` 30px 因为是仪式场景

### HUD Pill & Meter (`.hud-pill`, `.meter`)

- **Style:** 7px 圆角、Mine Black 76% 半透 + `backdrop-filter: blur(12px)`、inset highlight 顶光、外阴影 Hovering tier
- **Behavior:** 36px 固定高度。Meter 内嵌进度条用 `meter-fill.{red|green|blue}` 渐变，5px 高，999px 圆角

### Dialogue Panel (`.dialogue-panel`)

- **Style:** Imposing shadow + backdrop-blur，含 `whisper`（紫边）、`warning`（血红边）两种 tone 变体
- **Distinctive:** 克哈对话时左侧塞入像素 portrait（80×80, 紫描边, `image-rendering: pixelated`）
- **Voice:** 衬线 H1 不在这里出现；speaker 名 700 字重金色，正文 Lantern Cream 1.65 line-height

### Choice Panel (`.choice-panel`)

- **Style:** Imposing shadow + backdrop-blur + 中央居中 700px 最大宽度
- **Title:** Headline tier 衬线 28px（衬线允许的第二个位置）
- **Choice list:** 每项 `transform: translateX(2px)` on hover，物理向右推进暗示"做出选择"
- **Voice:** Title 衬线，描述 Body Mist 颜色，强 strong 是金色

### Inventory Item Row (`.item-row`)

- **Style:** 58px min-height、左 strong + small 信息块、右 em 属性块
- **Quality tints:** rare→Distant Lantern, corrupted→Khah's Whisper Violet, relic/mythic→Miner's Lantern Gold
- **⚠ Anti-pattern present:** 当前实现用 `inset 3px 0 0 <color>` 充当左侧 stripe，**这违反 impeccable 的 side-stripe ban**。本系统的合法替代方案：用左侧 icon + 整圈彩色 border（保留 quality 含义但消除条状），见 §6 Don't

### Vision Overlay (`.vision-overlay`)

- **Style:** 全屏 78% 透明黑底 + `backdrop-filter: blur(6px)` + 中央 `vision-frame` 12px 圆角 + Dramatic shadow + Khah's Whisper Violet 边
- **Behavior:** 点击 overlay 关闭；frame 内部 stopPropagation
- **Distinctive:** 唯一允许 12px 圆角的容器 —— 因为是仪式/幻象时刻，需要稍微更"精致"于常规 panel

## 6. Do's and Don'ts

### Do:
- **Do** verify every body text and label hits ≥ 4.5:1 against `#080a0b`. Lantern Cream (`#f2ead8`) and Tunnel Mist (`#b3aa98`) are safe; Soot Gray (`#7f786b`) is装饰天花板。
- **Do** 在所有 font-family 栈的首位放中文家族（"Microsoft YaHei" / "PingFang SC" / "Noto Sans SC"）。English 是 fallback、不是 base。
- **Do** 用 `font-variant-numeric: tabular-nums` 包住所有 HUD / stat-grid / 强度 数值。
- **Do** 给每条新动画写 `@media (prefers-reduced-motion: reduce)` 替代方案。当前 `disi-pulse` 缺这一条，是技术债。
- **Do** 用 Imposing/Dramatic shadow tier 强调仪式时刻（modal、choice、vision），让玩家停下来。
- **Do** 用 Khah's Whisper Violet 仅在污染语义触发时（觉醒第 2-3 阶、感染、低语对话、堕落装备、幻象）。
- **Do** 用 `text-wrap: balance` 包所有 h1-h3；`text-wrap: pretty` 包长 prose。

### Don't:
- **Don't** 在按钮、标签、表格、≤16px 任何文字上用 Georgia 衬线。Serif-Sanctuary Rule 死守。
- **Don't** 让 Miner's Lantern Gold 超过一屏 15% 屏占。超过这条线，矿灯变装饰品。
- **Don't** 在 `.panel-section` 或 `.creation-panel` 这类全屏 UI 用 `backdrop-filter: blur`。那是 PRODUCT.md 反对的 Notion/Linear SaaS 玻璃风。Backdrop-blur 只属于悬浮于 Phaser 之上的 HUD/dialogue/choice/vision。
- **Don't** 使用 side-stripe border（`border-left` / `border-right` 或 `inset Npx 0 0 color` > 1px）作为分类着色。**当前 `.item-row.rare/.corrupted/.relic` 是这一禁令的违反，待重构**：替代方案是整圈彩色 border + 左侧 quality icon。
- **Don't** 使用 24px+ 圆角。当前最大 12px（vision-frame），任何超过都是 PRODUCT.md 禁止的卡通调。
- **Don't** 写 `border: 1px solid X` 同时配 `box-shadow: 0 N Mpx ...` 且 M ≥ 16px 在同一按钮上 —— ghost-card 是 impeccable 通用 ban。Panel 例外（panel 必须有 shadow），按钮只能二选一。
- **Don't** 使用渐变文字（`background-clip: text` + gradient）。Miner's Lantern Gold 是单色 token，不要做成"金色渐变文字"。
- **Don't** 引入小写 ALL-CAPS tracked eyebrow（"ABOUT" / "PROCESS" / "01·"）在每个 section 顶部。当前 `brand-mark` 是合法的单一品牌 kicker，不要扩展成 section 通用 eyebrow。
- **Don't** 写 "X剧场 / 不只是 X / 真正的 X" 这类 marketing-meta 文案。当前 dialogue 文学风是合法的；任何"真正的容器觉醒"句式是 AI slop。
- **Don't** 在 SaaS 风格 `0 1px 2px rgba(0,0,0,0.1)` 阴影上浪费时间。在 Mine Black 上完全看不见。No-Mid-Shadow Rule。
- **Don't** 使用迷你传三 / Roblox 卡通 / AAA 全灰 / Notion SaaS 任何一种调子（PRODUCT.md anti-references 全部碾入这里）。
