use iced::{Task, Element};
use iced::widget::{container, Row, Space};
use std::time::Duration;
use tracing::{info, error};

use tiling_wm_status_bar::module::{Message, ModuleRegistry, Position};
use tiling_wm_status_bar::config::{BarConfig, ConfigLoader, BarPosition};
use tiling_wm_status_bar::modules;
use tiling_wm_status_bar::monitor;

struct StatusBar {
    /// Module registry
    modules: ModuleRegistry,
    
    /// Configuration
    config: BarConfig,
    
    /// IPC client
    #[allow(dead_code)]
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
        
        // Create module registry and load modules using ModuleFactory
        let mut modules = ModuleRegistry::new();
        
        // Load all modules from configuration
        let module_list = modules::ModuleFactory::create_all_modules(&config);
        for module in module_list {
            modules.register(module);
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

/// Calculate window position and size for a status bar on a given monitor
///
/// # Arguments
/// * `monitor_info` - Information about the monitor to position the bar on
/// * `bar_height` - Height of the status bar in pixels
/// * `position` - Whether the bar should be at the top or bottom
///
/// # Returns
/// A tuple of (x, y, width, height) for the status bar window
fn calculate_bar_geometry(
    monitor_info: &monitor::MonitorInfo,
    bar_height: u32,
    position: BarPosition,
) -> (i32, i32, u32, u32) {
    let (mon_x, mon_y, mon_width, mon_height) = monitor_info.work_area;
    
    let y = match position {
        BarPosition::Top => mon_y,
        BarPosition::Bottom => mon_y + mon_height as i32 - bar_height as i32,
    };
    
    (mon_x, y, mon_width, bar_height)
}

/// Get the monitors that should display status bars based on configuration
///
/// # Arguments
/// * `config` - The bar configuration
///
/// # Returns
/// A vector of MonitorInfo for monitors that should display status bars
fn get_target_monitors(config: &BarConfig) -> Vec<monitor::MonitorInfo> {
    if let Some(monitor_id) = config.bar.monitor {
        // Specific monitor requested
        monitor::get_monitor_by_index(monitor_id)
            .into_iter()
            .collect()
    } else {
        // All monitors
        monitor::enumerate_monitors()
    }
}

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_status_bar=debug,info")
        .with_target(false)
        .init();
    
    info!("Starting Tiling Window Manager Status Bar");
    
    // Load configuration to determine monitor setup
    let config = ConfigLoader::new()
        .and_then(|loader| loader.load())
        .unwrap_or_default();
    
    // Enumerate monitors and log information
    let monitors = monitor::enumerate_monitors();
    info!("Detected {} monitor(s)", monitors.len());
    
    for (i, mon) in monitors.iter().enumerate() {
        info!(
            "Monitor {}: {}x{} at ({}, {}), Primary: {}",
            i,
            mon.work_area.2,
            mon.work_area.3,
            mon.work_area.0,
            mon.work_area.1,
            mon.is_primary
        );
    }
    
    // Determine which monitors should display status bars
    let target_monitors = get_target_monitors(&config);
    info!("Status bar will appear on {} monitor(s)", target_monitors.len());
    
    // Calculate geometry for each target monitor
    for (i, mon) in target_monitors.iter().enumerate() {
        let (x, y, width, height) = calculate_bar_geometry(
            mon,
            config.bar.height,
            config.bar.position,
        );
        info!(
            "Bar {} geometry: {}x{} at ({}, {})",
            i, width, height, x, y
        );
    }
    
    // Run application using iced::application pattern
    // Note: The current iced::application API doesn't provide direct control over
    // window position and size in the builder pattern used here. For full multi-monitor
    // support with per-monitor windows, we would need to use iced::daemon or the
    // lower-level window management APIs. For now, we demonstrate the monitor enumeration
    // and position calculation logic which can be integrated when using those APIs.
    iced::application("Tiling WM Status Bar", StatusBar::update, StatusBar::view)
        .subscription(StatusBar::subscription)
        .theme(StatusBar::theme)
        .run_with(StatusBar::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bar_geometry_top() {
        let monitor = monitor::MonitorInfo {
            #[cfg(target_os = "windows")]
            handle: unsafe { std::mem::zeroed() },
            #[cfg(not(target_os = "windows"))]
            handle: 0,
            work_area: (100, 200, 1920, 1080),
            is_primary: true,
        };
        
        let (x, y, width, height) = calculate_bar_geometry(&monitor, 30, BarPosition::Top);
        
        assert_eq!(x, 100);
        assert_eq!(y, 200); // Top of work area
        assert_eq!(width, 1920);
        assert_eq!(height, 30);
    }

    #[test]
    fn test_calculate_bar_geometry_bottom() {
        let monitor = monitor::MonitorInfo {
            #[cfg(target_os = "windows")]
            handle: unsafe { std::mem::zeroed() },
            #[cfg(not(target_os = "windows"))]
            handle: 0,
            work_area: (100, 200, 1920, 1080),
            is_primary: true,
        };
        
        let (x, y, width, height) = calculate_bar_geometry(&monitor, 30, BarPosition::Bottom);
        
        assert_eq!(x, 100);
        assert_eq!(y, 200 + 1080 - 30); // Bottom of work area
        assert_eq!(width, 1920);
        assert_eq!(height, 30);
    }

    #[test]
    fn test_get_target_monitors_specific() {
        let mut config = BarConfig::default();
        config.bar.monitor = Some(0);
        
        let monitors = get_target_monitors(&config);
        
        // Should return at most 1 monitor (the requested one if it exists)
        assert!(monitors.len() <= 1);
    }

    #[test]
    fn test_get_target_monitors_all() {
        let config = BarConfig::default();
        
        let monitors = get_target_monitors(&config);
        
        // Should return all monitors (at least the mock one on non-Windows)
        #[cfg(not(target_os = "windows"))]
        assert_eq!(monitors.len(), 1);
        // On Windows, just verify the function works (count may vary)
    }

    #[test]
    fn test_calculate_bar_geometry_with_different_heights() {
        let monitor = monitor::MonitorInfo {
            #[cfg(target_os = "windows")]
            handle: unsafe { std::mem::zeroed() },
            #[cfg(not(target_os = "windows"))]
            handle: 0,
            work_area: (0, 0, 1920, 1080),
            is_primary: true,
        };
        
        // Test with different bar heights
        for bar_height in [20, 30, 40, 50] {
            let (_, _, _, height) = calculate_bar_geometry(&monitor, bar_height, BarPosition::Top);
            assert_eq!(height, bar_height);
        }
    }

    #[test]
    fn test_calculate_bar_geometry_preserves_monitor_width() {
        let monitor = monitor::MonitorInfo {
            #[cfg(target_os = "windows")]
            handle: unsafe { std::mem::zeroed() },
            #[cfg(not(target_os = "windows"))]
            handle: 0,
            work_area: (0, 0, 2560, 1440),
            is_primary: false,
        };
        
        let (_, _, width, _) = calculate_bar_geometry(&monitor, 30, BarPosition::Top);
        assert_eq!(width, 2560);
    }
}
