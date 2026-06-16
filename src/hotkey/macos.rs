//! macOS global hotkey implementation.
//!
//! Uses Carbon RegisterEventHotKey for global hotkey registration.
//! Note: Carbon is deprecated but still functional for this purpose.

use crate::error::KeyflowError;
use crate::hotkey::keys::{self, Key};
use crate::hotkey::{HotkeyCallback, HotkeyManager};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Convert a platform-agnostic Key to its macOS CGKeyCode.
fn key_to_macos(key: Key) -> u32 {
    match key {
        // Letters: kVK_ANSI_A = 0x00 (QWERTY physical position)
        Key::A => 0x00, Key::B => 0x0B, Key::C => 0x08, Key::D => 0x02,
        Key::E => 0x0E, Key::F => 0x03, Key::G => 0x05, Key::H => 0x04,
        Key::I => 0x22, Key::J => 0x26, Key::K => 0x28, Key::L => 0x25,
        Key::M => 0x2E, Key::N => 0x2D, Key::O => 0x1F, Key::P => 0x23,
        Key::Q => 0x0C, Key::R => 0x0F, Key::S => 0x01, Key::T => 0x11,
        Key::U => 0x20, Key::V => 0x09, Key::W => 0x0D, Key::X => 0x07,
        Key::Y => 0x10, Key::Z => 0x06,

        // Digits
        Key::Digit0 => 0x1D, Key::Digit1 => 0x12, Key::Digit2 => 0x13,
        Key::Digit3 => 0x14, Key::Digit4 => 0x15, Key::Digit5 => 0x17,
        Key::Digit6 => 0x16, Key::Digit7 => 0x1A, Key::Digit8 => 0x1C,
        Key::Digit9 => 0x19,

        // Function keys
        Key::F1 => 0x7A, Key::F2 => 0x78, Key::F3 => 0x63, Key::F4 => 0x76,
        Key::F5 => 0x60, Key::F6 => 0x61, Key::F7 => 0x62, Key::F8 => 0x64,
        Key::F9 => 0x65, Key::F10 => 0x6D, Key::F11 => 0x67, Key::F12 => 0x6F,
        Key::F13 => 0x69, Key::F14 => 0x6B, Key::F15 => 0x71, Key::F16 => 0x6A,
        Key::F17 => 0x40, Key::F18 => 0x4F, Key::F19 => 0x50, Key::F20 => 0x5A,
        // F21-F24 not standard on macOS
        Key::F21 => 0xFF, Key::F22 => 0xFF, Key::F23 => 0xFF, Key::F24 => 0xFF,

        // Navigation
        Key::Home => 0x73, Key::End => 0x77,
        Key::PageUp => 0x74, Key::PageDown => 0x79,
        Key::Up => 0x7E, Key::Down => 0x7D,
        Key::Left => 0x7B, Key::Right => 0x7C,
        Key::Insert => 0x72, Key::Delete => 0x75,
        Key::Tab => 0x30, Key::Enter => 0x24,
        Key::Escape => 0x35, Key::Backspace => 0x33,
        Key::Space => 0x31,

        // Punctuation
        Key::Minus => 0x1B, Key::Equal => 0x18,
        Key::BracketLeft => 0x21, Key::BracketRight => 0x1E,
        Key::Backslash => 0x2A, Key::Semicolon => 0x29,
        Key::Apostrophe => 0x27, Key::Grave => 0x32,
        Key::Comma => 0x2B, Key::Period => 0x2F, Key::Slash => 0x2C,
    }
}

/// Convert platform-agnostic modifier flags to macOS modifier flags.
fn modifiers_to_macos(modifiers: u16) -> u32 {
    let mut flags = 0u32;
    if modifiers & keys::modifiers::SHIFT != 0 { flags |= 1 << 17; }   // kCGEventFlagMaskShift
    if modifiers & keys::modifiers::CONTROL != 0 { flags |= 1 << 18; } // kCGEventFlagMaskControl
    if modifiers & keys::modifiers::ALT != 0 { flags |= 1 << 19; }     // kCGEventFlagMaskAlternate
    if modifiers & keys::modifiers::SUPER != 0 { flags |= 1 << 20; }   // kCGEventFlagMaskCommand
    flags
}

pub struct MacosHotkeyManager {
    callbacks: HashMap<u32, HotkeyCallback>, // hotkey_id -> callback
    next_id: u32,
    running: Arc<AtomicBool>,
}

impl MacosHotkeyManager {
    pub fn new() -> Result<Self, KeyflowError> {
        Ok(Self {
            callbacks: HashMap::new(),
            next_id: 1,
            running: Arc::new(AtomicBool::new(false)),
        })
    }
}

impl HotkeyManager for MacosHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        let _combo = keys::parse_hotkey(hotkey)?;
        // TODO: Implement RegisterEventHotKey
        // For now, store the callback for future implementation
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.insert(id, callback);
        log::warn!("macOS hotkey registration not yet implemented: {hotkey}");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::Release);
        log::info!("Hotkey manager started (macOS)");
        // TODO: Implement CFRunLoop or event tap
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
