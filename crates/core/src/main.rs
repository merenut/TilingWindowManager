mod window_manager;
mod event_loop;
mod utils;

use anyhow::Result;
use tracing::{info, error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use window_manager::WindowManager;
use event_loop::{EventLoop, WindowEvent};

#[cfg(target_os = "windows")]
use utils::win32::WindowHandle;

fn main() -> Result<()> {
    // Initialize logging
    initialize_logging();
    
    info!("Starting Tiling Window Manager v0.1.0");
    
    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C signal, initiating shutdown...");
        running_clone.store(false, Ordering::SeqCst);
    })?;
    
    // Initialize window manager
    info!("Initializing window manager...");
    let mut wm = WindowManager::new();
    wm.initialize()?;
    info!("Window manager initialized successfully");
    
    // Set up event loop
    info!("Starting event loop...");
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    info!("Event loop started successfully");
    
    // Scan and manage existing windows
    info!("Scanning for existing windows...");
    scan_and_manage_windows(&mut wm)?;
    
    info!("Tiling Window Manager is now running. Press Ctrl+C to exit.");
    
    // Main event loop
    run_event_loop(&mut wm, &mut event_loop, &running)?;
    
    // Clean shutdown
    info!("Stopping event loop...");
    event_loop.stop()?;
    info!("Tiling Window Manager stopped successfully");
    
    Ok(())
}

/// Initialize logging with appropriate levels and formatting.
fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_core=debug,info")
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();
}

/// Scan for existing windows and add them to management.
fn scan_and_manage_windows(wm: &mut WindowManager) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use utils::win32;
        use tracing::warn;
        
        match win32::enumerate_app_windows() {
            Ok(windows) => {
                info!("Found {} existing windows", windows.len());
                
                let mut managed_count = 0;
                for window in windows {
                    if wm.should_manage_window(&window).unwrap_or(false) {
                        let title = window.get_title().unwrap_or_else(|_| String::from("<unknown>"));
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
fn run_event_loop(
    wm: &mut WindowManager,
    event_loop: &mut EventLoop,
    running: &Arc<AtomicBool>,
) -> Result<()> {
    while running.load(Ordering::SeqCst) {
        // Process Windows messages
        if let Err(e) = event_loop.process_messages() {
            error!("Error processing messages: {}", e);
        }
        
        // Poll for window events
        for event in event_loop.poll_events() {
            if let Err(e) = handle_window_event(wm, event) {
                error!("Error handling window event: {}", e);
            }
        }
        
        // Small sleep to prevent 100% CPU usage
        std::thread::sleep(Duration::from_millis(50));
    }
    
    Ok(())
}

/// Handle a window event by dispatching to the appropriate window manager action.
#[cfg(target_os = "windows")]
fn handle_window_event(wm: &mut WindowManager, event: WindowEvent) -> Result<()> {
    use tracing::debug;
    
    match event {
        WindowEvent::WindowCreated(hwnd) => {
            debug!("Window created: {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);
            
            // Check if we should manage this window
            if wm.should_manage_window(&window)? {
                let title = window.get_title().unwrap_or_else(|_| String::from("<unknown>"));
                info!("Managing new window: {}", title);
                wm.manage_window(window)?;
            }
        }
        
        WindowEvent::WindowDestroyed(hwnd) => {
            debug!("Window destroyed: {:?}", hwnd);
            let window = WindowHandle::from_hwnd(hwnd);
            
            // Try to unmanage - it's okay if it wasn't managed
            if let Err(e) = wm.unmanage_window(&window) {
                debug!("Could not unmanage window {:?}: {}", hwnd, e);
            } else {
                debug!("Unmanaged window: {:?}", hwnd);
            }
        }
        
        WindowEvent::WindowShown(hwnd) => {
            debug!("Window shown: {:?}", hwnd);
            // Window became visible - might need to manage it
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                // Try to manage if not already managed
                if let Err(e) = wm.manage_window(window) {
                    debug!("Window already managed or error: {}", e);
                }
            }
        }
        
        WindowEvent::WindowHidden(hwnd) => {
            debug!("Window hidden: {:?}", hwnd);
            // Window was hidden - we keep it managed but it won't be visible
        }
        
        WindowEvent::WindowMoved(hwnd) => {
            debug!("Window moved: {:?}", hwnd);
            // User manually moved a window - we could re-tile here or track as floating
            // For now, we'll log it but not take action
        }
        
        WindowEvent::WindowMinimized(hwnd) => {
            debug!("Window minimized: {:?}", hwnd);
            // Window was minimized - keep managed but mark as minimized
        }
        
        WindowEvent::WindowRestored(hwnd) => {
            debug!("Window restored: {:?}", hwnd);
            // Window was restored from minimized - ensure it's tiled properly
            let window = WindowHandle::from_hwnd(hwnd);
            if wm.should_manage_window(&window)? {
                // Re-tile the current workspace
                wm.tile_workspace(wm.get_active_workspace())?;
            }
        }
        
        WindowEvent::WindowFocused(hwnd) => {
            debug!("Window focused: {:?}", hwnd);
            // Track which window has focus - could be used for focus-follows-mouse, etc.
        }
        
        WindowEvent::MonitorChanged => {
            info!("Monitor configuration changed");
            // Refresh monitor information and re-tile all workspaces
            wm.refresh_monitors()?;
            wm.tile_workspace(wm.get_active_workspace())?;
        }
    }
    
    Ok(())
}

/// Handle a window event by dispatching to the appropriate window manager action (stub for non-Windows).
#[cfg(not(target_os = "windows"))]
fn handle_window_event(wm: &mut WindowManager, event: WindowEvent) -> Result<()> {
    match event {
        WindowEvent::MonitorChanged => {
            info!("Monitor configuration changed");
            // Refresh monitor information and re-tile all workspaces
            wm.refresh_monitors()?;
            wm.tile_workspace(wm.get_active_workspace())?;
        }
    }
    
    Ok(())
}
