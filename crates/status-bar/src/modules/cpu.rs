//! CPU module - displays CPU usage

use crate::module::{Module, Message, ModuleConfig, Position, parse_color, styled_container};
use iced::{Task, Element};
use iced::widget::text;
use sysinfo::{System, CpuRefreshKind, RefreshKind};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CpuModule {
    config: ModuleConfig,
    usage: f32,
    cpu_config: CpuConfig,
    // Note: We don't store System because it doesn't implement Sync
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpuConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
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

impl CpuModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "cpu".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            usage: 0.0,
            cpu_config: CpuConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        // Create a temporary System instance to get CPU usage
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything())
        );
        system.refresh_cpu();
        self.usage = system.global_cpu_info().cpu_usage();
    }
    
    fn format_text(&self) -> String {
        self.cpu_config.format
            .replace("{usage}", &format!("{:.1}", self.usage))
    }
}

impl Module for CpuModule {
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
        self.cpu_config.interval
    }
}

impl Default for CpuModule {
    fn default() -> Self {
        Self::new()
    }
}
