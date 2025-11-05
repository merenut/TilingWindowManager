// Note: During Phase 2, some components are not yet fully integrated.
// Dead code warnings are expected and will be resolved in later phases.
#![allow(dead_code)]
#![allow(unused_imports)]

mod commands;
mod config;
mod event_loop;
mod keybinds;
mod rules;
mod utils;
mod window_manager;
mod workspace;

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use commands::{Command, CommandExecutor};
use config::{ConfigLoader, ConfigValidator, ConfigWatcher};
use event_loop::{EventLoop, WindowEvent};
use keybinds::KeybindManager;
use window_manager::WindowManager;

#[cfg(target_os = "windows")]
use utils::win32::WindowHandle;

fn main() -> Result<()> {
    // Initialize logging with command execution tracing
    initialize_logging();

    info!("==============================================");
    info!("Starting Tiling Window Manager v0.3.0");
    info!("Phase 4: Configuration Hot-Reload Active");
    info!("==============================================");

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C signal, initiating shutdown...");
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Load configuration
    info!("Loading configuration...");
    let config_loader = ConfigLoader::new()?;
    let config = config_loader.load()?;
    info!("Configuration loaded from: {:?}", config_loader.get_config_path());

    // Validate configuration
    if let Err(e) = ConfigValidator::validate(&config) {
        error!("Configuration validation failed: {}", e);
        return Err(e);
    }
    info!("Configuration validated successfully");

    // Initialize window manager with configuration
    info!("Initializing window manager...");
    let mut wm = WindowManager::new();
    wm.initialize()?;
    
    // Apply initial configuration
    if let Err(e) = wm.update_config(&config) {
        warn!("Failed to apply initial configuration: {}", e);
    }
    info!("Window manager initialized successfully");

    // Set up event loop
    info!("Starting event loop...");
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    info!("Event loop started successfully");

    // Set up keybind manager
    info!("Registering keybindings...");
    let mut keybind_manager = KeybindManager::new();
    match keybind_manager.register_keybinds(config.keybinds.clone()) {
        Ok(()) => info!("Keybindings registered successfully"),
        Err(e) => {
            warn!("Failed to register some keybindings: {}", e);
            warn!("Some hotkeys may not be available");
        }
    }

    // Scan and manage existing windows
    info!("Scanning for existing windows...");
    scan_and_manage_windows(&mut wm)?;

    // Set up configuration watcher for hot-reload
    info!("Starting configuration watcher...");
    let config_watcher = match ConfigWatcher::new(config_loader.get_config_path().clone()) {
        Ok(watcher) => {
            info!("Configuration hot-reload enabled");
            Some(watcher)
        }
        Err(e) => {
            warn!("Failed to start configuration watcher: {}", e);
            warn!("Hot-reload will not be available");
            None
        }
    };

    info!("==============================================");
    info!("Tiling Window Manager is now running");
    info!("Press Ctrl+C to exit");
    if config_watcher.is_some() {
        info!("Configuration hot-reload is active");
    }
    info!("==============================================");

    // Initialize command executor for handling commands
    // The executor is used to perform all window operations via the command system
    let executor = CommandExecutor::new();
    info!("Command executor initialized and ready");

    // Demonstrate command system integration
    demonstrate_command_system(&executor, &mut wm)?;

    info!("Starting main event loop with command system integration...");

    // Main event loop with command executor, keybinds, and config hot-reload
    run_event_loop(&mut wm, &mut event_loop, &executor, &mut keybind_manager, &running, config_watcher, &config_loader)?;

    // Clean shutdown
    info!("Stopping event loop...");
    event_loop.stop()?;
    info!("Tiling Window Manager stopped successfully");

    Ok(())
}

/// Initialize logging with appropriate levels and formatting.
///
/// Logging includes command execution traces and event processing information.
fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_core=debug,info")
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();
}

