//! Module trait and base types for status bar widgets
//!
//! This module defines the core abstractions for creating extensible status bar modules.

use iced::{widget::Container, Color, Element, Task};
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

/// Trait that all status bar modules must implement
pub trait Module: Send + Sync {
    /// Get the module's current view
    fn view(&self) -> Element<'_, Message>;
    
    /// Update the module with a message
    fn update(&mut self, message: Message) -> Option<Task<Message>>;
    
    /// Get the position where this module should be displayed
    fn position(&self) -> Position;
    
    /// Get the module's unique identifier
    fn name(&self) -> &str;
    
    /// Get the module's configuration
    fn config(&self) -> &ModuleConfig;
    
    /// Initialize the module (called once at startup)
    fn init(&mut self) -> Option<Task<Message>> {
        None
    }
    
    /// Cleanup when module is removed
    fn cleanup(&mut self) {}
    
    /// Get update interval in seconds (0 = no periodic updates)
    fn update_interval(&self) -> u64 {
        0
    }
}

/// Messages that modules can send and receive
#[derive(Debug, Clone)]
pub enum Message {
    /// Tick for periodic updates
    Tick,
    
    /// Module-specific message
    ModuleMessage {
        module_name: String,
        message: Box<ModuleMessage>,
    },
    
    /// IPC event received
    IpcEvent(IpcEvent),
    
    /// Request to switch workspace
    SwitchWorkspace(usize),
    
    /// Request to execute command
    ExecuteCommand(String),
}

/// Module-specific messages
#[derive(Debug, Clone)]
pub enum ModuleMessage {
    /// Workspace was clicked
    WorkspaceClicked(usize),
    
    /// Refresh the module
    Refresh,
    
    /// Custom string message
    Custom(String),
    
    /// Volume changed
    VolumeChanged(f32),
    
    /// Network interface clicked
    NetworkClicked(String),
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

/// Helper to convert hex color string to iced Color
pub fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
    
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
    } else {
        1.0
    };
    
    Color::from_rgba(r, g, b, a)
}

/// Helper to create a styled container for a module
pub fn styled_container<'a, M: 'a>(
    content: Element<'a, M>,
    style: &ModuleStyle,
) -> Container<'a, M> {
    let mut container = Container::new(content)
        .padding(style.padding);
    
    if let Some(ref bg) = style.background {
        let bg_color = parse_color(bg);
        container = container.style(move |_theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            }
        });
    }
    
    container
}

/// Module registry for managing loaded modules
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
    
    pub fn register(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }
    
    pub fn get_by_name(&self, name: &str) -> Option<&dyn Module> {
        self.modules.iter().find(|m| m.name() == name).map(|m| m.as_ref())
    }
    
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Box<dyn Module>> {
        self.modules.iter_mut().find(|m| m.name() == name)
    }
    
    pub fn get_by_position(&self, position: Position) -> Vec<&dyn Module> {
        self.modules
            .iter()
            .filter(|m| m.position() == position && m.config().enabled)
            .map(|m| m.as_ref())
            .collect()
    }
    
    pub fn get_by_position_mut(&mut self, position: Position) -> Vec<&mut Box<dyn Module>> {
        self.modules
            .iter_mut()
            .filter(|m| m.position() == position && m.config().enabled)
            .collect()
    }
    
    pub fn update_all(&mut self, message: Message) -> Vec<Task<Message>> {
        let mut tasks = Vec::new();
        
        for module in &mut self.modules {
            if let Some(task) = module.update(message.clone()) {
                tasks.push(task);
            }
        }
        
        tasks
    }
    
    pub fn count(&self) -> usize {
        self.modules.len()
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
            style: ModuleStyle::default(),
            config: serde_json::Value::Null,
        };
        
        assert!(config.enabled);
        assert_eq!(config.style.padding, 10);
    }
    
    #[test]
    fn test_parse_color_rgb() {
        let color = parse_color("#ff0000");
        assert_eq!(color, Color::from_rgb(1.0, 0.0, 0.0));
    }
    
    #[test]
    fn test_parse_color_rgba() {
        let color = parse_color("#ff000080");
        assert_eq!(color, Color::from_rgba(1.0, 0.0, 0.0, 0.5019608));
    }
    
    #[test]
    fn test_parse_color_with_hash() {
        let color = parse_color("#00ff00");
        assert_eq!(color, Color::from_rgb(0.0, 1.0, 0.0));
    }
    
    #[test]
    fn test_parse_color_without_hash() {
        let color = parse_color("0000ff");
        assert_eq!(color, Color::from_rgb(0.0, 0.0, 1.0));
    }
    
    #[test]
    fn test_parse_color_invalid() {
        // Should default to white (255, 255, 255) for invalid values
        let color = parse_color("#gggggg");
        assert_eq!(color, Color::from_rgb(1.0, 1.0, 1.0));
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
    fn test_module_style_default() {
        let style = ModuleStyle::default();
        assert_eq!(style.color, "#cdd6f4");
        assert_eq!(style.background, None);
        assert_eq!(style.font, None);
        assert_eq!(style.font_size, None);
        assert_eq!(style.padding, 10);
        assert_eq!(style.margin, 0);
    }
    
    #[test]
    fn test_module_config_serialization() {
        let config = ModuleConfig {
            name: "test".to_string(),
            position: Position::Left,
            enabled: true,
            style: ModuleStyle::default(),
            config: serde_json::Value::Null,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ModuleConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.position, Position::Left);
        assert!(deserialized.enabled);
    }
}
