//! CPU module - displays CPU usage

use dioxus::prelude::*;
use sysinfo::{System, CpuRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default = "default_interval")]
    pub interval: u64,
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

#[component]
pub fn Cpu(config: CpuConfig) -> Element {
    let mut usage = use_signal(|| 0.0f32);
    
    // Fetch CPU usage periodically
    let _ = use_resource(move || {
        let interval = config.interval;
        async move {
            loop {
                let mut system = System::new_with_specifics(
                    RefreshKind::new().with_cpu(CpuRefreshKind::everything())
                );
                system.refresh_cpu();
                let cpu_usage = system.global_cpu_info().cpu_usage();
                usage.set(cpu_usage);
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        }
    });
    
    let formatted_text = config.format.replace("{usage}", &format!("{:.1}", usage()));
    
    let cpu_class = if usage() > 90.0 {
        "module module-cpu critical"
    } else if usage() > 70.0 {
        "module module-cpu warning"
    } else {
        "module module-cpu"
    };
    
    rsx! {
        div { class: "{cpu_class}",
            "{formatted_text}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_replacement() {
        let config = CpuConfig::default();
        let formatted = config.format.replace("{usage}", "42.5");
        assert_eq!(formatted, " 42.5%");
    }
}
