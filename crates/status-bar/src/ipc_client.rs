//! IPC client for communicating with the window manager
//!
//! This module provides functionality to connect to the window manager's IPC server,
//! subscribe to events, and send commands.

use crate::module::IpcEvent;
use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

/// Windows file flag for overlapped I/O operations on named pipes
#[cfg(windows)]
const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

/// Default retry delay in seconds when connection is lost
const DEFAULT_RETRY_DELAY_SECS: u64 = 5;

/// Maximum message size in bytes (1MB) to prevent DoS via large allocations
#[cfg_attr(not(windows), allow(dead_code))]
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// IPC client for connecting to the window manager
#[derive(Clone)]
pub struct IpcClient {
    /// Named pipe path
    pipe_name: String,
    /// Event sender for broadcasting events to modules
    event_sender: Option<mpsc::UnboundedSender<IpcEvent>>,
    /// Connection state
    connected: Arc<Mutex<bool>>,
    /// Retry delay in seconds
    retry_delay_secs: u64,
}

impl IpcClient {
    /// Create a new IPC client with default settings
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tenraku".to_string(),
            event_sender: None,
            connected: Arc::new(Mutex::new(false)),
            retry_delay_secs: DEFAULT_RETRY_DELAY_SECS,
        }
    }

    /// Create a new IPC client with custom pipe name
    pub fn with_pipe_name(pipe_name: String) -> Self {
        Self {
            pipe_name,
            event_sender: None,
            connected: Arc::new(Mutex::new(false)),
            retry_delay_secs: DEFAULT_RETRY_DELAY_SECS,
        }
    }

    /// Set the retry delay in seconds
    pub fn with_retry_delay(mut self, secs: u64) -> Self {
        self.retry_delay_secs = secs;
        self
    }

    /// Get the pipe name
    pub fn pipe_name(&self) -> &str {
        &self.pipe_name
    }

    /// Set event sender for receiving IPC events
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<IpcEvent>) {
        self.event_sender = Some(sender);
    }

    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Connect to window manager and subscribe to events
    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to window manager IPC...");

        // Test connection by sending a ping
        match self
            .send_request(&serde_json::json!({
                "type": "ping"
            }))
            .await
        {
            Ok(_) => {
                *self.connected.lock().await = true;
                info!("Connected to window manager IPC");

                // Subscribe to events
                self.subscribe_to_events().await?;

                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to window manager: {}", e);
                Err(e)
            }
        }
    }

    /// Subscribe to window manager events
    async fn subscribe_to_events(&self) -> Result<()> {
        let request = serde_json::json!({
            "type": "subscribe",
            "events": [
                "workspace_changed",
                "window_focused",
                "window_created",
                "window_closed",
                "config_reloaded"
            ]
        });

        self.send_request(&request).await?;
        info!("Subscribed to window manager events");

        Ok(())
    }

    /// Get list of workspaces
    pub async fn get_workspaces(&self) -> Result<Vec<WorkspaceData>> {
        let request = serde_json::json!({
            "type": "get_workspaces"
        });

        let response = self.send_request(&request).await?;

        if let Some(data) = response.get("data") {
            let workspaces =
                serde_json::from_value(data.clone()).context("Failed to parse workspace data")?;
            Ok(workspaces)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get active window information
    pub async fn get_active_window(&self) -> Result<Option<WindowData>> {
        let request = serde_json::json!({
            "type": "get_active_window"
        });

        let response = self.send_request(&request).await?;

        if let Some(data) = response.get("data") {
            if data.is_null() {
                Ok(None)
            } else {
                let window =
                    serde_json::from_value(data.clone()).context("Failed to parse window data")?;
                Ok(Some(window))
            }
        } else {
            Ok(None)
        }
    }

    /// Switch to a workspace
    pub async fn switch_workspace(&self, id: usize) -> Result<()> {
        let request = serde_json::json!({
            "type": "switch_workspace",
            "id": id
        });

        let _response = self.send_request(&request).await?;
        debug!("Switched to workspace {}", id);

        Ok(())
    }

    /// Send a command to the window manager
    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<()> {
        let request = serde_json::json!({
            "type": "execute",
            "command": command,
            "args": args
        });

        let _response = self.send_request(&request).await?;
        debug!("Executed command: {} {:?}", command, args);

        Ok(())
    }

    /// Send a request and receive response
    #[cfg_attr(not(windows), allow(unused_variables))]
    async fn send_request(&self, request: &Value) -> Result<Value> {
        #[cfg(windows)]
        {
            // Open pipe connection
            let mut pipe = self.open_pipe()?;

            // Serialize and send request
            let data = serde_json::to_vec(request).context("Failed to serialize request")?;
            let len = data.len() as u32;

            pipe.write_all(&len.to_le_bytes())
                .context("Failed to write request length")?;
            pipe.write_all(&data)
                .context("Failed to write request data")?;
            pipe.flush().context("Failed to flush pipe")?;

            // Read response
            let mut len_buf = [0u8; 4];
            pipe.read_exact(&mut len_buf)
                .context("Failed to read response length")?;
            let response_len = u32::from_le_bytes(len_buf) as usize;

            // Validate message size to prevent DoS
            if response_len > MAX_MESSAGE_SIZE {
                anyhow::bail!(
                    "Response message too large: {} bytes (max: {} bytes)",
                    response_len,
                    MAX_MESSAGE_SIZE
                );
            }

            let mut response_data = vec![0u8; response_len];
            pipe.read_exact(&mut response_data)
                .context("Failed to read response data")?;

            // Parse response
            let response: Value =
                serde_json::from_slice(&response_data).context("Failed to parse response")?;

            // Check for errors in response
            if let Some(status) = response.get("status") {
                if status == "error" {
                    if let Some(message) = response.get("message") {
                        anyhow::bail!("Server error: {}", message);
                    } else {
                        anyhow::bail!("Server returned error status");
                    }
                }
            }

            Ok(response)
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!("IPC client only works on Windows")
        }
    }

    #[cfg(windows)]
    fn open_pipe(&self) -> Result<std::fs::File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(&self.pipe_name)
            .context("Failed to open named pipe")
    }

    /// Start event listener in background
    pub async fn start_event_listener(&self) -> Result<()> {
        let sender = self
            .event_sender
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Event sender not set"))?
            .clone();

        let pipe_name = self.pipe_name.clone();
        let connected = self.connected.clone();
        let retry_delay = self.retry_delay_secs;

        // Spawn background task for listening to events
        tokio::spawn(async move {
            loop {
                // Check if we're connected
                if !*connected.lock().await {
                    warn!("Not connected, waiting before retry...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay)).await;
                    continue;
                }

                match Self::listen_for_events(&pipe_name, &sender, &connected).await {
                    Ok(_) => {
                        info!("Event listener stopped normally");
                    }
                    Err(e) => {
                        error!("Event listener error: {}", e);
                        *connected.lock().await = false;
                        // Wait before retrying
                        tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay)).await;
                    }
                }
            }
        });

        Ok(())
    }

    #[cfg(windows)]
    async fn listen_for_events(
        pipe_name: &str,
        sender: &mpsc::UnboundedSender<IpcEvent>,
        connected: &Arc<Mutex<bool>>,
    ) -> Result<()> {
        info!("Starting event listener...");

        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(pipe_name)
            .context("Failed to open named pipe for events")?;

        loop {
            // Read event length
            let mut len_buf = [0u8; 4];
            match pipe.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Connection lost while reading event length: {}", e);
                    *connected.lock().await = false;
                    return Err(e.into());
                }
            }

            let event_len = u32::from_le_bytes(len_buf) as usize;

            // Validate message size to prevent DoS
            if event_len > MAX_MESSAGE_SIZE {
                error!("Event message too large: {} bytes, skipping", event_len);
                continue;
            }

            // Read event data
            let mut event_data = vec![0u8; event_len];
            match pipe.read_exact(&mut event_data) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Connection lost while reading event data: {}", e);
                    *connected.lock().await = false;
                    return Err(e.into());
                }
            }

            // Parse event
            let event_value: Value = match serde_json::from_slice(&event_data) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse event: {}", e);
                    continue;
                }
            };

            // Convert to IpcEvent and send
            if let Some(ipc_event) = Self::parse_event(&event_value) {
                debug!("Received IPC event: {:?}", ipc_event);
                if let Err(e) = sender.send(ipc_event) {
                    error!("Failed to send event to modules: {}", e);
                }
            }
        }
    }

    #[cfg(not(windows))]
    async fn listen_for_events(
        _pipe_name: &str,
        _sender: &mpsc::UnboundedSender<IpcEvent>,
        _connected: &Arc<Mutex<bool>>,
    ) -> Result<()> {
        anyhow::bail!("IPC client only works on Windows")
    }

    /// Parse JSON event into IpcEvent
    #[cfg_attr(not(windows), allow(dead_code))]
    fn parse_event(value: &Value) -> Option<IpcEvent> {
        // Check if this is an event response
        let event_type = value.get("type")?.as_str()?;

        if event_type == "event" {
            let name = value.get("name")?.as_str()?;
            let data = value.get("data")?;

            match name {
                "workspace_changed" => Some(IpcEvent::WorkspaceChanged {
                    from: data.get("from")?.as_u64()? as usize,
                    to: data.get("to")?.as_u64()? as usize,
                }),
                "window_focused" => Some(IpcEvent::WindowFocused {
                    hwnd: data.get("hwnd")?.as_str()?.to_string(),
                    title: data.get("title")?.as_str()?.to_string(),
                }),
                "window_created" => Some(IpcEvent::WindowCreated {
                    hwnd: data.get("hwnd")?.as_str()?.to_string(),
                    title: data.get("title")?.as_str()?.to_string(),
                }),
                "window_closed" => Some(IpcEvent::WindowClosed {
                    hwnd: data.get("hwnd")?.as_str()?.to_string(),
                }),
                "config_reloaded" => Some(IpcEvent::ConfigReloaded),
                _ => {
                    warn!("Unknown event type: {}", name);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Workspace data from window manager
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkspaceData {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub window_count: usize,
    pub active: bool,
}

/// Window data from window manager
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowData {
    pub hwnd: String,
    pub title: String,
    pub class: String,
    pub process_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_client_creation() {
        let client = IpcClient::new();
        assert_eq!(client.pipe_name(), r"\\.\pipe\tenraku");
    }

    #[tokio::test]
    async fn test_ipc_client_initially_not_connected() {
        let client = IpcClient::new();
        assert!(!client.is_connected().await);
    }

    #[test]
    fn test_parse_event_workspace_changed() {
        let json = serde_json::json!({
            "type": "event",
            "name": "workspace_changed",
            "data": {
                "from": 1,
                "to": 2
            }
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_some());

        if let Some(IpcEvent::WorkspaceChanged { from, to }) = event {
            assert_eq!(from, 1);
            assert_eq!(to, 2);
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_parse_event_window_focused() {
        let json = serde_json::json!({
            "type": "event",
            "name": "window_focused",
            "data": {
                "hwnd": "12345",
                "title": "Test Window"
            }
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_some());

        if let Some(IpcEvent::WindowFocused { hwnd, title }) = event {
            assert_eq!(hwnd, "12345");
            assert_eq!(title, "Test Window");
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_parse_event_window_created() {
        let json = serde_json::json!({
            "type": "event",
            "name": "window_created",
            "data": {
                "hwnd": "67890",
                "title": "New Window"
            }
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_some());

        if let Some(IpcEvent::WindowCreated { hwnd, title }) = event {
            assert_eq!(hwnd, "67890");
            assert_eq!(title, "New Window");
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_parse_event_window_closed() {
        let json = serde_json::json!({
            "type": "event",
            "name": "window_closed",
            "data": {
                "hwnd": "11111"
            }
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_some());

        if let Some(IpcEvent::WindowClosed { hwnd }) = event {
            assert_eq!(hwnd, "11111");
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_parse_event_config_reloaded() {
        let json = serde_json::json!({
            "type": "event",
            "name": "config_reloaded",
            "data": {}
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_some());
        assert!(matches!(event, Some(IpcEvent::ConfigReloaded)));
    }

    #[test]
    fn test_parse_event_unknown_type() {
        let json = serde_json::json!({
            "type": "event",
            "name": "unknown_event",
            "data": {}
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_event_invalid_format() {
        let json = serde_json::json!({
            "type": "response",
            "status": "success"
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_event_missing_data() {
        let json = serde_json::json!({
            "type": "event",
            "name": "workspace_changed"
        });

        let event = IpcClient::parse_event(&json);
        assert!(event.is_none());
    }

    #[test]
    fn test_with_pipe_name() {
        let client = IpcClient::with_pipe_name(r"\\.\pipe\custom-wm".to_string());
        assert_eq!(client.pipe_name(), r"\\.\pipe\custom-wm");
    }

    #[test]
    fn test_with_retry_delay() {
        let client = IpcClient::new().with_retry_delay(10);
        assert_eq!(client.retry_delay_secs, 10);
    }

    #[test]
    fn test_builder_pattern() {
        let client = IpcClient::with_pipe_name(r"\\.\pipe\test".to_string()).with_retry_delay(3);
        assert_eq!(client.pipe_name(), r"\\.\pipe\test");
        assert_eq!(client.retry_delay_secs, 3);
    }
}
