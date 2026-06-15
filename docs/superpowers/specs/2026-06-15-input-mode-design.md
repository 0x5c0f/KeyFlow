# Input Mode Design

## Problem Statement

KeyFlow's core purpose is to bypass paste-disabled input fields (e.g., VNC password boxes) by simulating keyboard input character-by-character. However, the current implementation has three issues:

1. **Format loss** — character-by-character input cannot preserve rich text formatting (bold, colors, fonts)
2. **CJK freeze** — Chinese/Japanese/Korean characters require X11 keycode remapping per character, causing UI freeze
3. **Mouse movement corruption** — focus changes during character-by-character input corrupt the output

The fundamental contradiction: paste-disabled fields require character typing, but normal editors should use clipboard paste (Ctrl+V) for format preservation and performance.

## Design Decisions

### Why not auto-detect?

X11 provides no reliable way to:
- Detect if the current input field accepts paste
- Detect if a Ctrl+V paste was consumed by the application
- Distinguish between "editor" and "password field" programmatically

Therefore, automatic detection is not feasible. The user must configure the input mode per binding.

### Why three modes?

| Mode | Behavior | Use Case |
|------|----------|----------|
| `auto` | Default, equivalent to `type` | Safe default when not configured |
| `type` | Character-by-character XTEST input | Paste-disabled fields (VNC, password boxes) |
| `paste` | Ctrl+V clipboard paste | Normal editors (format preserved) |

`auto` exists as a semantic default — it means "user didn't configure, use safe behavior." It is functionally identical to `type` but leaves room for future heuristics.

### Why remove batching?

The current implementation batches text into chunks of 10 characters with 20ms delays between batches. This was intended to prevent X11 event queue saturation but actually increases latency. enigo's internal 1ms keycode-switch delay is sufficient.

## Architecture

### InputMode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    Auto,
    Type,
    Paste,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Auto
    }
}
```

### Binding Changes

```rust
pub struct Binding {
    pub name: String,
    pub hotkey: String,
    pub provider: String,
    pub item_id: Option<String>,
    #[serde(default)]
    pub input_mode: InputMode,
}
```

### InputEngine Trait

Unchanged — the trait already has both `type_text` and `paste_from_clipboard` methods:

```rust
pub trait InputEngine: Send + Sync {
    fn get_mouse_position(&self) -> Result<(i32, i32), InputError>;
    fn click_at(&self, x: i32, y: i32) -> Result<(), InputError>;
    fn type_text(&self, text: &str) -> Result<(), InputError>;
    fn paste_from_clipboard(&self) -> Result<(), InputError>;
}
```

### Keyboard Module Changes

`type_text()` simplified — remove BATCH_SIZE/BATCH_DELAY_MS, call `enigo.text(text)` directly:

```rust
pub fn type_text(text: &str) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    enigo.text(text)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    Ok(())
}
```

### Daemon Dispatch

Step 5 in the daemon callback changes from always typing to mode-based dispatch:

```rust
match binding.input_mode {
    InputMode::Type | InputMode::Auto => {
        input.type_text(&text)?;
    }
    InputMode::Paste => {
        // Write to clipboard, then simulate Ctrl+V
        let mut cb = arboard::Clipboard::new()?;
        cb.set_text(&text)?;
        input.paste_from_clipboard()?;
    }
}
```

## Configuration Example

```toml
[settings]
clipboard_clear_after_secs = 5

# Paste to normal editor (format preserved)
[[bindings]]
name = "粘贴到编辑器"
hotkey = "F7"
provider = "clipboard"
input_mode = "paste"

# Type into paste-disabled field
[[bindings]]
name = "输入到密码框"
hotkey = "F8"
provider = "clipboard"
input_mode = "type"

# Bitwarden password (always type)
[[bindings]]
name = "Bitwarden 密码"
hotkey = "F9"
provider = "bitwarden"
item_id = "xxx"
# input_mode defaults to auto (equivalent to type)
```

## Error Handling

- `type_text` errors: `InputError::KeystrokeFailed`
- `paste_from_clipboard` errors: `InputError::KeystrokeFailed`
- Clipboard write errors: `InputError::KeystrokeFailed` (wrapped from arboard)

All errors are logged and the callback returns early (no retry).

## Testing

1. `cargo build` — compilation check
2. `input_mode = "type"` — test with ASCII and CJK text
3. `input_mode = "paste"` — test format preservation in editor
4. No `input_mode` configured — verify default `auto` behavior
5. `cargo test` — existing tests pass
