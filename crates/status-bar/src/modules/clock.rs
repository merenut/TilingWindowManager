//! Clock module - displays current time

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ClockModule {
    config: ModuleConfig,
    current_time: String,
    clock_config: ClockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClockConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default)]
    format_alt: Option<String>,
}

fn default_format() -> String {
    "%H:%M:%S".to_string()
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            format_alt: Some("%Y-%m-%d".to_string()),
        }
    }
}

impl ClockModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "clock".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            current_time: String::new(),
            clock_config: ClockConfig::default(),
        }
    }
    
    fn update_time(&mut self) {
        self.current_time = Local::now()
            .format(&self.clock_config.format)
            .to_string();
    }
}

impl Module for ClockModule {
    fn view(&self) -> Element<'_, Message> {
        let color = parse_color(&self.config.style.color);
        
        styled_container(
            text(&self.current_time)
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
            self.update_time();
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
        self.update_time();
        None
    }
    
    fn update_interval(&self) -> u64 {
        1 // Update every second
    }
}

impl Default for ClockModule {
    fn default() -> Self {
        Self::new()
    }
}