/// Demonstrate the command system integration.
///
/// This function shows examples of how commands are executed via the CommandExecutor.
/// In future phases, these commands will be triggered by hotkey bindings or IPC.
fn demonstrate_command_system(_executor: &CommandExecutor, wm: &mut WindowManager) -> Result<()> {
    info!("==============================================");
    info!("Command System Integration Examples:");
    info!("==============================================");

    // Example 1: Layout switching commands
    info!("Available layout commands:");
    info!("  - Command::SetLayoutDwindle  (smart tiling layout)");
    info!("  - Command::SetLayoutMaster   (master-stack layout)");

    // Example 2: Window manipulation commands
    info!("Available window commands:");
    info!("  - Command::ToggleFloating    (toggle tiled/floating)");
    info!("  - Command::ToggleFullscreen  (toggle fullscreen)");
    info!("  - Command::CloseActiveWindow (close focused window)");
    info!("  - Command::MinimizeActive    (minimize focused window)");

    // Example 3: Focus navigation commands
    info!("Available focus commands:");
    info!("  - Command::FocusLeft/Right/Up/Down");
    info!("  - Command::FocusPrevious/Next (Alt-Tab style)");

    // Example 4: Master layout adjustment commands
    info!("Available master layout commands:");
    info!("  - Command::IncreaseMasterCount/DecreaseMasterCount");
    info!("  - Command::IncreaseMasterFactor/DecreaseMasterFactor");

    // Example 5: Workspace commands
    info!("Available workspace commands:");
    info!("  - Command::SwitchWorkspace(id)");
    info!("  - Command::MoveToWorkspace(id)");
    info!("  - Command::MoveToWorkspaceAndFollow(id)");

    info!("==============================================");
    info!("Note: Commands will be bound to hotkeys in Phase 3");
    info!("For now, they can be executed programmatically");
    info!("==============================================");

    // Demonstrate actual command execution with logging
    info!("Demonstrating command execution with current layout...");
    let current_layout = wm.get_current_layout();
    info!("Current layout: {:?}", current_layout);

    // All command executions are logged by the CommandExecutor
    // See commands.rs for detailed execution logging

    Ok(())
}

