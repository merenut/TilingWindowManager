//! IPC request handler for window manager integration.
//!
//! This module provides the RequestHandler that processes IPC requests and forwards
//! them to the window manager, workspace manager, and command executor.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//! use tenraku_core::ipc::handler::RequestHandler;
//! use tenraku_core::ipc::protocol::Request;
//! use tenraku_core::window_manager::WindowManager;
//! use tenraku_core::workspace::WorkspaceManager;
//! use tenraku_core::commands::CommandExecutor;
//!
//! # async fn example() {
//! let wm = Arc::new(Mutex::new(WindowManager::new()));
//! let wsm = Arc::new(Mutex::new(WorkspaceManager::new()));
//! let executor = Arc::new(CommandExecutor::new());
//!
//! let handler = RequestHandler::new(wm, wsm, executor);
//!
//! // Handle a request
//! let request = Request::GetVersion;
//! let response = handler.handle_request(request).await;
//! # }
//! ```

use super::protocol::{
    ConfigInfo, MonitorInfo, Request, Response, VersionInfo,
    WorkspaceInfo,
};
use crate::commands::{Command, CommandExecutor};
use crate::window_manager::WindowManager;
use crate::workspace::core::WorkspaceManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Request handler that processes IPC requests and forwards them to the window manager.
///
/// The RequestHandler acts as a bridge between the IPC server and the window manager,
/// translating IPC requests into window manager operations and formatting responses.
pub struct RequestHandler {
    /// Window manager instance
    window_manager: Arc<Mutex<WindowManager>>,
    
    /// Workspace manager instance
    workspace_manager: Arc<Mutex<WorkspaceManager>>,
    
    /// Command executor for executing window manager commands
    command_executor: Arc<CommandExecutor>,
}

impl RequestHandler {
    /// Create a new request handler.
    ///
    /// # Arguments
    ///
    /// * `window_manager` - Arc-wrapped mutex-protected WindowManager
    /// * `workspace_manager` - Arc-wrapped mutex-protected WorkspaceManager
    /// * `command_executor` - Arc-wrapped CommandExecutor
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use tenraku_core::ipc::handler::RequestHandler;
    /// use tenraku_core::window_manager::WindowManager;
    /// use tenraku_core::workspace::WorkspaceManager;
    /// use tenraku_core::commands::CommandExecutor;
    ///
    /// let wm = Arc::new(Mutex::new(WindowManager::new()));
    /// let wsm = Arc::new(Mutex::new(WorkspaceManager::new()));
    /// let executor = Arc::new(CommandExecutor::new());
    ///
    /// let handler = RequestHandler::new(wm, wsm, executor);
    /// ```
    pub fn new(
        window_manager: Arc<Mutex<WindowManager>>,
        workspace_manager: Arc<Mutex<WorkspaceManager>>,
        command_executor: Arc<CommandExecutor>,
    ) -> Self {
        Self {
            window_manager,
            workspace_manager,
            command_executor,
        }
    }
    
