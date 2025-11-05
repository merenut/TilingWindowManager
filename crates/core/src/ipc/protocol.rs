//! IPC Protocol definitions for the Tiling Window Manager.
//!
//! This module defines the JSON-based protocol for inter-process communication,
//! including request and response types, data structures, and protocol versioning.

use serde::{Deserialize, Serialize};

/// Protocol version constant
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Protocol version structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub version: String,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
        }
    }
}

/// Request types for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    // Query requests
    /// Get information about the active window
    GetActiveWindow,
    
    /// Get list of all windows, optionally filtered by workspace
    GetWindows {
        #[serde(default)]
        workspace: Option<usize>,
    },
    
    /// Get list of all workspaces
    GetWorkspaces,
    
    /// Get list of all monitors
    GetMonitors,
    
    /// Get current configuration
    GetConfig,
    
    /// Get version information
    GetVersion,
    
    // Command execution
    /// Execute a generic command with arguments
    ///
    /// The command string should match one of the built-in commands supported
    /// by the window manager. Arguments are command-specific and should be
    /// validated by the command handler.
    Execute {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    
    // Window commands
    /// Close a window (active if hwnd is None)
    CloseWindow {
        hwnd: Option<String>,
    },
    
    /// Focus a specific window
    FocusWindow {
        hwnd: String,
    },
    
    /// Move a window to a different workspace
    MoveWindow {
        hwnd: String,
        workspace: usize,
    },
    
    /// Toggle floating state for a window (active if hwnd is None)
    ToggleFloating {
        hwnd: Option<String>,
    },
    
    /// Toggle fullscreen state for a window (active if hwnd is None)
    ToggleFullscreen {
        hwnd: Option<String>,
    },
    
    // Workspace commands
    /// Switch to a specific workspace
    SwitchWorkspace {
        id: usize,
    },
    
    /// Create a new workspace
    CreateWorkspace {
        name: String,
        monitor: usize,
    },
    
    /// Delete a workspace
    DeleteWorkspace {
        id: usize,
    },
    
    /// Rename a workspace
    RenameWorkspace {
        id: usize,
        name: String,
    },
    
    // Layout commands
    /// Set the layout for the current workspace
    ///
    /// Common layout names include "dwindle", "master", etc.
    /// Invalid layout names will be rejected by the window manager.
    SetLayout {
        layout: String,
    },
    
    /// Adjust the master area factor
    AdjustMasterFactor {
        delta: f32,
    },
    
    /// Increase the number of windows in master area
    IncreaseMasterCount,
    
    /// Decrease the number of windows in master area
    DecreaseMasterCount,
    
    // Event subscription
    /// Subscribe to specific events
    Subscribe {
        events: Vec<String>,
    },
    
    /// Unsubscribe from all events
    Unsubscribe,
    
    // Configuration
    /// Reload configuration from disk
    ReloadConfig,
    
    // System
    /// Ping the server (health check)
    Ping,
    
    /// Quit the window manager
    Quit,
}

/// Response types for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Successful response with optional data
    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    
    /// Error response with message and optional error code
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    
    /// Event notification
    Event {
        name: String,
        data: serde_json::Value,
    },
    
    /// Pong response to Ping request
    Pong,
}

impl Response {
    /// Create a success response with no data
    pub fn success() -> Self {
        Self::Success { data: None }
    }
    
    /// Create a success response with data
    pub fn success_with_data(data: serde_json::Value) -> Self {
        Self::Success { data: Some(data) }
    }
    
    /// Create an error response with a message
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
        }
    }
    
    /// Create an error response with a message and error code
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

/// Information about a window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Window handle as a string
    pub hwnd: String,
    
    /// Window title
    pub title: String,
    
    /// Window class name
    pub class: String,
    
    /// Process name
    pub process_name: String,
    
    /// Workspace ID the window belongs to
    pub workspace: usize,
    
    /// Monitor ID the window is on
    pub monitor: usize,
    
    /// Current window state
    pub state: WindowState,
    
    /// Window rectangle (position and size)
    pub rect: RectInfo,
    
    /// Whether this window is currently focused
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
}

