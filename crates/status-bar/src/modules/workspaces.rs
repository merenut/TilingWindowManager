//! Workspaces module - displays workspace indicators

use crate::module::{Module, Message, ModuleMessage, ModuleConfig, Position, IpcEvent, parse_color, styled_container};
use iced::{Task, Element, Length};
use iced::widget::{button, row, text, Row};
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
            Message::ModuleMessage { ref message, .. } => {
                if let ModuleMessage::WorkspaceClicked(id) = **message {
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
