# Phase 5: IPC & CLI - Detailed Task List

**Timeline:** Weeks 17-20 (4 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 5 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 5 Goals:**
- Implement named pipe IPC server for inter-process communication
- Create comprehensive JSON-based protocol for commands and queries
- Build CLI client application with full command set
- Implement event subscription system for real-time notifications
- Enable external scripting and automation
- Create documentation and example scripts
- Support multiple concurrent IPC connections
- Provide robust error handling and security

**Prerequisites:**
- Phase 1 completed successfully (project foundation, Win32 wrappers, tree structure)
- Phase 2 completed successfully (window management, layouts, focus management, commands)
- Phase 3 completed successfully (workspace system, Virtual Desktop integration, persistence)
- Phase 4 completed successfully (configuration system, window rules, keybindings, hot-reload)
- All Phase 1-4 tests passing
- Window manager fully operational with all core features
- Command system functional and tested

---

## Success Criteria for Phase 5 Completion

Phase 5 is considered complete when:

1. **IPC Protocol fully defined:**
   - JSON protocol specification complete
   - All request types documented
   - All response types documented
   - Event types clearly defined
   - Protocol versioning implemented

2. **Named pipe server operational:**
   - Server starts and listens on named pipe
   - Accepts multiple concurrent connections
   - Handles requests asynchronously
   - Returns properly formatted responses
   - Broadcasts events to subscribers
   - Handles client disconnects gracefully

3. **CLI client fully functional:**
   - Can query window manager state (windows, workspaces, monitors)
   - Can execute all commands via IPC
   - Supports all command-line arguments
   - Provides formatted output (JSON, table, etc.)
   - Has comprehensive help documentation
   - Error messages are clear and helpful

4. **Event system working:**
   - Window events broadcast correctly
   - Workspace events broadcast correctly
   - Configuration reload events work
   - Clients can subscribe to specific events
   - Clients can unsubscribe
   - Event filtering works correctly

5. **Integration complete:**
   - Window manager exposes all state via IPC
   - All commands work via IPC
   - IPC server starts with window manager
   - No performance impact from IPC
   - Memory usage is reasonable

6. **All tests passing:**
   - Unit tests for protocol serialization
   - Unit tests for server logic
   - Integration tests for CLI
   - End-to-end tests for IPC communication
   - Manual validation successful

---

## Task Breakdown

### Week 17: IPC Protocol Design and Data Structures

#### Task 5.1: Define IPC Protocol Schema

**Objective:** Create comprehensive data structures for the IPC protocol with full serialization support.

**File:** `crates/core/src/ipc/protocol.rs`

**Required Implementations:**

1. **Create IPC module structure:**
   ```bash
   mkdir -p crates/core/src/ipc
   touch crates/core/src/ipc/mod.rs
   touch crates/core/src/ipc/protocol.rs
   touch crates/core/src/ipc/server.rs
   touch crates/core/src/ipc/client.rs
   touch crates/core/src/ipc/events.rs
   ```

2. **Request enum with all request types:**

   ```rust
   use serde::{Serialize, Deserialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "type", rename_all = "snake_case")]
   pub enum Request {
       // Query requests
       GetActiveWindow,
       GetWindows {
           #[serde(default)]
           workspace: Option<usize>,
       },
       GetWorkspaces,
       GetMonitors,
       GetConfig,
       GetVersion,
       
       // Command execution
       Execute {
           command: String,
           #[serde(default)]
           args: Vec<String>,
       },
       
       // Window commands
       CloseWindow {
           hwnd: Option<String>,
       },
       FocusWindow {
           hwnd: String,
       },
       MoveWindow {
           hwnd: String,
           workspace: usize,
       },
       ToggleFloating {
           hwnd: Option<String>,
       },
       ToggleFullscreen {
           hwnd: Option<String>,
       },
       
       // Workspace commands
       SwitchWorkspace {
           id: usize,
       },
       CreateWorkspace {
           name: String,
           monitor: usize,
       },
       DeleteWorkspace {
           id: usize,
       },
       RenameWorkspace {
           id: usize,
           name: String,
       },
       
       // Layout commands
       SetLayout {
           layout: String,
       },
       AdjustMasterFactor {
           delta: f32,
       },
       IncreaseMasterCount,
       DecreaseMasterCount,
       
       // Event subscription
       Subscribe {
           events: Vec<String>,
       },
       Unsubscribe,
       
       // Configuration
       ReloadConfig,
       
       // System
       Ping,
       Quit,
   }
   ```

3. **Response enum:**

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "type", rename_all = "snake_case")]
   pub enum Response {
       Success {
           #[serde(skip_serializing_if = "Option::is_none")]
           data: Option<serde_json::Value>,
       },
       Error {
           message: String,
           #[serde(skip_serializing_if = "Option::is_none")]
           code: Option<String>,
       },
       Event {
           name: String,
           data: serde_json::Value,
       },
       Pong,
   }
   
   impl Response {
       pub fn success() -> Self {
           Self::Success { data: None }
       }
       
       pub fn success_with_data(data: serde_json::Value) -> Self {
           Self::Success { data: Some(data) }
       }
       
       pub fn error(message: impl Into<String>) -> Self {
           Self::Error {
               message: message.into(),
               code: None,
           }
       }
       
       pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
           Self::Error {
               message: message.into(),
               code: Some(code.into()),
           }
       }
   }
   ```

4. **Data structure definitions:**

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WindowInfo {
       pub hwnd: String,
       pub title: String,
       pub class: String,
       pub process_name: String,
       pub workspace: usize,
       pub monitor: usize,
       pub state: WindowState,
       pub rect: RectInfo,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub focused: Option<bool>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum WindowState {
       Tiled,
       Floating,
       Fullscreen,
       Minimized,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct RectInfo {
       pub x: i32,
       pub y: i32,
       pub width: i32,
       pub height: i32,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WorkspaceInfo {
       pub id: usize,
       pub name: String,
       pub monitor: usize,
       pub window_count: usize,
       pub active: bool,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub visible: Option<bool>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct MonitorInfo {
       pub id: usize,
       pub name: String,
       pub width: i32,
       pub height: i32,
       pub x: i32,
       pub y: i32,
       pub scale: f32,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub primary: Option<bool>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub active_workspace: Option<usize>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ConfigInfo {
       pub version: String,
       pub config_path: String,
       pub workspaces_count: usize,
       pub layouts: Vec<String>,
       pub current_layout: String,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct VersionInfo {
       pub version: String,
       pub build_date: String,
       pub git_commit: Option<String>,
       pub rustc_version: String,
   }
   ```

