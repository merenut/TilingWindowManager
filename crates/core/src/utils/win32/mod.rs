//! Windows API wrapper utilities for window management.
//!
//! This module provides safe Rust wrappers around Windows API functions for window management,
//! including window enumeration, property retrieval, and control operations.
//!
//! # Safety
//!
//! All unsafe Windows API calls are wrapped in safe Rust functions with proper error handling.
//! The module ensures no memory leaks by:
//! - Using proper buffer management for string operations
//! - Not storing raw pointers beyond their valid lifetime
//! - Properly handling Win32 API return values and error codes
//!
//! # Platform Support
//!
//! This module is only available on Windows platforms. Tests are conditional and will only
//! run on Windows (`#[cfg(target_os = "windows")]`).
//!
//! # Examples
//!
//! ```no_run
//! use tenraku_core::utils::win32::{enumerate_app_windows, get_foreground_window};
//!
//! // Get the currently focused window
//! if let Some(window) = get_foreground_window() {
//!     let title = window.get_title().unwrap_or_default();
//!     let pid = window.get_process_id();
//!     println!("Active: {} (PID: {})", title, pid);
//! }
//!
//! // Enumerate all application windows
//! let app_windows = enumerate_app_windows().unwrap();
//! for window in app_windows {
//!     let title = window.get_title().unwrap_or_default();
//!     println!("App: {}", title);
//! }
//! ```

mod enumeration;
mod filters;
mod handle;

pub use enumeration::{
    enumerate_app_windows, get_foreground_window,
};
pub use handle::WindowHandle;
