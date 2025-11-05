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
    
    /// Custom styling
    #[serde(default)]
    pub style: ModuleStyle,
    
    /// Module-specific configuration (JSON value)
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Styling options for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStyle {
    /// Foreground color
    #[serde(default = "default_foreground")]
    pub color: String,
    
    /// Background color
    #[serde(default)]
    pub background: Option<String>,
    
    /// Font family
    #[serde(default)]
    pub font: Option<String>,
    
    /// Font size
    #[serde(default)]
    pub font_size: Option<f32>,
    
    /// Padding (pixels)
    #[serde(default = "default_padding")]
    pub padding: u16,
    
    /// Margin (pixels)
    #[serde(default)]
    pub margin: u16,
}

impl Default for ModuleStyle {
    fn default() -> Self {
        Self {
            color: default_foreground(),
            background: None,
            font: None,
            font_size: None,
            padding: default_padding(),
            margin: 0,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_foreground() -> String {
    "#cdd6f4".to_string()
}

fn default_padding() -> u16 {
    10
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
    fn test_module_config_defaults() {
        let config = ModuleConfig {
            name: "test".to_string(),
            position: Position::Left,
            enabled: true,
            style: ModuleStyle::default(),
            config: serde_json::Value::Null,
        };
        
        assert!(config.enabled);
        assert_eq!(config.style.padding, 10);
    }
}
