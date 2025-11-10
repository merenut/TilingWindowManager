use dioxus::prelude::*;
use futures::StreamExt;
use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{error, info, warn};

mod appbar;
mod config;
mod ipc_client;
mod module;
mod modules;
mod monitor;

use config::{BarConfig, ConfigLoader};
use ipc_client::IpcClient;
use modules::{
    battery::{is_battery_available, Battery, BatteryConfig},
    clock::{Clock, ClockConfig},
    cpu::{Cpu, CpuConfig},
    memory::{Memory, MemoryConfig},
    window_title::{WindowTitle, WindowTitleConfig},
    workspaces::{WorkspaceInfo, Workspaces, WorkspacesConfig},
};

// Global CSS that will be injected
static CSS: &str = include_str!("../assets/status-bar.css");

// Global config storage
static GLOBAL_CONFIG: OnceLock<BarConfig> = OnceLock::new();

// Messages for IPC coroutine
#[derive(Debug, Clone)]
enum IpcMessage {
    SwitchWorkspace(usize),
    RefreshWorkspaces,
    RefreshWindow,
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("tenraku_bar=info")),
        )
        .init();

    info!("Tenraku Status Bar starting...");

    // Load configuration
    let config_loader = ConfigLoader::new()?;
    let config = config_loader.load()?;

    info!("Configuration loaded successfully");
    info!(
        "Height: {}, Position: {:?}",
        config.bar.height, config.bar.position
    );

    // Store config globally
    GLOBAL_CONFIG.set(config.clone()).ok();

    // Get monitors
    let monitors = monitor::enumerate_monitors();
    info!("Found {} monitor(s)", monitors.len());

    // Calculate status bar dimensions
    let (bar_x, bar_y, bar_width, bar_height) = if let Some(monitor_info) = monitors.first() {
        let (x, y, width, _height) = monitor_info.work_area;
        info!("Primary monitor: {}x{} at ({}, {})", width, _height, x, y);

        let bar_height = config.bar.height;
        (x, y, width as u32, bar_height)
    } else {
        // Fallback to reasonable defaults
        (0, 0, 1920, config.bar.height)
    };

    info!(
        "Launching status bar at ({}, {}) with size {}x{}",
        bar_x, bar_y, bar_width, bar_height
    );

    // Configure and launch the Dioxus app
    let window_config = dioxus::desktop::Config::new().with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title("Tenraku Status Bar")
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(false)
            .with_position(dioxus::desktop::LogicalPosition::new(
                bar_x as f64,
                bar_y as f64,
            ))
            .with_inner_size(dioxus::desktop::LogicalSize::new(
                bar_width as f64,
                bar_height as f64,
            ))
            .with_always_on_top(true),
    );

    dioxus::LaunchBuilder::new()
        .with_cfg(window_config)
        .launch(App);

    Ok(())
}

