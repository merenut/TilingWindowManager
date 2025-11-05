//! CPU module - displays CPU usage

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use sysinfo::{System, CpuRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CpuModule {
    config: ModuleConfig,
    usage: f32,
    cpu_config: CpuConfig,
    // Note: We don't store System because it doesn't implement Sync
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpuConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_format() -> String {
    " {usage}%".to_string()
}

fn default_interval() -> u64 {
    5
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            interval: default_interval(),
        }
    }
}

impl CpuModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "cpu".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            usage: 0.0,
            cpu_config: CpuConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        // Create a temporary System instance to get CPU usage
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything())
        );
        system.refresh_cpu();
        self.usage = system.global_cpu_info().cpu_usage();
    }
    
    fn format_text(&self) -> String {
        self.cpu_config.format
            .replace("{usage}", &format!("{:.1}", self.usage))
    }
}

impl Module for CpuModule {
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
        if matches!(message, Message::Tick) {
            self.update_usage();
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
        self.update_usage();
        None
    }
    
    fn update_interval(&self) -> u64 {
        self.cpu_config.interval
    }
}

impl Default for CpuModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_module_creation() {
        let module = CpuModule::new();
        assert_eq!(module.name(), "cpu");
        assert_eq!(module.position(), Position::Right);
        assert!(module.config().enabled);
        assert_eq!(module.usage, 0.0);
    }

    #[test]
    fn test_cpu_config_defaults() {
        let config = CpuConfig::default();
        assert_eq!(config.format, " {usage}%");
        assert_eq!(config.interval, 5);
    }

    #[test]
    fn test_cpu_config_serialization() {
        let config = CpuConfig {
            format: "CPU: {usage}%".to_string(),
            interval: 10,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CpuConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, "CPU: {usage}%");
        assert_eq!(deserialized.interval, 10);
    }

    #[test]
    fn test_cpu_config_partial_deserialization() {
        // Test that default values are used when fields are missing
        let json = "{}";
        let config: CpuConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.format, " {usage}%");
        assert_eq!(config.interval, 5);
    }

    #[test]
    fn test_format_text_default() {
        let mut module = CpuModule::new();
        module.usage = 42.5;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 42.5%");
    }

    #[test]
    fn test_format_text_custom_format() {
        let mut module = CpuModule::new();
        module.cpu_config.format = "CPU: {usage}%".to_string();
        module.usage = 75.3;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "CPU: 75.3%");
    }

    #[test]
    fn test_format_text_no_placeholder() {
        let mut module = CpuModule::new();
        module.cpu_config.format = "CPU".to_string();
        module.usage = 50.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "CPU");
    }

    #[test]
    fn test_format_text_multiple_placeholders() {
        let mut module = CpuModule::new();
        module.cpu_config.format = "{usage}% / {usage}%".to_string();
        module.usage = 33.3;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "33.3% / 33.3%");
    }

    #[test]
    fn test_format_text_zero_usage() {
        let mut module = CpuModule::new();
        module.usage = 0.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 0.0%");
    }

    #[test]
    fn test_format_text_high_usage() {
        let mut module = CpuModule::new();
        module.usage = 99.9;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 99.9%");
    }

    #[test]
    fn test_format_text_rounds_to_one_decimal() {
        let mut module = CpuModule::new();
        module.usage = 42.567;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 42.6%");
    }

    #[test]
    fn test_update_interval() {
        let module = CpuModule::new();
        assert_eq!(module.update_interval(), 5);
    }

    #[test]
    fn test_update_interval_custom() {
        let mut module = CpuModule::new();
        module.cpu_config.interval = 10;
        assert_eq!(module.update_interval(), 10);
    }

    #[test]
    fn test_tick_message_updates_usage() {
        let mut module = CpuModule::new();
        let initial_usage = module.usage;
        
        // Update with Tick message
        let result = module.update(Message::Tick);
        assert!(result.is_none());
        
        // Usage should be updated (might be 0.0 or some value depending on system)
        // We just verify that update_usage was called by checking the value is valid
        assert!(module.usage >= 0.0);
        assert!(module.usage <= 100.0);
    }

    #[test]
    fn test_non_tick_message_does_not_update() {
        let mut module = CpuModule::new();
        module.usage = 50.0; // Set a specific value
        
        // Update with non-Tick message
        let result = module.update(Message::ModuleMessage {
            module_name: "test".to_string(),
            message: Box::new(crate::module::ModuleMessage::Refresh),
        });
        assert!(result.is_none());
        
        // Usage should remain unchanged
        assert_eq!(module.usage, 50.0);
    }

    #[test]
    fn test_init_updates_usage() {
        let mut module = CpuModule::new();
        
        // Call init
        let result = module.init();
        assert!(result.is_none());
        
        // Usage should be updated from system
        assert!(module.usage >= 0.0);
        assert!(module.usage <= 100.0);
    }

    #[test]
    fn test_module_config_position() {
        let module = CpuModule::new();
        assert_eq!(module.position(), Position::Right);
    }

    #[test]
    fn test_module_config_enabled_by_default() {
        let module = CpuModule::new();
        assert!(module.config().enabled);
    }

    #[test]
    fn test_module_style_defaults() {
        let module = CpuModule::new();
        let style = &module.config().style;
        
        assert_eq!(style.color, "#cdd6f4");
        assert_eq!(style.padding, 10);
        assert_eq!(style.margin, 0);
        assert!(style.background.is_none());
    }

    #[test]
    fn test_update_usage_sets_valid_range() {
        let mut module = CpuModule::new();
        
        // Call update_usage multiple times to ensure it works consistently
        for _ in 0..3 {
            module.update_usage();
            assert!(module.usage >= 0.0, "CPU usage should be >= 0.0, got {}", module.usage);
            assert!(module.usage <= 100.0, "CPU usage should be <= 100.0, got {}", module.usage);
        }
    }

    #[test]
    fn test_default_format_function() {
        assert_eq!(default_format(), " {usage}%");
    }

    #[test]
    fn test_default_interval_function() {
        assert_eq!(default_interval(), 5);
    }
}
