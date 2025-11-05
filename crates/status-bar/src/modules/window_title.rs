//! Window title module - displays the active window's title

use crate::module::{Module, Message, ModuleConfig, Position, IpcEvent, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WindowTitleModule {
    config: ModuleConfig,
    window_title: String,
    window_config: WindowTitleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowTitleConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_max_length")]
    max_length: usize,
}

fn default_format() -> String {
    "{title}".to_string()
}

fn default_max_length() -> usize {
    50
}

impl Default for WindowTitleConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            max_length: default_max_length(),
        }
    }
}

impl WindowTitleModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "window-title".to_string(),
            position: Position::Center,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            window_title: String::new(),
            window_config: WindowTitleConfig::default(),
        }
    }
    
    fn truncate_title(&self, title: &str) -> String {
        // Use char_indices for safe Unicode truncation
        let char_count = title.chars().count();
        if char_count > self.window_config.max_length {
            let truncate_at = title
                .char_indices()
                .nth(self.window_config.max_length - 3)
                .map(|(idx, _)| idx)
                .unwrap_or(title.len());
            format!("{}...", &title[..truncate_at])
        } else {
            title.to_string()
        }
    }
    
    fn format_text(&self) -> String {
        let truncated = self.truncate_title(&self.window_title);
        self.window_config.format
            .replace("{title}", &truncated)
    }
}

impl Module for WindowTitleModule {
    fn view(&self) -> Element<'_, Message> {
        let color = parse_color(&self.config.style.color);
        
        styled_container(
            text(self.format_text())
                .style(move |_theme| {
                    iced::widget::text::Style {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Task<Message>> {
        match message {
            Message::IpcEvent(IpcEvent::WindowFocused { title, .. }) => {
                self.window_title = title;
            }
            Message::IpcEvent(IpcEvent::WindowClosed { .. }) => {
                // Clear title if no window focused
                self.window_title.clear();
            }
            _ => {}
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
}

impl Default for WindowTitleModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_title_module_creation() {
        let module = WindowTitleModule::new();
        assert_eq!(module.name(), "window-title");
        assert_eq!(module.position(), Position::Center);
        assert!(module.config().enabled);
    }

    #[test]
    fn test_default_config_values() {
        let module = WindowTitleModule::new();
        assert_eq!(module.window_config.format, "{title}");
        assert_eq!(module.window_config.max_length, 50);
        assert!(module.window_title.is_empty());
    }

    #[test]
    fn test_window_title_config_defaults() {
        let config = WindowTitleConfig::default();
        assert_eq!(config.format, "{title}");
        assert_eq!(config.max_length, 50);
    }

    #[test]
    fn test_window_title_config_serialization() {
        let config = WindowTitleConfig {
            format: "{title}".to_string(),
            max_length: 50,
        };
        
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: WindowTitleConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.max_length, config.max_length);
    }

    #[test]
    fn test_window_title_config_partial_deserialization() {
        // Test that default values are used when fields are missing
        let json = r#"{}"#;
        let config: WindowTitleConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.format, "{title}");
        assert_eq!(config.max_length, 50);
    }

    #[test]
    fn test_truncate_title_short_title() {
        let module = WindowTitleModule::new();
        let title = "Short Title";
        let truncated = module.truncate_title(title);
        assert_eq!(truncated, "Short Title");
    }

    #[test]
    fn test_truncate_title_long_title() {
        let mut module = WindowTitleModule::new();
        module.window_config.max_length = 20;
        
        let title = "This is a very long title that should be truncated";
        let truncated = module.truncate_title(title);
        
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() < title.len());
        // Should be 17 chars + "..." = 20 chars max in character count
        assert!(truncated.chars().count() <= 20);
    }

    #[test]
    fn test_truncate_title_unicode() {
        let mut module = WindowTitleModule::new();
        module.window_config.max_length = 15;
        
        let title = "🎨 Unicode Test 🚀 With Emojis 🌟";
        let truncated = module.truncate_title(title);
        
        // Should handle Unicode correctly
        assert!(truncated.ends_with("..."));
        assert!(truncated.chars().count() <= 15);
    }

    #[test]
    fn test_truncate_title_exact_length() {
        let mut module = WindowTitleModule::new();
        module.window_config.max_length = 11;
        
        let title = "Exact Title"; // Exactly 11 characters
        let truncated = module.truncate_title(title);
        
        assert_eq!(truncated, "Exact Title");
        assert!(!truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_title_one_over_limit() {
        let mut module = WindowTitleModule::new();
        module.window_config.max_length = 10;
        
        let title = "12345678901"; // 11 characters, 1 over limit
        let truncated = module.truncate_title(title);
        
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.chars().count(), 10); // "1234567..." = 10 chars
    }

    #[test]
    fn test_truncate_title_empty() {
        let module = WindowTitleModule::new();
        let title = "";
        let truncated = module.truncate_title(title);
        assert_eq!(truncated, "");
    }

    #[test]
    fn test_format_text_default() {
        let mut module = WindowTitleModule::new();
        module.window_title = "Test Window".to_string();
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Test Window");
    }

    #[test]
    fn test_format_text_custom_format() {
        let mut module = WindowTitleModule::new();
        module.window_title = "MyApp".to_string();
        module.window_config.format = "Title: {title}".to_string();
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Title: MyApp");
    }

    #[test]
    fn test_format_text_with_truncation() {
        let mut module = WindowTitleModule::new();
        module.window_config.max_length = 10;
        module.window_title = "This is a very long window title".to_string();
        
        let formatted = module.format_text();
        assert!(formatted.contains("..."));
        assert!(formatted.chars().count() <= 10);
    }

    #[test]
    fn test_format_text_empty_title() {
        let module = WindowTitleModule::new();
        let formatted = module.format_text();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_format_text_no_placeholder() {
        let mut module = WindowTitleModule::new();
        module.window_title = "MyApp".to_string();
        module.window_config.format = "Static Text".to_string();
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Static Text");
    }

    #[test]
    fn test_ipc_event_window_focused() {
        let mut module = WindowTitleModule::new();
        assert!(module.window_title.is_empty());
        
        let event = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x12345".to_string(),
            title: "New Window Title".to_string(),
        });
        
        module.update(event);
        assert_eq!(module.window_title, "New Window Title");
    }

    #[test]
    fn test_ipc_event_window_focused_updates_existing() {
        let mut module = WindowTitleModule::new();
        module.window_title = "Old Title".to_string();
        
        let event = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x67890".to_string(),
            title: "Updated Title".to_string(),
        });
        
        module.update(event);
        assert_eq!(module.window_title, "Updated Title");
    }

