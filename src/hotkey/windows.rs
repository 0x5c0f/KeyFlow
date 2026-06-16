//! Windows global hotkey implementation.
//!
//! Uses Win32 RegisterHotKey for global hotkey registration.
//! Requires a message-only window to receive WM_HOTKEY messages.

use crate::error::KeyflowError;
use crate::hotkey::keys::{self, Key};
use crate::hotkey::{HotkeyCallback, HotkeyManager};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Convert a platform-agnostic Key to its Windows Virtual Key code.
fn key_to_vk(key: Key) -> u32 {
    match key {
        // Letters: VK_A = 0x41
        Key::A => 0x41, Key::B => 0x42, Key::C => 0x43, Key::D => 0x44,
        Key::E => 0x45, Key::F => 0x46, Key::G => 0x47, Key::H => 0x48,
        Key::I => 0x49, Key::J => 0x4A, Key::K => 0x4B, Key::L => 0x4C,
        Key::M => 0x4D, Key::N => 0x4E, Key::O => 0x4F, Key::P => 0x50,
        Key::Q => 0x51, Key::R => 0x52, Key::S => 0x53, Key::T => 0x54,
        Key::U => 0x55, Key::V => 0x56, Key::W => 0x57, Key::X => 0x58,
        Key::Y => 0x59, Key::Z => 0x5A,

        // Digits: VK_0 = 0x30
        Key::Digit0 => 0x30, Key::Digit1 => 0x31, Key::Digit2 => 0x32,
        Key::Digit3 => 0x33, Key::Digit4 => 0x34, Key::Digit5 => 0x35,
        Key::Digit6 => 0x36, Key::Digit7 => 0x37, Key::Digit8 => 0x38,
        Key::Digit9 => 0x39,

        // Function keys: VK_F1 = 0x70
        Key::F1 => 0x70, Key::F2 => 0x71, Key::F3 => 0x72, Key::F4 => 0x73,
        Key::F5 => 0x74, Key::F6 => 0x75, Key::F7 => 0x76, Key::F8 => 0x77,
        Key::F9 => 0x78, Key::F10 => 0x79, Key::F11 => 0x7A, Key::F12 => 0x7B,
        Key::F13 => 0x7C, Key::F14 => 0x7D, Key::F15 => 0x7E, Key::F16 => 0x7F,
        Key::F17 => 0x80, Key::F18 => 0x81, Key::F19 => 0x82, Key::F20 => 0x83,
        Key::F21 => 0x84, Key::F22 => 0x85, Key::F23 => 0x86, Key::F24 => 0x87,

        // Navigation
        Key::Home => 0x24, Key::End => 0x23,
        Key::PageUp => 0x21, Key::PageDown => 0x22,
        Key::Up => 0x26, Key::Down => 0x28,
        Key::Left => 0x25, Key::Right => 0x27,
        Key::Insert => 0x2D, Key::Delete => 0x2E,
        Key::Tab => 0x09, Key::Enter => 0x0D,
        Key::Escape => 0x1B, Key::Backspace => 0x08,
        Key::Space => 0x20,

        // Punctuation
        Key::Minus => 0xBD, Key::Equal => 0xBB,
        Key::BracketLeft => 0xDB, Key::BracketRight => 0xDD,
        Key::Backslash => 0xDC, Key::Semicolon => 0xBA,
        Key::Apostrophe => 0xDE, Key::Grave => 0xC0,
        Key::Comma => 0xBC, Key::Period => 0xBE, Key::Slash => 0xBF,
    }
}

/// Convert platform-agnostic modifier flags to Windows modifier flags.
fn modifiers_to_win(modifiers: u16) -> u32 {
    let mut flags = 0u32;
    if modifiers & keys::modifiers::SHIFT != 0 { flags |= 0x0004; }   // MOD_SHIFT
    if modifiers & keys::modifiers::CONTROL != 0 { flags |= 0x0002; } // MOD_CONTROL
    if modifiers & keys::modifiers::ALT != 0 { flags |= 0x0001; }     // MOD_ALT
    if modifiers & keys::modifiers::SUPER != 0 { flags |= 0x0008; }   // MOD_WIN
    flags
}

pub struct WindowsHotkeyManager {
    callbacks: HashMap<u32, HotkeyCallback>, // id -> callback
    next_id: u32,
    running: Arc<AtomicBool>,
}

impl WindowsHotkeyManager {
    pub fn new() -> Result<Self, KeyflowError> {
        Ok(Self {
            callbacks: HashMap::new(),
            next_id: 1,
            running: Arc::new(AtomicBool::new(false)),
        })
    }
}

impl HotkeyManager for WindowsHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        let _combo = keys::parse_hotkey(hotkey)?;
        // TODO: Implement RegisterHotKey
        // For now, store the callback for future implementation
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.insert(id, callback);
        log::warn!("Windows hotkey registration not yet implemented: {hotkey}");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::Release);
        log::info!("Hotkey manager started (Windows)");
        // TODO: Implement message loop with GetMessage/PeekMessage
        while self.running.load(Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(100));
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