5. **Protocol version constant:**

   ```rust
   pub const PROTOCOL_VERSION: &str = "1.0.0";
   
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
   ```

**Acceptance Criteria:**
- [ ] All request types compile without errors
- [ ] All response types compile without errors
- [ ] Serialization to JSON works correctly
- [ ] Deserialization from JSON works correctly
- [ ] Optional fields are handled properly
- [ ] Protocol version is included
- [ ] Data structures are comprehensive
- [ ] Documentation is complete for all types

**Testing Requirements:**

Create `crates/core/src/ipc/protocol_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_serialization() {
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
    fn test_execute_request() {
        let request = Request::Execute {
            command: "workspace".to_string(),
            args: vec!["3".to_string()],
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("execute"));
        assert!(json.contains("workspace"));
    }
    
    #[test]
    fn test_response_success() {
        let response = Response::success();
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("success"));
    }
    
    #[test]
    fn test_response_error() {
        let response = Response::error("Test error");
        let json = serde_json::to_string(&response).unwrap();
        
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
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
        println!("{}", json);
        
        let deserialized: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hwnd, "12345");
        assert_eq!(deserialized.title, "Test Window");
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
            }
            _ => panic!("Wrong request type"),
        }
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core ipc::protocol
cargo clippy -p tiling-wm-core -- -D warnings
```

**Expected Output:**
- All tests pass
- JSON serialization is correct
- Protocol structures are properly defined

---

#### Task 5.2: Implement Event System

**Objective:** Create event broadcasting system for real-time notifications to IPC clients.

**File:** `crates/core/src/ipc/events.rs`

**Required Implementations:**

```rust
use super::protocol::Response;
use tokio::sync::broadcast::{channel, Sender, Receiver};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub enum Event {
    WindowCreated {
        hwnd: isize,
        title: String,
        workspace: usize,
    },
    WindowClosed {
        hwnd: isize,
    },
    WindowFocused {
        hwnd: isize,
    },
    WindowMoved {
        hwnd: isize,
        from_workspace: usize,
        to_workspace: usize,
    },
    WindowStateChanged {
        hwnd: isize,
        old_state: String,
        new_state: String,
    },
    WorkspaceChanged {
        from: usize,
        to: usize,
    },
    WorkspaceCreated {
        id: usize,
        name: String,
    },
    WorkspaceDeleted {
        id: usize,
    },
    MonitorChanged,
    ConfigReloaded,
    LayoutChanged {
        layout: String,
    },
}

pub struct EventBroadcaster {
    sender: Sender<Event>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = channel(100);
        Self {
            sender: tx,
        }
    }
    
    pub fn emit(&self, event: Event) {
        tracing::debug!("Broadcasting event: {:?}", event);
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> Receiver<Event> {
        self.sender.subscribe()
    }
    
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Event {
    pub fn to_response(&self) -> Response {
        let (name, data) = match self {
            Event::WindowCreated { hwnd, title, workspace } => {
                ("window_created", json!({
                    "hwnd": format!("{}", hwnd),
                    "title": title,
                    "workspace": workspace,
                }))
            }
            Event::WindowClosed { hwnd } => {
                ("window_closed", json!({ "hwnd": format!("{}", hwnd) }))
            }
            Event::WindowFocused { hwnd } => {
                ("window_focused", json!({ "hwnd": format!("{}", hwnd) }))
            }
            Event::WindowMoved { hwnd, from_workspace, to_workspace } => {
                ("window_moved", json!({
                    "hwnd": format!("{}", hwnd),
                    "from_workspace": from_workspace,
                    "to_workspace": to_workspace,
                }))
            }
            Event::WindowStateChanged { hwnd, old_state, new_state } => {
                ("window_state_changed", json!({
                    "hwnd": format!("{}", hwnd),
                    "old_state": old_state,
                    "new_state": new_state,
                }))
            }
            Event::WorkspaceChanged { from, to } => {
                ("workspace_changed", json!({ "from": from, "to": to }))
            }
            Event::WorkspaceCreated { id, name } => {
                ("workspace_created", json!({ "id": id, "name": name }))
            }
            Event::WorkspaceDeleted { id } => {
                ("workspace_deleted", json!({ "id": id }))
            }
            Event::MonitorChanged => {
                ("monitor_changed", json!({}))
            }
            Event::ConfigReloaded => {
                ("config_reloaded", json!({}))
            }
            Event::LayoutChanged { layout } => {
                ("layout_changed", json!({ "layout": layout }))
            }
        };
        
        Response::Event {
            name: name.to_string(),
            data,
        }
    }
    
    pub fn event_name(&self) -> &str {
        match self {
            Event::WindowCreated { .. } => "window_created",
            Event::WindowClosed { .. } => "window_closed",
            Event::WindowFocused { .. } => "window_focused",
            Event::WindowMoved { .. } => "window_moved",
            Event::WindowStateChanged { .. } => "window_state_changed",
            Event::WorkspaceChanged { .. } => "workspace_changed",
            Event::WorkspaceCreated { .. } => "workspace_created",
            Event::WorkspaceDeleted { .. } => "workspace_deleted",
            Event::MonitorChanged => "monitor_changed",
            Event::ConfigReloaded => "config_reloaded",
            Event::LayoutChanged { .. } => "layout_changed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_broadcaster() {
        let broadcaster = EventBroadcaster::new();
        let mut receiver = broadcaster.subscribe();
        
        let event = Event::WindowCreated {
            hwnd: 12345,
            title: "Test Window".to_string(),
            workspace: 1,
        };
        
        broadcaster.emit(event.clone());
        
        // Try to receive the event (non-blocking test)
        match receiver.try_recv() {
            Ok(received) => {
                assert_eq!(received.event_name(), "window_created");
            }
            Err(_) => {
                // Event may have been dropped if no receivers at time of send
            }
        }
    }
    
    #[test]
    fn test_event_to_response() {
        let event = Event::WindowCreated {
            hwnd: 12345,
            title: "Test Window".to_string(),
            workspace: 1,
        };
        
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "window_created");
                assert!(data.get("hwnd").is_some());
                assert!(data.get("title").is_some());
                assert!(data.get("workspace").is_some());
            }
            _ => panic!("Expected Event response"),
        }
    }
    
    #[test]
    fn test_workspace_changed_event() {
        let event = Event::WorkspaceChanged { from: 1, to: 2 };
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "workspace_changed");
                assert_eq!(data["from"], 1);
                assert_eq!(data["to"], 2);
            }
            _ => panic!("Expected Event response"),
        }
    }
    
    #[test]
    fn test_event_names() {
        let event = Event::ConfigReloaded;
        assert_eq!(event.event_name(), "config_reloaded");
        
        let event = Event::MonitorChanged;
        assert_eq!(event.event_name(), "monitor_changed");
    }
}
```

