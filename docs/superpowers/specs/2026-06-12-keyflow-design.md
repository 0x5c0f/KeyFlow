# KeyFlow — 非粘贴型密码框辅助输入工具 设计文档

## 1. 概述

### 1.1 项目背景

许多网站和应用通过 JavaScript 的 `onpaste`/`oncopy` 事件禁用密码框的粘贴功能。这迫使用户手动输入长复杂密码，极易出错。尤其在 VNC 远程登录、SSH 终端、远程桌面等场景下，问题更为突出。

### 1.2 项目目标

开发一个跨平台（Linux/Windows/macOS）的桌面小工具，通过模拟键盘逐字输入的方式绕过粘贴限制，帮助用户快速、准确地输入复杂密码。

### 1.3 核心价值

- **不粘贴** — 模拟物理键盘输入，绕过 `onpaste` 限制
- **不存储密码** — 密码不落地，从密码管理器实时获取或从剪贴板读取
- **一键触发** — 全局热键 + 鼠标位置自动聚焦，操作流程最简
- **可扩展** — 抽象的 PasswordProvider trait，方便集成各种密码管理器

## 2. 架构设计

### 2.1 项目结构

单 Package，lib + bin 双 target。遵循 Cargo 官方对小到中型项目的推荐。

```
keyflow/
├── Cargo.toml
├── keyflow.toml.example          # 配置文件示例
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs                    # 库入口，导出公共 API
│   ├── main.rs                   # CLI 二进制入口
│   ├── error.rs                  # 统一错误类型
│   ├── config/
│   │   ├── mod.rs                # Config 结构体、加载/保存
│   │   └── binding.rs            # Binding 定义（热键 → provider 映射）
│   ├── provider/
│   │   ├── mod.rs                # PasswordProvider trait 定义
│   │   ├── clipboard.rs          # 剪贴板模式（默认 provider）
│   │   └── bitwarden.rs          # Bitwarden CLI 集成
│   ├── input/
│   │   ├── mod.rs                # InputEngine trait + 工厂方法
│   │   ├── keyboard.rs           # 键盘模拟（封装 enigo）
│   │   └── mouse.rs              # 鼠标操作（获取位置、点击）
│   ├── hotkey/
│   │   ├── mod.rs                # HotkeyManager trait + 工厂方法
│   │   ├── linux.rs              # Linux 实现（X11/Wayland）
│   │   ├── windows.rs            # Windows 实现
│   │   └── macos.rs              # macOS 实现
│   ├── daemon/
│   │   └── mod.rs                # daemon 生命周期管理
│   └── cli/
│       ├── mod.rs                # clap 命令树
│       ├── run.rs                # keyflow run
│       ├── stop.rs               # keyflow stop
│       ├── status.rs             # keyflow status
│       ├── bind.rs               # keyflow bind add/remove/list
│       ├── config_cmd.rs         # keyflow config show/path
│       └── unlock.rs             # keyflow unlock
└── tests/
    ├── config_tests.rs
    ├── provider_tests.rs
    └── binding_tests.rs
```

### 2.2 核心 Trait

```rust
/// 密码提供者 trait — 可扩展的核心抽象
pub trait PasswordProvider {
    /// 获取密码（明文，仅在内存中短暂存在）
    fn get_password(&self) -> Result<String, ProviderError>;

    /// 提供者名称（用于配置和日志）
    fn name(&self) -> &str;
}

/// 输入引擎 trait — 键盘/鼠标模拟抽象
pub trait InputEngine {
    /// 获取当前鼠标位置
    fn get_mouse_position(&self) -> Result<(i32, i32), InputError>;

    /// 在指定位置点击鼠标
    fn click_at(&self, x: i32, y: i32) -> Result<(), InputError>;

    /// 模拟键盘输入文本
    fn type_text(&self, text: &str) -> Result<(), InputError>;
}

/// 热键管理器 trait — 全局热键抽象
pub trait HotkeyManager {
    /// 注册全局热键
    fn register(&mut self, hotkey: &str, callback: Box<dyn Fn()>) -> Result<(), HotkeyError>;

    /// 进入事件循环（阻塞）
    fn run(&self) -> Result<(), HotkeyError>;
}
```

### 2.3 核心数据流

```
用户将鼠标悬停在目标密码框上
    ↓
用户按全局热键（如 F7）
    ↓
HotkeyManager 捕获事件
    ↓
查找 Config 中对应的 Binding
    ↓
InputEngine.get_mouse_position() — 获取鼠标位置
    ↓
InputEngine.click_at(x, y) — 在鼠标位置点击，聚焦密码框
    ↓
根据 Binding.provider 类型调用 PasswordProvider.get_password()
    ├── clipboard: 从剪贴板读取密码
    └── bitwarden: 自动 unlock（如需要）→ 调用 `bw get password <item_id>`
    ↓
InputEngine.type_text(password) — 模拟键盘逐字输入
    ↓
清除剪贴板（安全清理）
```

