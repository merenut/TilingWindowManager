//! Keybinding system for Windows hotkey registration and handling.
//!
//! This module provides the infrastructure for registering global hotkeys
//! with Windows and mapping them to commands. It includes:
//!
//! - `KeybindManager`: Main struct for managing hotkey registration
//! - Key and modifier parsing utilities
//! - Integration with the Windows API for global hotkeys
//!
//! # Example
//!
//! ```no_run
//! use tenraku_core::keybinds::KeybindManager;
//! use tenraku_core::config::schema::Keybind;
//!
//! let mut manager = KeybindManager::new();
//!
//! let keybind = Keybind {
//!     modifiers: vec!["Win".to_string()],
//!     key: "Q".to_string(),
//!     command: "close".to_string(),
//!     args: vec![],
//! };
//!
//! manager.register_keybinds(vec![keybind]).ok();
//! ```

pub mod manager;
pub mod parser;

pub use manager::KeybindManager;
