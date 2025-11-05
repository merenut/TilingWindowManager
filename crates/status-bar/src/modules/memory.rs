//! Memory module - displays memory usage

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use sysinfo::{System, MemoryRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryModule {
    config: ModuleConfig,
    usage_percent: f32,
    used_memory: u64,
    total_memory: u64,
    memory_config: MemoryConfig,
    // Note: We don't store System because it doesn't implement Sync
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_format() -> String {
    " {percentage}%".to_string()
}

fn default_interval() -> u64 {
    5
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            interval: default_interval(),
        }
    }
}

impl MemoryModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "memory".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            usage_percent: 0.0,
            used_memory: 0,
            total_memory: 0,
            memory_config: MemoryConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        // Create a temporary System instance to get memory usage
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything())
        );
        system.refresh_memory();
        
        self.used_memory = system.used_memory();
        self.total_memory = system.total_memory();
        
        let total = self.total_memory as f32;
        let used = self.used_memory as f32;
        self.usage_percent = (used / total) * 100.0;
    }
    
    fn format_text(&self) -> String {
        let used_gb = (self.used_memory as f64) / (1024.0 * 1024.0 * 1024.0);
        let total_gb = (self.total_memory as f64) / (1024.0 * 1024.0 * 1024.0);
        
        self.memory_config.format
            .replace("{percentage}", &format!("{:.1}", self.usage_percent))
            .replace("{used}", &format!("{:.1}", used_gb))
            .replace("{total}", &format!("{:.1}", total_gb))
    }
}

impl Module for MemoryModule {
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
        self.memory_config.interval
    }
}

