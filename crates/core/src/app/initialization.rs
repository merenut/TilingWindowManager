//! Application initialization logic.
//!
//! This module contains functions for setting up logging, loading configuration,
//! initializing components, and scanning for existing windows.

use anyhow::Result;
use tracing::{info, warn};

use crate::config::{ConfigLoader, ConfigValidator};
use crate::window_manager::WindowManager;

/// Initialize logging with appropriate levels and formatting.
///
/// Logging includes command execution traces and event processing information.
/// Log level can be controlled via RUST_LOG environment variable.
pub fn initialize_logging() {
    use tracing_subscriber::EnvFilter;

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("tenraku_core=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();
}

/// Scan for existing windows and add them to management.
pub fn scan_and_manage_windows(wm: &mut WindowManager) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use crate::utils::win32;
        use tracing::{debug, warn};

        match win32::enumerate_app_windows() {
            Ok(windows) => {
                info!("Found {} existing windows", windows.len());

                let mut managed_count = 0;
                let mut skipped_count = 0;

                for window in windows {
                    match wm.should_manage_window(&window) {
                        Ok(true) => {
                            let title = window
                                .get_title()
                                .unwrap_or_else(|_| String::from("<unknown>"));
                            debug!("Managing existing window: {}", title);

                            if let Err(e) = wm.manage_window(window) {
                                warn!("Failed to manage window '{}': {}", title, e);
                                skipped_count += 1;
                            } else {
                                managed_count += 1;
                            }
                        }
                        Ok(false) => {
                            skipped_count += 1;
                        }
                        Err(e) => {
                            debug!("Error checking window: {}", e);
                            skipped_count += 1;
                        }
                    }
                }

                info!("Now managing {} window(s)", managed_count);
                if skipped_count > 0 {
                    debug!("Skipped {} non-application window(s)", skipped_count);
                }
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

/// Load and validate configuration.
///
/// Returns the ConfigLoader and loaded configuration if successful.
pub fn load_and_validate_config() -> Result<(ConfigLoader, crate::config::Config)> {
    info!("Loading configuration...");
    let config_loader = ConfigLoader::new()?;
    let config = config_loader.load()?;
    info!(
        "Configuration loaded from: {:?}",
        config_loader.get_config_path()
    );

    // Validate configuration
    ConfigValidator::validate(&config)?;
    info!("Configuration validated successfully");

    Ok((config_loader, config))
}

/// Demonstrate the command system integration.
///
/// This function shows examples of how commands are executed via the CommandExecutor.
/// Commands are triggered by hotkey bindings or IPC.
pub fn demonstrate_command_system(wm: &mut WindowManager) -> Result<()> {
    info!("[DEBUG] Command System Available:");
    info!("  Layouts: Dwindle, Master");
    info!("  Window: Float, Fullscreen, Close, Minimize");
    info!("  Focus: Left/Right/Up/Down, Previous/Next");
    info!("  Workspace: Switch, Move, MoveAndFollow");
    info!("  Current layout: {:?}", wm.get_current_layout());

    Ok(())
}
