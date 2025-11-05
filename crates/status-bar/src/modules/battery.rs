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