impl Default for MemoryModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test constant for GB conversion to avoid magic numbers
    const GB_IN_BYTES: u64 = 1024 * 1024 * 1024;

    #[test]
    fn test_memory_module_creation() {
        let module = MemoryModule::new();
        assert_eq!(module.name(), "memory");
        assert_eq!(module.position(), Position::Right);
        assert!(module.config().enabled);
        assert_eq!(module.usage_percent, 0.0);
        assert_eq!(module.used_memory, 0);
        assert_eq!(module.total_memory, 0);
    }

    #[test]
    fn test_memory_config_defaults() {
        let config = MemoryConfig::default();
        assert_eq!(config.format, " {percentage}%");
        assert_eq!(config.interval, 5);
    }

    #[test]
    fn test_memory_config_serialization() {
        let config = MemoryConfig {
            format: "Mem: {used}/{total} GB ({percentage}%)".to_string(),
            interval: 10,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MemoryConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, "Mem: {used}/{total} GB ({percentage}%)");
        assert_eq!(deserialized.interval, 10);
    }

    #[test]
    fn test_memory_config_partial_deserialization() {
        // Test that default values are used when fields are missing
        let json = "{}";
        let config: MemoryConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.format, " {percentage}%");
        assert_eq!(config.interval, 5);
    }

    #[test]
    fn test_format_text_default() {
        let mut module = MemoryModule::new();
        module.usage_percent = 65.5;
        module.used_memory = 8 * GB_IN_BYTES; // 8 GB in bytes
        module.total_memory = 16 * GB_IN_BYTES; // 16 GB in bytes
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 65.5%");
    }

    #[test]
    fn test_format_text_with_percentage() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "{percentage}%".to_string();
        module.usage_percent = 42.3;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "42.3%");
    }

    #[test]
    fn test_format_text_with_used_and_total() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "{used}/{total} GB".to_string();
        module.used_memory = 8 * GB_IN_BYTES; // 8 GB
        module.total_memory = 16 * GB_IN_BYTES; // 16 GB
        
        let formatted = module.format_text();
        assert_eq!(formatted, "8.0/16.0 GB");
    }

    #[test]
    fn test_format_text_with_all_placeholders() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "{used}/{total} GB ({percentage}%)".to_string();
        module.usage_percent = 50.0;
        module.used_memory = 4 * GB_IN_BYTES; // 4 GB
        module.total_memory = 8 * GB_IN_BYTES; // 8 GB
        
        let formatted = module.format_text();
        assert_eq!(formatted, "4.0/8.0 GB (50.0%)");
    }

    #[test]
    fn test_format_text_no_placeholder() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "Memory".to_string();
        module.usage_percent = 50.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Memory");
    }

    #[test]
    fn test_format_text_zero_usage() {
        let mut module = MemoryModule::new();
        module.usage_percent = 0.0;
        module.used_memory = 0;
        module.total_memory = 8 * GB_IN_BYTES; // 8 GB
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 0.0%");
    }

    #[test]
    fn test_format_text_high_usage() {
        let mut module = MemoryModule::new();
        module.usage_percent = 95.7;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 95.7%");
    }

    #[test]
    fn test_format_text_rounds_to_one_decimal() {
        let mut module = MemoryModule::new();
        module.usage_percent = 33.456;
        
        let formatted = module.format_text();
        assert_eq!(formatted, " 33.5%");
    }

    #[test]
    fn test_format_text_gb_conversion() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "{used} GB".to_string();
        // 1.5 GB in bytes
        module.used_memory = (1.5 * 1024.0 * 1024.0 * 1024.0) as u64;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "1.5 GB");
    }

    #[test]
    fn test_format_text_small_memory() {
        let mut module = MemoryModule::new();
        module.memory_config.format = "{used}/{total} GB".to_string();
        // 512 MB in bytes for used
        module.used_memory = 512 * 1024 * 1024;
        // 2 GB in bytes for total
        module.total_memory = 2 * GB_IN_BYTES;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "0.5/2.0 GB");
    }

    #[test]
    fn test_update_interval() {
        let module = MemoryModule::new();
        assert_eq!(module.update_interval(), 5);
    }

    #[test]
    fn test_update_interval_custom() {
        let mut module = MemoryModule::new();
        module.memory_config.interval = 10;
        assert_eq!(module.update_interval(), 10);
    }

    #[test]
    fn test_tick_message_updates_usage() {
        let mut module = MemoryModule::new();
        
        // Update with Tick message
        let result = module.update(Message::Tick);
        assert!(result.is_none());
        
        // Usage should be updated from system
        assert!(module.usage_percent >= 0.0);
        assert!(module.usage_percent <= 100.0);
        assert!(module.used_memory > 0); // System should have some memory used
        assert!(module.total_memory > 0); // System should have total memory
    }

    #[test]
    fn test_non_tick_message_does_not_update() {
        let mut module = MemoryModule::new();
        module.usage_percent = 50.0;
        module.used_memory = 1000;
        module.total_memory = 2000;
        
        // Update with non-Tick message
        let result = module.update(Message::ModuleMessage {
            module_name: "test".to_string(),
            message: Box::new(crate::module::ModuleMessage::Refresh),
        });
        assert!(result.is_none());
        
        // Values should remain unchanged
        assert_eq!(module.usage_percent, 50.0);
        assert_eq!(module.used_memory, 1000);
        assert_eq!(module.total_memory, 2000);
    }

    #[test]
    fn test_init_updates_usage() {
        let mut module = MemoryModule::new();
        
        // Call init
        let result = module.init();
        assert!(result.is_none());
        
        // Usage should be updated from system
        assert!(module.usage_percent >= 0.0);
        assert!(module.usage_percent <= 100.0);
        assert!(module.used_memory > 0);
        assert!(module.total_memory > 0);
    }

    #[test]
    fn test_module_config_position() {
        let module = MemoryModule::new();
        assert_eq!(module.position(), Position::Right);
    }

    #[test]
    fn test_module_config_enabled_by_default() {
        let module = MemoryModule::new();
        assert!(module.config().enabled);
    }

    #[test]
    fn test_module_style_defaults() {
        let module = MemoryModule::new();
        let style = &module.config().style;
        
        assert_eq!(style.color, "#cdd6f4");
        assert_eq!(style.padding, 10);
        assert_eq!(style.margin, 0);
        assert!(style.background.is_none());
    }

    #[test]
    fn test_update_usage_sets_valid_range() {
        let mut module = MemoryModule::new();
        
        // Call update_usage multiple times to ensure it works consistently
        for _ in 0..3 {
            module.update_usage();
            assert!(module.usage_percent >= 0.0, "Memory usage should be >= 0.0, got {}", module.usage_percent);
            assert!(module.usage_percent <= 100.0, "Memory usage should be <= 100.0, got {}", module.usage_percent);
            assert!(module.total_memory > 0, "Total memory should be > 0");
            assert!(module.used_memory <= module.total_memory, "Used memory should be <= total memory");
        }
    }

    #[test]
    fn test_update_usage_calculates_percentage_correctly() {
        let mut module = MemoryModule::new();
        module.update_usage();
        
        // Recalculate percentage manually to verify calculation
        let expected_percentage = (module.used_memory as f32 / module.total_memory as f32) * 100.0;
        
        // Allow for small floating point differences
        let diff = (module.usage_percent - expected_percentage).abs();
        assert!(diff < 0.01, "Percentage calculation mismatch: expected {}, got {}", expected_percentage, module.usage_percent);
    }

    #[test]
    fn test_default_format_function() {
        assert_eq!(default_format(), " {percentage}%");
    }

    #[test]
    fn test_default_interval_function() {
        assert_eq!(default_interval(), 5);
    }

    #[test]
    fn test_memory_calculation_edge_case_full() {
        let mut module = MemoryModule::new();
        // Simulate full memory usage
        module.used_memory = 16 * GB_IN_BYTES;
        module.total_memory = 16 * GB_IN_BYTES;
        module.usage_percent = (module.used_memory as f32 / module.total_memory as f32) * 100.0;
        
        assert_eq!(module.usage_percent, 100.0);
    }

    #[test]
    fn test_memory_calculation_edge_case_half() {
        let mut module = MemoryModule::new();
        // Simulate half memory usage
        module.used_memory = 8 * GB_IN_BYTES;
        module.total_memory = 16 * GB_IN_BYTES;
        module.usage_percent = (module.used_memory as f32 / module.total_memory as f32) * 100.0;
        
        assert_eq!(module.usage_percent, 50.0);
    }
}