**Acceptance Criteria:**
- [ ] EventBroadcaster can emit events
- [ ] Clients can subscribe to events
- [ ] Events convert to proper Response format
- [ ] Event names are correct
- [ ] Subscriber count tracking works
- [ ] Broadcast channel has proper capacity
- [ ] Events include all necessary data

**Testing Requirements:**
- All event tests pass
- Broadcast mechanism works
- Event serialization is correct

---

### Week 18: Named Pipe Server Implementation

#### Task 5.3: Implement Named Pipe IPC Server

**Objective:** Create the named pipe server that handles IPC connections and processes requests.

**File:** `crates/core/src/ipc/server.rs`

**Required Implementations:**

```rust
use super::protocol::{Request, Response};
use super::events::{Event, EventBroadcaster};
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use anyhow::{Result, Context};
use tracing::{info, error, debug, warn};

pub struct IpcServer {
    pipe_name: String,
    event_broadcaster: Arc<EventBroadcaster>,
    running: Arc<RwLock<bool>>,
    connection_count: Arc<Mutex<usize>>,
}

impl IpcServer {
    pub fn new(event_broadcaster: Arc<EventBroadcaster>) -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
            event_broadcaster,
            running: Arc::new(RwLock::new(false)),
            connection_count: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn with_pipe_name(mut self, name: impl Into<String>) -> Self {
        self.pipe_name = format!(r"\\.\pipe\{}", name.into());
        self
    }
    
    pub async fn start(self: Arc<Self>) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting IPC server on {}", self.pipe_name);
        
        loop {
            // Check if we should stop
            if !*self.running.read().await {
                break;
            }
            
            // Create server instance
            let server = match ServerOptions::new()
                .first_pipe_instance(false)
                .create(&self.pipe_name)
            {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create named pipe: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
            };
            
            let server_clone = Arc::clone(&self);
            
            // Spawn handler for this connection
            tokio::spawn(async move {
                if let Err(e) = server_clone.handle_client(server).await {
                    error!("Client handler error: {}", e);
                }
            });
        }
        
        info!("IPC server stopped");
        Ok(())
    }
    
    pub async fn stop(&self) {
        info!("Stopping IPC server");
        let mut running = self.running.write().await;
        *running = false;
    }
    
    async fn handle_client(&self, server: NamedPipeServer) -> Result<()> {
        // Increment connection count
        {
            let mut count = self.connection_count.lock().await;
            *count += 1;
            debug!("Client connected. Total connections: {}", *count);
        }
        
        let result = self.process_client(server).await;
        
        // Decrement connection count
        {
            let mut count = self.connection_count.lock().await;
            *count -= 1;
            debug!("Client disconnected. Total connections: {}", *count);
        }
        
        result
    }
    
    async fn process_client(&self, mut server: NamedPipeServer) -> Result<()> {
        // Wait for client to connect
        server.connect().await
            .context("Failed to connect to client")?;
        
        let mut reader = BufReader::new(&mut server);
        let mut writer = BufWriter::new(&mut server);
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        loop {
            // If subscribed, wait for either a request or an event
            if subscribed {
                tokio::select! {
                    // Handle incoming requests
                    request_result = Self::read_request(&mut reader) => {
                        match request_result {
                            Ok(Some(request)) => {
                                let response = self.process_request(request, &mut subscribed, &mut event_receiver).await;
                                Self::write_response(&mut writer, &response).await?;
                            }
                            Ok(None) => break, // Client disconnected
                            Err(e) => {
                                error!("Failed to read request: {}", e);
                                break;
                            }
                        }
                    }
                    
                    // Forward events to client
                    event = Self::receive_event(&mut event_receiver) => {
                        if let Some(evt) = event {
                            let response = evt.to_response();
                            if let Err(e) = Self::write_response(&mut writer, &response).await {
                                error!("Failed to send event: {}", e);
                                break;
                            }
                        }
                    }
                }
            } else {
                // Not subscribed, just handle requests
                match Self::read_request(&mut reader).await? {
                    Some(request) => {
                        let response = self.process_request(request, &mut subscribed, &mut event_receiver).await;
                        Self::write_response(&mut writer, &response).await?;
                    }
                    None => break, // Client disconnected
                }
            }
        }
        
        Ok(())
    }
    
    async fn read_request<R>(reader: &mut R) -> Result<Option<Request>>
    where
        R: AsyncReadExt + Unpin,
    {
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        match reader.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None); // Client disconnected
            }
            Err(e) => return Err(e.into()),
        }
        
        let len = u32::from_le_bytes(len_buf) as usize;
        
        // Sanity check
        if len > 10 * 1024 * 1024 {
            anyhow::bail!("Request too large: {} bytes", len);
        }
        
        // Read request data
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data).await?;
        
        // Parse JSON
        let request: Request = serde_json::from_slice(&data)
            .context("Failed to parse request JSON")?;
        
        Ok(Some(request))
    }
    
    async fn write_response<W>(writer: &mut W, response: &Response) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        // Serialize response
        let data = serde_json::to_vec(response)
            .context("Failed to serialize response")?;
        
        // Write length prefix
        let len = data.len() as u32;
        writer.write_all(&len.to_le_bytes()).await?;
        
        // Write response data
        writer.write_all(&data).await?;
        
        // Flush
        writer.flush().await?;
        
        Ok(())
    }
    
    async fn receive_event(
        event_receiver: &mut Option<tokio::sync::broadcast::Receiver<Event>>
    ) -> Option<Event> {
        if let Some(ref mut receiver) = event_receiver {
            match receiver.recv().await {
                Ok(event) => Some(event),
                Err(_) => None,
            }
        } else {
            // No receiver, wait indefinitely
            std::future::pending().await
        }
    }
    
    async fn process_request(
        &self,
        request: Request,
        subscribed: &mut bool,
        event_receiver: &mut Option<tokio::sync::broadcast::Receiver<Event>>,
    ) -> Response {
        debug!("Processing request: {:?}", request);
        
        match request {
            Request::Ping => Response::Pong,
            
            Request::Subscribe { events } => {
                if events.is_empty() {
                    Response::error("No events specified")
                } else {
                    *subscribed = true;
                    *event_receiver = Some(self.event_broadcaster.subscribe());
                    Response::success_with_data(serde_json::json!({
                        "subscribed": events,
                    }))
                }
            }
            
            Request::Unsubscribe => {
                *subscribed = false;
                *event_receiver = None;
                Response::success()
            }
            
            // Other requests would be forwarded to window manager
            // For now, return placeholder responses
            _ => Response::error("Not implemented"),
        }
    }
    
    pub async fn get_connection_count(&self) -> usize {
        *self.connection_count.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ipc_server_creation() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        assert!(server.pipe_name.contains("tiling-wm"));
    }
    
    #[tokio::test]
    async fn test_custom_pipe_name() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster)
            .with_pipe_name("test-pipe");
        
        assert!(server.pipe_name.contains("test-pipe"));
    }
}
```

