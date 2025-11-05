//! Clock module - displays current time

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ClockModule {
    config: ModuleConfig,
    current_time: String,
    clock_config: ClockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClockConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default)]
    format_alt: Option<String>,
}

fn default_format() -> String {
    "%H:%M:%S".to_string()
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            format_alt: Some("%Y-%m-%d".to_string()),
        }
    }
}

impl ClockModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "clock".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            current_time: String::new(),
            clock_config: ClockConfig::default(),
        }
    }
    
    fn update_time(&mut self) {
        self.current_time = Local::now()
            .format(&self.clock_config.format)
            .to_string();
    }
}

impl Module for ClockModule {
    fn view(&self) -> Element<'_, Message> {
        let color = parse_color(&self.config.style.color);
        
        styled_container(
            text(&self.current_time)
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
        if matches!(message, Message::Tick) {
            self.update_time();
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
    
    fn init(&mut self) -> Option<Task<Message>> {
        self.update_time();
        None
    }
    
    fn update_interval(&self) -> u64 {
        1 // Update every second
    }
}

impl Default for ClockModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_module_creation() {
        let module = ClockModule::new();
        assert_eq!(module.name(), "clock");
        assert_eq!(module.position(), Position::Right);
        assert!(module.config().enabled);
    }

    #[test]
    fn test_default_format() {
        let module = ClockModule::new();
        assert_eq!(module.clock_config.format, "%H:%M:%S");
        assert_eq!(
            module.clock_config.format_alt,
            Some("%Y-%m-%d".to_string())
        );
    }

    #[test]
    fn test_clock_config_defaults() {
        let config = ClockConfig::default();
        assert_eq!(config.format, "%H:%M:%S");
        assert_eq!(config.format_alt, Some("%Y-%m-%d".to_string()));
    }

    #[test]
    fn test_update_time_formats_correctly() {
        let mut module = ClockModule::new();
        module.update_time();
        
        // The time should be formatted as HH:MM:SS
        assert!(!module.current_time.is_empty());
        
        // Should match the pattern HH:MM:SS (e.g., "14:30:45")
        let parts: Vec<&str> = module.current_time.split(':').collect();
        assert_eq!(parts.len(), 3, "Time should have three parts separated by colons");
        
        // Each part should be a valid number
        for part in &parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "Each time component should be a number"
            );
        }
    }

    #[test]
    fn test_custom_format() {
        let mut module = ClockModule::new();
        module.clock_config.format = "%Y-%m-%d".to_string();
        module.update_time();
        
        // Should be formatted as YYYY-MM-DD
        assert!(!module.current_time.is_empty());
        
        // Verify date format YYYY-MM-DD
        let parts: Vec<&str> = module.current_time.split('-').collect();
        assert_eq!(parts.len(), 3, "Date should have three parts separated by dashes");
    }

    #[test]
    fn test_time_format_with_day_name() {
        let mut module = ClockModule::new();
        module.clock_config.format = "%A".to_string();
        module.update_time();
        
        // Should be a day name like "Monday", "Tuesday", etc.
        assert!(!module.current_time.is_empty());
        assert!(module.current_time.len() >= 6); // At least "Friday"
    }

    #[test]
    fn test_time_format_12_hour() {
        let mut module = ClockModule::new();
        module.clock_config.format = "%I:%M:%S %p".to_string();
        module.update_time();
        
        // Should include AM or PM
        assert!(
            module.current_time.contains("AM") || module.current_time.contains("PM"),
            "12-hour format should include AM or PM"
        );
    }

    #[test]
    fn test_update_interval() {
        let module = ClockModule::new();
        assert_eq!(module.update_interval(), 1);
    }

    #[test]
    fn test_init_updates_time() {
        let mut module = ClockModule::new();
        assert_eq!(module.current_time, "");
        
        module.init();
        assert!(!module.current_time.is_empty());
    }

    #[test]
    fn test_tick_message_updates_time() {
        let mut module = ClockModule::new();
        module.current_time = String::new();
        
        module.update(Message::Tick);
        assert!(!module.current_time.is_empty());
    }

    #[test]
    fn test_non_tick_message_does_not_update() {
        let mut module = ClockModule::new();
        module.update_time();
        let initial_time = module.current_time.clone();
        
        // Send a different message type
        module.update(Message::ModuleMessage {
            module_name: "test".to_string(),
            message: Box::new(crate::module::ModuleMessage::Refresh),
        });
        
        // Time should not have changed from update
        assert_eq!(module.current_time, initial_time);
    }

    #[test]
    fn test_format_validation_with_valid_formats() {
        // Test various valid chrono format strings
        let valid_formats = vec![
            "%H:%M:%S",
            "%Y-%m-%d",
            "%Y-%m-%d %H:%M:%S",
            "%I:%M:%S %p",
            "%A, %B %d, %Y",
            "%c",
            "%x %X",
        ];

        for format_str in valid_formats {
            let mut module = ClockModule::new();
            module.clock_config.format = format_str.to_string();
            
            // This should not panic
            module.update_time();
            assert!(
                !module.current_time.is_empty(),
                "Format {} should produce non-empty output",
                format_str
            );
        }
    }

    #[test]
    fn test_empty_format_produces_empty_string() {
        let mut module = ClockModule::new();
        module.clock_config.format = "".to_string();
        module.update_time();
        
        // Empty format should produce empty string
        assert_eq!(module.current_time, "");
    }

    #[test]
    fn test_literal_text_in_format() {
        let mut module = ClockModule::new();
        module.clock_config.format = "Current time: %H:%M".to_string();
        module.update_time();
        
        assert!(module.current_time.starts_with("Current time: "));
    }

    #[test]
    fn test_module_position_is_right() {
        let module = ClockModule::new();
        assert_eq!(module.position(), Position::Right);
    }

    #[test]
    fn test_module_config_enabled_by_default() {
        let module = ClockModule::new();
        assert!(module.config().enabled);
    }

    #[test]
    fn test_module_style_defaults() {
        let module = ClockModule::new();
        let style = &module.config().style;
        
        assert_eq!(style.padding, 10);
        assert_eq!(style.margin, 0);
        assert!(style.background.is_none());
    }

    #[test]
    fn test_clock_config_serialization() {
        let config = ClockConfig {
            format: "%H:%M:%S".to_string(),
            format_alt: Some("%Y-%m-%d".to_string()),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("%H:%M:%S"));

        // Test deserialization from JSON
        let deserialized: ClockConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.format_alt, config.format_alt);
    }

    #[test]
    fn test_clock_config_partial_deserialization() {
        // Test that missing fields use defaults
        let json = r#"{"format": "%I:%M %p"}"#;
        let config: ClockConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.format, "%I:%M %p");
        // format_alt should use default (None in this case will use the default)
    }

    #[test]
    fn test_multiple_updates_change_time() {
        let mut module = ClockModule::new();
        module.update_time();
        let time1 = module.current_time.clone();
        
        // Wait a tiny bit to ensure time might change
        // Note: This test might occasionally fail if run exactly at a second boundary
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        module.update_time();
        let time2 = module.current_time.clone();
        
        // Both times should be valid (non-empty)
        assert!(!time1.is_empty());
        assert!(!time2.is_empty());
    }
}
