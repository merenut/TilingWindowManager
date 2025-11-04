//! Workspace management module
//!
//! This module provides functionality for managing workspaces, including
//! integration with Windows Virtual Desktops.

pub mod virtual_desktop;

pub use virtual_desktop::VirtualDesktopManager;
