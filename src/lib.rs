//! KeyFlow — Non-paste password input assistant
//!
//! Simulates keystrokes to bypass paste-disabled password fields.
//! Integrates with Bitwarden CLI for secure password retrieval.

pub mod config;
pub mod error;
pub mod provider;
pub mod input;
pub mod hotkey;
pub mod daemon;
pub mod cli;

#[cfg(target_os = "windows")]
pub mod windows_service;