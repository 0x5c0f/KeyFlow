//! macOS global hotkey implementation.
//!
//! Uses Carbon RegisterEventHotKey for global hotkey registration.
//! Note: Carbon is deprecated but still functional for this purpose.

use crate::error::KeyflowError;
use crate::hotkey::keys::{self, Key};
use crate::hotkey::{HotkeyCallback, HotkeyManager};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

// Carbon API types
type OSStatus = i32;
type EventHotKeyRef = *mut std::ffi::c_void;
type EventHandlerRef = *mut std::ffi::c_void;
type EventTargetRef = *mut std::ffi::c_void;
type EventRef = *mut std::ffi::c_void;
type CFRunLoopRef = *mut std::ffi::c_void;
type EventHandlerCallRef = *mut std::ffi::c_void;

// Carbon event constants
const kEventClassKeyboard: u32 = 0x6B657962; // 'keyb'
const kEventHotKeyPressed: u32 = 5;

// Modifier key constants
const cmdKey: u32 = 1 << 8;
const shiftKey: u32 = 1 << 9;
const optionKey: u32 = 1 << 11;
const controlKey: u32 = 1 << 12;

// Carbon event structures
#[repr(C)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[repr(C)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

// Carbon API functions
extern "C" {
    fn RegisterEventHotKey(
        inHotKeyCode: u32,
        inHotKeyModifiers: u32,
        inHotKeyID: EventHotKeyID,
        inTarget: EventTargetRef,
        inOptions: u32,
        outRef: *mut EventHotKeyRef,
    ) -> OSStatus;

    fn UnregisterEventHotKey(inHotKeyRef: EventHotKeyRef) -> OSStatus;

    fn GetApplicationEventTarget() -> EventTargetRef;

    fn InstallEventHandler(
        inTarget: EventTargetRef,
        inHandler: Option<unsafe extern "C" fn(EventHandlerCallRef, EventRef, *mut std::ffi::c_void) -> OSStatus>,
        inNumTypes: u32,
        inList: *const EventTypeSpec,
        inUserData: *mut std::ffi::c_void,
        outRef: *mut EventHandlerRef,
    ) -> OSStatus;

    fn GetEventParameter(
        inEvent: EventRef,
        inName: u32,
        inDesiredType: u32,
        outActualType: *mut u32,
        inBufferSize: u32,
        outBufferSize: *mut u32,
        outData: *mut std::ffi::c_void,
    ) -> OSStatus;

    fn CFRunLoopRun();
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopStop(inRunLoop: CFRunLoopRef);
}

// Global callback storage - shared between main thread and event handler
static HOTKEY_CALLBACKS: OnceLock<Mutex<HashMap<u32, HotkeyCallback>>> = OnceLock::new();

fn get_callbacks() -> &'static Mutex<HashMap<u32, HotkeyCallback>> {
    HOTKEY_CALLBACKS.get_or_init(|| Mutex::new(HashMap::new()))
}

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
    if modifiers & keys::modifiers::SHIFT != 0 { flags |= shiftKey; }
    if modifiers & keys::modifiers::CONTROL != 0 { flags |= controlKey; }
    if modifiers & keys::modifiers::ALT != 0 { flags |= optionKey; }
    if modifiers & keys::modifiers::SUPER != 0 { flags |= cmdKey; }
    flags
}

/// Carbon event handler callback for hotkey events.
unsafe extern "C" fn hotkey_handler(
    _next_handler: EventHandlerCallRef,
    event: EventRef,
    _user_data: *mut std::ffi::c_void,
) -> OSStatus {
    let mut hotkey_id = EventHotKeyID { signature: 0, id: 0 };
    let mut size: u32 = 0;

    let status = GetEventParameter(
        event,
        1, // kEventParamDirectObject
        0x686B4944, // typeEventHotKeyID (hkid)
        std::ptr::null_mut(),
        std::mem::size_of::<EventHotKeyID>() as u32,
        &mut size,
        &mut hotkey_id as *mut EventHotKeyID as *mut std::ffi::c_void,
    );

    if status == 0 {
        let id = hotkey_id.id;
        log::debug!("Hotkey event received: id={id}");

        // Invoke the callback from the global map
        if let Ok(callbacks) = get_callbacks().lock() {
            if let Some(callback) = callbacks.get(&id) {
                callback();
            }
        }
    }

    0 // noErr
}

