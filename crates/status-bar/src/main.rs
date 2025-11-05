use iced::{Task, Element};
use iced::widget::{container, Row, Space};
use std::time::Duration;
use tracing::{info, error};

use tiling_wm_status_bar::module::{Message, ModuleRegistry, Position};
use tiling_wm_status_bar::config::{BarConfig, ConfigLoader, BarPosition};
use tiling_wm_status_bar::modules;

struct StatusBar {
    /// Module registry
    modules: ModuleRegistry,
    
    /// Configuration
    config: BarConfig,
    
    /// IPC client
    ipc_client: Option<tiling_wm_status_bar::ipc_client::IpcClient>,
}

impl StatusBar {
    fn new() -> (Self, Task<Message>) {
        // Load configuration
        let config_loader = match ConfigLoader::new() {
            Ok(loader) => loader,
            Err(e) => {
                error!("Failed to create config loader: {}", e);
                return (
                    Self {
                        modules: ModuleRegistry::new(),
                        config: BarConfig::default(),
                        ipc_client: None,
                    },
                    Task::none(),
                );
            }
        };
        
        let config = match config_loader.load() {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load configuration: {}", e);
                BarConfig::default()
            }
        };
        
        info!("Status bar starting...");
        info!("Height: {}, Position: {:?}", config.bar.height, config.bar.position);
        
        // Create module registry and load modules
        let mut modules = ModuleRegistry::new();
        
        // Register modules based on configuration
        modules.register(Box::new(modules::workspaces::WorkspacesModule::new()));
        modules.register(Box::new(modules::window_title::WindowTitleModule::new()));
        modules.register(Box::new(modules::clock::ClockModule::new()));
        modules.register(Box::new(modules::cpu::CpuModule::new()));
        modules.register(Box::new(modules::memory::MemoryModule::new()));
        
        // Only add battery module if battery is available
        if modules::battery::BatteryModule::is_available() {
            modules.register(Box::new(modules::battery::BatteryModule::new()));
        }
        
        info!("Loaded {} modules", modules.count());
        
        // Initialize IPC client
        let ipc_client = tiling_wm_status_bar::ipc_client::IpcClient::new();
        
        (
            Self {
                modules,
                config,
                ipc_client: Some(ipc_client),
            },
            Task::none(),
        )
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Update all modules that need periodic updates
                let tasks: Vec<Task<Message>> = self.modules
                    .update_all(Message::Tick);
                
                Task::batch(tasks)
            }
            
            Message::ModuleMessage { module_name, message } => {
                // Route message to specific module
                if let Some(module) = self.modules.get_by_name_mut(&module_name) {
                    if let Some(task) = module.update(Message::ModuleMessage {
                        module_name: module_name.clone(),
                        message,
                    }) {
                        return task;
                    }
                }
                Task::none()
            }
            
            Message::IpcEvent(event) => {
                // Broadcast event to all modules
                let tasks = self.modules.update_all(Message::IpcEvent(event));
                Task::batch(tasks)
            }
            
            Message::SwitchWorkspace(id) => {
                // Send workspace switch command via IPC
                info!("Switching to workspace {}", id);
                // TODO: Implement IPC command sending when IPC client is fully implemented
                Task::none()
            }
            
            Message::ExecuteCommand(cmd) => {
                // Execute arbitrary command via IPC
                info!("Executing command: {}", cmd);
                // TODO: Implement IPC command execution
                Task::none()
            }
        }
    }
    
    fn view(&self) -> Element<'_, Message> {
        // Create rows for each position
        let left_modules = self.create_module_row(Position::Left);
        let center_modules = self.create_module_row(Position::Center);
        let right_modules = self.create_module_row(Position::Right);
        
        // Create main row with spacing
        let main_row = Row::new()
            .push(left_modules)
            .push(Space::with_width(iced::Length::Fill))
            .push(center_modules)
            .push(Space::with_width(iced::Length::Fill))
            .push(right_modules);
        
        // Wrap in container with styling
        let background_color = tiling_wm_status_bar::module::parse_color(&self.config.style.background_color);
        
        container(main_row)
            .width(iced::Length::Fill)
            .height(iced::Length::Fixed(self.config.bar.height as f32))
            .style(move |_theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(background_color)),
                    ..Default::default()
                }
            })
            .into()
    }
    
    fn subscription(&self) -> iced::Subscription<Message> {
        // Subscribe to time ticks every second
        iced::time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    }
    
    #[allow(dead_code)]
    fn title(&self) -> String {
        "Tiling WM Status Bar".to_string()
    }
    
    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
    
    /// Create a row of modules for a specific position
    fn create_module_row(&self, position: Position) -> Row<'_, Message> {
        let mut row = Row::new().spacing(10);
        
        for module in self.modules.get_by_position(position) {
            if module.config().enabled {
                row = row.push(module.view());
            }
        }
        
        row
    }
}

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_status_bar=debug,info")
        .with_target(false)
        .init();
    
    info!("Starting Tiling Window Manager Status Bar");
    
    // Run application using iced::application pattern
    // Window configuration (size, position, always-on-top, etc.) will be implemented
    // in a future task (Task 6.11 - Multi-Monitor Support) using platform-specific code
    iced::application("Tiling WM Status Bar", StatusBar::update, StatusBar::view)
        .subscription(StatusBar::subscription)
        .theme(StatusBar::theme)
        .run_with(|| StatusBar::new())
}
