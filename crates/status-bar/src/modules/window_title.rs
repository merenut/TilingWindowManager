//! Window title module - displays the active window's title

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowTitleConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default = "default_max_length")]
    pub max_length: usize,
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

fn truncate_title(title: &str, max_length: usize) -> String {
    let char_count = title.chars().count();
    if char_count > max_length {
        let truncate_at = title
            .char_indices()
            .nth(max_length - 3)
            .map(|(idx, _)| idx)
            .unwrap_or(title.len());
        format!("{}...", &title[..truncate_at])
    } else {
        title.to_string()
    }
}

#[component]
pub fn WindowTitle(
    config: WindowTitleConfig,
    window_title: Signal<String>,
) -> Element {
    let truncated = truncate_title(&window_title(), config.max_length);
    let formatted_text = config.format.replace("{title}", &truncated);
    
    let class_name = if window_title().is_empty() {
        "module module-window-title empty"
    } else {
        "module module-window-title"
    };
    
    rsx! {
        div { class: "{class_name}",
            "{formatted_text}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            max_length: 100,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WindowTitleConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.max_length, 100);
    }

    #[test]
    fn test_truncate_title_short() {
        let title = "Short Title";
        let truncated = truncate_title(title, 50);
        assert_eq!(truncated, "Short Title");
    }

    #[test]
    fn test_truncate_title_long() {
        let title = "This is a very long window title that should be truncated";
        let truncated = truncate_title(title, 20);
        assert_eq!(truncated.len(), 20);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_title_exact_length() {
        let title = "Exactly fifty characters long title for testing it";
        let truncated = truncate_title(title, 50);
        assert_eq!(truncated, title);
    }

    #[test]
    fn test_truncate_title_unicode() {
        let title = "Unicode 🚀 characters 🎨 in title";
        let truncated = truncate_title(title, 15);
        assert!(truncated.len() <= 18); // Approximate due to Unicode
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_format_replacement() {
        let config = WindowTitleConfig::default();
        let formatted = config.format.replace("{title}", "Test Window");
        assert_eq!(formatted, "Test Window");
    }
}
