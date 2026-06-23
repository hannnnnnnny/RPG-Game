# 《潮蚀之环》构建 & 发布到 itch.io

本项目已验证可在 **Godot 4.3** 下运行并导出。本文档讲怎么打包 + 免费发到 itch.io。

> 已确认：所有 GDScript 通过解析校验，主场景 + 黑潮矿区场景 headless 运行零报错，Web 导出成功（约 39MB）。

## 0. 前置

- **Godot 4.3 standard 版**（非 .NET）
- **导出模板 4.3.stable** 已安装到：
  - Windows: `%APPDATA%\Godot\export_templates\4.3.stable\`
  - 检查方法：编辑器 → Editor → Manage Export Templates，应显示 "4.3.stable installed"

`export_presets.cfg` 已随仓库提交，含两个预设：
- **Web**（单线程变体，itch.io 友好，无需 SharedArrayBuffer）
- **Windows Desktop**（x86_64 exe）

## 1. 命令行导出（推荐，可脚本化）

把 `<godot>` 换成你的 Godot 可执行文件路径（本机绿色版在 `C:\Users\harry\GodotPortable\Godot_v4.3-stable_win64_console.exe`）。

**Web 版**（浏览器玩）：
```bash
<godot> --headless --path godot --export-release "Web" dist/web/index.html
```

**Windows exe**（发朋友 / GitHub Release）：
```bash
<godot> --headless --path godot --export-release "Windows Desktop" dist/windows/TidesOfKhah.exe
```

输出落在仓库根的 `dist/`（已 gitignore，不进版本库）。

> 编辑器里导出：Project → Export → 选预设 → Export Project。

## 2. 本地测试 Web 版

Web 构建必须用 HTTP 服务（不能直接双击 `file://` 打开）。本项目用单线程导出，**普通静态服务器即可**，不需要 COOP/COEP 头：

```bash
npx --yes http-server dist/web -p 8099 -c-1
# 浏览器打开 http://127.0.0.1:8099/
```

加载约几秒（34MB wasm）。看到角色创建界面就成功。

## 3. 发布到 itch.io（完全免费）

### 3.1 建页面
1. 注册 https://itch.io 账号
2. 右上 → Upload new project
3. **Kind of project** 选 **HTML**
4. 标题：潮蚀之环 / Tides of Khah

### 3.2 上传 Web 构建
1. 把 `dist/web/` 整个文件夹打包成 zip：
   ```bash
   cd dist/web && zip -r ../tides-web.zip . && cd ../..
   ```
2. 在 itch 页面上传 `tides-web.zip`
3. 勾选该文件的 **"This file will be played in the browser"**
4. **Embed options**：
   - Viewport：1280 × 720（匹配项目分辨率）
   - 勾 **Mobile friendly**（可选）
   - 勾 **Fullscreen button**
   - **SharedArrayBuffer support**：本项目单线程导出，**不需要**勾（勾了也行）
5. **Visibility**：先设 Draft / Restricted 自己测，OK 后再 Public

### 3.3 Windows 版（可选，作为下载）
1. `dist/windows/` 打包 zip 一起传
2. 不勾 "played in browser"，作为可下载文件
3. 玩家下载解压双击 `TidesOfKhah.exe`

## 4. 用 butler 自动化上传（可选，进阶）

[butler](https://itch.io/docs/butler/) 是 itch 官方命令行推送工具，适合频繁更新：

```bash
# 安装后登录
butler login
# 推送 Web 版（user/game:channel）
butler push dist/web your-itch-name/tides-of-khah:html
# 推送 Windows 版
butler push dist/windows your-itch-name/tides-of-khah:windows
```

之后每次更新只跑 `butler push`，itch 自动增量同步、玩家自动拿到新版。

## 5. AI 生成内容声明

商店页/上传时如实勾选"包含 AI 生成内容"——本项目的立绘 jpg 是 AI 生成的。itch.io 允许，但需声明。Steam 上架时同理。

## 6. 当前内容范围（重要）

发布前清楚知道：**现在只有黑潮矿区开场**（约 2-3 分钟流程）。适合作为：
- **Demo / Prototype**：免费发，攒反馈、攒关注
- **开发日志**：itch 支持 devlog，边做边发

**不建议现在标价卖**——内容还远不够。按 GAME_DESIGN.md 把内容做到 2-4 小时流程、几个地牢 + Boss 后，再考虑正式发售（或上 Steam）。

## 7. 已知占位（不影响运行，后续替换）

- 角色/敌人/瓦片都是**程序化烘焙**的像素艺术，非手绘。换真素材见 `README.md` 对应章节
- 无音效
- 单地图（矿区），灰灯镇及之后未做