## 3. 配置文件

### 3.1 文件路径

| 平台 | 路径 |
|------|------|
| Linux | `~/.config/keyflow/keyflow.toml` |
| macOS | `~/Library/Application Support/keyflow/keyflow.toml` |
| Windows | `%APPDATA%\keyflow\keyflow.toml` |

### 3.2 格式

```toml
# keyflow.toml

# 全局设置
[settings]
# 输入完成后自动清除剪贴板（秒），0 表示不清除
clipboard_clear_after_secs = 5

# 密码提供者配置
[[providers]]
type = "clipboard"
# 无需额外配置，直接从剪贴板读取

[[providers]]
type = "bitwarden"
# 主密码通过环境变量 BW_PASSWORD 传入（用户自行设置）
# 会话通过环境变量 BW_SESSION 管理（工具自动 unlock）
# cli_path = "bw"  # 可选，默认在 PATH 中查找

# 热键绑定
[[bindings]]
name = "vnc-server-1"
hotkey = "F7"
provider = "bitwarden"
# Bitwarden 条目 ID（通过 `bw list items --search "xxx"` 查询）
item_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

[[bindings]]
name = "ssh-gateway"
hotkey = "F8"
provider = "bitwarden"
item_id = "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy"

[[bindings]]
name = "from-clipboard"
hotkey = "F9"
provider = "clipboard"
# 无需 item_id，直接从剪贴板读取
```

## 4. Bitwarden 集成

### 4.1 认证流程

KeyFlow 不存储任何 Bitwarden 凭证。认证完全依赖 Bitwarden CLI 自身的会话管理。

```
用户设置环境变量 BW_PASSWORD（写入 .bashrc/.zshrc）
    ↓
KeyFlow 检查 BW_SESSION 是否有效（bw status）
    ├── 已解锁（status = "unlocked"）→ 直接使用
    └── 未解锁 → 自动调用 `bw unlock --passwordenv BW_PASSWORD`
    ↓
使用 BW_SESSION 调用 `bw get password <item_id>`
    ↓
返回密码，用于键盘模拟输入
```

### 4.2 关键特性

- **BW_SESSION 无时间过期** — 根据 Bitwarden 官方文档，session 在 `bw lock` 或 `bw logout` 之前一直有效
- **非交互式解锁** — 通过 `bw unlock --passwordenv BW_PASSWORD` 实现，无需手动输入主密码
- **密码不落地** — 密码仅在内存中短暂存在，用于键盘输入后即丢弃

### 4.3 用户配置步骤

```bash
# 1. 安装 Bitwarden CLI
# 2. 首次登录（只需一次）
bw login

# 3. 设置主密码环境变量（写入 shell 配置文件）
export BW_PASSWORD="your-master-password"

# 4. KeyFlow 首次运行时会自动 unlock，之后一直有效
```

## 5. CLI 命令设计

### 5.1 命令树

```
keyflow
├── run                # 启动 daemon（前台/后台）
├── stop               # 停止 daemon
├── status             # 查看 daemon 状态、BW_SESSION 状态
├── bind
│   ├── add            # 添加热键绑定
│   ├── remove         # 删除热键绑定
│   └── list           # 列出所有绑定
├── config
│   ├── show           # 显示当前配置
│   └── path           # 显示配置文件路径
└── unlock             # 解锁 Bitwarden（封装 bw unlock）
```

### 5.2 CLI = 配置管理，Daemon = 核心能力

- **CLI** 负责配置和管理（`bind`、`config`、`unlock`）
- **Daemon** 负责核心能力（热键监听、鼠标点击、键盘输入）
- 触发输入的唯一路径是 daemon + 热键，不提供 CLI 触发输入的方式

### 5.3 用法示例

```bash
# 启动 daemon
keyflow run                    # 前台运行（调试用）
keyflow run --daemon           # 后台运行

# 停止 daemon
keyflow stop

# 查看状态
keyflow status

# 管理热键绑定
keyflow bind add \
  --name "vnc-server" \
  --hotkey "F7" \
  --provider bitwarden \
  --item-id "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

keyflow bind remove --name "vnc-server"
keyflow bind list

# 配置
keyflow config show
keyflow config path

# Bitwarden 解锁
keyflow unlock
```

## 6. 依赖选型

### 6.1 核心依赖

| 依赖 | 用途 | 选型理由 |
|------|------|---------|
| `clap` (v4) | CLI 参数解析 | Rust CLI 标准选择，derive macro 支持好 |
| `enigo` (v0.6) | 键盘模拟 + 鼠标操作 | 跨平台，API 简洁，支持 X11/Wayland/Windows/macOS |
| `arboard` | 剪贴板读写 | 跨平台剪贴板库，比 `clipboard` crate 更活跃 |
| `serde` + `toml` | 配置文件序列化 | TOML 格式解析，Rust 生态标准 |
| `dirs` | 获取用户配置目录 | `~/.config/keyflow/` 跨平台解析 |
| `anyhow` | 应用级错误处理 | 简化错误传播，适合 CLI 应用 |
| `thiserror` | 自定义错误类型 | 用于库层的结构化错误定义 |
| `log` + `env_logger` | 日志 | daemon 运行时的日志输出 |