fn App() -> Element {
    // Get config from global storage
    let _config = GLOBAL_CONFIG.get().expect("Config not initialized").clone();

    // Register as AppBar on mount
    use_effect(move || {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

            // Wait a bit for the window to be created
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Find our window by title
            let title: Vec<u16> = "Tenraku Status Bar\0".encode_utf16().collect();

            unsafe {
                let hwnd = FindWindowW(None, windows::core::PCWSTR(title.as_ptr()));

                if hwnd.0 != 0 {
                    info!("Found status bar window, registering as AppBar");

                    // Get monitors for AppBar registration
                    let monitors = monitor::enumerate_monitors();
                    if let Some(monitor_info) = monitors.first() {
                        let (x, y, width, _height) = monitor_info.work_area;
                        let bar_height = 30;

                        // Register as AppBar
                        if let Err(e) =
                            appbar::register_appbar(hwnd, x, y, width as i32, bar_height)
                        {
                            warn!("Failed to register AppBar: {}", e);
                        } else {
                            info!("Successfully registered as AppBar");
                        }
                    }
                } else {
                    warn!("Could not find status bar window by title");
                }
            }
        }
    });

    // CSS hot-reload - track CSS content with a signal
    let mut css_content = use_signal(|| CSS.to_string());

    // Spawn CSS file watcher
    use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        let css_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("status-bar.css");

        info!("Watching CSS file: {:?}", css_path);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Clone path for the file read operation
        let css_path_clone = css_path.clone();

        // Spawn file watcher in blocking thread
        std::thread::spawn(move || {
            let mut watcher =
                notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            let _ = tx.send(());
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                })
                .expect("Failed to create watcher");

            if let Err(e) = watcher.watch(&css_path, RecursiveMode::NonRecursive) {
                error!("Failed to watch CSS file: {:?}", e);
                return;
            }

            // Keep watcher alive
            loop {
                std::thread::park_timeout(std::time::Duration::from_secs(1));
            }
        });

        // Listen for file change events (non-blocking async)
        let mut rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        while let Some(_) = rx.next().await {
            info!("CSS file changed, reloading...");

            match std::fs::read_to_string(&css_path_clone) {
                Ok(new_css) => {
                    css_content.set(new_css);
                    info!("CSS reloaded successfully");
                }
                Err(e) => {
                    error!("Failed to read CSS file: {}", e);
                }
            }
        }
    });

    // Global state for IPC-driven data
    let active_workspace = use_signal(|| 1usize);
    let workspaces = use_signal(|| {
        (1..=10)
            .map(|i| WorkspaceInfo {
                id: i,
                name: i.to_string(),
                window_count: 0,
            })
            .collect::<Vec<_>>()
    });
    let window_title = use_signal(|| String::new());

    // IPC integration - spawn coroutine to handle window manager communication
    let ipc_tx = use_coroutine(|mut rx: UnboundedReceiver<IpcMessage>| async move {
        let client = IpcClient::new();

        // Try to connect to window manager
        match client.connect().await {
            Ok(_) => {
                info!("IPC client connected");

                // Start event listener in background
                let client_clone = client.clone();
                tokio::spawn(async move {
                    if let Err(e) = client_clone.start_event_listener().await {
                        error!("Event listener error: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to connect to window manager: {}. Status bar will display with mock data.", e);
            }
        }

        // Handle commands from UI
        while let Some(msg) = rx.next().await {
            match msg {
                IpcMessage::SwitchWorkspace(id) => {
                    if let Err(e) = client.switch_workspace(id).await {
                        error!("Failed to switch workspace: {}", e);
                    }
                }
                IpcMessage::RefreshWorkspaces => {
                    // Fetch workspace list periodically
                    match client.get_workspaces().await {
                        Ok(_ws_list) => {
                            // TODO: Update workspaces signal
                        }
                        Err(e) => {
                            error!("Failed to get workspaces: {}", e);
                        }
                    }
                }
                IpcMessage::RefreshWindow => {
                    // Fetch active window info
                    match client.get_active_window().await {
                        Ok(Some(_window)) => {
                            // TODO: Update window_title signal
                        }
                        Ok(None) => {
                            // No active window
                        }
                        Err(e) => {
                            error!("Failed to get active window: {}", e);
                        }
                    }
                }
            }
        }
    });

    // Parse module configurations
    let clock_config = ClockConfig::default();
    let cpu_config = CpuConfig::default();
    let memory_config = MemoryConfig::default();
    let battery_config = BatteryConfig::default();
    let workspaces_config = WorkspacesConfig::default();
    let window_title_config = WindowTitleConfig::default();

    // Check if battery is available
    let has_battery = is_battery_available();

    // Workspace click handler
    let on_workspace_click = move |workspace_id: usize| {
        info!("Workspace {} clicked", workspace_id);
        ipc_tx.send(IpcMessage::SwitchWorkspace(workspace_id));
    };

    rsx! {
        style { {css_content()} }
        div { class: "status-bar",
            // Left modules
            div { class: "modules-left",
                Workspaces {
                    config: workspaces_config.clone(),
                    active_workspace,
                    workspaces,
                    on_workspace_click,
                }
            }

            // Center modules
            div { class: "modules-center",
                WindowTitle {
                    config: window_title_config.clone(),
                    window_title,
                }
            }

            // Right modules
            div { class: "modules-right",
                Cpu { config: cpu_config.clone() }
                Memory { config: memory_config.clone() }
                if has_battery {
                    Battery { config: battery_config.clone() }
                }
                Clock { config: clock_config.clone() }
            }
        }
    }
}
