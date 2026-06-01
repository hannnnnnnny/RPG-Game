# 开发计划

## 当前目标

先完成《潮蚀之环》的正式文档和第一版开发基线。第一版只实现黑潮矿区开局场景，但架构按完整游戏预留。

## 阶段

| 阶段 | 状态 | 内容 |
|---|---|---|
| 1 | complete | 讨论并锁定核心游戏设计 |
| 2 | complete | 创建 GAME_DESIGN.md |
| 3 | complete | 创建 TECH_ARCHITECTURE.md |
| 4 | complete | 建立项目规划与进度记录 |
| 5 | complete | 搭建 Vite + React + TypeScript + Phaser 项目骨架 |
| 6 | complete | 实现黑潮矿区开局垂直切片 |
| 7 | complete | 实现基础战斗、掉落、存档和 AIDLC 雏形 |

## 第一版范围

第一版只做黑潮矿区开局：

- 角色创建。
- 玩家名字全局存储。
- 克哈低语。
- 黑潮矿区地图。
- 基础移动、攻击、翻滚、体力。
- 简单敌人。
- 第一个永久选择。
- 图腾残片互动。
- 基础装备和金币掉落。
- 存档和 worldState。
- AIDLC 规则审批雏形。

## 暂缓内容

- 伊芙正式出场。
- 希多正式出场。
- 13 个完整地牢。
- 完整装备强化和重铸界面。
- 完整 AI NPC 接入。
- 全部结局和阵营任务。

## 错误记录

| 日期 | 问题 | 处理 |
|---|---|---|
| 2026-06-01 | PowerShell 阻止执行 `npm.ps1` | 改用 `npm.cmd` 运行 npm 命令 |
| 2026-06-01 | Vite 8 类型解析要求更现代的 moduleResolution | 将 TypeScript 配置改为 `moduleResolution: Bundler` |
| 2026-06-01 | TypeScript project reference 不允许被引用项目 `noEmit` | 改为构建脚本显式执行两个 `tsc --noEmit` 类型检查 |
| 2026-06-01 | Playwright 用中文按钮名定位失败 | 改用稳定 CSS 选择器 `.primary-action` 继续验证 |
| 2026-06-01 | 矿井出口重复按 E 会反复写入 `escape_mine` 日志 | 在 AIDLC 审批规则和出口交互处增加一次性保护 |
