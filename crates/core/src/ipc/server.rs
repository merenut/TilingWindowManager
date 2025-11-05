//! IPC server implementation using Windows named pipes.
//!
//! This module provides an async server that listens for IPC connections over Windows
//! named pipes, processes requests, and broadcasts events to subscribed clients.
//!
//! # Features
//!
//! - Async server implementation using tokio
//! - Multiple concurrent client connections
//! - Request/response framing with 4-byte length prefix
//! - Event subscription and broadcasting
//! - Graceful connection and disconnection handling
//! - Connection counting and tracking
//! - Robust error handling
//!
//! # Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use tiling_wm_core::ipc::server::IpcServer;
//! use tiling_wm_core::ipc::EventBroadcaster;
//!
//! #[tokio::main]
//! async fn main() {
//!     let broadcaster = Arc::new(EventBroadcaster::new());
//!     let server = Arc::new(IpcServer::new(broadcaster));
//!     
//!     // Start server
//!     tokio::spawn(async move {
//!         server.start().await.unwrap();
//!     });
//! }
//! ```

use super::events::{Event, EventBroadcaster};
use super::handler::RequestHandler;
use super::protocol::{Request, Response};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

/// IPC server for handling named pipe connections
///
/// The server listens on a Windows named pipe and processes incoming requests
/// from IPC clients. It supports multiple concurrent connections and can broadcast
/// events to subscribed clients.
pub struct IpcServer {
    /// Named pipe path
    pipe_name: String,
    
    /// Event broadcaster for sending events to clients
    event_broadcaster: Arc<EventBroadcaster>,
    
    /// Request handler for processing IPC requests
    request_handler: Option<Arc<RequestHandler>>,
    
    /// Server running state
    running: Arc<RwLock<bool>>,
    
    /// Number of active connections
    connection_count: Arc<Mutex<usize>>,
}

impl IpcServer {
    /// Create a new IPC server with the default pipe name
    ///
    /// # Arguments
    ///
    /// * `event_broadcaster` - Event broadcaster for sending events to clients
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tiling_wm_core::ipc::server::IpcServer;
    /// use tiling_wm_core::ipc::EventBroadcaster;
    ///
    /// let broadcaster = Arc::new(EventBroadcaster::new());
    /// let server = IpcServer::new(broadcaster);
    /// ```
    pub fn new(event_broadcaster: Arc<EventBroadcaster>) -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
            event_broadcaster,
            request_handler: None,
            running: Arc::new(RwLock::new(false)),
            connection_count: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Create a new IPC server with a custom pipe name
    ///
    /// # Arguments
    ///
    /// * `event_broadcaster` - Event broadcaster for sending events to clients
    /// * `name` - Custom pipe name (without the `\\.\pipe\` prefix)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tiling_wm_core::ipc::server::IpcServer;
    /// use tiling_wm_core::ipc::EventBroadcaster;
    ///
    /// let broadcaster = Arc::new(EventBroadcaster::new());
    /// let server = IpcServer::new(broadcaster)
    ///     .with_pipe_name("my-custom-pipe");
    /// ```
    pub fn with_pipe_name(mut self, name: impl Into<String>) -> Self {
        self.pipe_name = format!(r"\\.\pipe\{}", name.into());
        self
    }
    
    /// Set the request handler for processing IPC requests
    ///
    /// # Arguments
    ///
    /// * `handler` - Request handler for processing IPC requests
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use tiling_wm_core::ipc::{IpcServer, EventBroadcaster, RequestHandler};
    /// use tiling_wm_core::window_manager::WindowManager;
    /// use tiling_wm_core::workspace::WorkspaceManager;
    /// use tiling_wm_core::commands::CommandExecutor;
    ///
    /// let broadcaster = Arc::new(EventBroadcaster::new());
    /// let wm = Arc::new(Mutex::new(WindowManager::new()));
    /// let wsm = Arc::new(Mutex::new(WorkspaceManager::new()));
    /// let executor = Arc::new(CommandExecutor::new());
    /// let handler = Arc::new(RequestHandler::new(wm, wsm, executor));
    ///
    /// let server = IpcServer::new(broadcaster)
    ///     .with_handler(handler);
    /// ```
    pub fn with_handler(mut self, handler: Arc<RequestHandler>) -> Self {
        self.request_handler = Some(handler);
        self
    }
    
    /// Get the pipe name being used by this server
    pub fn pipe_name(&self) -> &str {
        &self.pipe_name
    }
    
    /// Start the IPC server
    ///
    /// This method starts the server and begins listening for connections.
    /// It will continue running until `stop()` is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot be started or if there's an
    /// error creating the named pipe.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tiling_wm_core::ipc::server::IpcServer;
    /// use tiling_wm_core::ipc::EventBroadcaster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let broadcaster = Arc::new(EventBroadcaster::new());
    ///     let server = Arc::new(IpcServer::new(broadcaster));
    ///     
    ///     tokio::spawn(async move {
    ///         server.start().await.unwrap();
    ///     });
    /// }
    /// ```
    #[cfg(windows)]
    pub async fn start(self: Arc<Self>) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                info!("IPC server already running");
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting IPC server on {}", self.pipe_name);
        
