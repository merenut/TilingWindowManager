//! Workspaces module - displays workspace indicators

use crate::module::{Module, Message, ModuleMessage, ModuleConfig, Position, IpcEvent, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::{button, text, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WorkspacesModule {
    config: ModuleConfig,
    workspaces: Vec<WorkspaceInfo>,
    active_workspace: usize,
    workspace_config: WorkspacesConfig,
}

#[derive(Debug, Clone)]
struct WorkspaceInfo {
    id: usize,
    name: String,
    window_count: usize,
    visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspacesConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default)]
    icons: std::collections::HashMap<String, String>,
    
    #[serde(default = "default_active_color")]
    active_color: String,
    
    #[serde(default = "default_inactive_color")]
    inactive_color: String,
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
            icons: std::collections::HashMap::new(),
            active_color: default_active_color(),
            inactive_color: default_inactive_color(),
        }
    }
}

impl WorkspacesModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "workspaces".to_string(),
            position: Position::Left,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        // Create initial workspaces (will be updated from IPC)
        let workspaces = (1..=10)
            .map(|i| WorkspaceInfo {
                id: i,
                name: i.to_string(),
                window_count: 0,
                visible: i == 1,
            })
            .collect();
        
        Self {
            config,
            workspaces,
            active_workspace: 1,
            workspace_config: WorkspacesConfig::default(),
        }
    }
    
    fn format_workspace(&self, ws: &WorkspaceInfo) -> String {
        // Use icon if available, otherwise use name
        let icon = self.workspace_config.icons
            .get(&ws.id.to_string())
            .cloned()
            .unwrap_or_else(|| ws.name.clone());
        
        self.workspace_config.format
            .replace("{icon}", &icon)
            .replace("{name}", &ws.name)
            .replace("{windows}", &ws.window_count.to_string())
    }
    
    fn get_workspace_color(&self, ws: &WorkspaceInfo) -> iced::Color {
        if ws.visible {
            parse_color(&self.workspace_config.active_color)
        } else {
            parse_color(&self.workspace_config.inactive_color)
        }
    }
}

impl Module for WorkspacesModule {
    fn view(&self) -> Element<'_, Message> {
        let mut workspace_row = Row::new().spacing(5);
        
        for ws in &self.workspaces {
            let label = self.format_workspace(ws);
            let color = self.get_workspace_color(ws);
            
            let btn = button(
                text(label)
                    .style(move |_theme| {
                        iced::widget::text::Style {
                            color: Some(color),
                        }
                    })
                    .size(self.config.style.font_size.unwrap_or(12.0))
            )
            .on_press(Message::ModuleMessage {
                module_name: self.name().to_string(),
                message: Box::new(ModuleMessage::WorkspaceClicked(ws.id)),
            })
            .style(|_theme, _status| {
                iced::widget::button::Style {
                    background: None,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            });
            
            workspace_row = workspace_row.push(btn);
        }
        
        styled_container(workspace_row.into(), &self.config.style).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Task<Message>> {
        match message {
            Message::ModuleMessage { message: module_msg, .. } => {
                if let ModuleMessage::WorkspaceClicked(id) = *module_msg {
                    // Send workspace switch command
                    return Some(Task::done(Message::SwitchWorkspace(id)));
                }
            }
            Message::IpcEvent(IpcEvent::WorkspaceChanged { to, .. }) => {
                // Update active workspace
                self.active_workspace = to;
                for ws in &mut self.workspaces {
                    ws.visible = ws.id == to;
                }
            }
            _ => {}
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
}

impl Default for WorkspacesModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_module_creation() {
        let module = WorkspacesModule::new();
        assert_eq!(module.name(), "workspaces");
        assert_eq!(module.position(), Position::Left);
        assert_eq!(module.workspaces.len(), 10);
        assert_eq!(module.active_workspace, 1);
        assert!(module.config().enabled);
    }
    
    #[test]
    fn test_initial_workspace_state() {
        let module = WorkspacesModule::new();
        
        // First workspace should be visible
        assert!(module.workspaces[0].visible);
        assert_eq!(module.workspaces[0].id, 1);
        
        // Other workspaces should not be visible
        for i in 1..10 {
            assert!(!module.workspaces[i].visible);
            assert_eq!(module.workspaces[i].id, i + 1);
        }
    }
    
    #[test]
    fn test_workspace_formatting_default() {
        let module = WorkspacesModule::new();
        let ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 3,
            visible: true,
        };
        
        let formatted = module.format_workspace(&ws);
        // Default format is "{icon}", and without custom icons, should use name
        assert_eq!(formatted, "Main");
    }
    
    #[test]
    fn test_workspace_formatting_with_custom_icon() {
        let mut module = WorkspacesModule::new();
        
        // Add custom icon for workspace 1
        module.workspace_config.icons.insert("1".to_string(), "󰖟".to_string());
        
        let ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 3,
            visible: true,
        };
        
        let formatted = module.format_workspace(&ws);
        assert_eq!(formatted, "󰖟");
    }
    
