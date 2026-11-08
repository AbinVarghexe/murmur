//! Murmur core: everything that can be decided without touching the operating system.
//!
//! The Tauri app wraps this crate with the parts that must talk to Windows or macOS
//! (microphone, keyboard, clipboard, windows). Keeping the decisions here makes them
//! fast to test and identical on every platform.

#![forbid(unsafe_code)]

pub mod app_match;
pub mod audio_utils;
pub mod chunk;
pub mod controller;
pub mod format;
pub mod hotkeys;
pub mod keys;
pub mod memory;
pub mod planner;
pub mod resample;
pub mod settings;
pub mod urls;
pub mod vad;
