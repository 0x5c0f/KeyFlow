# KeyFlow

[English](README.en.md) | 简体中文

非粘贴型密码框辅助输入工具 — 通过模拟键盘输入绕过禁止粘贴的密码框，同时支持正常编辑器的粘贴模式。

## 适用场景

- 任何禁用粘贴功能的输入框
- 配合 Bitwarden 等密码管理器使用
- 正常编辑器中的格式化粘贴
- 固定文本快速输入（邮箱、API Key 等）

## 工作原理

1. 用户将鼠标悬停在目标输入框上
2. 按下全局热键（如 F7）
3. KeyFlow 自动：获取鼠标位置 → 点击聚焦 → 从密码管理器获取密码 → 输入密码
4. 根据 `input_mode` 配置选择输入方式：
   - `type`（默认）：模拟键盘逐字输入，绕过禁粘贴字段
   - `paste`：通过剪贴板 + Ctrl+V 粘贴，保留格式

## 系统要求

| 平台 | 状态 | 依赖 |
|------|------|------|
| Linux (X11) | ✅ 支持 | 无（纯 Rust X11 后端） |
| Linux (Wayland) | ❌ 待支持 | — |
| macOS | ❌ 待支持 | — |
| Windows | ❌ 待支持 | — |

> **注意：** Linux X11 版本使用纯 Rust 的 `x11rb` 后端，无需安装额外系统依赖。

## 安装

### 从源码构建

```bash
git clone https://github.com/0x5c0f/keyflow.git
cd keyflow
make build
make install  # 安装到 ~/.local/bin/（无需 sudo）
```

### 从 tarball 安装

```bash
# 下载并解压
tar -xzf keyflow-*-x86_64-linux.tar.gz
cd keyflow-*-x86_64-linux

# 一键安装（二进制 + 配置 + systemd 服务）
make install

# 卸载
make uninstall

# 升级（停止 → 安装 → 启动）
make upgrade
```

### systemd 服务（推荐）

安装后自动启用 systemd 用户服务，实现开机自启和进程守护：

```bash
# 启动服务
systemctl --user start keyflow

# 查看状态
systemctl --user status keyflow

# 查看日志
journalctl --user -u keyflow -f
```

### 开发模式

```bash
make build
# 二进制文件在 target/debug/keyflow
```

## 快速开始

### 1. 查看帮助

```bash
keyflow --help
```

### 2. 配置 Bitwarden（可选）

```bash
# 安装 Bitwarden CLI
npm install -g @bitwarden/cli

# 首次登录
bw login

# 解锁并保存会话（交互式输入密码）
keyflow unlock
```

### 3. 添加热键绑定

```bash
# 剪贴板 + 逐字符输入（绕过禁粘贴字段）
keyflow bind add --name "my-server" --hotkey "F7" --provider clipboard

# Bitwarden + 逐字符输入
keyflow bind add --name "vnc-server" --hotkey "F8" --provider bitwarden --item-id "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

# 静态文本（明文）
keyflow bind add --name "邮箱" --hotkey "F9" --provider static --content "user@example.com"

# 静态文本（加密）
keyflow bind add --name "API Key" --hotkey "F10" --provider static --content "enc:v1:xxx" --encrypted

# 组合键 — 使用修饰键
keyflow bind add --name "secure" --hotkey "Ctrl+Shift+F7" --provider clipboard
```

### 热键格式

支持单键和组合键，用 `+` 连接修饰键：

| 格式 | 示例 | 说明 |
|------|------|------|
| 单键 | `F7` | 功能键直接使用 |
| 修饰键+键 | `Ctrl+F7` | 一个修饰键 |
| 多修饰键+键 | `Ctrl+Shift+F7` | 多个修饰键 |
| 修饰键+字母 | `Ctrl+P` | 修饰键+普通键 |

**支持的修饰键：** `Ctrl`、`Shift`、`Alt`、`Super`

**支持的按键：** `F1`-`F24`、`A`-`Z`、`0`-`9`、`Space`、`Tab`、`Esc`、`Enter`、`Backspace`、`Delete`、`Insert`、`Home`、`End`、`PageUp`、`PageDown`、方向键、标点符号

查看绑定：
```bash
keyflow bind list
```

### 4. 启动 daemon

```bash
# 前台运行（调试用）
keyflow run

# 后台运行
keyflow run --daemon
```

### 5. 使用

1. 将鼠标悬停在目标输入框上
2. 按 F7（或你配置的热键）
3. 密码自动输入

## 输入模式

每个 binding 可配置 `input_mode`，控制文本如何传递到目标字段：

| 模式 | 行为 | 适用场景 |
|------|------|---------|
| `auto` | 默认，等同于 `type` | 未配置时的安全默认值 |
| `type` | 逐字符键盘输入 | 禁粘贴字段（VNC、密码框） |
| `paste` | Ctrl+V 剪贴板粘贴 | 正常编辑器（保留格式） |

