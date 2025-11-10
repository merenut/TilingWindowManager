//! Clock module - displays current time

use dioxus::prelude::*;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClockConfig {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "%H:%M:%S".to_string()
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
        }
    }
}

#[component]
pub fn Clock(config: ClockConfig) -> Element {
    let time = use_signal(|| String::new());
    
    // Update time every second
    use_effect(move || {
        let mut time_mut = time.clone();
        let format = config.format.clone();
        spawn(async move {
            loop {
                let formatted = Local::now().format(&format).to_string();
                time_mut.set(formatted);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    });
    
    rsx! {
        div { class: "module module-clock",
            "{time}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        let config = ClockConfig::default();
        assert_eq!(config.format, "%H:%M:%S");
    }

    #[test]
    fn test_clock_config_serialization() {
        let config = ClockConfig {
            format: "%Y-%m-%d".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ClockConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.format, "%Y-%m-%d");
    }

    #[test]
    fn test_format_time() {
        let config = ClockConfig {
            format: "%H:%M:%S".to_string(),
        };
        
        let formatted = Local::now().format(&config.format).to_string();
        assert!(!formatted.is_empty());
        
        let parts: Vec<&str> = formatted.split(':').collect();
        assert_eq!(parts.len(), 3);
    }
}