        loop {
            // Check if we should stop
            if !*self.running.read().await {
                info!("IPC server stop requested");
                break;
            }
            
            // Create server instance for this connection
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
    
    /// Start the IPC server (non-Windows platforms)
    ///
    /// On non-Windows platforms, this method returns an error since named pipes
    /// are a Windows-specific feature.
    #[cfg(not(windows))]
    pub async fn start(self: Arc<Self>) -> Result<()> {
        anyhow::bail!("Named pipes are only supported on Windows");
    }
    
    /// Stop the IPC server
    ///
    /// This method signals the server to stop accepting new connections.
    /// Existing connections will be allowed to complete.
    pub async fn stop(&self) {
        info!("Stopping IPC server");
        let mut running = self.running.write().await;
        *running = false;
    }
    
    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Get the current connection count
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tiling_wm_core::ipc::server::IpcServer;
    /// use tiling_wm_core::ipc::EventBroadcaster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let broadcaster = Arc::new(EventBroadcaster::new());
    ///     let server = Arc::new(IpcServer::new(broadcaster));
    ///     
    ///     let count = server.get_connection_count().await;
    ///     println!("Active connections: {}", count);
    /// }
    /// ```
    pub async fn get_connection_count(&self) -> usize {
        *self.connection_count.lock().await
    }
    
    /// Handle a client connection
    #[cfg(windows)]
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
    
    /// Process client requests and handle event subscriptions
    #[cfg(windows)]
    async fn process_client(&self, mut server: NamedPipeServer) -> Result<()> {
        // Wait for client to connect
        server
            .connect()
            .await
            .context("Failed to connect to client")?;
        
        debug!("Client connected to named pipe");
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        loop {
            // If subscribed, wait for either a request or an event
            if subscribed {
                tokio::select! {
                    // Handle incoming requests
                    request_result = Self::read_request(&mut server) => {
                        match request_result {
                            Ok(Some(request)) => {
                                let response = self.process_request(request, &mut subscribed, &mut event_receiver).await;
                                Self::write_response(&mut server, &response).await?;
                            }
                            Ok(None) => {
                                debug!("Client disconnected");
                                break;
                            }
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
                            if let Err(e) = Self::write_response(&mut server, &response).await {
                                error!("Failed to send event: {}", e);
                                break;
                            }
                        }
                    }
                }
            } else {
                // Not subscribed, just handle requests
                match Self::read_request(&mut server).await? {
                    Some(request) => {
                        let response = self
                            .process_request(request, &mut subscribed, &mut event_receiver)
                            .await;
                        Self::write_response(&mut server, &response).await?;
                    }
                    None => {
                        debug!("Client disconnected");
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Read a request from the client with length prefix framing
    ///
    /// Messages are framed with a 4-byte little-endian length prefix followed
    /// by the JSON payload.
    #[cfg(windows)]
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
        
        // Sanity checks
        if len == 0 {
            anyhow::bail!("Request length cannot be zero");
        }
        if len > 10 * 1024 * 1024 {
            anyhow::bail!("Request too large: {} bytes (max 10MB)", len);
        }
        
        // Read request data
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data).await?;
        
        // Parse JSON
        let request: Request =
            serde_json::from_slice(&data).context("Failed to parse request JSON")?;
        
        debug!("Received request: {:?}", request);
        Ok(Some(request))
    }
    
    /// Write a response to the client with length prefix framing
    #[cfg(windows)]
    async fn write_response<W>(writer: &mut W, response: &Response) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        // Serialize response
        let data = serde_json::to_vec(response).context("Failed to serialize response")?;
        
        // Write length prefix (4 bytes, little-endian)
        let len = data.len() as u32;
        writer.write_all(&len.to_le_bytes()).await?;
        
        // Write response data
        writer.write_all(&data).await?;
        
        // Flush to ensure data is sent
        writer.flush().await?;
        
        debug!("Sent response: {} bytes", data.len());
        Ok(())
    }
    
    /// Receive an event from the event receiver
    ///
    /// Returns None if there is no receiver or if the channel is closed.
    async fn receive_event(
        event_receiver: &mut Option<tokio::sync::broadcast::Receiver<Event>>,
    ) -> Option<Event> {
        if let Some(ref mut receiver) = event_receiver {
            match receiver.recv().await {
                Ok(event) => {
                    debug!("Received event: {:?}", event);
                    Some(event)
                }
                Err(e) => {
                    warn!("Event receiver error: {}", e);
                    None
                }
            }
        } else {
            // No receiver, wait indefinitely
            std::future::pending().await
        }
    }
    
    /// Process a request and return a response
    ///
    /// This method handles subscription requests directly and forwards all other
    /// requests to the request handler if one is configured.
    async fn process_request(
        &self,
        request: Request,
        subscribed: &mut bool,
        event_receiver: &mut Option<tokio::sync::broadcast::Receiver<Event>>,
    ) -> Response {
        debug!("Processing request: {:?}", request);
        
        match request {
            // Handle subscription requests directly (server-level)
            Request::Subscribe { events } => {
                if events.is_empty() {
                    Response::error("No events specified")
                } else {
                    *subscribed = true;
                    *event_receiver = Some(self.event_broadcaster.subscribe());
                    info!("Client subscribed to events: {:?}", events);
                    Response::success_with_data(serde_json::json!({
                        "subscribed": events,
                    }))
                }
            }
            
            Request::Unsubscribe => {
                *subscribed = false;
                *event_receiver = None;
                info!("Client unsubscribed from events");
                Response::success()
            }
            
            // Forward all other requests to the handler
            _ => {
                if let Some(handler) = &self.request_handler {
                    handler.handle_request(request).await
                } else {
                    Response::error("Request handler not configured. Server requires integration with window manager.")
                }
            }
        }
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
        assert_eq!(server.get_connection_count().await, 0);
        assert!(!server.is_running().await);
    }
    
    #[tokio::test]
    async fn test_custom_pipe_name() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster).with_pipe_name("test-pipe");
        
        assert!(server.pipe_name.contains("test-pipe"));
        assert_eq!(server.pipe_name, r"\\.\pipe\test-pipe");
    }
    
    #[tokio::test]
    async fn test_pipe_name_getter() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster).with_pipe_name("test-pipe");
        
