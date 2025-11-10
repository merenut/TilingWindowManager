//! Workspaces module - displays workspace indicators

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspacesConfig {
    #[serde(default = "default_format")]
    pub format: String,
    
    #[serde(default)]
    pub icons: HashMap<String, String>,
    
    #[serde(default = "default_active_color")]
    pub active_color: String,
    
    #[serde(default = "default_inactive_color")]
    pub inactive_color: String,
}

fn default_format() -> String {
    "{icon}".to_string()
}

fn default_active_color() -> String {
    "#89b4fa".to_string()
}

fn default_inactive_color() -> String {
    "#585b70".to_string()
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            icons: HashMap::new(),
            active_color: default_active_color(),
            inactive_color: default_inactive_color(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceInfo {
    pub id: usize,
    pub name: String,
    pub window_count: usize,
}

#[component]
pub fn Workspaces(
    config: WorkspacesConfig,
    active_workspace: Signal<usize>,
    workspaces: Signal<Vec<WorkspaceInfo>>,
    on_workspace_click: EventHandler<usize>,
) -> Element {
    rsx! {
        div { class: "module module-workspaces",
            for ws in workspaces().iter() {
                {
                    let is_active = ws.id == active_workspace();
                    let ws_id = ws.id;
                    let icon = config.icons
                        .get(&ws.id.to_string())
                        .cloned()
                        .unwrap_or_else(|| ws.name.clone());
                    
                    let class_name = if is_active {
                        "workspace-button workspace-button-active"
                    } else if ws.window_count > 0 {
                        "workspace-button workspace-button-occupied"
                    } else {
                        "workspace-button"
                    };
                    
                    rsx! {
                        button {
                            key: "{ws_id}",
                            class: "{class_name}",
                            onclick: move |_| on_workspace_click.call(ws_id),
                            "{icon}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspaces_config_defaults() {
        let config = WorkspacesConfig::default();
        assert_eq!(config.format, "{icon}");
        assert_eq!(config.active_color, "#89b4fa");
        assert_eq!(config.inactive_color, "#585b70");
        assert!(config.icons.is_empty());
    }

    #[test]
    fn test_workspaces_config_serialization() {
        let mut icons = HashMap::new();
        icons.insert("1".to_string(), "".to_string());
        
        let config = WorkspacesConfig {
            format: "{icon}".to_string(),
            icons,
            active_color: "#89b4fa".to_string(),
            inactive_color: "#585b70".to_string(),
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkspacesConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.active_color, config.active_color);
        assert_eq!(deserialized.inactive_color, config.inactive_color);
    }

    #[test]
    fn test_workspace_info_creation() {
        let ws = WorkspaceInfo {
            id: 1,
            name: "1".to_string(),
            window_count: 3,
        };
        
        assert_eq!(ws.id, 1);
        assert_eq!(ws.name, "1");
        assert_eq!(ws.window_count, 3);
    }
}
