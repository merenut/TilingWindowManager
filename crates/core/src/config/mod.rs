//! Configuration module for the tiling window manager
//! 
//! This module provides comprehensive configuration management including:
//! - TOML-based configuration schema with serde support
//! - Configuration file parsing and validation
//! - Hot-reload capability with file watching
//! - Default configuration generation

pub mod schema;
pub mod parser;
pub mod validator;
pub mod watcher;

pub use schema::*;
pub use parser::ConfigLoader;
pub use validator::ConfigValidator;
pub use watcher::ConfigWatcher;
