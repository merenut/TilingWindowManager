//! Battery module - displays battery status (optional, only on laptops)

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use battery::{Manager, State};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct BatteryModule {
    config: ModuleConfig,
    percentage: f32,
    state: BatteryState,
    battery_config: BatteryConfig,
    // Note: We don't store Manager because it doesn't implement Sync
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatteryConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_warning_level")]
    warning_level: u32,
    
    #[serde(default = "default_critical_level")]
    critical_level: u32,
}

fn default_format() -> String {
    "{icon} {percentage}%".to_string()
}

fn default_warning_level() -> u32 {
    30
}

fn default_critical_level() -> u32 {
    15
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            warning_level: default_warning_level(),
            critical_level: default_critical_level(),
        }
    }
}

impl BatteryModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "battery".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            percentage: 0.0,
            state: BatteryState::Unknown,
            battery_config: BatteryConfig::default(),
        }
    }
    
    /// Check if battery is available on this system
    pub fn is_available() -> bool {
        Manager::new()
            .ok()
            .and_then(|manager| manager.batteries().ok())
            .map(|mut batteries| batteries.next().is_some())
            .unwrap_or(false)
    }
    
    fn update_status(&mut self) {
        // Create a temporary Manager instance to get battery status
        if let Ok(manager) = Manager::new() {
            if let Ok(batteries) = manager.batteries() {
                if let Some(Ok(battery)) = batteries.into_iter().next() {
                    self.percentage = battery.state_of_charge().value * 100.0;
                    self.state = match battery.state() {
                        State::Charging => BatteryState::Charging,
                        State::Discharging => BatteryState::Discharging,
                        State::Full => BatteryState::Full,
                        _ => BatteryState::Unknown,
                    };
                }
            }
        }
    }
    
    fn get_icon(&self) -> &str {
        match self.state {
            BatteryState::Charging => "🔌",
            BatteryState::Full => "🔋",
            BatteryState::Discharging => {
                if self.percentage > 50.0 {
                    "🔋"
                } else {
                    "🪫"
                }
            }
            BatteryState::Unknown => "❓",
        }
    }
    
    fn format_text(&self) -> String {
        self.battery_config.format
            .replace("{icon}", self.get_icon())
            .replace("{percentage}", &format!("{:.0}", self.percentage))
            .replace("{state}", match self.state {
                BatteryState::Charging => "Charging",
                BatteryState::Discharging => "Discharging",
                BatteryState::Full => "Full",
                BatteryState::Unknown => "Unknown",
            })
    }
    
    fn get_color(&self) -> String {
        if self.percentage <= self.battery_config.critical_level as f32 {
            "#f38ba8".to_string() // Critical - red
        } else if self.percentage <= self.battery_config.warning_level as f32 {
            "#f9e2af".to_string() // Warning - yellow
        } else {
            self.config.style.color.clone()
        }
    }
}

impl Module for BatteryModule {
    fn view(&self) -> Element<'_, Message> {
        let color = parse_color(&self.get_color());
        
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
            self.update_status();
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
        self.update_status();
        None
    }
    
    fn update_interval(&self) -> u64 {
        30 // Update every 30 seconds
    }
}