    /// Handle an IPC request and return a response.
    ///
    /// This is the main entry point for request processing. It routes requests
    /// to the appropriate handler method and ensures proper error handling.
    ///
    /// # Arguments
    ///
    /// * `request` - The IPC request to handle
    ///
    /// # Returns
    ///
    /// A Response containing either success with optional data or an error message.
    pub async fn handle_request(&self, request: Request) -> Response {
        debug!("Handling IPC request: {:?}", request);
        
        match request {
            // Query requests
            Request::GetActiveWindow => self.get_active_window().await,
            Request::GetWindows { workspace } => self.get_windows(workspace).await,
            Request::GetWorkspaces => self.get_workspaces().await,
            Request::GetMonitors => self.get_monitors().await,
            Request::GetConfig => self.get_config().await,
            Request::GetVersion => self.get_version().await,
            
            // Command execution
            Request::Execute { command, args } => self.execute_command(command, args).await,
            
            // Window commands
            Request::CloseWindow { hwnd } => self.close_window(hwnd).await,
            Request::FocusWindow { hwnd } => self.focus_window(hwnd).await,
            Request::MoveWindow { hwnd, workspace } => self.move_window(hwnd, workspace).await,
            Request::ToggleFloating { hwnd } => self.toggle_floating(hwnd).await,
            Request::ToggleFullscreen { hwnd } => self.toggle_fullscreen(hwnd).await,
            
            // Workspace commands
            Request::SwitchWorkspace { id } => self.switch_workspace(id).await,
            Request::CreateWorkspace { name, monitor } => self.create_workspace(name, monitor).await,
            Request::DeleteWorkspace { id } => self.delete_workspace(id).await,
            Request::RenameWorkspace { id, name } => self.rename_workspace(id, name).await,
            
            // Layout commands
            Request::SetLayout { layout } => self.set_layout(layout).await,
            Request::AdjustMasterFactor { delta } => self.adjust_master_factor(delta).await,
            Request::IncreaseMasterCount => self.increase_master_count().await,
            Request::DecreaseMasterCount => self.decrease_master_count().await,
            
            // Configuration
            Request::ReloadConfig => self.reload_config().await,
            
            // System commands - These are handled by the server
            Request::Ping => Response::Pong,
            Request::Subscribe { .. } => Response::error("Subscribe must be handled by IPC server"),
            Request::Unsubscribe => Response::error("Unsubscribe must be handled by IPC server"),
            Request::Quit => self.quit().await,
        }
    }
    
    // Query handlers
    
    async fn get_active_window(&self) -> Response {
        debug!("Getting active window");
        
        let wm = self.window_manager.lock().await;
        
        if let Some(window) = wm.get_active_window() {
            Response::success_with_data(serde_json::json!({
                "hwnd": window.handle.hwnd().0,
                "title": window.title,
                "exe": window.process_name,
                "class": window.class,
                "workspace": window.workspace
            }))
        } else {
            Response::error("No active window")
        }
    }
    
    async fn get_windows(&self, workspace: Option<usize>) -> Response {
        debug!("Getting windows list for workspace: {:?}", workspace);
        
        let wm = self.window_manager.lock().await;
        let windows = wm.get_windows(workspace);
        
        let window_list: Vec<_> = windows.iter().map(|w| {
            serde_json::json!({
                "hwnd": w.handle.hwnd().0,
                "title": w.title,
                "exe": w.process_name,
                "class": w.class,
                "workspace": w.workspace
            })
        }).collect();
        
        Response::success_with_data(serde_json::json!(window_list))
    }
    
