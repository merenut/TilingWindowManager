//! Application modules.
//!
//! This module contains the main application logic split into focused components.

pub mod commands;
pub mod event_handling;
pub mod initialization;

pub use event_handling::run_event_loop;
pub use initialization::{
    demonstrate_command_system, initialize_logging, load_and_validate_config,
    scan_and_manage_windows,
};