impl Default for BatteryModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_module_creation() {
        let module = BatteryModule::new();
        assert_eq!(module.name(), "battery");
        assert_eq!(module.position(), Position::Right);
        assert!(module.config().enabled);
    }

    #[test]
    fn test_default_format() {
        let module = BatteryModule::new();
        assert_eq!(module.battery_config.format, "{icon} {percentage}%");
        assert_eq!(module.battery_config.warning_level, 30);
        assert_eq!(module.battery_config.critical_level, 15);
    }

    #[test]
    fn test_battery_config_defaults() {
        let config = BatteryConfig::default();
        assert_eq!(config.format, "{icon} {percentage}%");
        assert_eq!(config.warning_level, 30);
        assert_eq!(config.critical_level, 15);
    }

    #[test]
    fn test_battery_state_initial() {
        let module = BatteryModule::new();
        assert_eq!(module.state, BatteryState::Unknown);
        assert_eq!(module.percentage, 0.0);
    }

    #[test]
    fn test_get_icon_charging() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Charging;
        module.percentage = 50.0;
        assert_eq!(module.get_icon(), "🔌");
    }

    #[test]
    fn test_get_icon_full() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Full;
        module.percentage = 100.0;
        assert_eq!(module.get_icon(), "🔋");
    }

    #[test]
    fn test_get_icon_discharging_high() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 75.0;
        assert_eq!(module.get_icon(), "🔋");
    }

    #[test]
    fn test_get_icon_discharging_medium() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 50.0;
        // At or below 50% threshold shows low battery icon
        assert_eq!(module.get_icon(), "🪫");
    }

    #[test]
    fn test_get_icon_discharging_low() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 25.0;
        assert_eq!(module.get_icon(), "🪫");
    }

    #[test]
    fn test_get_icon_discharging_critical() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 10.0;
        assert_eq!(module.get_icon(), "🪫");
    }

    #[test]
    fn test_get_icon_unknown() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Unknown;
        assert_eq!(module.get_icon(), "❓");
    }

    #[test]
    fn test_format_text_default() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 75.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🔋 75%");
    }

    #[test]
    fn test_format_text_charging() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Charging;
        module.percentage = 50.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🔌 50%");
    }

    #[test]
    fn test_format_text_with_state() {
        let mut module = BatteryModule::new();
        module.battery_config.format = "{state} {percentage}%".to_string();
        module.state = BatteryState::Charging;
        module.percentage = 85.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Charging 85%");
    }

    #[test]
    fn test_format_text_full() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Full;
        module.percentage = 100.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🔋 100%");
    }

    #[test]
    fn test_format_text_rounds_percentage() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 42.567;
        
        let formatted = module.format_text();
        // Below 50% threshold shows low battery icon
        assert_eq!(formatted, "🪫 43%");
    }

    #[test]
    fn test_get_color_normal() {
        let mut module = BatteryModule::new();
        module.percentage = 50.0;
        
        let color = module.get_color();
        assert_eq!(color, module.config.style.color);
    }

    #[test]
    fn test_get_color_warning() {
        let mut module = BatteryModule::new();
        module.percentage = 25.0;
        module.battery_config.warning_level = 30;
        
        let color = module.get_color();
        assert_eq!(color, "#f9e2af"); // Warning - yellow
    }

    #[test]
    fn test_get_color_critical() {
        let mut module = BatteryModule::new();
        module.percentage = 10.0;
        module.battery_config.critical_level = 15;
        
        let color = module.get_color();
        assert_eq!(color, "#f38ba8"); // Critical - red
    }

    #[test]
    fn test_get_color_at_warning_boundary() {
        let mut module = BatteryModule::new();
        module.percentage = 30.0;
        module.battery_config.warning_level = 30;
        
        let color = module.get_color();
        assert_eq!(color, "#f9e2af"); // Warning - yellow (at boundary)
    }

    #[test]
    fn test_get_color_at_critical_boundary() {
        let mut module = BatteryModule::new();
        module.percentage = 15.0;
        module.battery_config.critical_level = 15;
        
        let color = module.get_color();
        assert_eq!(color, "#f38ba8"); // Critical - red (at boundary)
    }

    #[test]
    fn test_get_color_just_above_warning() {
        let mut module = BatteryModule::new();
        module.percentage = 31.0;
        module.battery_config.warning_level = 30;
        
        let color = module.get_color();
        assert_eq!(color, module.config.style.color); // Normal color
    }

    #[test]
    fn test_get_color_custom_warning_level() {
        let mut module = BatteryModule::new();
        module.percentage = 40.0;
        module.battery_config.warning_level = 50;
        module.battery_config.critical_level = 20;
        
        let color = module.get_color();
        assert_eq!(color, "#f9e2af"); // Warning - yellow
    }

    #[test]
    fn test_get_color_custom_critical_level() {
        let mut module = BatteryModule::new();
        module.percentage = 18.0;
        module.battery_config.warning_level = 30;
        module.battery_config.critical_level = 20;
        
        let color = module.get_color();
        assert_eq!(color, "#f38ba8"); // Critical - red
    }

    #[test]
    fn test_update_interval() {
        let module = BatteryModule::new();
        assert_eq!(module.update_interval(), 30);
    }

    #[test]
    fn test_module_config_position() {
        let module = BatteryModule::new();
        assert_eq!(module.position(), Position::Right);
    }

    #[test]
    fn test_module_enabled_by_default() {
        let module = BatteryModule::new();
        assert!(module.config().enabled);
    }

    #[test]
    fn test_module_name() {
        let module = BatteryModule::new();
        assert_eq!(module.name(), "battery");
    }

    #[test]
    fn test_tick_message_updates_status() {
        let mut module = BatteryModule::new();
        
        // Update with Tick message
        let result = module.update(Message::Tick);
        assert!(result.is_none());
        
        // Percentage should be valid (0-100)
        assert!(module.percentage >= 0.0);
        assert!(module.percentage <= 100.0);
        
        // State should be one of the valid states
        assert!(matches!(
            module.state,
            BatteryState::Charging | BatteryState::Discharging | BatteryState::Full | BatteryState::Unknown
        ));
    }

    #[test]
    fn test_non_tick_message_does_not_update() {
        let mut module = BatteryModule::new();
        module.percentage = 75.0;
        module.state = BatteryState::Discharging;
        
        // Update with non-Tick message
        let result = module.update(Message::ModuleMessage {
            module_name: "test".to_string(),
            message: Box::new(crate::module::ModuleMessage::Refresh),
        });
        assert!(result.is_none());
        
        // State should remain unchanged
        assert_eq!(module.percentage, 75.0);
        assert_eq!(module.state, BatteryState::Discharging);
    }

    #[test]
    fn test_init_updates_status() {
        let mut module = BatteryModule::new();
        
        // Initial state should be Unknown with 0%
        assert_eq!(module.state, BatteryState::Unknown);
        assert_eq!(module.percentage, 0.0);
        
        // Call init
        let result = module.init();
        assert!(result.is_none());
        
        // After init, percentage should be valid
        assert!(module.percentage >= 0.0);
        assert!(module.percentage <= 100.0);
    }

    #[test]
    fn test_is_available_returns_bool() {
        // This test verifies the function doesn't panic on systems with or without batteries
        let available = BatteryModule::is_available();
        // Result can be true or false depending on system
        assert!(available == true || available == false);
    }

    #[test]
    fn test_battery_state_equality() {
        assert_eq!(BatteryState::Charging, BatteryState::Charging);
        assert_eq!(BatteryState::Discharging, BatteryState::Discharging);
        assert_eq!(BatteryState::Full, BatteryState::Full);
        assert_eq!(BatteryState::Unknown, BatteryState::Unknown);
        
        assert_ne!(BatteryState::Charging, BatteryState::Discharging);
        assert_ne!(BatteryState::Full, BatteryState::Unknown);
    }

    #[test]
    fn test_format_text_custom_format() {
        let mut module = BatteryModule::new();
        module.battery_config.format = "Battery: {percentage}% ({state})".to_string();
        module.state = BatteryState::Charging;
        module.percentage = 65.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Battery: 65% (Charging)");
    }

    #[test]
    fn test_format_text_icon_only() {
        let mut module = BatteryModule::new();
        module.battery_config.format = "{icon}".to_string();
        module.state = BatteryState::Full;
        module.percentage = 100.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🔋");
    }

    #[test]
    fn test_format_text_percentage_only() {
        let mut module = BatteryModule::new();
        module.battery_config.format = "{percentage}%".to_string();
        module.state = BatteryState::Discharging;
        module.percentage = 42.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "42%");
    }

    #[test]
    fn test_format_text_state_only() {
        let mut module = BatteryModule::new();
        module.battery_config.format = "{state}".to_string();
        module.state = BatteryState::Discharging;
        module.percentage = 50.0;
        
        let formatted = module.format_text();
        assert_eq!(formatted, "Discharging");
    }

    #[test]
    fn test_color_priority_critical_over_warning() {
        let mut module = BatteryModule::new();
        module.percentage = 10.0;
        module.battery_config.warning_level = 30;
        module.battery_config.critical_level = 15;
        
        // Should show critical color, not warning
        let color = module.get_color();
        assert_eq!(color, "#f38ba8"); // Critical - red
    }

    #[test]
    fn test_battery_config_serialization() {
        let config = BatteryConfig {
            format: "{icon} {percentage}%".to_string(),
            warning_level: 25,
            critical_level: 10,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BatteryConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.warning_level, config.warning_level);
        assert_eq!(deserialized.critical_level, config.critical_level);
    }

    #[test]
    fn test_battery_config_partial_deserialization() {
        let json = r#"{"format": "BAT {percentage}%"}"#;
        let config: BatteryConfig = serde_json::from_str(json).unwrap();
        
        assert_eq!(config.format, "BAT {percentage}%");
        assert_eq!(config.warning_level, 30); // Should use default
        assert_eq!(config.critical_level, 15); // Should use default
    }

    #[test]
    fn test_icon_boundary_at_50_percent() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        
        // At exactly 50% - not greater than 50, so shows low icon
        module.percentage = 50.0;
        assert_eq!(module.get_icon(), "🪫");
        
        // Just above 50% - shows normal icon
        module.percentage = 50.1;
        assert_eq!(module.get_icon(), "🔋");
        
        // Just below 50% - shows low icon
        module.percentage = 49.9;
        assert_eq!(module.get_icon(), "🪫");
    }

    #[test]
    fn test_module_style_defaults() {
        let module = BatteryModule::new();
        assert_eq!(module.config.style.padding, 10);
        assert_eq!(module.config.style.margin, 0);
    }

    #[test]
    fn test_zero_percentage() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Discharging;
        module.percentage = 0.0;
        
        let color = module.get_color();
        assert_eq!(color, "#f38ba8"); // Critical - red
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🪫 0%");
    }

    #[test]
    fn test_hundred_percentage() {
        let mut module = BatteryModule::new();
        module.state = BatteryState::Full;
        module.percentage = 100.0;
        
        let color = module.get_color();
        assert_eq!(color, module.config.style.color); // Normal color
        
        let formatted = module.format_text();
        assert_eq!(formatted, "🔋 100%");
    }
}