**Acceptance Criteria:**
- [ ] Server starts and listens on named pipe
- [ ] Accepts multiple concurrent connections
- [ ] Processes requests correctly
- [ ] Handles client disconnects gracefully
- [ ] Event subscription works
- [ ] Request/response framing is correct
- [ ] Error handling is comprehensive
- [ ] Connection counting works

**Testing Requirements:**
- Server creation tests pass
- Pipe naming tests pass
- Integration tests with actual connections

---

#### Task 5.4: Integrate IPC Server with Window Manager

**Objective:** Connect the IPC server to the window manager to handle actual requests.

**File:** `crates/core/src/ipc/handler.rs`

**Required Implementations:**

```rust
use super::protocol::{Request, Response, WindowInfo, WorkspaceInfo, MonitorInfo, ConfigInfo, VersionInfo};
use crate::window_manager::WindowManager;
use crate::workspace::manager::WorkspaceManager;
use crate::commands::{Command, CommandExecutor};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct RequestHandler {
    window_manager: Arc<Mutex<WindowManager>>,
    workspace_manager: Arc<Mutex<WorkspaceManager>>,
    command_executor: Arc<CommandExecutor>,
}

impl RequestHandler {
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
    
    pub async fn handle_request(&self, request: Request) -> Response {
        match request {
            Request::GetActiveWindow => self.get_active_window().await,
            Request::GetWindows { workspace } => self.get_windows(workspace).await,
            Request::GetWorkspaces => self.get_workspaces().await,
            Request::GetMonitors => self.get_monitors().await,
            Request::GetConfig => self.get_config().await,
            Request::GetVersion => self.get_version().await,
            
            Request::Execute { command, args } => self.execute_command(command, args).await,
            
            Request::CloseWindow { hwnd } => self.close_window(hwnd).await,
            Request::FocusWindow { hwnd } => self.focus_window(hwnd).await,
            Request::MoveWindow { hwnd, workspace } => self.move_window(hwnd, workspace).await,
            Request::ToggleFloating { hwnd } => self.toggle_floating(hwnd).await,
            Request::ToggleFullscreen { hwnd } => self.toggle_fullscreen(hwnd).await,
            
            Request::SwitchWorkspace { id } => self.switch_workspace(id).await,
            Request::CreateWorkspace { name, monitor } => self.create_workspace(name, monitor).await,
            Request::DeleteWorkspace { id } => self.delete_workspace(id).await,
            Request::RenameWorkspace { id, name } => self.rename_workspace(id, name).await,
            
            Request::SetLayout { layout } => self.set_layout(layout).await,
            Request::AdjustMasterFactor { delta } => self.adjust_master_factor(delta).await,
            Request::IncreaseMasterCount => self.increase_master_count().await,
            Request::DecreaseMasterCount => self.decrease_master_count().await,
            
            Request::ReloadConfig => self.reload_config().await,
            
            Request::Ping => Response::Pong,
            Request::Quit => self.quit().await,
            
            _ => Response::error("Request type not handled by this handler"),
        }
    }
    
    async fn get_active_window(&self) -> Response {
        let wm = self.window_manager.lock().await;
        
        if let Some(window) = wm.get_active_window() {
            let info = WindowInfo {
                hwnd: format!("{}", window.handle.0.0),
                title: window.title.clone(),
                class: window.class.clone(),
                process_name: window.process_name.clone(),
                workspace: window.workspace,
                monitor: window.monitor,
                state: match window.state {
                    crate::window_manager::window::WindowState::Tiled => 
                        super::protocol::WindowState::Tiled,
                    crate::window_manager::window::WindowState::Floating => 
                        super::protocol::WindowState::Floating,
                    crate::window_manager::window::WindowState::Fullscreen => 
                        super::protocol::WindowState::Fullscreen,
                    crate::window_manager::window::WindowState::Minimized => 
                        super::protocol::WindowState::Minimized,
                },
                rect: super::protocol::RectInfo {
                    x: 0,  // TODO: Get actual rect
                    y: 0,
                    width: 0,
                    height: 0,
                },
                focused: Some(true),
            };
            
            Response::success_with_data(serde_json::to_value(info).unwrap())
        } else {
            Response::error("No active window")
        }
    }
    
    async fn get_windows(&self, workspace: Option<usize>) -> Response {
        // Implementation to get all windows, optionally filtered by workspace
        Response::success_with_data(serde_json::json!([]))
    }
    
    async fn get_workspaces(&self) -> Response {
        let wsm = self.workspace_manager.lock().await;
        let workspaces: Vec<WorkspaceInfo> = wsm.get_all_workspaces()
            .iter()
            .map(|ws| WorkspaceInfo {
                id: ws.id,
                name: ws.name.clone(),
                monitor: ws.monitor,
                window_count: ws.window_count(),
                active: ws.visible,
                visible: Some(ws.visible),
            })
            .collect();
        
        Response::success_with_data(serde_json::to_value(workspaces).unwrap())
    }
    
    async fn get_monitors(&self) -> Response {
        // Implementation to get monitor information
        Response::success_with_data(serde_json::json!([]))
    }
    
    async fn get_config(&self) -> Response {
        // Implementation to get config information
        Response::success_with_data(serde_json::json!({}))
    }
    
    async fn get_version(&self) -> Response {
        let info = VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: env!("BUILD_DATE").unwrap_or("unknown").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(String::from),
            rustc_version: env!("RUSTC_VERSION").unwrap_or("unknown").to_string(),
        };
        
        Response::success_with_data(serde_json::to_value(info).unwrap())
    }
    
    async fn execute_command(&self, command: String, args: Vec<String>) -> Response {
        // Parse command string into Command enum and execute
        Response::success()
    }
    
    async fn close_window(&self, hwnd: Option<String>) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn focus_window(&self, hwnd: String) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn move_window(&self, hwnd: String, workspace: usize) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn toggle_floating(&self, hwnd: Option<String>) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn toggle_fullscreen(&self, hwnd: Option<String>) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn switch_workspace(&self, id: usize) -> Response {
        let mut wsm = self.workspace_manager.lock().await;
        match wsm.switch_to(id) {
            Ok(_) => Response::success(),
            Err(e) => Response::error(format!("Failed to switch workspace: {}", e)),
        }
    }
    
    async fn create_workspace(&self, name: String, monitor: usize) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn delete_workspace(&self, id: usize) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn rename_workspace(&self, id: usize, name: String) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn set_layout(&self, layout: String) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn adjust_master_factor(&self, delta: f32) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn increase_master_count(&self) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn decrease_master_count(&self) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn reload_config(&self) -> Response {
        // Implementation
        Response::success()
    }
    
    async fn quit(&self) -> Response {
        // Signal application to quit
        Response::success()
    }
}
```

