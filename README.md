# 番茄钟 — 桌面应用

一个基于 Tauri v2 的番茄钟桌面应用，Rust 后端精确计时，Web 前端禅意极简设计。

## 功能特性

- **高精度计时** — Rust `std::time::Instant` 追踪墙钟时间，毫秒级精度，不受浏览器节流影响
- **单锁架构** — 所有状态（计时、设置、时间戳）统一由单个 `Mutex` 管理，彻底消除死锁
- **精美弹窗提醒** — 涟漪动画 + 毛玻璃卡片 + 自动关闭进度条，替代传统系统通知
- **强制弹窗** — 工作结束时窗口自动置顶显示，任务栏闪烁提醒（可关闭）
- **手动确认** — 工作/休息结束后需点击确认才开始下一阶段，避免自动切换
- **视觉反馈** — 圆环进度条呼吸动画 + 闪烁高亮，三种模式独立配色
- **自定义设置** — 工作/短休息/长休息时间、轮数均可调整，支持小数分钟（测试用）
- **数据持久化** — 设置和统计自动保存到本地 JSON 配置文件
- **禅意 UI** — 深靛蓝底色 + 暖琥珀主调 + 毛玻璃质感 + 噪点纹理

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (最新稳定版)

### 克隆仓库

```bash
git clone https://github.com/10086wanghu007/pomodoro-timer.git
cd pomodoro-timer
```

### 安装与运行

```bash
# 安装 Node.js 依赖
npm install

# 开发模式运行（首次会自动编译 Rust 后端，需等待几分钟）
npm run tauri dev

# 构建发布包（生成 EXE + MSI）
npm run tauri build
```

构建完成后：
- 安装包：`src-tauri/target/release/bundle/msi/`
- 可执行文件：`src-tauri/target/release/pomodoro-app.exe`

## 使用说明

| 操作 | 说明 |
|------|------|
| 开始 | 启动工作计时（默认 25 分钟） |
| 暂停 | 暂停计时，可恢复继续 |
| 重置 | 回到初始状态 |
| 保存设置 | 修改时间/轮数后点击保存 |

### 计时流程

1. 点击「开始」进入工作模式
2. 工作时间到 → 弹窗提醒「休息一下吧」
3. 点击「好的」开始休息，或点击「再专注 5 分钟」延长工作
4. 休息时间到 → 弹窗提醒「开始工作吧」
5. 点击「好的」开始下一轮工作

### 提醒方式

当一个阶段结束时：
1. **精美弹窗** — 毛玻璃卡片 + 涟漪动画 + 8秒自动关闭
2. **强制弹窗** — 窗口自动置顶显示（可关闭），任务栏闪烁
3. **系统通知** — Windows 右下角弹窗
4. **蜂鸣声** — 两声系统提示音
5. **圆环闪烁** — 界面圆环放大高亮闪烁

### 自定义设置

- **工作时间** — 默认 25 分钟，支持小数（如 0.5 = 30 秒，用于测试）
- **短休息** — 默认 5 分钟
- **长休息** — 默认 15 分钟
- **轮数** — 默认 4 轮
- **强制弹窗提醒** — 默认开启，工作结束时强制窗口置顶

## 项目结构

```
pomodoro-app/
├── src/                        # 前端
│   ├── index.html              # 禅意极简 UI + 精美弹窗组件
│   ├── styles.css              # 深靛蓝 + 琥珀色 + 毛玻璃 + 弹窗动画
│   └── main.js                 # 前端逻辑（Tauri invoke + event listen + 弹窗交互）
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # Windows 子系统入口（隐藏控制台）
│   │   ├── lib.rs              # Tauri 配置 + 后台计时循环 + 通知/声音 + 强制弹窗
│   │   ├── timer.rs            # PomodoroTimer 核心逻辑（单锁 Instant + 等待确认）
│   │   └── commands.rs         # Tauri IPC 命令 + 配置读写 + 延长/确认命令
│   ├── capabilities/           # Tauri 权限配置（窗口操作权限）
│   ├── icons/                  # 应用图标
│   ├── Cargo.toml              # Rust 依赖声明
│   ├── Cargo.lock              # Rust 版本锁定
│   ├── build.rs                # 构建脚本
│   └── tauri.conf.json         # Tauri 应用配置
├── package.json                # Node.js 依赖声明
├── package-lock.json           # Node.js 版本锁定
├── .gitignore
└── README.md
```

## 技术栈

- **后端**: Rust + Tauri v2 + serde + serde_json
- **前端**: HTML5 + CSS3 + 原生 JavaScript（零依赖）
- **通知**: tauri-plugin-notification + Windows MessageBeep API
- **窗口管理**: Tauri Window API（置顶、最小化、焦点控制）
- **字体**: 思源宋体 (Noto Serif SC) + 思源黑体 (Noto Sans SC)

## 核心功能实现

### 精美弹窗系统
- 毛玻璃背景 + 渐变卡片设计
- 涟漪扩散动画效果
- 浮动时钟图标
- 自动关闭进度条（8秒）
- 支持「好的」确认和「再专注 5 分钟」延长

### 强制弹窗机制
- 工作结束时自动取消最小化
- 窗口置顶显示（3秒后自动取消）
- 任务栏图标闪烁提醒
- 可在设置中自由开关

### 手动确认流程
- 工作/休息结束后暂停计时
- 显示「等待确认」状态
- 用户点击后才开始下一阶段
- 支持「再专注 5 分钟」延长当前工作

### 测试时间支持
- 时间输入支持小数（最小 0.1 分钟 = 6 秒）
- 步进值 0.1，方便快速调整
- 适合开发测试使用

## 开发工具

本项目由 AI 辅助开发，使用以下工具链：

- **Agent**: [Claude Code](https://docs.anthropic.com/en/docs/claude-code) — Anthropic 官方 CLI 编程助手
- **Model**: mimo-v2.5-pro
- **Skills**:
  - `tauri-v2` — Tauri v2 框架开发指导（项目初始化、IPC 命令、插件配置）
  - `frontend-design` — 前端 UI 设计指导（禅意极简风格、CSS 动效、色彩体系）
