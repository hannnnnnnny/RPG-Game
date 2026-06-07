# Product

## Register

product

> 注：项目是 split register。开局 / 角色创建画面走 brand 调性（大字号衬线、`潮蚀之环` brand-mark、克苏鲁文学开场白）；进入游戏后切换到 product UI（HUD、背包、对话、任务、日志）。主 register 是 product —— 玩家在 in-game shell 里花的时间最长，所有决策都围绕这里展开。brand-flavored 入口按 brand 标准单独评审。

## Users

魂类 / 黑暗向 ARPG 玩家、暗黑破坏神风格刷装党、克苏鲁与多结局叙事爱好者。

使用场景：单人沉浸 session，桌面浏览器为主（1100px+ 是主断点，<1100px 切栈式布局）。玩家进来想要的不是"快速完成任务"，而是"被一个世界吞掉"。

要做的事：
- 创建角色，给自己起一个全局同步的名字
- 在 2D Phaser 矿区里探索、战斗、翻滚
- 听克哈低语、读日志、做永久选择
- 拣装备、读词条、试搭配
- 看着污染 / 觉醒 / 理智 三个数值，理解自己正在变成什么

## Product Purpose

《潮蚀之环》是一款 Web-first 2D 暗黑克苏鲁 ARPG。第一版交付黑潮矿区开局垂直切片，后续按地图顺序扩展。

核心机制：
- **AIDLC**：NPC 记住玩家行为，规则系统影响局部世界状态
- **永久选择**：玩家决定不可回滚，至少 8 主结局 + 2 特殊分支
- **刷装循环**：随机词条 + 装备品质等级（rare / corrupted / relic / mythic）
- **碎片叙事**：玩家失忆开局，世界真相通过低语、幻象、日志拼回

成功标准：玩家在矿区跑完一遍后，会主动想知道"我到底是谁"、"如果我刚才选了另一个会怎样"，而不是"下一关在哪"。

## Brand Personality

三轴并行，缺一不可：

1. **潮湿 / 腐烂 / 不洁**：UI 表面应该有"被黑潮浸过"的质感。背景的 `repeating-linear-gradient` 噪点、`vision-frame` 紫边、`disi-avatar.stage-3` 的脉冲都是这一轴。
2. **冷峻 / 工业 / 矮人机械**：金色 (`#d9b662`) + 黑铁，符文凿刻感的字重对比，HUD pill 像合金铭牌。`tabular-nums` 是这一轴的实现。
3. **梦境 / 失忆 / 不可信**：偶尔的字号错位、blur 叠层、双重视觉、`whisper` tone 紫边对话。玩家应该不确定刚才看到的对话是不是真发生过。

声调：第三人称冷峻叙述 + 偶尔直击玩家的低语 ("血没有溅到你身上，它像认识你一样避开了")。不解释，不安慰，不给确定性。

## Anti-references

绝不要变成的东西：

- **迷你传三 / 卡牌手游头部面板**：顶部三段渐变金边、堆叠的"·1·2·3"章节卡、土豪带货 CTA
- **Roblox / 卡通 Mini 游戏调**：亮色饱和、厚字、≥16px 圆角、Material You 味
- **AAA 全灰设置菜单**：黑灰加青色强调、信息密度低、个性归零的通用 dark mode
- **Notion / Linear 型 SaaS 净色面板**：玻璃柔和渐变、为数据查看而设计、零沉浸感

特别警惕（impeccable 通用 ban 在本项目尤其相关）：
- 不要 24-32px+ 圆角的卡片（当前 `--radius: 8px` 已合适，别瞎改）
- 不要"X theater / 不只是 X"式 marketing 文案
- 不要 1px border + 大 box-shadow 双戴的 ghost-card
- 不要为了"现代感"把 muted 灰文字加到接近背景 —— 暗调更要拉够对比

## Design Principles

1. **沉浸高于易用**：UI 是世界的一部分，不是工具栏。氛围优先于功能直白；但 a11y 不让步。
2. **文字是主角**：对话面板、日志、任务和战斗 HUD 同等重要。文字承载叙事，排版承载文字。
3. **不解释**：克苏鲁感来自留白和暗示，不是科普。空状态、错误状态都用世界内语言（"矿袋空着，只剩黑潮和铁锈的气味"），不是 SaaS 标准文案。
4. **选择有重量**：所有不可逆动作（永久选择、装备替换、重置旅程）必须有视觉权重和文案警示。`destructive` 不靠红色，靠犹豫。
5. **中文优先**：所有排版决策按中文衡量。Aptos / 微软雅黑 / PingFang SC 是基线，Georgia 只在文学段落（h1、章节标题、vision caption）。英文是充气符。

## Accessibility & Inclusion

- **WCAG AA 体文 4.5:1**：所有 body / 提示 / 标签必须达标。已知风险点：`--dim: #7f786b` on `--bg: #080a0b` 实测约 3.6:1，**不能用于正文**，只能用作非关键装饰（如 `stat-grid dt` 标签）。`--muted: #b3aa98` 约 8:1，OK。审查时优先盯 `.muted`、`.choice-list span`、`.disi-avatar-meta` 这些位置。
- **prefers-reduced-motion 必须支持**：当前 `disi-pulse`、`backdrop-filter: blur`、`transform` 过渡都需要 reduced-motion fallback（crossfade 或瞬切）。新增任何动效都要双轨。
- **中文字体优先**：font-stack 头部必须是中文家族（Aptos / 微软雅黑 / PingFang SC / Noto Sans SC）。新加的 web font 要确认中文字形完整或显式 fallback。
- **键盘全控 + focus-visible**：Phaser 画布以外，所有 React UI 都要可纯键盘操作。`focus-visible` 已有金色 outline，新组件不能 outline-none 跳过。choice-panel、vision-overlay 这类 modal 要正确的 focus trap。