/// Window state enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowState {
    /// Window is tiled (managed by layout)
    Tiled,
    
    /// Window is floating (not managed by layout)
    Floating,
    
    /// Window is in fullscreen mode
    Fullscreen,
    
    /// Window is minimized
    Minimized,
}

/// Rectangle information (position and size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectInfo {
    /// X coordinate
    pub x: i32,
    
    /// Y coordinate
    pub y: i32,
    
    /// Width in pixels
    pub width: i32,
    
    /// Height in pixels
    pub height: i32,
}

/// Information about a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Workspace ID
    pub id: usize,
    
    /// Workspace name
    pub name: String,
    
    /// Monitor ID this workspace is assigned to
    pub monitor: usize,
    
    /// Number of windows in this workspace
    pub window_count: usize,
    
    /// Whether this workspace is currently active
    pub active: bool,
    
    /// Whether this workspace is visible on its monitor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

/// Information about a monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// Monitor ID
    pub id: usize,
    
    /// Monitor name
    pub name: String,
    
    /// Monitor width in pixels
    pub width: i32,
    
    /// Monitor height in pixels
    pub height: i32,
    
    /// Monitor X position
    pub x: i32,
    
    /// Monitor Y position
    pub y: i32,
    
    /// DPI scale factor
    pub scale: f32,
    
    /// Whether this is the primary monitor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    
    /// Active workspace ID on this monitor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_workspace: Option<usize>,
}

/// Configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Configuration version
    pub version: String,
    
    /// Path to the configuration file
    pub config_path: String,
    
    /// Number of configured workspaces
    pub workspaces_count: usize,
    
    /// Available layouts
    pub layouts: Vec<String>,
    
    /// Current layout name
    pub current_layout: String,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string
    pub version: String,
    
    /// Build date
    pub build_date: String,
    
    /// Git commit hash (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    
    /// Rust compiler version
    pub rustc_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_version() {
        let version = ProtocolVersion::default();
        assert_eq!(version.version, "1.0.0");
    }
    
    #[test]
    fn test_request_get_windows_serialization() {
        let request = Request::GetWindows { workspace: Some(1) };
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            Request::GetWindows { workspace } => {
                assert_eq!(workspace, Some(1));
            }
            _ => panic!("Wrong request type"),
        }
    }
    
    #[test]
    fn test_request_get_windows_no_workspace() {
        let request = Request::GetWindows { workspace: None };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("get_windows"));
    }
    
    #[test]
    fn test_execute_request() {
        let request = Request::Execute {
            command: "workspace".to_string(),
            args: vec!["3".to_string()],
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("execute"));
        assert!(json.contains("workspace"));
        
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        match deserialized {
            Request::Execute { command, args } => {
                assert_eq!(command, "workspace");
                assert_eq!(args, vec!["3".to_string()]);
            }
            _ => panic!("Wrong request type"),
        }
    }
    
    #[test]
    fn test_response_success() {
        let response = Response::success();
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("success"));
        
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Success { data } => {
                assert!(data.is_none());
            }
            _ => panic!("Wrong response type"),
        }
    }
    
    #[test]
    fn test_response_success_with_data() {
        let data = serde_json::json!({"test": "value"});
        let response = Response::success_with_data(data.clone());
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("success"));
        assert!(json.contains("test"));
        
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Success { data: Some(d) } => {
                assert_eq!(d, data);
            }
            _ => panic!("Wrong response type or missing data"),
        }
    }
    
    #[test]
    fn test_response_error() {
        let response = Response::error("Test error");
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
        
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Error { message, code } => {
                assert_eq!(message, "Test error");
                assert!(code.is_none());
            }
            _ => panic!("Wrong response type"),
        }
    }
    
    #[test]
    fn test_response_error_with_code() {
        let response = Response::error_with_code("Test error", "ERR_TEST");
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
        assert!(json.contains("ERR_TEST"));
        
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Error { message, code } => {
                assert_eq!(message, "Test error");
                assert_eq!(code, Some("ERR_TEST".to_string()));
            }
            _ => panic!("Wrong response type"),
        }
    }
    
    #[test]
    fn test_window_info_serialization() {
        let info = WindowInfo {
            hwnd: "12345".to_string(),
            title: "Test Window".to_string(),
            class: "TestClass".to_string(),
            process_name: "test.exe".to_string(),
            workspace: 1,
            monitor: 0,
            state: WindowState::Tiled,
            rect: RectInfo {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            focused: Some(true),
        };
        
        let json = serde_json::to_string_pretty(&info).unwrap();
        
        let deserialized: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hwnd, "12345");
        assert_eq!(deserialized.title, "Test Window");
        assert_eq!(deserialized.workspace, 1);
        assert_eq!(deserialized.focused, Some(true));
    }
    
    #[test]
    fn test_window_state_serialization() {
        let states = vec![
            WindowState::Tiled,
            WindowState::Floating,
            WindowState::Fullscreen,
            WindowState::Minimized,
        ];
        
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: WindowState = serde_json::from_str(&json).unwrap();
            
            match (&state, &deserialized) {
                (WindowState::Tiled, WindowState::Tiled) => {},
                (WindowState::Floating, WindowState::Floating) => {},
                (WindowState::Fullscreen, WindowState::Fullscreen) => {},
                (WindowState::Minimized, WindowState::Minimized) => {},
                _ => panic!("State mismatch"),
            }
        }
    }
    
    #[test]
    fn test_workspace_info() {
        let info = WorkspaceInfo {
            id: 1,
            name: "Workspace 1".to_string(),
            monitor: 0,
            window_count: 5,
            active: true,
            visible: Some(true),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: WorkspaceInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.window_count, 5);
        assert_eq!(deserialized.active, true);
        assert_eq!(deserialized.visible, Some(true));
    }
    
    #[test]
    fn test_monitor_info() {
        let info = MonitorInfo {
            id: 0,
            name: "Monitor 1".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            scale: 1.0,
            primary: Some(true),
            active_workspace: Some(1),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: MonitorInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, 0);
        assert_eq!(deserialized.width, 1920);
        assert_eq!(deserialized.primary, Some(true));
    }
    
    #[test]
    fn test_subscribe_request() {
        let request = Request::Subscribe {
            events: vec![
                "window_created".to_string(),
                "workspace_changed".to_string(),
            ],
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            Request::Subscribe { events } => {
                assert_eq!(events.len(), 2);
                assert_eq!(events[0], "window_created");
                assert_eq!(events[1], "workspace_changed");
            }
            _ => panic!("Wrong request type"),
        }
    }
    
    #[test]
    fn test_all_request_types_roundtrip() {
        let requests = vec![
            Request::GetActiveWindow,
            Request::GetWorkspaces,
            Request::GetMonitors,
            Request::GetConfig,
            Request::GetVersion,
            Request::Ping,
            Request::Unsubscribe,
            Request::ReloadConfig,
            Request::IncreaseMasterCount,
            Request::DecreaseMasterCount,
            Request::Quit,
        ];
        
        for request in requests {
            let json = serde_json::to_string(&request).unwrap();
            let _deserialized: Request = serde_json::from_str(&json).unwrap();
        }
    }
    
    #[test]
    fn test_config_info() {
        let info = ConfigInfo {
            version: "1.0.0".to_string(),
            config_path: "/path/to/config.toml".to_string(),
            workspaces_count: 9,
            layouts: vec!["dwindle".to_string(), "master".to_string()],
            current_layout: "dwindle".to_string(),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ConfigInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.layouts.len(), 2);
    }
    
    #[test]
    fn test_version_info() {
        let info = VersionInfo {
            version: "0.1.0".to_string(),
            build_date: "2024-01-01".to_string(),
            git_commit: Some("abc123".to_string()),
            rustc_version: "1.70.0".to_string(),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, "0.1.0");
        assert_eq!(deserialized.git_commit, Some("abc123".to_string()));
    }
    
    #[test]
    fn test_pong_response() {
        let response = Response::Pong;
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("pong"));
        
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Pong => {},
            _ => panic!("Wrong response type"),
        }
    }
}
