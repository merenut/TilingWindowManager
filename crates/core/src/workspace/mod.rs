//! Workspace management module
//!
//! This module provides functionality for managing workspaces, including
//! integration with Windows Virtual Desktops and automatic state persistence.

pub mod auto_save;
pub mod core;
pub mod monitor_integration;
pub mod persistence;
pub mod state;
pub mod switching;
pub mod virtual_desktop;
pub mod window_ops;

pub use core::WorkspaceManager;
