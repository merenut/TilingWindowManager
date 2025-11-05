//! Inter-Process Communication (IPC) module for the Tiling Window Manager.
//!
//! This module provides a JSON-based IPC protocol over Windows named pipes,
//! enabling external programs to query window manager state and execute commands.
//!
//! # Features
//!
//! - **Query Operations**: Get information about windows, workspaces, and monitors
//! - **Command Execution**: Control windows and workspaces remotely
//! - **Event System**: Subscribe to real-time notifications about state changes
//! - **Protocol Versioning**: Ensure compatibility between client and server
//!
//! # Protocol
//!
//! The IPC protocol uses JSON messages with a 4-byte length prefix (little-endian).
//! All requests and responses are serialized using serde_json.
//!
//! # Example
//!
//! ```rust,no_run
//! use tiling_wm_core::ipc::protocol::{Request, Response};
//!
//! // Create a request
//! let request = Request::GetWorkspaces;
//!
//! // Serialize to JSON
//! let json = serde_json::to_string(&request).unwrap();
//!
//! // Send over named pipe...
//! ```

pub mod events;
pub mod protocol;

// Server and client modules are placeholders for future implementation
#[allow(dead_code)]
mod server;
#[allow(dead_code)]
mod client;

// Re-export commonly used types
pub use events::{Event, EventBroadcaster};
pub use protocol::{
    ConfigInfo, MonitorInfo, ProtocolVersion, RectInfo, Request, Response, VersionInfo,
    WindowInfo, WindowState, WorkspaceInfo, PROTOCOL_VERSION,
};
