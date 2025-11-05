//! Memory module - displays memory usage

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use sysinfo::{System, MemoryRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryModule {
    config: ModuleConfig,
    usage_percent: f32,
    used_memory: u64,
    total_memory: u64,
    memory_config: MemoryConfig,
    // Note: We don't store System because it doesn't implement Sync
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
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

impl MemoryModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "memory".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            usage_percent: 0.0,
            used_memory: 0,
            total_memory: 0,
            memory_config: MemoryConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        // Create a temporary System instance to get memory usage
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything())
        );
        system.refresh_memory();
        
        self.used_memory = system.used_memory();
        self.total_memory = system.total_memory();
        
        let total = self.total_memory as f32;
        let used = self.used_memory as f32;
        self.usage_percent = (used / total) * 100.0;
    }
    
    fn format_text(&self) -> String {
        let used_gb = (self.used_memory as f64) / (1024.0 * 1024.0 * 1024.0);
        let total_gb = (self.total_memory as f64) / (1024.0 * 1024.0 * 1024.0);
        
        self.memory_config.format
            .replace("{percentage}", &format!("{:.1}", self.usage_percent))
            .replace("{used}", &format!("{:.1}", used_gb))
            .replace("{total}", &format!("{:.1}", total_gb))
    }
}

impl Module for MemoryModule {
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
        if matches!(message, Message::Tick) {
            self.update_usage();
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
        self.update_usage();
        None
    }
    
    fn update_interval(&self) -> u64 {
        self.memory_config.interval
    }
}

impl Default for MemoryModule {
    fn default() -> Self {
        Self::new()
    }
}
