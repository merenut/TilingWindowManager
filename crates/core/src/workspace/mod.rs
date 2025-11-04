//! Workspace management module
//!
//! This module provides functionality for managing workspaces, including
//! integration with Windows Virtual Desktops and automatic state persistence.

pub mod auto_save;
pub mod manager;
pub mod persistence;
pub mod virtual_desktop;

#[cfg(test)]
mod manager_tests;

pub use auto_save::AutoSaver;
pub use manager::{Workspace, WorkspaceConfig, WorkspaceManager};
pub use persistence::{PersistenceManager, SessionState, WindowState, WorkspaceState};
pub use virtual_desktop::VirtualDesktopManager;
