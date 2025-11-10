mod app;
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
use tracing::info;

use commands::CommandExecutor;
use config::ConfigWatcher;
use event_loop::EventLoop;
use keybinds::KeybindManager;
use window_manager::WindowManager;

fn main() -> Result<()> {
    // Initialize logging with command execution tracing
    app::initialize_logging();

    info!("==============================================");
    info!("Tenraku v0.3.0");
    info!("A dynamic tiling window manager for Windows");
    info!("==============================================");

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C signal, initiating shutdown...");
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Load and validate configuration
    let (config_loader, config) = app::load_and_validate_config()?;

    // Initialize window manager with configuration
    info!("Initializing window manager...");
    let mut wm = WindowManager::new();
    wm.initialize()?;

    // Apply initial configuration
    if let Err(e) = wm.update_config(&config) {
        use tracing::warn;
        warn!("Failed to apply configuration: {}", e);
        warn!("Continuing with default settings");
    } else {
        info!("Configuration applied successfully");
    }

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
            use tracing::warn;
            warn!("Failed to register some keybindings: {}", e);
            warn!("Some hotkeys may not be available");
        }
    }

    // Scan and manage existing windows
    info!("Scanning for existing windows...");
    app::scan_and_manage_windows(&mut wm)?;

    // Set up configuration watcher for hot-reload
    info!("Starting configuration watcher...");
    let config_watcher = match ConfigWatcher::new(config_loader.get_config_path().clone()) {
        Ok(watcher) => {
            info!("Configuration hot-reload enabled");
            Some(watcher)
        }
        Err(e) => {
            use tracing::warn;
            warn!("Failed to start configuration watcher: {}", e);
            warn!("Hot-reload will not be available");
            None
        }
    };

    info!("==============================================");
    info!("✓ Tenraku is now running");
    if config_watcher.is_some() {
        info!("✓ Configuration hot-reload enabled");
    }
    info!("  Press Ctrl+C to exit");
    info!("==============================================");

    // Initialize command executor for handling commands
    let executor = CommandExecutor::new();

    // Demonstrate command system (only in debug mode)
    #[cfg(debug_assertions)]
    app::demonstrate_command_system(&mut wm)?;

    // Main event loop with command executor, keybinds, and config hot-reload
    app::run_event_loop(
        &mut wm,
        &mut event_loop,
        &executor,
        &mut keybind_manager,
        &running,
        config_watcher,
        &config_loader,
    )?;

    // Clean shutdown
    info!("Stopping event loop...");
    event_loop.stop()?;
    info!("Tenraku stopped successfully");

    Ok(())
}