    #[test]
    fn test_workspace_formatting_with_window_count() {
        let mut module = WorkspacesModule::new();
        module.workspace_config.format = "{name} ({windows})".to_string();
        
        let ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 3,
            visible: true,
        };
        
        let formatted = module.format_workspace(&ws);
        assert_eq!(formatted, "Main (3)");
    }
    
    #[test]
    fn test_workspace_color_active() {
        let module = WorkspacesModule::new();
        let ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 0,
            visible: true,
        };
        
        let color = module.get_workspace_color(&ws);
        let expected_color = parse_color(&module.workspace_config.active_color);
        assert_eq!(color, expected_color);
    }
    
    #[test]
    fn test_workspace_color_inactive() {
        let module = WorkspacesModule::new();
        let ws = WorkspaceInfo {
            id: 2,
            name: "Work".to_string(),
            window_count: 0,
            visible: false,
        };
        
        let color = module.get_workspace_color(&ws);
        let expected_color = parse_color(&module.workspace_config.inactive_color);
        assert_eq!(color, expected_color);
    }
    
    #[test]
    fn test_workspace_color_custom_colors() {
        let mut module = WorkspacesModule::new();
        module.workspace_config.active_color = "#ff0000".to_string();
        module.workspace_config.inactive_color = "#00ff00".to_string();
        
        let active_ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 0,
            visible: true,
        };
        
        let inactive_ws = WorkspaceInfo {
            id: 2,
            name: "Work".to_string(),
            window_count: 0,
            visible: false,
        };
        
        let active_color = module.get_workspace_color(&active_ws);
        let inactive_color = module.get_workspace_color(&inactive_ws);
        
        assert_eq!(active_color, parse_color("#ff0000"));
        assert_eq!(inactive_color, parse_color("#00ff00"));
    }
    
    #[test]
    fn test_ipc_event_workspace_changed() {
        let mut module = WorkspacesModule::new();
        assert_eq!(module.active_workspace, 1);
        assert!(module.workspaces[0].visible);
        
        // Simulate workspace change event
        let event = IpcEvent::WorkspaceChanged { from: 1, to: 3 };
        module.update(Message::IpcEvent(event));
        
        // Verify state updated
        assert_eq!(module.active_workspace, 3);
        assert!(!module.workspaces[0].visible); // Workspace 1 should not be visible
        assert!(module.workspaces[2].visible); // Workspace 3 should be visible
    }
    
    #[test]
    fn test_ipc_event_multiple_workspace_changes() {
        let mut module = WorkspacesModule::new();
        
        // Change from 1 to 5
        module.update(Message::IpcEvent(IpcEvent::WorkspaceChanged { from: 1, to: 5 }));
        assert_eq!(module.active_workspace, 5);
        assert!(module.workspaces[4].visible);
        
        // Change from 5 to 2
        module.update(Message::IpcEvent(IpcEvent::WorkspaceChanged { from: 5, to: 2 }));
        assert_eq!(module.active_workspace, 2);
        assert!(!module.workspaces[4].visible);
        assert!(module.workspaces[1].visible);
    }
    
    #[test]
    fn test_workspace_click_generates_switch_message() {
        let mut module = WorkspacesModule::new();
        
        // Simulate clicking on workspace 5
        let click_message = Message::ModuleMessage {
            module_name: "workspaces".to_string(),
            message: Box::new(ModuleMessage::WorkspaceClicked(5)),
        };
        
        let result = module.update(click_message);
        assert!(result.is_some());
        // The task would produce a SwitchWorkspace message when awaited
    }
    
    #[test]
    fn test_workspace_config_defaults() {
        let config = WorkspacesConfig::default();
        assert_eq!(config.format, "{icon}");
        assert_eq!(config.active_color, "#89b4fa");
        assert_eq!(config.inactive_color, "#585b70");
        assert!(config.icons.is_empty());
    }
    
    #[test]
    fn test_module_config_position() {
        let module = WorkspacesModule::new();
        assert_eq!(module.position(), Position::Left);
        assert_eq!(module.config.position, Position::Left);
    }
    
    #[test]
    fn test_module_enabled_by_default() {
        let module = WorkspacesModule::new();
        assert!(module.config().enabled);
    }
}