**Acceptance Criteria:**
- [ ] All request types are handled
- [ ] Queries return correct data
- [ ] Commands execute successfully
- [ ] Error messages are informative
- [ ] Handler integrates with window manager
- [ ] Async/await is used correctly

---

### Week 19: CLI Client Implementation

#### Task 5.5: Create CLI Client Application

**Objective:** Build a command-line client that communicates with the IPC server.

**File:** `crates/cli/Cargo.toml`

First, update the CLI crate dependencies:

```toml
[package]
name = "tiling-wm-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "twm"
path = "src/main.rs"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
clap = { version = "4.4", features = ["derive", "cargo"] }
tabling = "0.15"  # For table formatting
colored = "2.1"   # For colored output
```

**File:** `crates/cli/src/main.rs`

```rust
use clap::{Parser, Subcommand, ValueEnum};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use anyhow::{Result, Context};
use tabling::{Table, Tabled, settings::Style};
use colored::*;

#[derive(Parser)]
#[command(name = "twm")]
#[command(about = "Tiling Window Manager CLI", long_about = None)]
#[command(version)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,
    
    /// Named pipe path
    #[arg(long, default_value = r"\\.\pipe\tiling-wm")]
    pipe: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Compact,
}

#[derive(Subcommand)]
enum Commands {
    /// Get list of windows
    Windows {
        /// Filter by workspace
        #[arg(short, long)]
        workspace: Option<usize>,
    },
    
    /// Get active window information
    ActiveWindow,
    
    /// Get list of workspaces
    Workspaces,
    
    /// Get list of monitors
    Monitors,
    
    /// Get configuration info
    Config,
    
    /// Get version information
    Version,
    
    /// Execute a command
    #[command(subcommand)]
    Exec(ExecCommands),
    
    /// Switch to workspace
    Workspace {
        /// Workspace ID
        id: usize,
    },
    
    /// Close active or specified window
    Close {
        /// Window HWND (hex or decimal)
        #[arg(short, long)]
        window: Option<String>,
    },
    
    /// Focus a window
    Focus {
        /// Window HWND (hex or decimal)
        window: String,
    },
    
    /// Move window to workspace
    Move {
        /// Window HWND
        window: String,
        /// Target workspace ID
        workspace: usize,
    },
    
    /// Toggle floating for active or specified window
    ToggleFloat {
        /// Window HWND
        #[arg(short, long)]
        window: Option<String>,
    },
    
    /// Toggle fullscreen for active or specified window
    ToggleFullscreen {
        /// Window HWND
        #[arg(short, long)]
        window: Option<String>,
    },
    
    /// Create a new workspace
    CreateWorkspace {
        /// Workspace name
        name: String,
        /// Monitor ID
        #[arg(short, long, default_value = "0")]
        monitor: usize,
    },
    
    /// Delete a workspace
    DeleteWorkspace {
        /// Workspace ID
        id: usize,
    },
    
    /// Rename a workspace
    RenameWorkspace {
        /// Workspace ID
        id: usize,
        /// New name
        name: String,
    },
    
    /// Set layout
    Layout {
        /// Layout name (dwindle, master)
        name: String,
    },
    
    /// Reload configuration
    Reload,
    
    /// Subscribe to events (listen mode)
    Listen {
        /// Events to subscribe to
        #[arg(short, long, value_delimiter = ',')]
        events: Vec<String>,
    },
    
    /// Ping the server
    Ping,
}

#[derive(Subcommand)]
enum ExecCommands {
    /// Adjust master factor
    MasterFactor {
        /// Delta value
        delta: f32,
    },
    
    /// Increase master count
    IncreaseMaster,
    
    /// Decrease master count
    DecreaseMaster,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Connect to named pipe
    let mut client = ClientOptions::new()
        .open(&cli.pipe)
        .context("Failed to connect to window manager. Is it running?")?;
    
    // Build request
    let request = build_request(&cli.command)?;
    
    // Send request
    send_request(&mut client, &request).await?;
    
    // Handle response based on command type
    match cli.command {
        Commands::Listen { .. } => {
            // Listen mode: keep receiving events
            loop {
                let response = receive_response(&mut client).await?;
                print_response(&response, cli.format);
            }
        }
        _ => {
            // Single request: receive one response
            let response = receive_response(&mut client).await?;
            print_response(&response, cli.format);
        }
    }
    
    Ok(())
}

fn build_request(command: &Commands) -> Result<Value> {
    let request = match command {
        Commands::Windows { workspace } => {
            serde_json::json!({
                "type": "get_windows",
                "workspace": workspace,
            })
        }
        Commands::ActiveWindow => {
            serde_json::json!({
                "type": "get_active_window"
            })
        }
        Commands::Workspaces => {
            serde_json::json!({
                "type": "get_workspaces"
            })
        }
        Commands::Monitors => {
            serde_json::json!({
                "type": "get_monitors"
            })
        }
        Commands::Config => {
            serde_json::json!({
                "type": "get_config"
            })
        }
        Commands::Version => {
            serde_json::json!({
                "type": "get_version"
            })
        }
        Commands::Workspace { id } => {
            serde_json::json!({
                "type": "switch_workspace",
                "id": id,
            })
        }
        Commands::Close { window } => {
            serde_json::json!({
                "type": "close_window",
                "hwnd": window,
            })
        }
        Commands::Focus { window } => {
            serde_json::json!({
                "type": "focus_window",
                "hwnd": window,
            })
        }
        Commands::Move { window, workspace } => {
            serde_json::json!({
                "type": "move_window",
                "hwnd": window,
                "workspace": workspace,
            })
        }
        Commands::ToggleFloat { window } => {
            serde_json::json!({
                "type": "toggle_floating",
                "hwnd": window,
            })
        }
        Commands::ToggleFullscreen { window } => {
            serde_json::json!({
                "type": "toggle_fullscreen",
                "hwnd": window,
            })
        }
        Commands::CreateWorkspace { name, monitor } => {
            serde_json::json!({
                "type": "create_workspace",
                "name": name,
                "monitor": monitor,
            })
        }
        Commands::DeleteWorkspace { id } => {
            serde_json::json!({
                "type": "delete_workspace",
                "id": id,
            })
        }
        Commands::RenameWorkspace { id, name } => {
            serde_json::json!({
                "type": "rename_workspace",
                "id": id,
                "name": name,
            })
        }
        Commands::Layout { name } => {
            serde_json::json!({
                "type": "set_layout",
                "layout": name,
            })
        }
        Commands::Reload => {
            serde_json::json!({
                "type": "reload_config"
            })
        }
        Commands::Listen { events } => {
            serde_json::json!({
                "type": "subscribe",
                "events": events,
            })
        }
        Commands::Ping => {
            serde_json::json!({
                "type": "ping"
            })
        }
        Commands::Exec(exec_cmd) => {
            match exec_cmd {
                ExecCommands::MasterFactor { delta } => {
                    serde_json::json!({
                        "type": "adjust_master_factor",
                        "delta": delta,
                    })
                }
                ExecCommands::IncreaseMaster => {
                    serde_json::json!({
                        "type": "increase_master_count"
                    })
                }
                ExecCommands::DecreaseMaster => {
                    serde_json::json!({
                        "type": "decrease_master_count"
                    })
                }
            }
        }
    };
    
    Ok(request)
}

async fn send_request<W>(writer: &mut W, request: &Value) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let data = serde_json::to_vec(request)?;
    let len = data.len() as u32;
    
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(&data).await?;
    writer.flush().await?;
    
    Ok(())
}

async fn receive_response<R>(reader: &mut R) -> Result<Value>
where
    R: AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await?;
    
    let response: Value = serde_json::from_slice(&data)?;
    Ok(response)
}

fn print_response(response: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Table => {
            print_table(response);
        }
        OutputFormat::Compact => {
            print_compact(response);
        }
    }
}

fn print_table(response: &Value) {
    // Check response type
    if let Some(response_type) = response.get("type").and_then(|t| t.as_str()) {
        match response_type {
            "success" => {
                if let Some(data) = response.get("data") {
                    // Try to format as table if it's an array
                    if let Some(arr) = data.as_array() {
                        if !arr.is_empty() {
                            // Format array as table
                            println!("{}", "Success".green());
                            // Table formatting would go here
                        }
                    } else {
                        println!("{}: {}", "Success".green(), data);
                    }
                } else {
                    println!("{}", "Success".green());
                }
            }
            "error" => {
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("{}: {}", "Error".red(), message);
            }
            "event" => {
                let name = response.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let data = response.get("data").unwrap_or(&Value::Null);
                println!("{} {}: {}", "Event".cyan(), name, data);
            }
            "pong" => {
                println!("{}", "Pong".green());
            }
            _ => {
                println!("{}", serde_json::to_string_pretty(response).unwrap());
            }
        }
    }
}

fn print_compact(response: &Value) {
    if let Some(response_type) = response.get("type").and_then(|t| t.as_str()) {
        match response_type {
            "success" => {
                if let Some(data) = response.get("data") {
                    println!("{}", serde_json::to_string(data).unwrap());
                } else {
                    println!("ok");
                }
            }
            "error" => {
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("error");
                eprintln!("{}", message);
            }
            "event" => {
                println!("{}", serde_json::to_string(response).unwrap());
            }
            "pong" => {
                println!("pong");
            }
            _ => {
                println!("{}", serde_json::to_string(response).unwrap());
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] CLI compiles without errors
- [ ] All commands are implemented
- [ ] Can connect to IPC server
- [ ] Requests are sent correctly
- [ ] Responses are received and parsed
- [ ] Output formatting works (JSON, table, compact)
- [ ] Error messages are helpful
- [ ] Help text is comprehensive

**Testing Requirements:**
- Manual testing with running window manager
- Test all CLI commands
- Test different output formats
- Test error cases (server not running, etc.)

---

### Week 20: Integration, Testing, and Documentation

#### Task 5.6: Create Example Scripts

**Objective:** Create example scripts demonstrating IPC usage for common tasks.

**File:** `examples/ipc/README.md`

```markdown
# IPC Examples

