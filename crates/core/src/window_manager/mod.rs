//! Window manager module for managing window trees, workspaces, and monitors.
//!
//! This module provides the core WindowManager struct that orchestrates:
//! - Window trees for each workspace
//! - Workspace management and switching
//! - Monitor information and DPI handling
//! - Window filtering and management rules
//!
//! # Example
//!
//! ```no_run
//! use tenraku_core::window_manager::WindowManager;
//!
//! let mut wm = WindowManager::new();
//! wm.initialize().expect("Failed to initialize window manager");
//!
//! // The window manager is now ready to manage windows
//! ```

// Public submodules
pub mod focus;
pub mod layout;
pub mod monitor;
pub mod tree;
pub mod window;

// Internal implementation modules
mod core;
mod layout_operations;
mod monitor_ops;
mod window_operations;
mod workspace_operations;

#[cfg(test)]
mod tree_tests;

// Layout types are exported for public API use in later integration tasks
pub use focus::FocusManager;
pub use layout::{DwindleLayout, MasterLayout};
pub use monitor::MonitorInfo;
pub use tree::{Rect, Split, TreeNode};
pub use window::{ManagedWindow, WindowRegistry, WindowState};

// Re-export the WindowManager from core module
pub use self::core::WindowManager;

/// Layout algorithm type.
///
/// Determines how windows are arranged in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    /// Dwindle layout with smart split direction
    Dwindle,
    /// Master-stack layout
    Master,
}
