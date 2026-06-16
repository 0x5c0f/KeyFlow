//! Linux global hotkey implementation.
//!
//! Uses X11 XGrabKey for global hotkey registration.
//! Wayland support will be added later via libei.

use crate::error::KeyflowError;
use crate::hotkey::keys::{self, Key};
use crate::hotkey::{HotkeyCallback, HotkeyManager};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

/// Convert a platform-agnostic Key to its X11 keysym value.
fn key_to_x11_keysym(key: Key) -> u32 {
    match key {
        // Letters: XK_a = 0x61, XK_A = 0x41 (we use lowercase)
        Key::A => 0x61, Key::B => 0x62, Key::C => 0x63, Key::D => 0x64,
        Key::E => 0x65, Key::F => 0x66, Key::G => 0x67, Key::H => 0x68,
        Key::I => 0x69, Key::J => 0x6A, Key::K => 0x6B, Key::L => 0x6C,
        Key::M => 0x6D, Key::N => 0x6E, Key::O => 0x6F, Key::P => 0x70,
        Key::Q => 0x71, Key::R => 0x72, Key::S => 0x73, Key::T => 0x74,
        Key::U => 0x75, Key::V => 0x76, Key::W => 0x77, Key::X => 0x78,
        Key::Y => 0x79, Key::Z => 0x7A,

        // Digits: XK_0 = 0x30
        Key::Digit0 => 0x30, Key::Digit1 => 0x31, Key::Digit2 => 0x32,
        Key::Digit3 => 0x33, Key::Digit4 => 0x34, Key::Digit5 => 0x35,
        Key::Digit6 => 0x36, Key::Digit7 => 0x37, Key::Digit8 => 0x38,
        Key::Digit9 => 0x39,

        // Function keys: XK_F1 = 0xFFBE
        Key::F1 => 0xFFBE, Key::F2 => 0xFFBF, Key::F3 => 0xFFC0, Key::F4 => 0xFFC1,
        Key::F5 => 0xFFC2, Key::F6 => 0xFFC3, Key::F7 => 0xFFC4, Key::F8 => 0xFFC5,
        Key::F9 => 0xFFC6, Key::F10 => 0xFFC7, Key::F11 => 0xFFC8, Key::F12 => 0xFFC9,
        Key::F13 => 0xFFCA, Key::F14 => 0xFFCB, Key::F15 => 0xFFCC, Key::F16 => 0xFFCD,
        Key::F17 => 0xFFCE, Key::F18 => 0xFFCF, Key::F19 => 0xFFD0, Key::F20 => 0xFFD1,
        Key::F21 => 0xFFD2, Key::F22 => 0xFFD3, Key::F23 => 0xFFD4, Key::F24 => 0xFFD5,

        // Navigation
        Key::Home => 0xFF50, Key::End => 0xFF57,
        Key::PageUp => 0xFF55, Key::PageDown => 0xFF56,
        Key::Up => 0xFF52, Key::Down => 0xFF54,
        Key::Left => 0xFF51, Key::Right => 0xFF53,
        Key::Insert => 0xFF63, Key::Delete => 0xFFFF,
        Key::Tab => 0xFF09, Key::Enter => 0xFF0D,
        Key::Escape => 0xFF1B, Key::Backspace => 0xFF08,
        Key::Space => 0x0020,

        // Punctuation
        Key::Minus => 0x002D, Key::Equal => 0x003D,
        Key::BracketLeft => 0x005B, Key::BracketRight => 0x005D,
        Key::Backslash => 0x005C, Key::Semicolon => 0x003B,
        Key::Apostrophe => 0x0027, Key::Grave => 0x0060,
        Key::Comma => 0x002C, Key::Period => 0x002E, Key::Slash => 0x002F,
    }
}

/// Convert platform-agnostic modifier flags to X11 modifier mask.
fn modifiers_to_x11(modifiers: u16) -> u16 {
    let mut mask = 0u16;
    if modifiers & keys::modifiers::SHIFT != 0 { mask |= 1 << 0; }   // ShiftMask
    if modifiers & keys::modifiers::CONTROL != 0 { mask |= 1 << 2; } // ControlMask
    if modifiers & keys::modifiers::ALT != 0 { mask |= 1 << 3; }     // Mod1Mask (Alt)
    if modifiers & keys::modifiers::SUPER != 0 { mask |= 1 << 6; }   // Mod4Mask (Super)
    mask
}

