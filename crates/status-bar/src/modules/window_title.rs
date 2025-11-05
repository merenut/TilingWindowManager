//! Window title module - displays the active window's title

use crate::module::{Module, Message, ModuleConfig, Position, IpcEvent, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WindowTitleModule {
    config: ModuleConfig,
    window_title: String,
    window_config: WindowTitleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowTitleConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_max_length")]
    max_length: usize,
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

impl WindowTitleModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "window-title".to_string(),
            position: Position::Center,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            window_title: String::new(),
            window_config: WindowTitleConfig::default(),
        }
    }
    
    fn truncate_title(&self, title: &str) -> String {
        if title.len() > self.window_config.max_length {
            format!("{}...", &title[..self.window_config.max_length - 3])
        } else {
            title.to_string()
        }
    }
    
    fn format_text(&self) -> String {
        let truncated = self.truncate_title(&self.window_title);
        self.window_config.format
            .replace("{title}", &truncated)
    }
}

impl Module for WindowTitleModule {
    fn view(&self) -> Element<'_, Message> {
        let color = parse_color(&self.config.style.color);
        
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
        match message {
            Message::IpcEvent(IpcEvent::WindowFocused { title, .. }) => {
                self.window_title = title;
            }
            Message::IpcEvent(IpcEvent::WindowClosed { .. }) => {
                // Clear title if no window focused
                self.window_title.clear();
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

impl Default for WindowTitleModule {
    fn default() -> Self {
        Self::new()
    }
}
