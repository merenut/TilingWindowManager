//! Memory module - displays memory usage

use dioxus::prelude::*;
use sysinfo::{System, MemoryRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default = "default_interval")]
    pub interval: u64,
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

#[component]
pub fn Memory(config: MemoryConfig) -> Element {
    let mut percentage = use_signal(|| 0.0f32);
    let mut used_gb = use_signal(|| 0.0f64);
    let mut total_gb = use_signal(|| 0.0f64);
    
    // Fetch memory usage periodically
    let _ = use_resource(move || {
        let interval = config.interval;
        async move {
            loop {
                let mut system = System::new_with_specifics(
                    RefreshKind::new().with_memory(MemoryRefreshKind::everything())
                );
                system.refresh_memory();
                
                let used = system.used_memory();
                let total = system.total_memory();
                
                percentage.set((used as f32 / total as f32) * 100.0);
                used_gb.set((used as f64) / (1024.0 * 1024.0 * 1024.0));
                total_gb.set((total as f64) / (1024.0 * 1024.0 * 1024.0));
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        }
    });
    
    let formatted_text = config.format
        .replace("{percentage}", &format!("{:.1}", percentage()))
        .replace("{used}", &format!("{:.1}", used_gb()))
        .replace("{total}", &format!("{:.1}", total_gb()));
    
    let memory_class = if percentage() > 90.0 {
        "module module-memory critical"
    } else if percentage() > 70.0 {
        "module module-memory warning"
    } else {
        "module module-memory"
    };
    
    rsx! {
        div { class: "{memory_class}",
            "{formatted_text}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_defaults() {
        let config = MemoryConfig::default();
        assert_eq!(config.format, " {percentage}%");
        assert_eq!(config.interval, 5);
    }

    #[test]
    fn test_memory_config_serialization() {
        let config = MemoryConfig {
            format: "{used}/{total} GB".to_string(),
            interval: 10,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MemoryConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, "{used}/{total} GB");
        assert_eq!(deserialized.interval, 10);
    }

    #[test]
    fn test_format_replacement() {
        let config = MemoryConfig::default();
        let formatted = config.format.replace("{percentage}", "65.5");
        assert_eq!(formatted, " 65.5%");
    }

    #[test]
    fn test_gb_conversion() {
        let bytes: u64 = 8 * 1024 * 1024 * 1024; // 8 GB
        let gb = (bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        assert_eq!(gb, 8.0);
    }
}