This directory contains example scripts demonstrating how to interact with the Tiling Window Manager via IPC.

## Prerequisites

- Window manager running with IPC enabled
- `twm` CLI tool installed

## Examples

### 1. Query Window Information

```bash
# Get all windows
twm windows

# Get windows in specific workspace
twm windows --workspace 1

# Get active window
twm active-window
```

### 2. Workspace Management

```bash
# List all workspaces
twm workspaces

# Switch to workspace 3
twm workspace 3

# Create new workspace
twm create-workspace "Development" --monitor 0

# Rename workspace
twm rename-workspace 1 "Main"
```

### 3. Window Operations

```bash
# Close active window
twm close

# Toggle floating
twm toggle-float

# Toggle fullscreen
twm toggle-fullscreen

# Move window to workspace 2
twm move <hwnd> 2
```

### 4. Layout Control

```bash
# Set layout
twm layout dwindle
twm layout master

# Adjust master factor
twm exec master-factor 0.05

# Change master count
twm exec increase-master
twm exec decrease-master
```

### 5. Configuration

```bash
# Reload configuration
twm reload

# Get configuration info
twm config

# Get version
twm version
```

### 6. Event Subscription

```bash
# Listen to all events
twm listen --events window_created,workspace_changed,config_reloaded

# Monitor in JSON format
twm --format json listen --events window_created
```

## PowerShell Scripts

### Auto-tile new windows

```powershell
# monitor-windows.ps1
$events = "window_created"
& twm listen --events $events | ForEach-Object {
    $event = $_ | ConvertFrom-Json
    Write-Host "New window: $($event.data.title)"
}
```

