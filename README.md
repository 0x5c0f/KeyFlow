# KeyFlow

非粘贴型密码框辅助输入工具 — 通过模拟键盘输入绕过禁止粘贴的密码框。

## 适用场景

- VNC 远程登录时输入长复杂密码
- SSH 终端中输入密码
- 任何禁用粘贴功能的密码输入框
- 配合 Bitwarden 等密码管理器使用

## 工作原理

1. 用户将鼠标悬停在目标密码框上
2. 按下全局热键（如 F7）
3. KeyFlow 自动：获取鼠标位置 → 点击聚焦 → 从密码管理器获取密码 → 模拟键盘逐字输入
4. 密码不经过剪贴板，不落盘，安全输入

## 系统要求

| 平台 | 依赖 |
|------|------|
| Linux (X11) | `libxdo-dev` |
| Linux (Wayland) | 待支持 |
| macOS | 待支持 |
| Windows | 待支持 |

### 安装系统依赖

**Debian / Ubuntu:**
```bash
sudo apt-get install -y libxdo-dev
```

**Fedora:**
```bash
sudo dnf install -y libXtst-devel
```

**Arch Linux:**
```bash
sudo pacman -S xdotool
```

## 安装

### 从源码构建

```bash
git clone https://github.com/your-user/keyflow.git
cd keyflow
make build
sudo make install  # 安装到 /usr/local/bin/
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

# 设置主密码环境变量（写入 ~/.bashrc 或 ~/.zshrc）
export BW_PASSWORD="your-master-password"
```

### 3. 添加热键绑定

```bash
# 剪贴板模式 — 从剪贴板读取密码
keyflow bind add --name "my-server" --hotkey "F7" --provider clipboard

# Bitwarden 模式 — 直接从 Bitwarden 获取密码
keyflow bind add --name "vnc-server" --hotkey "F8" --provider bitwarden --item-id "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

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

> **注意：** 当前版本的热键管理器是 stub 实现，组合键的实际匹配尚未接入 X11 事件循环。

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

1. 将鼠标悬停在目标密码框上
2. 按 F7（或你配置的热键）
3. 密码自动输入

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
└── unlock           # 解锁 Bitwarden
```

## 配置文件

配置文件路径：`~/.config/keyflow/keyflow.toml`

```toml
[settings]
clipboard_clear_after_secs = 5

[[providers]]
type = "clipboard"

[[providers]]
type = "bitwarden"

[[bindings]]
name = "my-server"
hotkey = "F7"                                # 单键
provider = "bitwarden"
item_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

[[bindings]]
name = "from-clipboard"
hotkey = "Ctrl+Shift+F9"                     # 组合键
provider = "clipboard"
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
├── config/         # 配置管理（TOML 解析）
├── provider/       # 密码提供者（Clipboard / Bitwarden）
├── input/          # 输入模拟（键盘 / 鼠标，基于 enigo）
├── hotkey/         # 全局热键管理
├── daemon/         # 后台守护进程
└── cli/            # CLI 命令定义
```

## License

MIT