**配置示例：**

```toml
# 逐字符输入（绕过禁粘贴字段）
[[bindings]]
name = "VNC 密码"
hotkey = "F7"
provider = "clipboard"
input_mode = "type"

# 粘贴模式（保留格式）
[[bindings]]
name = "编辑器粘贴"
hotkey = "F8"
provider = "clipboard"
input_mode = "paste"
```

## 静态文本输入

除了从剪贴板或 Bitwarden 获取内容，还可以直接在配置文件中定义要输入的文本：

### 明文模式

```toml
[[bindings]]
name = "邮箱地址"
hotkey = "F7"
provider = "static"
content = "user@example.com"
```

### 加密模式

对于敏感内容（如 API Key），可以使用加密存储：

1. 在配置文件中设置加密密钥：

```toml
[settings]
encryption_key = "your-secret-key"
```

2. 使用 `keyflow encrypt` 命令加密内容：

```bash
keyflow encrypt "your-api-key"
# 输出: enc:v1:aGVsbG8gd29ybGQ...
```

3. 在绑定中使用加密内容：

```toml
[[bindings]]
name = "API Key"
hotkey = "F8"
provider = "static"
content = "enc:v1:aGVsbG8gd29ybGQ..."
encrypted = true
```

**安全说明：** 加密密钥存储在配置文件中，与加密内容在同一位置。这提供了基本的保护，防止配置文件被直接查看时泄露敏感内容。

## 剪贴板清理

每个 binding 可独立配置剪贴板清理时间：

```toml
# 全局默认 5 秒后清理
[settings]
clipboard_clear_after_secs = 5

# 此 binding 3 秒后清理
[[bindings]]
name = "快速清理"
hotkey = "F7"
provider = "clipboard"
clipboard_clear_after_secs = 3

# 此 binding 不清理
[[bindings]]
name = "保留剪贴板"
hotkey = "F8"
provider = "clipboard"
clipboard_clear_after_secs = 0
```

**优先级：** binding 级别 > 全局设置

**安全机制：** 清理前会检查剪贴板内容是否仍为输入的文本。如果用户在等待期间复制了新内容，不会误删。

## CLI 命令

```
keyflow
├── run              # 启动 daemon（监听热键）
├── stop             # 停止 daemon
├── status           # 查看 daemon 和 Bitwarden 状态
├── bind
│   ├── add          # 添加热键绑定
│   ├── remove       # 删除热键绑定
│   └── list         # 列出所有绑定
├── config
│   ├── show         # 显示当前配置
│   └── path         # 显示配置文件路径
├── unlock           # 解锁 Bitwarden
└── encrypt          # 加密文本（用于 static 绑定）
```

## 配置文件

配置文件路径：`~/.config/keyflow/keyflow.toml`

完整配置示例见 [`keyflow.toml.example`](keyflow.toml.example)。

```toml
[settings]
clipboard_clear_after_secs = 5
# encryption_key = "your-secret-key"  # 用于加密 static 绑定的内容

[[bindings]]
name = "VNC 密码"
hotkey = "F7"
provider = "bitwarden"
item_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
input_mode = "type"
cache_secs = 300

[[bindings]]
name = "编辑器粘贴"
hotkey = "F8"
provider = "clipboard"
input_mode = "paste"
clipboard_clear_after_secs = 0

[[bindings]]
name = "邮箱地址"
hotkey = "F9"
provider = "static"
content = "user@example.com"
```

## 开发

```bash
# 构建
make build

# 运行测试
make test

# 代码检查
make check

# 清理
make clean

# 显示所有可用命令
make help
```

## 架构

```
src/
├── lib.rs          # 库入口
├── main.rs         # CLI 入口
├── error.rs        # 统一错误类型
├── crypto.rs       # 加密/解密（AES-256-GCM + Argon2id）
├── config/         # 配置管理（TOML 解析）
│   ├── mod.rs      # Config、Settings
│   └── binding.rs  # Binding、InputMode
├── provider/       # 密码提供者
│   ├── mod.rs      # PasswordProvider trait
│   ├── clipboard.rs# 剪贴板提供者
│   ├── bitwarden.rs# Bitwarden CLI 提供者
│   ├── static_provider.rs # 静态文本提供者
│   └── cached.rs   # 密码缓存包装器
├── input/          # 输入模拟（键盘 / 鼠标，基于 enigo）
│   ├── mod.rs      # InputEngine trait
│   ├── keyboard.rs # 键盘输入（type_text / paste_from_clipboard）
│   └── mouse.rs    # 鼠标操作
├── hotkey/         # 全局热键管理
│   ├── mod.rs      # HotkeyManager trait
│   ├── keys.rs     # 快捷键字符串解析
│   └── linux.rs    # X11 实现
├── daemon.rs       # 后台守护进程
└── cli/            # CLI 命令定义
```

## License

MIT
