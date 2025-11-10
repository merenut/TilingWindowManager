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