    async fn get_workspaces(&self) -> Response {
        debug!("Getting workspaces list");
        
        let wsm = self.workspace_manager.lock().await;
        let active_workspace = wsm.active_workspace();
        
        // Iterate through workspace IDs
        // Note: We check up to 20 workspaces to accommodate various configurations
        // Only workspaces that exist will be included in the response
        let mut workspace_infos: Vec<WorkspaceInfo> = Vec::new();
        
        for id in 1..=20 {
            if let Some(ws) = wsm.get_workspace(id) {
                workspace_infos.push(WorkspaceInfo {
                    id: ws.id,
                    name: ws.name.clone(),
                    monitor: ws.monitor,
                    window_count: ws.windows.len(),
                    active: ws.id == active_workspace,
                    visible: Some(ws.visible),
                });
            }
        }
        
        match serde_json::to_value(workspace_infos) {
            Ok(data) => Response::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize workspaces: {}", e);
                Response::error(format!("Failed to serialize workspaces: {}", e))
            }
        }
    }
    
    async fn get_monitors(&self) -> Response {
        debug!("Getting monitors list");
        
        let wm = self.window_manager.lock().await;
        let monitors = wm.get_monitors();
        
        let monitor_infos: Vec<MonitorInfo> = monitors
            .iter()
            .enumerate()
            .map(|(idx, mon)| MonitorInfo {
                id: idx,
                name: mon.name.clone(),
                width: mon.work_area.width,
                height: mon.work_area.height,
                x: mon.work_area.x,
                y: mon.work_area.y,
                scale: mon.dpi_scale,
                // Note: Primary monitor detection would ideally come from the OS
                // For now, we assume the first monitor is primary as a reasonable default
                primary: Some(idx == 0),
                active_workspace: mon.active_workspace,
            })
            .collect();
        
        match serde_json::to_value(monitor_infos) {
            Ok(data) => Response::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize monitors: {}", e);
                Response::error(format!("Failed to serialize monitors: {}", e))
            }
        }
    }
    
    async fn get_config(&self) -> Response {
        debug!("Getting configuration");
        
        // Note: This is a placeholder implementation
        // In a full implementation, this would return actual configuration
        let wsm = self.workspace_manager.lock().await;
        let workspace_count = wsm.workspace_count();
        drop(wsm);
        
        let config_info = ConfigInfo {
            version: "1.0.0".to_string(),
            config_path: "config.toml".to_string(),
            workspaces_count: workspace_count,
            layouts: vec!["dwindle".to_string(), "master".to_string()],
            current_layout: "dwindle".to_string(),
        };
        
        match serde_json::to_value(config_info) {
            Ok(data) => Response::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize config: {}", e);
                Response::error(format!("Failed to serialize config: {}", e))
            }
        }
    }
    
    async fn get_version(&self) -> Response {
        debug!("Getting version information");
        
        let version_info = VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: option_env!("BUILD_DATE").unwrap_or("unknown").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(String::from),
            rustc_version: option_env!("RUSTC_VERSION").unwrap_or("unknown").to_string(),
        };
        
        match serde_json::to_value(version_info) {
            Ok(data) => Response::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize version: {}", e);
                Response::error(format!("Failed to serialize version: {}", e))
            }
        }
    }
    
    // Command handlers
    
    async fn execute_command(&self, command: String, args: Vec<String>) -> Response {
        debug!("Executing command: {} with args: {:?}", command, args);
        
        // Parse command string into Command enum
        let cmd = match command.as_str() {
            "close" => Some(Command::CloseActiveWindow),
            "toggle_floating" | "toggle-floating" => Some(Command::ToggleFloating),
            "toggle_fullscreen" | "toggle-fullscreen" => Some(Command::ToggleFullscreen),
            "focus_left" | "focus-left" => Some(Command::FocusLeft),
            "focus_right" | "focus-right" => Some(Command::FocusRight),
            "focus_up" | "focus-up" => Some(Command::FocusUp),
            "focus_down" | "focus-down" => Some(Command::FocusDown),
            "layout_dwindle" | "layout-dwindle" => Some(Command::SetLayoutDwindle),
            "layout_master" | "layout-master" => Some(Command::SetLayoutMaster),
            "increase_master" | "increase-master" => Some(Command::IncreaseMasterCount),
            "decrease_master" | "decrease-master" => Some(Command::DecreaseMasterCount),
            "workspace" if !args.is_empty() => {
                if let Ok(id) = args[0].parse::<usize>() {
                    Some(Command::SwitchWorkspace(id))
                } else {
                    None
                }
            }
            _ => None,
        };
        
        if let Some(cmd) = cmd {
            let mut wm = self.window_manager.lock().await;
            match self.command_executor.execute(cmd, &mut wm) {
                Ok(_) => {
                    info!("Command executed successfully: {}", command);
                    Response::success()
                }
                Err(e) => {
                    error!("Command execution failed: {}", e);
                    Response::error(format!("Command execution failed: {}", e))
                }
            }
        } else {
            Response::error(format!("Unknown command: {}", command))
        }
    }
    
    async fn close_window(&self, _hwnd: Option<String>) -> Response {
        debug!("Closing window");
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::CloseActiveWindow, &mut wm) {
            Ok(_) => {
                info!("Window closed successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to close window: {}", e);
                Response::error(format!("Failed to close window: {}", e))
            }
        }
    }
    
    async fn focus_window(&self, hwnd: String) -> Response {
        debug!("Focusing window: {}", hwnd);
        
        // Parse HWND from string
        let hwnd_value = match hwnd.parse::<isize>() {
            Ok(v) => v,
            Err(e) => return Response::error(format!("Invalid HWND format: {}", e))
        };
        
        let mut wm = self.window_manager.lock().await;
        match wm.focus_window_by_hwnd(hwnd_value) {
            Ok(_) => {
                info!("Window focused successfully: {}", hwnd);
                Response::success()
            }
            Err(e) => {
                error!("Failed to focus window: {}", e);
                Response::error(format!("Failed to focus window: {}", e))
            }
        }
    }
    
    async fn move_window(&self, hwnd: String, workspace: usize) -> Response {
        debug!("Moving window {} to workspace {}", hwnd, workspace);
        
        // Parse HWND from string
        let hwnd_value = match hwnd.parse::<isize>() {
            Ok(v) => v,
            Err(e) => return Response::error(format!("Invalid HWND format: {}", e))
        };
        
        // Validate workspace ID
        if workspace < 1 || workspace > 10 {
            return Response::error(format!("Invalid workspace ID: {}", workspace));
        }
        
        let mut wm = self.window_manager.lock().await;
        
        // Get the window from registry and update its workspace
        if let Some(managed_window) = wm.registry_mut().get_mut(hwnd_value) {
            let old_workspace = managed_window.workspace;
            
            if old_workspace == workspace {
                return Response::success(); // Already on target workspace
            }
            
            managed_window.workspace = workspace;
            
            debug!(
                "Moved window {} from workspace {} to {}",
                hwnd_value, old_workspace, workspace
            );
            
            // Hide the window if it's not on the current workspace
            use crate::utils::win32::WindowHandle;
            use windows::Win32::Foundation::HWND;
            let window_handle = WindowHandle::from_hwnd(HWND(hwnd_value));
            
            if wm.get_active_workspace() != workspace {
                #[cfg(target_os = "windows")]
                window_handle.hide();
            }
            
            // Retile both workspaces
            if let Err(e) = wm.retile_workspace(old_workspace) {
                error!("Failed to retile old workspace: {}", e);
            }
            if let Err(e) = wm.retile_workspace(workspace) {
                error!("Failed to retile new workspace: {}", e);
            }
            
            info!("Window moved successfully to workspace {}", workspace);
            Response::success()
        } else {
            Response::error(format!("Window {} is not managed", hwnd))
        }
    }
    
    async fn toggle_floating(&self, _hwnd: Option<String>) -> Response {
        debug!("Toggling floating");
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::ToggleFloating, &mut wm) {
            Ok(_) => {
                info!("Toggled floating successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to toggle floating: {}", e);
                Response::error(format!("Failed to toggle floating: {}", e))
            }
        }
    }
    
    async fn toggle_fullscreen(&self, _hwnd: Option<String>) -> Response {
        debug!("Toggling fullscreen");
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::ToggleFullscreen, &mut wm) {
            Ok(_) => {
                info!("Toggled fullscreen successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to toggle fullscreen: {}", e);
                Response::error(format!("Failed to toggle fullscreen: {}", e))
            }
        }
    }
    
    async fn switch_workspace(&self, id: usize) -> Response {
        debug!("Switching to workspace {}", id);
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::SwitchWorkspace(id), &mut wm) {
            Ok(_) => {
                info!("Switched to workspace {} successfully", id);
                Response::success()
            }
            Err(e) => {
                error!("Failed to switch workspace: {}", e);
                Response::error(format!("Failed to switch workspace: {}", e))
            }
        }
    }
    
    async fn create_workspace(&self, name: String, monitor: usize) -> Response {
        debug!("Creating workspace {} on monitor {}", name, monitor);
        
        let mut wsm = self.workspace_manager.lock().await;
        
        // Get the monitor area - for now use a default size
        // In a full implementation, this would query the actual monitor dimensions
        let area = crate::window_manager::Rect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        
        let workspace_id = wsm.create_workspace(name.clone(), monitor, area);
        
        info!("Created workspace {} with ID {}", name, workspace_id);
        Response::success_with_data(serde_json::json!({
            "workspace_id": workspace_id
        }))
    }
    
    async fn delete_workspace(&self, id: usize) -> Response {
        debug!("Deleting workspace {}", id);
        
        let mut wsm = self.workspace_manager.lock().await;
        
        // Use workspace 1 as the fallback for moved windows
        match wsm.delete_workspace(id, 1) {
            Ok(_) => {
                info!("Deleted workspace {} successfully", id);
                Response::success()
            }
            Err(e) => {
                error!("Failed to delete workspace: {}", e);
                Response::error(format!("Failed to delete workspace: {}", e))
            }
        }
    }
    
    async fn rename_workspace(&self, id: usize, name: String) -> Response {
        debug!("Renaming workspace {} to {}", id, name);
        
        let mut wsm = self.workspace_manager.lock().await;
        
        match wsm.rename_workspace(id, name.clone()) {
            Ok(_) => {
                info!("Renamed workspace {} to {} successfully", id, name);
                Response::success()
            }
            Err(e) => {
                error!("Failed to rename workspace: {}", e);
                Response::error(format!("Failed to rename workspace: {}", e))
            }
        }
    }
    
    async fn set_layout(&self, layout: String) -> Response {
        debug!("Setting layout to {}", layout);
        
        let cmd = match layout.as_str() {
            "dwindle" => Command::SetLayoutDwindle,
            "master" => Command::SetLayoutMaster,
            _ => {
                return Response::error(format!("Unknown layout: {}", layout));
            }
        };
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(cmd, &mut wm) {
            Ok(_) => {
                info!("Layout set to {} successfully", layout);
                Response::success()
            }
            Err(e) => {
                error!("Failed to set layout: {}", e);
                Response::error(format!("Failed to set layout: {}", e))
            }
        }
    }
    
    async fn adjust_master_factor(&self, delta: f32) -> Response {
        debug!("Adjusting master factor by {}", delta);
        
        let cmd = if delta > 0.0 {
            Command::IncreaseMasterFactor
        } else {
            Command::DecreaseMasterFactor
        };
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(cmd, &mut wm) {
            Ok(_) => {
                info!("Master factor adjusted successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to adjust master factor: {}", e);
                Response::error(format!("Failed to adjust master factor: {}", e))
            }
        }
    }
    
    async fn increase_master_count(&self) -> Response {
        debug!("Increasing master count");
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::IncreaseMasterCount, &mut wm) {
            Ok(_) => {
                info!("Master count increased successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to increase master count: {}", e);
                Response::error(format!("Failed to increase master count: {}", e))
            }
        }
    }
    
    async fn decrease_master_count(&self) -> Response {
        debug!("Decreasing master count");
        
        let mut wm = self.window_manager.lock().await;
        match self.command_executor.execute(Command::DecreaseMasterCount, &mut wm) {
            Ok(_) => {
                info!("Master count decreased successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to decrease master count: {}", e);
                Response::error(format!("Failed to decrease master count: {}", e))
            }
        }
    }
    
    async fn reload_config(&self) -> Response {
        debug!("Reloading configuration");
        
        // Load config from default location
        use crate::config::ConfigLoader;
        
        let config_loader = match ConfigLoader::new() {
            Ok(loader) => loader,
            Err(e) => {
                error!("Failed to create config loader: {}", e);
                return Response::error(format!("Failed to load config: {}", e));
            }
        };
        
        let config = match config_loader.load() {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load config: {}", e);
                return Response::error(format!("Failed to load config: {}", e));
            }
        };
        
        // Apply configuration to window manager
        let mut wm = self.window_manager.lock().await;
        match wm.update_config(&config) {
            Ok(_) => {
                info!("Configuration reloaded successfully");
                Response::success()
            }
            Err(e) => {
                error!("Failed to apply config: {}", e);
                Response::error(format!("Failed to apply config: {}", e))
            }
        }
    }
    
    async fn quit(&self) -> Response {
        info!("Quit command received");
        // Note: The actual quit logic should be handled by the main application
        // This just returns success to acknowledge the request
        Response::success()
    }
}