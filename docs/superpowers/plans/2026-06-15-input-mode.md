# Input Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `input_mode` configuration to KeyFlow bindings, supporting `type` (character-by-character), `paste` (Ctrl+V), and `auto` (default, equivalent to `type`) modes.

**Architecture:** Add `InputMode` enum to the config binding module. Simplify keyboard input by removing artificial batching. Update daemon to dispatch input based on the configured mode.

**Tech Stack:** Rust, enigo 0.2, arboard 3, serde

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/config/binding.rs` | Modify | Add `InputMode` enum and `input_mode` field to `Binding` |
| `src/input/keyboard.rs` | Modify | Remove batching constants, simplify `type_text()` |
| `src/daemon.rs` | Modify | Add mode-based input dispatch in Step 5 |

---

### Task 1: Add InputMode Enum to binding.rs

**Files:**
- Modify: `src/config/binding.rs`

- [ ] **Step 1: Add InputMode enum above Binding struct**

```rust
//! Hotkey binding definition.

use serde::{Deserialize, Serialize};

/// Input mode for a binding — controls how text is delivered to the target field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    /// Default — currently equivalent to `type`. Reserved for future heuristics.
    Auto,
    /// Character-by-character XTEST input. Works in paste-disabled fields.
    Type,
    /// Ctrl+V clipboard paste. Preserves formatting, fast for CJK.
    Paste,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// A binding maps a hotkey to a password provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// Human-readable name for this binding.
    pub name: String,
    /// Hotkey to trigger this binding (e.g., "F7", "F8").
    pub hotkey: String,
    /// Provider type to use ("clipboard" or "bitwarden").
    pub provider: String,
    /// Item ID for provider-specific lookup (e.g., Bitwarden item ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    /// Input mode — how text is delivered to the target field.
    #[serde(default)]
    pub input_mode: InputMode,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1`
Expected: `Finished` (no errors — new field has `#[serde(default)]` so existing configs still parse)

- [ ] **Step 3: Commit**

```bash
git add src/config/binding.rs
git commit -m "feat(config): add InputMode enum to binding"
```

---

### Task 2: Simplify type_text in keyboard.rs

**Files:**
- Modify: `src/input/keyboard.rs`

- [ ] **Step 1: Replace type_text with simplified version**

Replace the entire file content with:

```rust
//! Keyboard simulation via enigo.

use crate::error::InputError;
use enigo::{Enigo, Keyboard, Settings};

/// Type text by simulating keystrokes, character by character.
///
/// Uses enigo's built-in pacing (1ms between different keycodes, configurable
/// delay for same-keycode repeats). No manual batching needed.
pub fn type_text(text: &str) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    enigo
        .text(text)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    Ok(())
}

/// Paste text from clipboard by simulating Ctrl+V.
/// This is faster and preserves formatting, but requires the text
/// to already be in the clipboard.
pub fn paste_from_clipboard() -> Result<(), InputError> {
    use enigo::{Direction, Key};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    // Simulate Ctrl+V
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1`
Expected: `Finished` (no errors)

- [ ] **Step 3: Commit**

```bash
git add src/input/keyboard.rs
git commit -m "refactor(input): simplify type_text by removing manual batching"
```

---

### Task 3: Add Mode-Based Dispatch in daemon.rs

**Files:**
- Modify: `src/daemon.rs`

- [ ] **Step 1: Add InputMode import**

At line 6, change:

```rust
use crate::config::{Config, ProviderConfig};
```

to:

```rust
use crate::config::{Config, ProviderConfig};
use crate::config::binding::InputMode;
```

- [ ] **Step 2: Clone input_mode before closure**

After line 50 (`let clear_secs = config.settings.clipboard_clear_after_secs;`), add:

```rust
let input_mode = binding.input_mode;
```

- [ ] **Step 3: Replace Step 5 with mode-based dispatch**

Replace lines 102-107:

```rust
            // 5. Type the password character by character
            log::debug!("[{binding_name}] Step 5: Typing password ({} chars)...", password.len());
            if let Err(e) = input.type_text(&password) {
                log::error!("[{binding_name}] Failed to type password: {e}");
                return;
            }
```

with:

```rust
            // 5. Input text using the configured mode
            log::debug!("[{binding_name}] Step 5: Input mode={input_mode:?}, text_len={}", password.len());
            let input_result = match input_mode {
                InputMode::Type | InputMode::Auto => {
                    log::debug!("[{binding_name}] Typing character by character...");
                    input.type_text(&password)
                }
                InputMode::Paste => {
                    log::debug!("[{binding_name}] Pasting via clipboard + Ctrl+V...");
                    // Write to clipboard, then simulate Ctrl+V
                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&password)) {
                        Ok(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            input.paste_from_clipboard()
                        }
                        Err(e) => Err(crate::error::InputError::KeystrokeFailed(e.to_string())),
                    }
                }
            };
            if let Err(e) = input_result {
                log::error!("[{binding_name}] Failed to input text: {e}");
                return;
            }
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build 2>&1`
Expected: `Finished` (no errors)

- [ ] **Step 5: Commit**

```bash
git add src/daemon.rs
git commit -m "feat(daemon): add mode-based input dispatch (type/paste/auto)"
```

---

### Task 4: Build and Integration Test

- [ ] **Step 1: Run all tests**

Run: `cargo test 2>&1`
Expected: all tests pass

- [ ] **Step 2: Build release binary**

Run: `cargo build --release 2>&1`
Expected: `Finished` (no errors)

- [ ] **Step 3: Install binary**

Run: `cargo install --path . --force 2>&1 && cp ~/.cargo/bin/keyflow ~/.local/bin/keyflow`
Expected: `Installed package`

- [ ] **Step 4: Verify help output**

Run: `keyflow --help 2>&1`
Expected: help text displayed without errors