    #[test]
    fn test_ipc_event_window_closed() {
        let mut module = WindowTitleModule::new();
        module.window_title = "Some Window".to_string();
        
        let event = Message::IpcEvent(IpcEvent::WindowClosed {
            hwnd: "0x12345".to_string(),
        });
        
        module.update(event);
        assert!(module.window_title.is_empty());
    }

    #[test]
    fn test_ipc_event_window_closed_when_empty() {
        let mut module = WindowTitleModule::new();
        assert!(module.window_title.is_empty());
        
        let event = Message::IpcEvent(IpcEvent::WindowClosed {
            hwnd: "0x12345".to_string(),
        });
        
        module.update(event);
        assert!(module.window_title.is_empty());
    }

    #[test]
    fn test_non_ipc_event_does_not_update() {
        let mut module = WindowTitleModule::new();
        module.window_title = "Original Title".to_string();
        
        let event = Message::Tick;
        module.update(event);
        
        assert_eq!(module.window_title, "Original Title");
    }

    #[test]
    fn test_module_position_is_center() {
        let module = WindowTitleModule::new();
        assert_eq!(module.position(), Position::Center);
    }

    #[test]
    fn test_module_name() {
        let module = WindowTitleModule::new();
        assert_eq!(module.name(), "window-title");
    }

    #[test]
    fn test_module_enabled_by_default() {
        let module = WindowTitleModule::new();
        assert!(module.config().enabled);
    }

    #[test]
    fn test_module_config_position() {
        let module = WindowTitleModule::new();
        assert_eq!(module.config.position, Position::Center);
    }

    #[test]
    fn test_module_style_defaults() {
        let module = WindowTitleModule::new();
        assert_eq!(module.config.style.padding, 10);
    }

    #[test]
    fn test_ipc_event_workspace_changed_does_not_affect_title() {
        let mut module = WindowTitleModule::new();
        module.window_title = "Test Window".to_string();
        
        let event = Message::IpcEvent(IpcEvent::WorkspaceChanged {
            from: 1,
            to: 2,
        });
        
        module.update(event);
        assert_eq!(module.window_title, "Test Window");
    }

    #[test]
    fn test_multiple_focus_events_in_sequence() {
        let mut module = WindowTitleModule::new();
        
        let event1 = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x1".to_string(),
            title: "Window 1".to_string(),
        });
        module.update(event1);
        assert_eq!(module.window_title, "Window 1");
        
        let event2 = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x2".to_string(),
            title: "Window 2".to_string(),
        });
        module.update(event2);
        assert_eq!(module.window_title, "Window 2");
        
        let event3 = Message::IpcEvent(IpcEvent::WindowClosed {
            hwnd: "0x2".to_string(),
        });
        module.update(event3);
        assert!(module.window_title.is_empty());
    }

    #[test]
    fn test_special_characters_in_title() {
        let mut module = WindowTitleModule::new();
        
        let event = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x123".to_string(),
            title: "Test <>&\"' Window".to_string(),
        });
        
        module.update(event);
        assert_eq!(module.window_title, "Test <>&\"' Window");
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Test <>&\"' Window");
    }

    #[test]
    fn test_update_returns_none() {
        let mut module = WindowTitleModule::new();
        
        let event = Message::IpcEvent(IpcEvent::WindowFocused {
            hwnd: "0x123".to_string(),
            title: "Test".to_string(),
        });
        
        let result = module.update(event);
        assert!(result.is_none());
    }

    #[test]
    fn test_default_format_function() {
        assert_eq!(default_format(), "{title}");
    }

    #[test]
    fn test_default_max_length_function() {
        assert_eq!(default_max_length(), 50);
    }
}
