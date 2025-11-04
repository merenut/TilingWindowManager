//! Workspace management module
//!
//! This module provides functionality for managing workspaces, including
//! integration with Windows Virtual Desktops.

pub mod manager;
pub mod virtual_desktop;

#[cfg(test)]
mod manager_tests;

pub use manager::{Workspace, WorkspaceConfig, WorkspaceManager};
pub use virtual_desktop::VirtualDesktopManager;
