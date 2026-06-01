# RPG-Game

《潮蚀之环》第一版原型。当前版本实现黑潮矿区开局垂直切片，后续会按地图顺序继续扩展。

## 技术栈

- Vite
- React
- TypeScript
- Phaser 3
- Zustand

## 本地运行

```bash
npm install
npm run dev
```

然后打开：

```text
http://127.0.0.1:5173
```

Windows PowerShell 如果拦截 `npm.ps1`，使用：

```bash
npm.cmd install
npm.cmd run dev
```

## 当前可玩内容

- 创建角色并同步玩家名字。
- 进入黑潮矿区开局场景。
- 克哈低语。
- 2D Phaser 矿区画面。
- 移动、攻击、翻滚、体力、生命。
- 感染矮人敌人。
- 第一个永久选择。
- 图腾残片互动。
- 基础金币掉落。
- 随机词条装备掉落。
- 背包、任务、日志、HUD。
- 本地存档。
- AIDLC 状态变更审批雏形。

## 操作

- WASD / 方向键：移动。
- J：攻击。
- Space：翻滚。
- E：互动。

## 验证

```bash
npm run build
npm audit --omit=optional
```

当前已验证：

- 构建通过。
- 安全审计 0 漏洞。
- Playwright 可打开页面并验证角色创建、Phaser 画面、永久选择、图腾互动、装备掉落和日志更新。