### Workspace switcher

```powershell
# switch-workspace.ps1
param([int]$WorkspaceId)

& twm workspace $WorkspaceId
if ($LASTEXITCODE -eq 0) {
    Write-Host "Switched to workspace $WorkspaceId" -ForegroundColor Green
} else {
    Write-Host "Failed to switch workspace" -ForegroundColor Red
}
```

## Python Scripts

### Window monitor

```python
#!/usr/bin/env python3
import subprocess
import json

def monitor_windows():
    proc = subprocess.Popen(
        ['twm', '--format', 'json', 'listen', '--events', 'window_created,window_closed'],
        stdout=subprocess.PIPE,
        text=True
    )
    
    for line in proc.stdout:
        event = json.loads(line)
        print(f"Event: {event['name']}")
        print(f"Data: {event['data']}")

if __name__ == '__main__':
    monitor_windows()
```

### Workspace status

```python
#!/usr/bin/env python3
import subprocess
import json

def get_workspaces():
    result = subprocess.run(
        ['twm', '--format', 'json', 'workspaces'],
        capture_output=True,
        text=True
    )
    
    response = json.loads(result.stdout)
    if response['type'] == 'success':
        workspaces = response['data']
        for ws in workspaces:
            status = "●" if ws['active'] else "○"
            print(f"{status} Workspace {ws['id']}: {ws['name']} ({ws['window_count']} windows)")

if __name__ == '__main__':
    get_workspaces()
```
```

**Create example files:**
- `examples/ipc/powershell/monitor-windows.ps1`
- `examples/ipc/powershell/switch-workspace.ps1`
- `examples/ipc/python/window-monitor.py`
- `examples/ipc/python/workspace-status.py`

**Acceptance Criteria:**
- [ ] Example scripts are functional
- [ ] Scripts demonstrate key features
- [ ] Documentation is clear
- [ ] Scripts handle errors gracefully
- [ ] Both PowerShell and Python examples provided

---

#### Task 5.7: Write IPC Documentation

**Objective:** Create comprehensive documentation for the IPC protocol and CLI.

**File:** `docs/IPC.md`

```markdown
# IPC Protocol Documentation

## Overview

The Tiling Window Manager provides an IPC (Inter-Process Communication) interface for external control and automation. The IPC uses a JSON-based protocol over Windows named pipes.

## Connection

**Named Pipe:** `\\.\pipe\tiling-wm`

### Connecting from PowerShell

```powershell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "tiling-wm", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect()
```

### Connecting from Python

```python
import win32pipe
import win32file

pipe = win32file.CreateFile(
    r'\\.\pipe\tiling-wm',
    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
    0, None,
    win32file.OPEN_EXISTING,
    0, None
)
```

## Protocol Format

Messages are framed with a 4-byte length prefix (little-endian uint32) followed by JSON data.

```
[4 bytes: length] [N bytes: JSON payload]
```

## Request Types

### Query Requests

#### Get Active Window
```json
{
  "type": "get_active_window"
}
```

Response:
```json
{
  "type": "success",
  "data": {
    "hwnd": "12345",
    "title": "Window Title",
    "class": "WindowClass",
    "process_name": "process.exe",
    "workspace": 1,
    "monitor": 0,
    "state": "tiled",
    "rect": {
      "x": 0,
      "y": 0,
      "width": 1920,
      "height": 1080
    },
    "focused": true
  }
}
```

#### Get Windows
```json
{
  "type": "get_windows",
  "workspace": 1  // optional
}
```

#### Get Workspaces
```json
{
  "type": "get_workspaces"
}
```

Response:
```json
{
  "type": "success",
  "data": [
    {
      "id": 1,
      "name": "Workspace 1",
      "monitor": 0,
      "window_count": 3,
      "active": true,
      "visible": true
    }
  ]
}
```

### Command Requests

#### Switch Workspace
```json
{
  "type": "switch_workspace",
  "id": 2
}
```

#### Toggle Floating
```json
{
  "type": "toggle_floating",
  "hwnd": "12345"  // optional, defaults to active window
}
```

#### Move Window
```json
{
  "type": "move_window",
  "hwnd": "12345",
  "workspace": 2
}
```

### Event Subscription

#### Subscribe
```json
{
  "type": "subscribe",
  "events": ["window_created", "workspace_changed"]
}
```

Response:
```json
{
  "type": "success",
  "data": {
    "subscribed": ["window_created", "workspace_changed"]
  }
}
```

#### Unsubscribe
```json
{
  "type": "unsubscribe"
}
```

## Event Types

When subscribed, the server sends events in this format:

```json
{
  "type": "event",
  "name": "window_created",
  "data": {
    "hwnd": "12345",
    "title": "New Window",
    "workspace": 1
  }
}
```

### Available Events

- `window_created`: New window opened
- `window_closed`: Window closed
- `window_focused`: Window gained focus
- `window_moved`: Window moved to different workspace
- `window_state_changed`: Window state changed (tiled/floating/fullscreen)
- `workspace_changed`: Active workspace changed
- `workspace_created`: New workspace created
- `workspace_deleted`: Workspace deleted
- `monitor_changed`: Monitor configuration changed
- `config_reloaded`: Configuration reloaded
- `layout_changed`: Layout changed

## Error Handling

Errors are returned in this format:

```json
{
  "type": "error",
  "message": "Error description",
  "code": "ERROR_CODE"  // optional
}
```

## CLI Tool

The `twm` CLI tool provides a convenient interface to the IPC system.

### Installation

```bash
cargo install --path crates/cli
```

### Basic Usage

```bash
# Get help
twm --help

# List windows
twm windows

# Switch workspace
twm workspace 3

# Listen to events
twm listen --events window_created,workspace_changed
```

### Output Formats

```bash
# JSON output
twm --format json workspaces

# Table output (default)
twm --format table workspaces

# Compact output
twm --format compact workspaces
```

## Security Considerations

- Named pipes are local-only (cannot be accessed remotely)
- No authentication required (running as same user)
- Commands execute with same privileges as window manager
- Consider implementing access control for production use

## Protocol Version

Current protocol version: **1.0.0**

