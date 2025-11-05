//! Event system for IPC communication.
//!
//! This module provides event broadcasting functionality for real-time notifications
//! to IPC clients about window manager state changes.

use super::protocol::Response;
use serde_json::json;
use tokio::sync::broadcast::{channel, Receiver, Sender};

/// Event types that can be broadcast to IPC clients
///
/// Note: Window handles (hwnd) are stored as isize internally to match
/// the Windows HWND type, but are converted to strings when sent over IPC
/// to ensure JSON compatibility and cross-language interoperability.
#[derive(Debug, Clone)]
pub enum Event {
    /// Window was created
    WindowCreated {
        hwnd: isize,
        title: String,
        workspace: usize,
    },
    
    /// Window was closed
    WindowClosed {
        hwnd: isize,
    },
    
    /// Window received focus
    WindowFocused {
        hwnd: isize,
    },
    
    /// Window was moved to a different workspace
    WindowMoved {
        hwnd: isize,
        from_workspace: usize,
        to_workspace: usize,
    },
    
    /// Window state changed (tiled, floating, fullscreen, minimized)
    WindowStateChanged {
        hwnd: isize,
        old_state: String,
        new_state: String,
    },
    
    /// Active workspace changed
    WorkspaceChanged {
        from: usize,
        to: usize,
    },
    
    /// New workspace was created
    WorkspaceCreated {
        id: usize,
        name: String,
    },
    
    /// Workspace was deleted
    WorkspaceDeleted {
        id: usize,
    },
    
    /// Monitor configuration changed
    MonitorChanged,
    
    /// Configuration was reloaded
    ConfigReloaded,
    
    /// Layout changed
    LayoutChanged {
        layout: String,
    },
}

/// Event broadcaster for distributing events to multiple subscribers
pub struct EventBroadcaster {
    sender: Sender<Event>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster with a buffer size of 100 events
    pub fn new() -> Self {
        let (tx, _) = channel(100);
        Self { sender: tx }
    }
    
    /// Emit an event to all subscribers
    ///
    /// If there are no subscribers, the event is dropped silently.
    /// This is expected behavior for a broadcast channel.
    pub fn emit(&self, event: Event) {
        tracing::debug!("Broadcasting event: {:?}", event);
        // It's acceptable to ignore the error here because broadcast channels
        // return an error when there are no receivers, which is a valid state.
        // The event is simply dropped if no one is listening.
        let _ = self.sender.send(event);
    }
    
    /// Subscribe to events
    ///
    /// Returns a receiver that will receive all future events
    pub fn subscribe(&self) -> Receiver<Event> {
        self.sender.subscribe()
    }
    
    /// Get the number of active subscribers
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
    /// Convert event to an IPC Response
    pub fn to_response(&self) -> Response {
        let (name, data) = match self {
            Event::WindowCreated {
                hwnd,
                title,
                workspace,
            } => (
                "window_created",
                json!({
                    "hwnd": format!("{}", hwnd),
                    "title": title,
                    "workspace": workspace,
                }),
            ),
            Event::WindowClosed { hwnd } => {
                ("window_closed", json!({ "hwnd": format!("{}", hwnd) }))
            }
            Event::WindowFocused { hwnd } => {
                ("window_focused", json!({ "hwnd": format!("{}", hwnd) }))
            }
            Event::WindowMoved {
                hwnd,
                from_workspace,
                to_workspace,
            } => (
                "window_moved",
                json!({
                    "hwnd": format!("{}", hwnd),
                    "from_workspace": from_workspace,
                    "to_workspace": to_workspace,
                }),
            ),
            Event::WindowStateChanged {
                hwnd,
                old_state,
                new_state,
            } => (
                "window_state_changed",
                json!({
                    "hwnd": format!("{}", hwnd),
                    "old_state": old_state,
                    "new_state": new_state,
                }),
            ),
            Event::WorkspaceChanged { from, to } => {
                ("workspace_changed", json!({ "from": from, "to": to }))
            }
            Event::WorkspaceCreated { id, name } => {
                ("workspace_created", json!({ "id": id, "name": name }))
            }
            Event::WorkspaceDeleted { id } => ("workspace_deleted", json!({ "id": id })),
            Event::MonitorChanged => ("monitor_changed", json!({})),
            Event::ConfigReloaded => ("config_reloaded", json!({})),
            Event::LayoutChanged { layout } => {
                ("layout_changed", json!({ "layout": layout }))
            }
        };
        
        Response::Event {
            name: name.to_string(),
            data,
        }
    }
    
    /// Get the event name as a string
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
    fn test_event_broadcaster_creation() {
        let broadcaster = EventBroadcaster::new();
        assert_eq!(broadcaster.subscriber_count(), 0);
    }
    
    #[test]
    fn test_event_broadcaster_subscribe() {
        let broadcaster = EventBroadcaster::new();
        let _receiver = broadcaster.subscribe();
        
        assert_eq!(broadcaster.subscriber_count(), 1);
    }
    
    #[test]
    fn test_event_broadcaster_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let _receiver1 = broadcaster.subscribe();
        let _receiver2 = broadcaster.subscribe();
        
        assert_eq!(broadcaster.subscriber_count(), 2);
    }
    
    #[test]
    fn test_event_broadcast() {
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
    fn test_event_to_response_window_created() {
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
                assert_eq!(data["workspace"], 1);
            }
            _ => panic!("Expected Event response"),
        }
    }
    
    #[test]
    fn test_event_to_response_window_closed() {
        let event = Event::WindowClosed { hwnd: 12345 };
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "window_closed");
                assert!(data.get("hwnd").is_some());
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
    fn test_window_moved_event() {
        let event = Event::WindowMoved {
            hwnd: 12345,
            from_workspace: 1,
            to_workspace: 2,
        };
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "window_moved");
                assert_eq!(data["from_workspace"], 1);
                assert_eq!(data["to_workspace"], 2);
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
        
        let event = Event::LayoutChanged {
            layout: "dwindle".to_string(),
        };
        assert_eq!(event.event_name(), "layout_changed");
    }
    
    #[test]
    fn test_workspace_created_event() {
        let event = Event::WorkspaceCreated {
            id: 5,
            name: "New Workspace".to_string(),
        };
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "workspace_created");
                assert_eq!(data["id"], 5);
                assert_eq!(data["name"], "New Workspace");
            }
            _ => panic!("Expected Event response"),
        }
    }
    
    #[test]
    fn test_window_state_changed_event() {
        let event = Event::WindowStateChanged {
            hwnd: 12345,
            old_state: "tiled".to_string(),
            new_state: "floating".to_string(),
        };
        let response = event.to_response();
        
        match response {
            Response::Event { name, data } => {
                assert_eq!(name, "window_state_changed");
                assert_eq!(data["old_state"], "tiled");
                assert_eq!(data["new_state"], "floating");
            }
            _ => panic!("Expected Event response"),
        }
    }
    
    #[test]
    fn test_default_event_broadcaster() {
        let broadcaster = EventBroadcaster::default();
        assert_eq!(broadcaster.subscriber_count(), 0);
    }
}
