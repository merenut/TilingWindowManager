//! IPC client for communicating with the window manager
//!
//! This module provides functionality to connect to the window manager's IPC server
//! and receive real-time events.

/// IPC client for connecting to the window manager
#[derive(Clone)]
pub struct IpcClient {
    pipe_name: String,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
        }
    }
    
    /// Get the pipe name
    pub fn pipe_name(&self) -> &str {
        &self.pipe_name
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipc_client_creation() {
        let client = IpcClient::new();
        assert_eq!(client.pipe_name(), r"\\.\pipe\tiling-wm");
    }
}