        assert_eq!(server.pipe_name(), r"\\.\pipe\test-pipe");
    }
    
    #[tokio::test]
    async fn test_connection_count_initialization() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        assert_eq!(server.get_connection_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_server_not_running_initially() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        assert!(!server.is_running().await);
    }
    
    #[tokio::test]
    async fn test_stop_server() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        // Manually set running to true for this test
        {
            let mut running = server.running.write().await;
            *running = true;
        }
        
        assert!(server.is_running().await);
        
        server.stop().await;
        
        assert!(!server.is_running().await);
    }
    
    #[tokio::test]
    async fn test_process_ping_request() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        let response = server
            .process_request(Request::Ping, &mut subscribed, &mut event_receiver)
            .await;
        
        matches!(response, Response::Pong);
    }
    
    #[tokio::test]
    async fn test_process_subscribe_request() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        let events = vec!["window_created".to_string(), "workspace_changed".to_string()];
        let response = server
            .process_request(
                Request::Subscribe {
                    events: events.clone(),
                },
                &mut subscribed,
                &mut event_receiver,
            )
            .await;
        
        assert!(subscribed);
        assert!(event_receiver.is_some());
        
        match response {
            Response::Success { data } => {
                assert!(data.is_some());
            }
            _ => panic!("Expected Success response"),
        }
    }
    
    #[tokio::test]
    async fn test_process_subscribe_empty_events() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        let response = server
            .process_request(
                Request::Subscribe { events: vec![] },
                &mut subscribed,
                &mut event_receiver,
            )
            .await;
        
        assert!(!subscribed);
        assert!(event_receiver.is_none());
        
        match response {
            Response::Error { message, .. } => {
                assert!(message.contains("No events specified"));
            }
            _ => panic!("Expected Error response"),
        }
    }
    
    #[tokio::test]
    async fn test_process_unsubscribe_request() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(Arc::clone(&broadcaster));
        
        let mut subscribed = true;
        let mut event_receiver = Some(broadcaster.subscribe());
        
        let response = server
            .process_request(Request::Unsubscribe, &mut subscribed, &mut event_receiver)
            .await;
        
        assert!(!subscribed);
        assert!(event_receiver.is_none());
        
        matches!(response, Response::Success { .. });
    }
    
    #[tokio::test]
    async fn test_process_unimplemented_request() {
        let broadcaster = Arc::new(EventBroadcaster::new());
        let server = IpcServer::new(broadcaster);
        
        let mut subscribed = false;
        let mut event_receiver = None;
        
        let response = server
            .process_request(
                Request::GetWorkspaces,
                &mut subscribed,
                &mut event_receiver,
            )
            .await;
        
        match response {
            Response::Error { message, .. } => {
                assert!(message.contains("not implemented"));
            }
            _ => panic!("Expected Error response"),
        }
    }
    
    #[cfg(windows)]
    #[tokio::test]
    async fn test_request_framing_size_check() {
        // Test that oversized requests are rejected
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Max u32 value
        let mut cursor = std::io::Cursor::new(data);
        
        let result = IpcServer::read_request(&mut cursor).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("too large"));
        }
    }
    
    #[cfg(windows)]
    #[tokio::test]
    async fn test_request_framing_zero_length() {
        // Test that zero-length requests are rejected
        let data = vec![0x00, 0x00, 0x00, 0x00]; // Zero length
        let mut cursor = std::io::Cursor::new(data);
        
        let result = IpcServer::read_request(&mut cursor).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("cannot be zero"));
        }
    }
}
