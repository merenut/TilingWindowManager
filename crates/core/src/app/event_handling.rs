//! Event handling logic.
//!
//! This module contains the main event loop and event handlers.

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::commands::CommandExecutor;
use crate::config::{ConfigLoader, ConfigValidator, ConfigWatcher};
use crate::event_loop::{EventLoop, WindowEvent};
use crate::keybinds::KeybindManager;
use crate::window_manager::{WindowManager, WindowState};

#[cfg(target_os = "windows")]
use crate::utils::win32::WindowHandle;

/// Main event loop that processes Windows events and manages windows.
///
/// This is the core loop that:
/// - Processes Windows messages
/// - Polls for window events and hotkeys
/// - Checks for configuration changes and reloads
/// - Uses CommandExecutor for window operations
/// - Logs all significant events and command executions
pub fn run_event_loop(
    wm: &mut WindowManager,
    event_loop: &mut EventLoop,
    executor: &CommandExecutor,
    keybind_manager: &mut KeybindManager,
    running: &Arc<AtomicBool>,
    mut config_watcher: Option<ConfigWatcher>,
    config_loader: &ConfigLoader,
) -> Result<()> {
    debug!("Event loop started");

    while running.load(Ordering::SeqCst) {
        // Check for configuration changes
        if let Some(ref mut watcher) = config_watcher {
            if watcher.check_for_changes() {
                info!("Configuration changed, reloading...");
                match reload_configuration(wm, keybind_manager, config_loader) {
                    Ok(()) => {
                        info!("✓ Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("✗ Failed to reload configuration: {}", e);
                        error!("  Continuing with previous configuration");
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
    let config = config_loader
        .load()
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
    match event {
        WindowEvent::WindowCreated(hwnd) => {
            debug!("EVENT: Window created {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);

            // Check if we should manage this window
            if wm.should_manage_window(&window)? {
                let title = window
                    .get_title()
                    .unwrap_or_else(|_| String::from("<unknown>"));
                info!("Managing new window: {}", title);

                // Use window manager directly for window lifecycle management
                // The command system focuses on user-initiated operations
                wm.manage_window(window)?;
            }
        }

        WindowEvent::WindowDestroyed(hwnd) => {
            debug!("Window destroyed: {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);

            // Try to unmanage - it's okay if it wasn't managed
            if let Err(e) = wm.unmanage_window(&window) {
                debug!("Could not unmanage window: {}", e);
            }
        }

        WindowEvent::WindowShown(hwnd) => {
            debug!("Window shown: {:?}", hwnd);
            // Window became visible - might need to manage it
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                // Try to manage if not already managed
                if let Err(e) = wm.manage_window(window) {
                    debug!("Window already managed: {}", e);
                }
            }
        }

        WindowEvent::WindowHidden(hwnd) => {
            debug!("Window hidden: {:?}", hwnd);
            // Window was hidden - we keep it managed but it won't be visible
        }

        WindowEvent::WindowMoved(hwnd) => {
            debug!("Window moved: {:?}", hwnd);
            // User manually moved a window
            let window = WindowHandle::from_hwnd(hwnd);

            // Check if this is a managed window
            if wm.is_window_managed(&window) {
                // Get the window's state to check if it's tiled
                if let Some(managed_window) = wm.get_window(window.hwnd().0) {
                    if managed_window.state == WindowState::Tiled {
                        debug!("Tiled window was moved manually, retiling workspace");
                        // Retile the workspace to restore the window's position
                        if let Err(e) = wm.tile_workspace(wm.get_active_workspace()) {
                            error!("Failed to retile after window move: {}", e);
                        }
                    }
                }
            }
        }

        WindowEvent::WindowMinimized(hwnd) => {
            debug!("Window minimized: {:?}", hwnd);
        }

        WindowEvent::WindowRestored(hwnd) => {
            debug!("Window restored: {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                // Re-tile the current workspace
                wm.tile_workspace(wm.get_active_workspace())?;
            }
        }

        WindowEvent::WindowFocused(hwnd) => {
            debug!("Window focused: {:?}", hwnd);
            // Track which window has focus
            // Future: Could integrate with FocusManager via command system
            // executor.execute(Command::FocusWindow(hwnd), wm)
        }

        WindowEvent::MonitorChanged => {
            info!("Monitor configuration changed");
            // Refresh monitor information and re-tile all workspaces
            wm.refresh_monitors()?;
            wm.tile_workspace(wm.get_active_workspace())?;
            info!("Monitors refreshed");
        }

        WindowEvent::HotkeyPressed(hotkey_id) => {
            debug!("Hotkey pressed: {}", hotkey_id);

            // Look up the command for this hotkey
            if let Some((command, args)) = keybind_manager.get_command(hotkey_id) {
                debug!("Executing command: {} {:?}", command, args); // Parse and execute the command
                if let Err(e) =
                    crate::app::commands::execute_command_from_string(executor, wm, command, args)
                {
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
