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

// Win32 API constants
const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const WM_HOTKEY: u32 = 0x0312;
const WM_DESTROY: u32 = 0x0002;
const WM_QUIT: u32 = 0x0012;
const HWND_MESSAGE: isize = -3isize;

// Win32 API types
type HWND = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;
type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type BOOL = i32;
type UINT = u32;
type DWORD = u32;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINT,
    lPrivate: DWORD,
}

#[repr(C)]
struct WNDCLASSW {
    style: UINT,
    lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: *mut std::ffi::c_void,
    hCursor: *mut std::ffi::c_void,
    hbrBackground: *mut std::ffi::c_void,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
}

// Win32 API functions
extern "system" {
    fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: *mut std::ffi::c_void,
        hInstance: HINSTANCE,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn PostQuitMessage(nExitCode: i32);
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: UINT, vk: UINT) -> BOOL;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

/// Convert a platform-agnostic Key to its Windows Virtual Key code.
fn key_to_vk(key: Key) -> u16 {
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
    if modifiers & keys::modifiers::SHIFT != 0 { flags |= MOD_SHIFT; }
    if modifiers & keys::modifiers::CONTROL != 0 { flags |= MOD_CONTROL; }
    if modifiers & keys::modifiers::ALT != 0 { flags |= MOD_ALT; }
    if modifiers & keys::modifiers::SUPER != 0 { flags |= MOD_WIN; }
    flags
}

/// Window procedure for handling WM_HOTKEY messages.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let id = wparam as u32;
            log::debug!("WM_HOTKEY received: id={id}");
            // Store the hotkey ID for the callback
            HOTKEY_ID.store(id, Ordering::SeqCst);
            HOTKEY_RECEIVED.store(true, Ordering::SeqCst);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Global state for hotkey callback communication
static HOTKEY_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static HOTKEY_RECEIVED: AtomicBool = AtomicBool::new(false);

pub struct WindowsHotkeyManager {
    callbacks: HashMap<u32, HotkeyCallback>,
    next_id: u32,
    running: Arc<AtomicBool>,
    hwnd: Option<HWND>,
}

// Safety: HWND is a handle to a message-only window that we own and control.
// It's only accessed from the hotkey manager thread.
unsafe impl Send for WindowsHotkeyManager {}

impl WindowsHotkeyManager {
    pub fn new() -> Result<Self, KeyflowError> {
        Ok(Self {
            callbacks: HashMap::new(),
            next_id: 1,
            running: Arc::new(AtomicBool::new(false)),
            hwnd: None,
        })
    }

    /// Create a message-only window to receive WM_HOTKEY messages.
    fn create_message_window(&mut self) -> Result<(), KeyflowError> {
        unsafe {
            let class_name: Vec<u16> = "KeyFlowHotkey\0".encode_utf16().collect();

            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(std::ptr::null()),
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                0,                    // dwExStyle
                class_name.as_ptr(),  // lpClassName
                std::ptr::null(),     // lpWindowName
                0,                    // dwStyle
                0, 0, 0, 0,          // x, y, width, height
                HWND_MESSAGE as HWND, // hWndParent (message-only window)
                std::ptr::null_mut(), // hMenu
                GetModuleHandleW(std::ptr::null()), // hInstance
                std::ptr::null_mut(), // lpParam
            );

            if hwnd.is_null() {
                return Err(KeyflowError::Io(std::io::Error::other("Failed to create message window")));
            }

            self.hwnd = Some(hwnd);
            log::debug!("Message window created: {hwnd:?}");
        }
        Ok(())
    }
}

impl Drop for WindowsHotkeyManager {
    fn drop(&mut self) {
        // Unregister all hotkeys
        if let Some(hwnd) = self.hwnd {
            for id in self.callbacks.keys() {
                unsafe {
                    UnregisterHotKey(hwnd, *id as i32);
                }
            }
        }
    }
}

impl HotkeyManager for WindowsHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        let combo = keys::parse_hotkey(hotkey)?;

        // Ensure window is created
        if self.hwnd.is_none() {
            self.create_message_window()?;
        }

        let id = self.next_id;
        self.next_id += 1;

        let vk = key_to_vk(combo.key);
        let mods = modifiers_to_win(combo.modifiers);

        // Register the hotkey
        unsafe {
            let hwnd = self.hwnd.unwrap();
            let result = RegisterHotKey(hwnd, id as i32, mods, vk as u32);
            if result == 0 {
                return Err(KeyflowError::HotkeyRegistration {
                    hotkey: hotkey.to_string(),
                    reason: format!("RegisterHotKey failed (vk=0x{vk:02X}, mods=0x{mods:02X})"),
                });
            }
        }

        self.callbacks.insert(id, callback);
        log::info!("Registered hotkey: {hotkey} (id={id}, vk=0x{vk:02X}, mods=0x{mods:02X})");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::Release);
        log::info!("Hotkey manager started (Windows)");

        let mut msg = MSG {
            hwnd: std::ptr::null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
            lPrivate: 0,
        };

        unsafe {
            let hwnd = self.hwnd.unwrap_or(std::ptr::null_mut());

            while self.running.load(Ordering::Acquire) {
                // Use GetMessageW to wait for messages
                let result = GetMessageW(&mut msg, hwnd, 0, 0);

                if result == 0 {
                    // WM_QUIT received
                    break;
                } else if result == -1 {
                    // Error
                    log::error!("GetMessageW returned error");
                    break;
                }

                // Check if this is a WM_HOTKEY message
                if msg.message == WM_HOTKEY {
                    let id = msg.wParam as u32;
                    if let Some(callback) = self.callbacks.get(&id) {
                        log::debug!("Invoking callback for hotkey id={id}");
                        callback();
                    }
                }
            }
        }

        log::info!("Hotkey manager stopped");
        Ok(())
    }

    fn stop(&self) {
        self.running.store(false, Ordering::Release);
        // Post WM_QUIT to break GetMessageW
        unsafe {
            PostQuitMessage(0);
        }
    }

    fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}