/// Shared callback wrapper (allows one callback to be registered for multiple keycodes).
type SharedCallback = Arc<HotkeyCallback>;

/// Modifier masks for Num Lock and Caps Lock.
/// These are detected at startup via GetModifierMapping.
#[derive(Clone, Copy)]
struct LockModifiers {
    num_lock: u16,
    caps_lock: u16,
    scroll_lock: u16,
}

pub struct LinuxHotkeyManager {
    connection: RustConnection,
    root_window: Window,
    keymap: HashMap<u32, Vec<u8>>,               // keysym -> keycodes
    callbacks: HashMap<(u8, u16), SharedCallback>, // (keycode, base_modifiers) -> callback
    registered_keys: Vec<(u8, u16)>,              // (keycode, full_modifiers) for cleanup
    lock_mods: LockModifiers,
    running: Arc<AtomicBool>,
}

impl LinuxHotkeyManager {
    pub fn new() -> Result<Self, KeyflowError> {
        let (connection, screen_num) = RustConnection::connect(None).map_err(|e| {
            KeyflowError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Failed to connect to X11 display: {e}. Is X running? (Note: Wayland is not yet supported)"),
            ))
        })?;
        let screen = &connection.setup().roots[screen_num];
        let root_window = screen.root;

        let keymap = build_keymap(&connection)?;
        let lock_mods = detect_lock_modifiers(&connection)?;

        Ok(Self {
            connection,
            root_window,
            keymap,
            callbacks: HashMap::new(),
            registered_keys: Vec::new(),
            lock_mods,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get all combinations of lock modifier masks (for Num Lock / Caps Lock compatibility).
    fn all_lock_masks(&self) -> Vec<u16> {
        let lm = &self.lock_mods;
        let mut masks = vec![0u16];
        if lm.num_lock != 0 {
            let mut new = masks.clone();
            new.iter_mut().for_each(|m| *m |= lm.num_lock);
            masks.extend(new);
        }
        if lm.caps_lock != 0 {
            let mut new = masks.clone();
            new.iter_mut().for_each(|m| *m |= lm.caps_lock);
            masks.extend(new);
        }
        if lm.scroll_lock != 0 {
            let mut new = masks.clone();
            new.iter_mut().for_each(|m| *m |= lm.scroll_lock);
            masks.extend(new);
        }
        masks
    }

    fn handle_event(&self, event: Event) {
        if let Event::KeyPress(ev) = event {
            let keycode = ev.detail;
            let state: u16 = ev.state.into();
            let state = state
                & !(self.lock_mods.num_lock | self.lock_mods.caps_lock | self.lock_mods.scroll_lock);
            if let Some(callback) = self.callbacks.get(&(keycode, state)) {
                log::debug!("Hotkey triggered: keycode={keycode}, state=0x{state:04X}");
                callback();
            }
        }
    }
}

impl HotkeyManager for LinuxHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        let combo = keys::parse_hotkey(hotkey)?;
        let keysym = key_to_x11_keysym(combo.key);
        let x11_mods = modifiers_to_x11(combo.modifiers);

        let keycodes = self.keymap.get(&keysym).ok_or_else(|| {
            KeyflowError::HotkeyRegistration {
                hotkey: hotkey.to_string(),
                reason: format!("keysym 0x{keysym:04X} not found on keyboard"),
            }
        })?;

        let lock_masks = self.all_lock_masks();
        for &keycode in keycodes {
            for &lock_mask in &lock_masks {
                let full_modifiers = x11_mods | lock_mask;
                match self.connection.grab_key(
                    true,
                    self.root_window,
                    full_modifiers.into(),
                    keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                ) {
                    Ok(_) => {
                        self.registered_keys.push((keycode, full_modifiers));
                    }
                    Err(e) => {
                        let reason = format_x11_error("grab_key", &e);
                        return Err(KeyflowError::HotkeyRegistration {
                            hotkey: hotkey.to_string(),
                            reason,
                        });
                    }
                }
            }
        }

        let shared = Arc::new(callback);
        for &keycode in keycodes {
            self.callbacks.insert((keycode, x11_mods), shared.clone());
        }

        self.connection.flush().map_err(|e| {
            KeyflowError::HotkeyRegistration {
                hotkey: hotkey.to_string(),
                reason: format!("flush failed: {e}"),
            }
        })?;

        log::info!("Registered hotkey: {hotkey} (keysym=0x{keysym:04X}, mods=0x{x11_mods:04X})");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::Release);
        log::info!("Hotkey manager started (Linux X11)");

        while self.running.load(Ordering::Acquire) {
            match self.connection.poll_for_event() {
                Ok(Some(event)) => {
                    self.handle_event(event);
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => {
                    log::warn!("X11 event error: {e:?}");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        log::info!("Hotkey manager stopped");
        Ok(())
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}

impl Drop for LinuxHotkeyManager {
    fn drop(&mut self) {
        for &(keycode, modifiers) in &self.registered_keys {
            let _ = self.connection.ungrab_key(keycode, self.root_window, modifiers.into());
        }
        let _ = self.connection.flush();
        log::debug!("X11 hotkeys ungrabbed");
    }
}

/// Format X11 errors into human-readable messages.
fn format_x11_error(operation: &str, error: &impl std::fmt::Debug) -> String {
    let msg = format!("{error:?}");
    if msg.contains("Access") {
        format!("{operation} failed: hotkey already grabbed by another application")
    } else if msg.contains("Value") {
        format!("{operation} failed: invalid keycode or modifier")
    } else if msg.contains("Match") {
        format!("{operation} failed: incompatible configuration")
    } else {
        format!("{operation} failed: {msg}")
    }
}

/// Build a mapping from keysym to keycode(s) using GetKeyboardMapping.
fn build_keymap(connection: &RustConnection) -> Result<HashMap<u32, Vec<u8>>, KeyflowError> {
    let setup = connection.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    let count = max_keycode - min_keycode + 1;

    let reply = connection
        .get_keyboard_mapping(min_keycode, count)
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_keyboard_mapping failed: {e}"))))?
        .reply()
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_keyboard_mapping reply failed: {e}"))))?;

    let keysyms_per_keycode = reply.keysyms_per_keycode as usize;
    let mut keymap: HashMap<u32, Vec<u8>> = HashMap::new();

    for (i, keycode) in (min_keycode..=max_keycode).enumerate() {
        let base = i * keysyms_per_keycode;
        for j in 0..keysyms_per_keycode {
            let keysym = reply.keysyms[base + j];
            if keysym != 0 {
                keymap.entry(keysym).or_default().push(keycode);
            }
        }
    }

    Ok(keymap)
}

/// Detect modifier masks for Num Lock, Caps Lock, Scroll Lock.
fn detect_lock_modifiers(connection: &RustConnection) -> Result<LockModifiers, KeyflowError> {
    let reply = connection
        .get_modifier_mapping()
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_modifier_mapping failed: {e}"))))?
        .reply()
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_modifier_mapping reply failed: {e}"))))?;
    let modmap = &reply.keycodes;
    let per_mod = reply.keycodes_per_modifier() as usize;

    let mut num_lock = 0u16;
    let mut caps_lock = 0u16;
    let mut scroll_lock = 0u16;

    let setup = connection.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    let count = max_keycode - min_keycode + 1;
    let km_reply = connection
        .get_keyboard_mapping(min_keycode, count)
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_keyboard_mapping failed: {e}"))))?
        .reply()
        .map_err(|e| KeyflowError::Io(std::io::Error::other(format!("get_keyboard_mapping reply failed: {e}"))))?;
    let kspkc = km_reply.keysyms_per_keycode as usize;

    let has_keysym = |keycode: u8, target: u32| -> bool {
        let idx = (keycode - min_keycode) as usize * kspkc;
        for j in 0..kspkc {
            if km_reply.keysyms[idx + j] == target {
                return true;
            }
        }
        false
    };

    // XK_Num_Lock = 0xFF7F, XK_Caps_Lock = 0xFFE5, XK_Scroll_Lock = 0xFF14
    for mod_idx in 0..8 {
        let mask: u16 = 1 << mod_idx;
        for k in 0..per_mod {
            let keycode = modmap[mod_idx * per_mod + k];
            if keycode == 0 {
                continue;
            }
            if has_keysym(keycode, 0xFF7F) {
                num_lock = mask;
            }
            if has_keysym(keycode, 0xFFE5) {
                caps_lock = mask;
            }
            if has_keysym(keycode, 0xFF14) {
                scroll_lock = mask;
            }
        }
    }

    Ok(LockModifiers {
        num_lock,
        caps_lock,
        scroll_lock,
    })
}