The protocol version is included in the `GetVersion` response.
```

**Acceptance Criteria:**
- [ ] Documentation is comprehensive
- [ ] All request types are documented
- [ ] All event types are documented
- [ ] Examples are provided
- [ ] Security considerations are mentioned
- [ ] Protocol format is clearly explained

---

## Phase 5 Completion Checklist

### Build & Compilation
- [ ] `cargo build --workspace` succeeds without errors
- [ ] `cargo build --workspace --release` succeeds
- [ ] No warnings from `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted with `cargo fmt --workspace --check`

### Core Functionality
- [ ] IPC protocol structures compile correctly
- [ ] Protocol serialization/deserialization works
- [ ] Named pipe server starts and listens
- [ ] Server accepts multiple connections
- [ ] Requests are processed correctly
- [ ] Responses are formatted properly
- [ ] Event broadcasting works
- [ ] Event subscription/unsubscription works
- [ ] CLI client compiles and runs
- [ ] All CLI commands work
- [ ] Output formatting works correctly

### Testing
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Protocol serialization tests pass
- [ ] Server logic tests pass
- [ ] Integration tests pass
- [ ] End-to-end IPC tests pass
- [ ] CLI tests pass
- [ ] No test failures or panics

### Integration
- [ ] IPC server starts with window manager
- [ ] All window manager state is accessible via IPC
- [ ] All commands work via IPC
- [ ] Events are broadcast for all relevant actions
- [ ] Performance impact is minimal
- [ ] Memory usage is reasonable
- [ ] No race conditions or deadlocks

### Documentation
- [ ] All public APIs have doc comments
- [ ] `cargo doc --no-deps` builds successfully
- [ ] IPC protocol documentation complete
- [ ] CLI documentation complete
- [ ] Example scripts work correctly
- [ ] README updated with Phase 5 features

### Manual Validation
- [ ] Start window manager with IPC enabled
- [ ] Connect with CLI tool
- [ ] Execute query commands (windows, workspaces, monitors)
- [ ] Execute window commands (close, focus, move)
- [ ] Execute workspace commands (switch, create, delete)
- [ ] Subscribe to events and verify they're received
- [ ] Test with multiple concurrent connections
- [ ] Verify error handling with invalid requests
- [ ] Test performance under load
- [ ] Application runs stable for 15+ minutes with IPC active
- [ ] CPU usage remains reasonable
- [ ] Memory usage is stable

---

## Deliverables for Phase 5

At the end of Phase 5, you should have:

1. **Complete IPC Protocol:**
   - JSON-based protocol specification
   - Request and response types for all operations
   - Event types for all relevant actions
   - Protocol versioning
   - Comprehensive documentation

2. **Named Pipe Server:**
   - Async server implementation
   - Multiple concurrent connection support
   - Request processing and routing
   - Event broadcasting
   - Error handling and recovery

3. **CLI Client Application:**
   - Full-featured command-line tool
   - All query commands
   - All control commands
   - Event listening mode
   - Multiple output formats
   - Comprehensive help text

4. **Event System:**
   - Event broadcaster
   - Subscription management
   - Event filtering
   - Real-time event delivery

5. **Integration:**
   - IPC server integrated with window manager
   - All state accessible via IPC
   - All commands executable via IPC
   - Events emitted for all actions

6. **Documentation and Examples:**
   - IPC protocol documentation
   - CLI documentation
   - Example PowerShell scripts
   - Example Python scripts
   - Usage guides

7. **Quality Assurance:**
   - Comprehensive unit tests
   - Integration tests
   - Manual validation
   - Performance testing
   - Documentation complete

---

## Success Criteria Summary

Phase 5 is complete when:

1. ✅ **IPC protocol is fully defined:**
   - All request types documented and implemented
   - All response types working correctly
   - Event types complete and tested
   - Protocol versioning in place

2. ✅ **Named pipe server is operational:**
   - Starts and listens on named pipe
   - Handles multiple concurrent connections
   - Processes all request types
   - Broadcasts events to subscribers
   - Error handling is robust

3. ✅ **CLI client is fully functional:**
   - All commands implemented
   - Output formatting works
   - Error handling is user-friendly
   - Help documentation is complete

4. ✅ **Event system works correctly:**
   - All events are broadcast
   - Subscription mechanism works
   - Event filtering functions
   - No event loss under normal load

5. ✅ **Integration is complete:**
   - Window manager exposes all functionality
   - No performance degradation
   - Stable under load
   - Memory efficient

6. ✅ **Quality standards met:**
   - All tests passing
   - No clippy warnings
   - Documentation complete
   - Examples work correctly

---

## Next Steps

After completing Phase 5, proceed to **Phase 6: Status Bar Implementation** (Weeks 21-26), which will implement:

- Separate status bar application
- Modular widget system
- System information widgets (CPU, memory, network, battery)
- Workspace indicator
- Window title display
- Custom modules support
- IPC integration with window manager
- CSS-like styling

See DETAILED_ROADMAP.md for Phase 6 specifications.

---

## Troubleshooting

### Common Issues

**Issue: Cannot connect to named pipe**
- Solution: Verify window manager is running
- Check pipe name is correct (`\\.\pipe\tiling-wm`)
- Ensure you have necessary permissions
- Try running as administrator if needed

**Issue: Requests timeout**
- Solution: Check server is processing requests
- Verify no deadlocks in handler code
- Check async runtime is configured correctly
- Increase timeout values if needed

**Issue: Events not received**
- Solution: Verify subscription was successful
- Check event names are correct
- Ensure connection is still alive
- Test with simple events first

**Issue: JSON parsing errors**
- Solution: Validate JSON format
- Check protocol version compatibility
- Verify all required fields are present
- Test with CLI tool first

**Issue: CLI tool crashes**
- Solution: Check window manager is running
- Verify pipe path is correct
- Test with simple commands first
- Check for networking/permission issues

**Issue: High memory usage with many connections**
- Solution: Implement connection limits
- Add timeout for idle connections
- Review event subscriber count
- Check for memory leaks in handlers

---

## Notes for Autonomous Agents

When executing this task list:

1. **Follow order strictly**: IPC tasks build on each other
2. **Validate each step**: Run tests after each task
3. **Test incrementally**: Don't wait until end to test
4. **Handle async carefully**: Tokio runtime must be configured correctly
5. **Test with real connections**: Use named pipe clients for testing
6. **Check compatibility**: Windows named pipes are platform-specific
7. **Monitor performance**: IPC shouldn't impact main application
8. **Handle errors gracefully**: IPC errors shouldn't crash window manager
9. **Document extensively**: IPC protocol needs clear documentation
10. **Reference phases 1-4**: Build on existing command system

---

**End of Phase 5 Task Document**