### 6.2 全局热键方案

| 平台 | 方案 | 说明 |
|------|------|------|
| Linux | X11 `XGrabKey` / Wayland `libei` | 底层事件监听 |
| Windows | `RegisterHotKey` Win32 API | 系统级热键注册 |
| macOS | `CGEventTap` | 全局事件拦截 |

> 注意：`enigo` 不提供全局热键功能，需要自行实现或使用专门的 crate（如 `global-hotkey`）。

## 7. 错误处理

### 7.1 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum KeyflowError {
    #[error("配置文件未找到: {path}")]
    ConfigNotFound { path: String },

    #[error("配置解析失败: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("热键注册失败: {hotkey} — {reason}")]
    HotkeyRegistration { hotkey: String, reason: String },

    #[error("密码提供者错误: {0}")]
    Provider(#[from] ProviderError),

    #[error("输入模拟失败: {0}")]
    Input(#[from] InputError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Bitwarden 未解锁，请设置 BW_PASSWORD 环境变量")]
    BitwardenLocked,

    #[error("Bitwarden 条目未找到: {item_id}")]
    BitwardenItemNotFound { item_id: String },

    #[error("剪贴板为空")]
    ClipboardEmpty,

    #[error("bw 命令执行失败: {stderr}")]
    BitwardenCliError { stderr: String },
}
```

### 7.2 错误处理原则

1. **用户友好的错误消息** — 不暴露内部细节，给出可操作的建议
2. **Provider 错误优雅降级** — Bitwarden 失败时提示用户检查状态，不 panic
3. **daemon 不因单次错误退出** — 热键触发失败时 log 错误，继续运行
4. **退出码规范** — 0 成功，1 用户错误，2 系统错误

## 8. 测试策略

| 层级 | 测试类型 | 覆盖内容 |
|------|---------|---------|
| 单元测试 | Provider trait 测试 | Mock provider，验证 get_password 逻辑 |
| 单元测试 | Config 解析测试 | TOML 解析、默认值、边界情况 |
| 单元测试 | Binding 匹配测试 | 热键 → provider 映射查找 |
| 集成测试 | Bitwarden 集成 | 需要真实 `bw` CLI，标记 `#[ignore]` |
| 集成测试 | 剪贴板读写 | 跨平台剪贴板操作验证 |
| 手动测试 | 端到端流程 | daemon 启动 → 热键触发 → 鼠标点击 → 键盘输入 |

### 8.1 测试原则

1. Provider 层用 trait mock，不依赖真实密码管理器
2. 平台相关代码用 `cfg` 条件编译测试
3. 集成测试标记 `#[ignore]`，需要外部工具的测试默认跳过
4. 不测试键盘模拟的实际效果（依赖 OS 环境，手动验证）

## 9. MVP 功能清单

| # | 功能 | 优先级 | 说明 |
|---|------|--------|------|
| 1 | 从剪贴板读取密码 → 模拟键盘输入 | P0 | 核心能力 |
| 2 | 鼠标位置点击聚焦 + 键盘输入 | P0 | 正确的交互方式 |
| 3 | 全局热键触发 | P0 | daemon 核心 |
| 4 | Bitwarden 集成（BW_SESSION 自动管理） | P0 | 主要密码来源 |
| 5 | 预配置绑定（热键 → Bitwarden 条目） | P0 | 一键输入 |
| 6 | 配置文件管理（bind add/remove/list） | P0 | 用户可配置 |
| 7 | `keyflow run/stop/status` | P0 | daemon 生命周期 |
| 8 | `keyflow unlock` | P1 | Bitwarden 解锁便利 |
| 9 | 按键延迟/速度配置 | P2 | 后续版本 |
| 10 | 特殊键支持（Tab、Enter） | P2 | 后续版本 |
| 11 | TUI 交互界面 | P3 | 后续版本 |
| 12 | GUI/系统托盘 | P3 | 后续版本 |

## 10. 后续扩展方向

- **更多密码管理器** — KeePass、1Password、LastPass 等（实现 PasswordProvider trait）
- **特殊键序列** — 支持 `{TAB}{ENTER}` 等自动序列（类似 KeePass Auto-Type）
- **按键延迟配置** — 适配不同应用的输入速度要求
- **TUI 界面** — 终端内交互式管理绑定
- **GUI/系统托盘** — Windows 用户友好的图形界面
- **两通道混淆** — 抵抗键盘记录器（KeePass 的 Two-Channel Auto-Type Obfuscation）