/// Scan for existing windows and add them to management.
fn scan_and_manage_windows(wm: &mut WindowManager) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use tracing::{debug, warn};
        use utils::win32;

        match win32::enumerate_app_windows() {
            Ok(windows) => {
                info!("Found {} existing windows", windows.len());

                let mut managed_count = 0;
                for window in windows {
                    if wm.should_manage_window(&window).unwrap_or(false) {
                        let title = window
                            .get_title()
                            .unwrap_or_else(|_| String::from("<unknown>"));
                        debug!("Managing existing window: {}", title);

                        if let Err(e) = wm.manage_window(window) {
                            warn!("Failed to manage window '{}': {}", title, e);
                        } else {
                            managed_count += 1;
                        }
                    }
                }

                info!("Managing {} windows", managed_count);
            }
            Err(e) => {
                warn!("Failed to enumerate existing windows: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = wm; // Suppress unused variable warning
        info!("Window enumeration is only supported on Windows");
    }

    Ok(())
}

/// Main event loop that processes Windows events and manages windows.
///
/// This is the core loop that:
/// - Processes Windows messages
/// - Polls for window events and hotkeys
/// - Checks for configuration changes and reloads
/// - Uses CommandExecutor for window operations
/// - Logs all significant events and command executions
fn run_event_loop(
    wm: &mut WindowManager,
    event_loop: &mut EventLoop,
    executor: &CommandExecutor,
    keybind_manager: &mut KeybindManager,
    running: &Arc<AtomicBool>,
    mut config_watcher: Option<ConfigWatcher>,
    config_loader: &ConfigLoader,
) -> Result<()> {
    info!("Event loop running - processing window events via command system");

    while running.load(Ordering::SeqCst) {
        // Check for configuration changes
        if let Some(ref mut watcher) = config_watcher {
            if watcher.check_for_changes() {
                info!("Configuration change detected, reloading...");
                match reload_configuration(wm, keybind_manager, config_loader) {
                    Ok(()) => {
                        info!("✓ Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("✗ Failed to reload configuration: {}", e);
                        error!("Continuing with previous configuration");
                    }
                }
            }
        }

        // Process Windows messages (includes hotkeys)
        if let Err(e) = event_loop.process_messages() {
            error!("Error processing messages: {}", e);
        }

        // Poll for window events and handle them via command system
        for event in event_loop.poll_events() {
            if let Err(e) = handle_event(wm, executor, keybind_manager, event) {
                error!("Error handling event: {}", e);
            }
        }

        // Small sleep to prevent 100% CPU usage
        std::thread::sleep(Duration::from_millis(50));
    }

    info!("Event loop shutting down gracefully");
    Ok(())
}

/// Reload configuration from disk and apply to window manager
///
/// This function:
/// 1. Loads the new configuration from disk
/// 2. Validates the configuration
/// 3. Applies it to the window manager
/// 4. Updates rules and keybindings
///
/// If any step fails, the previous configuration remains active.
fn reload_configuration(
    wm: &mut WindowManager,
    keybind_manager: &mut KeybindManager,
    config_loader: &ConfigLoader,
) -> Result<()> {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Load new configuration
    let config = config_loader.load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    
    // Validate configuration
    ConfigValidator::validate(&config)
        .map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;
    
    // Apply to window manager
    wm.update_config(&config)
        .map_err(|e| anyhow::anyhow!("Failed to apply configuration: {}", e))?;
    
    // Update keybindings
    keybind_manager
        .register_keybinds(config.keybinds.clone())
        .map_err(|e| anyhow::anyhow!("Failed to register keybindings: {}", e))?;
    
    let elapsed = start.elapsed();
    info!("Configuration reload completed in {:?}", elapsed);
    
    // Check if reload meets performance target
    if elapsed > Duration::from_millis(100) {
        warn!(
            "Configuration reload took {:?}, exceeds 100ms target",
            elapsed
        );
    }
    
    Ok(())
}

/// Handle an event (window or hotkey) by dispatching to the appropriate action.
///
/// This function serves as the bridge between raw events and the command system.
/// All operations go through the CommandExecutor for consistent handling and logging.
#[cfg(target_os = "windows")]
fn handle_event(
    wm: &mut WindowManager,
    executor: &CommandExecutor,
    keybind_manager: &KeybindManager,
    event: WindowEvent,
) -> Result<()> {
    use tracing::debug;

    match event {
        WindowEvent::WindowCreated(hwnd) => {
            debug!("EVENT: Window created {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);

            // Check if we should manage this window
            if wm.should_manage_window(&window)? {
                let title = window
                    .get_title()
                    .unwrap_or_else(|_| String::from("<unknown>"));
                info!("EVENT: Managing new window: {} (HWND: {:?})", title, hwnd);

                // Use window manager directly for window lifecycle management
                // The command system focuses on user-initiated operations
                wm.manage_window(window)?;
                info!("RESULT: Window successfully added to workspace");
            }
        }

        WindowEvent::WindowDestroyed(hwnd) => {
            debug!("EVENT: Window destroyed {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);

            // Try to unmanage - it's okay if it wasn't managed
            if let Err(e) = wm.unmanage_window(&window) {
                debug!("Could not unmanage window {:?}: {}", hwnd, e);
            } else {
                info!("RESULT: Window removed from management: {:?}", hwnd);
            }
        }

        WindowEvent::WindowShown(hwnd) => {
            debug!("EVENT: Window shown {:?}", hwnd);
            // Window became visible - might need to manage it
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                // Try to manage if not already managed
                if let Err(e) = wm.manage_window(window) {
                    debug!("Window already managed or error: {}", e);
                } else {
                    info!("RESULT: Window shown and added to management");
                }
            }
        }

        WindowEvent::WindowHidden(hwnd) => {
            debug!("EVENT: Window hidden {:?}", hwnd);
            // Window was hidden - we keep it managed but it won't be visible
            // No command needed - state tracked automatically
        }

        WindowEvent::WindowMoved(hwnd) => {
            debug!("EVENT: Window moved {:?}", hwnd);
            // User manually moved a window
            // Future: Could use executor.execute(Command::ToggleFloating, wm) here
            // to automatically mark manually moved windows as floating
        }

        WindowEvent::WindowMinimized(hwnd) => {
            debug!("EVENT: Window minimized {:?}", hwnd);
            // Window was minimized - CommandExecutor.minimize_active could be used
            // for programmatic minimize, but this is user-initiated via OS
            info!("RESULT: Window minimized by user");
        }

        WindowEvent::WindowRestored(hwnd) => {
            debug!("EVENT: Window restored {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                info!("RESULT: Window restored - retiling workspace");
                // Re-tile the current workspace
                wm.tile_workspace(wm.get_active_workspace())?;
            }
        }

        WindowEvent::WindowFocused(hwnd) => {
            debug!("EVENT: Window focused {:?}", hwnd);
            // Track which window has focus
            // Future: Could integrate with FocusManager via command system
            // executor.execute(Command::FocusWindow(hwnd), wm)
        }

        WindowEvent::MonitorChanged => {
            info!("EVENT: Monitor configuration changed");
            // Refresh monitor information and re-tile all workspaces
            wm.refresh_monitors()?;
            wm.tile_workspace(wm.get_active_workspace())?;
            info!("RESULT: Monitors refreshed and workspace retiled");
        }

        WindowEvent::HotkeyPressed(hotkey_id) => {
            debug!("EVENT: Hotkey pressed (ID: {})", hotkey_id);
            
            // Look up the command for this hotkey
            if let Some((command, args)) = keybind_manager.get_command(hotkey_id) {
                info!("HOTKEY: Executing command '{}' with args {:?}", command, args);
                
                // Parse and execute the command
                if let Err(e) = execute_command_from_string(executor, wm, command, args) {
                    error!("Failed to execute hotkey command '{}': {}", command, e);
                }
            } else {
                warn!("Received hotkey event for unknown ID: {}", hotkey_id);
            }
        }
    }

    Ok(())
}

/// Handle an event by dispatching to the appropriate action (stub for non-Windows).
#[cfg(not(target_os = "windows"))]
fn handle_event(
    wm: &mut WindowManager,
    _executor: &CommandExecutor,
    _keybind_manager: &KeybindManager,
    event: WindowEvent,
) -> Result<()> {
    match event {
        WindowEvent::MonitorChanged => {
            info!("EVENT: Monitor configuration changed");
            // Refresh monitor information and re-tile all workspaces
            wm.refresh_monitors()?;
            wm.tile_workspace(wm.get_active_workspace())?;
            info!("RESULT: Monitors refreshed and workspace retiled");
        }
        WindowEvent::HotkeyPressed(_) => {
            // Hotkeys not supported on non-Windows platforms
            warn!("Hotkey events are only supported on Windows");
        }
    }

    Ok(())
}

/// Execute a command from a string representation.
///
/// This function parses a command string and optional arguments, then executes
/// the corresponding command through the CommandExecutor.
fn execute_command_from_string(
    executor: &CommandExecutor,
    wm: &mut WindowManager,
    command_str: &str,
    _args: &[String],
) -> Result<()> {
    use tracing::debug;
    
    // Parse command string to Command enum
    let command = match command_str {
        // Window commands
        "close" => Command::CloseActiveWindow,
        "toggle-floating" => Command::ToggleFloating,
        "toggle-fullscreen" => Command::ToggleFullscreen,
        "minimize" => Command::MinimizeActive,
        "restore" => Command::RestoreActive,
        
        // Focus commands
        "focus-left" => Command::FocusLeft,
        "focus-right" => Command::FocusRight,
        "focus-up" => Command::FocusUp,
        "focus-down" => Command::FocusDown,
        "focus-previous" => Command::FocusPrevious,
        "focus-next" => Command::FocusNext,
        
        // Move commands
        "move-left" => Command::MoveWindowLeft,
        "move-right" => Command::MoveWindowRight,
        "move-up" => Command::MoveWindowUp,
        "move-down" => Command::MoveWindowDown,
        "swap-master" => Command::SwapWithMaster,
        
        // Layout commands
        "layout-dwindle" => Command::SetLayoutDwindle,
        "layout-master" => Command::SetLayoutMaster,
        "increase-master" => Command::IncreaseMasterCount,
        "decrease-master" => Command::DecreaseMasterCount,
        "increase-master-factor" => Command::IncreaseMasterFactor,
        "decrease-master-factor" => Command::DecreaseMasterFactor,
        
        // Workspace commands
        "workspace-1" => Command::SwitchWorkspace(1),
        "workspace-2" => Command::SwitchWorkspace(2),
        "workspace-3" => Command::SwitchWorkspace(3),
        "workspace-4" => Command::SwitchWorkspace(4),
        "workspace-5" => Command::SwitchWorkspace(5),
        "workspace-6" => Command::SwitchWorkspace(6),
        "workspace-7" => Command::SwitchWorkspace(7),
        "workspace-8" => Command::SwitchWorkspace(8),
        "workspace-9" => Command::SwitchWorkspace(9),
        "workspace-10" => Command::SwitchWorkspace(10),
        
        "move-to-workspace-1" => Command::MoveToWorkspace(1),
        "move-to-workspace-2" => Command::MoveToWorkspace(2),
        "move-to-workspace-3" => Command::MoveToWorkspace(3),
        "move-to-workspace-4" => Command::MoveToWorkspace(4),
        "move-to-workspace-5" => Command::MoveToWorkspace(5),
        
        // System commands
        "reload-config" => Command::Reload,
        "exit" | "quit" => Command::Quit,
        
        // Unknown command
        _ => {
            warn!("Unknown command: {}", command_str);
            return Ok(());
        }
    };
    
    debug!("Parsed command: {:?}", command);
    
    // Execute the command
    executor.execute(command, wm)?;
    
    Ok(())
}
