# KeyFlow

English | [简体中文](README.md)

A non-paste password input assistant — bypass paste-disabled password fields by simulating keyboard input, with support for normal paste mode in editors.

## Use Cases

- Any input field that disables paste functionality
- Integration with password managers like Bitwarden
- Formatted paste in normal editors

## How It Works

1. Hover the mouse over the target input field
2. Press a global hotkey (e.g., F7)
3. KeyFlow automatically: captures mouse position → clicks to focus → retrieves password → inputs text
4. Based on `input_mode` configuration:
   - `type` (default): simulates keyboard input character by character, bypasses paste-disabled fields
   - `paste`: clipboard + Ctrl+V paste, preserves formatting

## System Requirements

| Platform | Status | Dependency |
|----------|--------|------------|
| Linux (X11) | ✅ Supported | `libxdo-dev` |
| Linux (Wayland) | ❌ Planned | — |
| macOS | ❌ Planned | — |
| Windows | ❌ Planned | — |

### Install System Dependencies

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

## Installation

### Build from Source

```bash
git clone https://github.com/your-user/keyflow.git
cd keyflow
make build
make install  # Install to ~/.local/bin/ (no sudo required)
```

### Install from Tarball

```bash
# Download and extract
tar -xzf keyflow-*-x86_64-linux.tar.gz
cd keyflow-*-x86_64-linux

# One-click install (binary + config + systemd service)
make install

# Uninstall
make uninstall

# Upgrade (stop → install → start)
make upgrade
```

### systemd Service (Recommended)

After installation, the systemd user service is automatically enabled for auto-start and process supervision:

```bash
# Start service
systemctl --user start keyflow

# Check status
systemctl --user status keyflow

# View logs
journalctl --user -u keyflow -f
```

### Development Mode

```bash
make build
# Binary at target/debug/keyflow
```

## Quick Start

### 1. View Help

```bash
keyflow --help
```

### 2. Configure Bitwarden (Optional)

```bash
# Install Bitwarden CLI
npm install -g @bitwarden/cli

# First login
bw login

# Unlock and save session (interactive password input)
keyflow unlock
```

### 3. Add Hotkey Bindings

```bash
# Clipboard + character input (bypass paste-disabled fields)
keyflow bind add --name "my-server" --hotkey "F7" --provider clipboard

# Bitwarden + character input
keyflow bind add --name "vnc-server" --hotkey "F8" --provider bitwarden --item-id "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

# Combination keys
keyflow bind add --name "secure" --hotkey "Ctrl+Shift+F7" --provider clipboard
```

### Hotkey Format

Single keys and combinations supported, modifiers connected with `+`:

| Format | Example | Description |
|--------|---------|-------------|
| Single key | `F7` | Function keys |
| Modifier+key | `Ctrl+F7` | Single modifier |
| Multi-modifier+key | `Ctrl+Shift+F7` | Multiple modifiers |
| Modifier+letter | `Ctrl+P` | Modifier + regular key |

**Supported modifiers:** `Ctrl`, `Shift`, `Alt`, `Super`

**Supported keys:** `F1`-`F24`, `A`-`Z`, `0`-`9`, `Space`, `Tab`, `Esc`, `Enter`, `Backspace`, `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, arrow keys, punctuation

View bindings:
```bash
keyflow bind list
```

### 4. Start Daemon

```bash
# Foreground (for debugging)
keyflow run

# Background
keyflow run --daemon
```

### 5. Usage

1. Hover mouse over target input field
2. Press F7 (or your configured hotkey)
3. Password is input automatically

## Input Modes

Each binding can configure `input_mode` to control how text is delivered:

| Mode | Behavior | Use Case |
|------|----------|----------|
| `auto` | Default, equivalent to `type` | Safe default when not configured |
| `type` | Character-by-character keyboard input | Paste-disabled fields (VNC, password boxes) |
| `paste` | Ctrl+V clipboard paste | Normal editors (preserves formatting) |

**Configuration Example:**

```toml
# Character input (bypass paste-disabled fields)
[[bindings]]
name = "VNC Password"
hotkey = "F7"
provider = "clipboard"
input_mode = "type"

# Paste mode (preserves formatting)
[[bindings]]
name = "Editor Paste"
hotkey = "F8"
provider = "clipboard"
input_mode = "paste"
```

## Clipboard Clearing

Each binding can independently configure clipboard clearing time:

```toml
# Global default: clear after 5 seconds
[settings]
clipboard_clear_after_secs = 5

# This binding: clear after 3 seconds
[[bindings]]
name = "Quick Clear"
hotkey = "F7"
provider = "clipboard"
clipboard_clear_after_secs = 3

# This binding: don't clear
[[bindings]]
name = "Keep Clipboard"
hotkey = "F8"
provider = "clipboard"
clipboard_clear_after_secs = 0
```

**Priority:** binding-level > global setting

**Safety:** Before clearing, KeyFlow checks if the clipboard still contains the input text. If the user copied new content during the wait period, it won't be deleted.

## CLI Commands

```
keyflow
├── run              # Start daemon (listen for hotkeys)
├── stop             # Stop daemon
├── status           # Show daemon and Bitwarden status
├── bind
│   ├── add          # Add hotkey binding
│   ├── remove       # Remove binding
│   └── list         # List all bindings
├── config
│   ├── show         # Show current configuration
│   └── path         # Show config file path
└── unlock           # Unlock Bitwarden vault
```

## Configuration File

Config path: `~/.config/keyflow/keyflow.toml`

Full configuration example: [`keyflow.toml.example`](keyflow.toml.example)

```toml
[settings]
clipboard_clear_after_secs = 5

[[bindings]]
name = "VNC Password"
hotkey = "F7"
provider = "bitwarden"
item_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
input_mode = "type"
cache_secs = 300

[[bindings]]
name = "Editor Paste"
hotkey = "F8"
provider = "clipboard"
input_mode = "paste"
clipboard_clear_after_secs = 0
```

## Development

```bash
# Build
make build

# Run tests
make test

# Code check
make check

# Clean
make clean

# Show all available commands
make help
```

## Architecture

```
src/
├── lib.rs          # Library entry
├── main.rs         # CLI entry
├── error.rs        # Unified error types
├── config/         # Configuration management (TOML parsing)
│   ├── mod.rs      # Config, Settings
│   └── binding.rs  # Binding, InputMode
├── provider/       # Password providers
│   ├── mod.rs      # PasswordProvider trait
│   ├── clipboard.rs# Clipboard provider
│   ├── bitwarden.rs# Bitwarden CLI provider
│   └── cached.rs   # Password cache wrapper
├── input/          # Input simulation (keyboard / mouse, based on enigo)
│   ├── mod.rs      # InputEngine trait
│   ├── keyboard.rs # Keyboard input (type_text / paste_from_clipboard)
│   └── mouse.rs    # Mouse operations
├── hotkey/         # Global hotkey management
│   ├── mod.rs      # HotkeyManager trait
│   ├── keys.rs     # Hotkey string parser
│   └── linux.rs    # X11 implementation
├── daemon.rs       # Background daemon
└── cli/            # CLI command definitions
```

## License

MIT
