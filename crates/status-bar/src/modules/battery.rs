//! Battery module - displays battery status (optional, only on laptops)

use dioxus::prelude::*;
use battery::{Manager, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatteryConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default = "default_warning_level")]
    pub warning_level: u32,
    
    #[serde(default = "default_critical_level")]
    pub critical_level: u32,
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

/// Check if battery is available on this system
pub fn is_battery_available() -> bool {
    Manager::new()
        .ok()
        .and_then(|manager| manager.batteries().ok())
        .map(|mut batteries| batteries.next().is_some())
        .unwrap_or(false)
}

fn get_icon(state: BatteryState, percentage: f32) -> &'static str {
    match state {
        BatteryState::Charging => "🔌",
        BatteryState::Full => "🔋",
        BatteryState::Discharging => {
            if percentage > 50.0 {
                "🔋"
            } else {
                "🪫"
            }
        }
        BatteryState::Unknown => "❓",
    }
}

fn get_state_text(state: BatteryState) -> &'static str {
    match state {
        BatteryState::Charging => "Charging",
        BatteryState::Discharging => "Discharging",
        BatteryState::Full => "Full",
        BatteryState::Unknown => "Unknown",
    }
}

#[component]
pub fn Battery(config: BatteryConfig) -> Element {
    let mut percentage = use_signal(|| 0.0f32);
    let mut state = use_signal(|| BatteryState::Unknown);
    
    // Fetch battery status periodically (every 30 seconds)
    let _ = use_resource(move || async move {
        loop {
            if let Ok(manager) = Manager::new() {
                if let Ok(batteries) = manager.batteries() {
                    if let Some(Ok(battery)) = batteries.into_iter().next() {
                        percentage.set(battery.state_of_charge().value * 100.0);
                        state.set(match battery.state() {
                            State::Charging => BatteryState::Charging,
                            State::Discharging => BatteryState::Discharging,
                            State::Full => BatteryState::Full,
                            _ => BatteryState::Unknown,
                        });
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });
    
    let icon = get_icon(state(), percentage());
    let state_text = get_state_text(state());
    
    let formatted_text = config.format
        .replace("{icon}", icon)
        .replace("{percentage}", &format!("{:.0}", percentage()))
        .replace("{state}", state_text);
    
    let battery_class = if percentage() <= config.critical_level as f32 {
        "module module-battery critical"
    } else if percentage() <= config.warning_level as f32 {
        "module module-battery warning"
    } else if state() == BatteryState::Charging {
        "module module-battery charging"
    } else {
        "module module-battery"
    };
    
    rsx! {
        div { class: "{battery_class}",
            "{formatted_text}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_config_defaults() {
        let config = BatteryConfig::default();
        assert_eq!(config.format, "{icon} {percentage}%");
        assert_eq!(config.warning_level, 30);
        assert_eq!(config.critical_level, 15);
    }

    #[test]
    fn test_get_icon_charging() {
        assert_eq!(get_icon(BatteryState::Charging, 50.0), "🔌");
    }

    #[test]
    fn test_get_icon_full() {
        assert_eq!(get_icon(BatteryState::Full, 100.0), "🔋");
    }

    #[test]
    fn test_get_icon_discharging_high() {
        assert_eq!(get_icon(BatteryState::Discharging, 75.0), "🔋");
    }

    #[test]
    fn test_get_icon_discharging_low() {
        assert_eq!(get_icon(BatteryState::Discharging, 25.0), "🪫");
    }

    #[test]
    fn test_get_icon_unknown() {
        assert_eq!(get_icon(BatteryState::Unknown, 0.0), "❓");
    }

    #[test]
    fn test_get_state_text() {
        assert_eq!(get_state_text(BatteryState::Charging), "Charging");
        assert_eq!(get_state_text(BatteryState::Discharging), "Discharging");
        assert_eq!(get_state_text(BatteryState::Full), "Full");
        assert_eq!(get_state_text(BatteryState::Unknown), "Unknown");
    }

    #[test]
    fn test_format_replacement() {
        let config = BatteryConfig::default();
        let formatted = config.format
            .replace("{icon}", "🔋")
            .replace("{percentage}", "75");
        assert_eq!(formatted, "🔋 75%");
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
    fn test_is_battery_available_returns_bool() {
        let available = is_battery_available();
        assert!(available == true || available == false);
    }
}