pub struct MacosHotkeyManager {
    next_id: u32,
    running: Arc<AtomicBool>,
    handler_ref: Option<EventHandlerRef>,
    hotkey_refs: Vec<EventHotKeyRef>,
    cf_loop: RefCell<Option<CFRunLoopRef>>,
}

impl MacosHotkeyManager {
    pub fn new() -> Result<Self, KeyflowError> {
        Ok(Self {
            next_id: 1,
            running: Arc::new(AtomicBool::new(false)),
            handler_ref: None,
            hotkey_refs: Vec::new(),
            cf_loop: RefCell::new(None),
        })
    }

    /// Install the Carbon event handler for hotkey events.
    fn install_handler(&mut self) -> Result<(), KeyflowError> {
        let event_types = [
            EventTypeSpec {
                event_class: kEventClassKeyboard,
                event_kind: kEventHotKeyPressed,
            },
        ];

        let mut handler_ref: EventHandlerRef = std::ptr::null_mut();

        unsafe {
            let status = InstallEventHandler(
                GetApplicationEventTarget(),
                Some(hotkey_handler),
                event_types.len() as u32,
                event_types.as_ptr(),
                std::ptr::null_mut(),
                &mut handler_ref,
            );

            if status != 0 {
                return Err(KeyflowError::Io(std::io::Error::other(
                    format!("InstallEventHandler failed: {status}"),
                )));
            }
        }

        self.handler_ref = Some(handler_ref);
        log::debug!("Carbon event handler installed");
        Ok(())
    }
}

impl Drop for MacosHotkeyManager {
    fn drop(&mut self) {
        // Unregister all hotkeys
        for hotkey_ref in &self.hotkey_refs {
            unsafe {
                UnregisterEventHotKey(*hotkey_ref);
            }
        }

        // Stop CFRunLoop if running
        if let Some(cf_loop) = *self.cf_loop.borrow() {
            unsafe {
                CFRunLoopStop(cf_loop);
            }
        }

        // Clear global callbacks
        if let Ok(mut callbacks) = get_callbacks().lock() {
            callbacks.clear();
        }
    }
}

impl HotkeyManager for MacosHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        let combo = keys::parse_hotkey(hotkey)?;

        // Install handler if not already done
        if self.handler_ref.is_none() {
            self.install_handler()?;
        }

        let id = self.next_id;
        self.next_id += 1;

        let keycode = key_to_macos(combo.key);
        let mods = modifiers_to_macos(combo.modifiers);

        let hotkey_id = EventHotKeyID {
            signature: 0x4B46, // 'KF' - KeyFlow signature
            id,
        };

        let mut hotkey_ref: EventHotKeyRef = std::ptr::null_mut();

        unsafe {
            let status = RegisterEventHotKey(
                keycode,
                mods,
                hotkey_id,
                GetApplicationEventTarget(),
                0,
                &mut hotkey_ref,
            );

            if status != 0 {
                return Err(KeyflowError::HotkeyRegistration {
                    hotkey: hotkey.to_string(),
                    reason: format!("RegisterEventHotKey failed: {status}"),
                });
            }
        }

        self.hotkey_refs.push(hotkey_ref);

        // Store callback in global map for the event handler
        if let Ok(mut callbacks) = get_callbacks().lock() {
            callbacks.insert(id, callback);
        }

        log::info!("Registered hotkey: {hotkey} (id={id}, keycode=0x{keycode:02X}, mods=0x{mods:02X})");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::Release);
        log::info!("Hotkey manager started (macOS)");

        // CFRunLoop needs to run on the main thread for hotkey events to work
        unsafe {
            let cf_loop = CFRunLoopGetCurrent();
            *self.cf_loop.borrow_mut() = Some(cf_loop);

            // This will block until CFRunLoopStop is called
            CFRunLoopRun();
        }

        log::info!("Hotkey manager stopped");
        Ok(())
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Release);
        // Stop CFRunLoop
        if let Some(cf_loop) = *self.cf_loop.borrow() {
            unsafe {
                CFRunLoopStop(cf_loop);
            }
        }
    }

    fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}
