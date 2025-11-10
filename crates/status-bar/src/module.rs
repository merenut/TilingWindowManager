//! Module trait and base types for status bar widgets
//!
//! This module defines the core abstractions for creating extensible status bar modules.

use serde::{Deserialize, Serialize};

/// Position of a module on the status bar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Center,
    Right,
}

/// Base configuration for all modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Module name
    pub name: String,

    /// Position on bar
    pub position: Position,

    /// Whether module is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Module-specific configuration (JSON value)
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_true() -> bool {
    true
}

/// Messages that modules can send and receive
#[derive(Debug, Clone)]
pub enum Message {
    /// Tick for periodic updates
    Tick,

    /// IPC event received
    IpcEvent(IpcEvent),

    /// Request to switch workspace
    SwitchWorkspace(usize),

    /// Request to execute command
    ExecuteCommand(String),
}

/// IPC events from window manager
#[derive(Debug, Clone)]
pub enum IpcEvent {
    WorkspaceChanged { from: usize, to: usize },
    WindowFocused { hwnd: String, title: String },
    WindowCreated { hwnd: String, title: String },
    WindowClosed { hwnd: String },
    ConfigReloaded,
}

/// Module registry for managing loaded modules
pub struct ModuleRegistry {
    enabled_modules: Vec<ModuleConfig>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            enabled_modules: Vec::new(),
        }
    }

    pub fn register(&mut self, config: ModuleConfig) {
        if config.enabled {
            self.enabled_modules.push(config);
        }
    }

    pub fn get_by_position(&self, position: Position) -> Vec<&ModuleConfig> {
        self.enabled_modules
            .iter()
            .filter(|m| m.position == position)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.enabled_modules.len()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_serialization() {
        let pos = Position::Left;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""left""#);

        let deserialized: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Position::Left);
    }

    #[test]
    fn test_position_serialization_center() {
        let pos = Position::Center;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""center""#);

        let deserialized: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Position::Center);
    }

    #[test]
    fn test_position_serialization_right() {
        let pos = Position::Right;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""right""#);

        let deserialized: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Position::Right);
    }

    #[test]
    fn test_module_config_defaults() {
        let config = ModuleConfig {
            name: "test".to_string(),
            position: Position::Left,
            enabled: true,
            config: serde_json::Value::Null,
        };

        assert!(config.enabled);
    }

    #[test]
    fn test_module_registry_new() {
        let registry = ModuleRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_module_registry_default() {
        let registry = ModuleRegistry::default();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_module_config_serialization() {
        let config = ModuleConfig {
            name: "test".to_string(),
            position: Position::Left,
            enabled: true,
            config: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ModuleConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.position, Position::Left);
        assert!(deserialized.enabled);
    }
}
