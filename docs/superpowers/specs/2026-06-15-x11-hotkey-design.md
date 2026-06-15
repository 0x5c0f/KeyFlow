# X11 全局热键实现设计

**日期:** 2026-06-15
**状态:** 已批准
**范围:** 替换 LinuxHotkeyManager stub，实现真实的 X11 全局热键注册与事件监听

---

## 1. 背景

当前 `LinuxHotkeyManager` 是一个纯 stub——`run()` 只是 sleep 循环，`register()` 仅存储回调但不执行 X11 注册。按热键无任何反应。`x11rb` 依赖已添加到 Cargo.toml 但未使用。

## 2. 需求

| 需求 | 决策 |
|------|------|
| 热键冲突处理 | 报错并退出 daemon |
| Num Lock / Caps Lock 兼容 | 自动兼容（对每个锁定键组合都注册） |
| 组合键范围 | 完整组合键（字母/数字/功能键 + 修饰键） |
| 事件循环方式 | poll_for_event + sleep(10ms) |
| 延迟要求 | < 10ms（poll + sleep(10ms)，用户无法感知） |

## 3. 架构

### 3.1 热键字符串解析器 (`src/hotkey/keys.rs`)

**职责：** 将 `"Ctrl+Shift+F7"` 解析为 `(keysym: u32, modifiers: u16)`

**解析规则：**
- 按 `+` 分割输入字符串
- 识别修饰键：`Ctrl`/`Control` → ControlMask, `Shift` → ShiftMask, `Alt` → Mod1Mask, `Super`/`Win`/`Meta` → Mod4Mask
- 识别普通键：功能键 F1-F24、字母 A-Z、数字 0-9、特殊键（Space/Tab/Esc/Enter 等）、方向键
- 剩余部分作为 keysym 名称查询

**错误情况：**
- 空字符串 → HotkeyRegistration 错误
- 未知键名 → HotkeyRegistration 错误
- 无普通键（只有修饰键）→ HotkeyRegistration 错误

### 3.2 X11 连接与按键码映射 (`src/hotkey/linux.rs`)

**启动流程：**
1. `x11rb::connect(None)` 连接 X Server
2. `get_keyboard_mapping()` 获取当前键盘布局
3. 构建 `HashMap<u32, Vec<u8>>` 映射表（keysym → keycodes）

**为什么需要映射表：**
- `XGrabKey` 需要 keycode（物理键位置），不是 keysym（逻辑键含义）
- 一个 keysym 可能映射到多个 keycode（主键盘区和数字小键盘）

**Num Lock / Caps Lock 兼容：**
- 通过 `GetModifierMapping` 从 X Server 获取 NumLock 和 CapsLock 的实际 modifier 掩码（不是固定值）
- 注册时对每个锁定键组合（0, NumLock, CapsLock, NumLock|CapsLock）都调用一次 `grab_key`
- 匹配事件时从 state 中去掉锁定键位再查表

### 3.3 热键注册 (`register` 方法)

1. 调用 `parse_hotkey()` 解析字符串 → (keysym, modifiers)
2. 从 keymap 查 keycode：`keymap.get(keysym)` → Vec<keycode>
3. 对每个 keycode，对每个 lock_mask 组合调用 `grab_key(keycode, modifiers | lock_mask)`
4. 存储回调：`callbacks.insert((keycode, stripped_modifiers), callback)`
5. 记录已注册的 (keycode, modifiers) 用于 Drop 清理

**冲突处理：**
- `grab_key()` 返回错误时，返回 `KeyflowError::HotkeyRegistration`
- daemon 收到错误后退出

### 3.4 事件循环 (`run` 方法)

```
loop:
  poll_for_event()           // 非阻塞检查
  ├─ KeyPress 事件?
  │   提取 keycode + state
  │   state &= !(NumLock | CapsLock)   // 去掉锁定键位
  │   callbacks.get((keycode, state)) → 调用回调
  ├─ 其他事件 → 忽略
  └─ 无事件 → sleep(10ms)
  检查 running 标志 → false 则退出
```

**选择 poll + sleep 而非 wait_for_event 的原因：**
- `wait_for_event()` 阻塞，无法响应 stop 信号
- `sleep(10ms)` 延迟用户无法感知

### 3.5 Drop 清理

- 对所有已注册的 (keycode, modifiers) 执行 `ungrab_key`
- `connection.flush()` 确保请求发送到 X Server

## 4. 数据结构

```rust
struct LinuxHotkeyManager {
    connection: RustConnection,
    root_window: u32,
    keymap: HashMap<u32, Vec<u8>>,              // keysym → keycodes
    callbacks: HashMap<(u8, u16), HotkeyCallback>, // (keycode, mod) → callback
    registered_keys: Vec<(u8, u16)>,            // 用于 Drop 清理
    running: Arc<AtomicBool>,
}
```

## 5. 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `Cargo.toml` | 已改 | 添加 `x11rb = "0.13"` |
| `src/hotkey/keys.rs` | 新建 | 热键字符串解析器 |
| `src/hotkey/linux.rs` | 重写 | 真实 X11 实现 |
| `src/hotkey/mod.rs` | 小改 | 添加 `pub mod keys;` |
| `tests/hotkey_tests.rs` | 新建 | 解析器单元测试 |

## 6. 错误处理

| 场景 | 处理 |
|------|------|
| X11 连接失败 | `KeyflowError::Io` |
| 热键注册冲突 | `KeyflowError::HotkeyRegistration` |
| 解析失败 | `KeyflowError::HotkeyRegistration` |
| 事件循环异常 | 日志警告，继续运行 |

## 7. 测试

**单元测试 (`tests/hotkey_tests.rs`)：**
- `"F7"` → keysym=F7, modifiers=0
- `"Ctrl+F7"` → keysym=F7, modifiers=ControlMask
- `"Ctrl+Shift+F9"` → keysym=F9, modifiers=ControlMask|ShiftMask
- `"Super+A"` → keysym=A, modifiers=Mod4Mask
- `""` → 错误
- `"Ctrl"` → 错误（无普通键）
- `"Unknown+F7"` → 错误

**集成验证：**
- `cargo build` 编译通过
- `cargo test` 所有测试通过
- `cargo run -- run` daemon 启动，日志显示热键注册成功
- 按下注册的热键 → 回调触发，密码被输入

## 8. YAGNI（不做之事）

- ❌ Wayland 支持（后续单独实现）
- ❌ 热键动态添加/删除
- ❌ 热键录制功能
- ❌ 多 X11 screen 支持（仅用默认 screen）
