//! Standalone test for IPC protocol serialization/deserialization.
//!
//! This example tests the IPC protocol types without requiring Windows dependencies.

use tiling_wm_core::ipc::protocol::*;
use tiling_wm_core::ipc::events::*;

fn main() {
    println!("Testing IPC Protocol Implementation...\n");
    
    // Test protocol version
    println!("1. Testing Protocol Version:");
    let version = ProtocolVersion::default();
    assert_eq!(version.version, PROTOCOL_VERSION);
    println!("   ✓ Protocol version: {}", PROTOCOL_VERSION);
    
    // Test Request serialization
    println!("\n2. Testing Request Serialization:");
    test_requests();
    
    // Test Response serialization
    println!("\n3. Testing Response Serialization:");
    test_responses();
    
    // Test Data Structures
    println!("\n4. Testing Data Structures:");
    test_data_structures();
    
    // Test Events
    println!("\n5. Testing Event System:");
    test_events();
    
    println!("\n✅ All IPC protocol tests passed!");
}

fn test_requests() {
    // Test GetWindows request
    let request = Request::GetWindows { workspace: Some(1) };
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: Request = serde_json::from_str(&json).unwrap();
    match deserialized {
        Request::GetWindows { workspace } => {
            assert_eq!(workspace, Some(1));
            println!("   ✓ GetWindows request serialization");
        }
        _ => panic!("Wrong request type"),
    }
    
    // Test Execute request
    let request = Request::Execute {
        command: "workspace".to_string(),
        args: vec!["3".to_string()],
    };
    let json = serde_json::to_string(&request).unwrap();
    let _deserialized: Request = serde_json::from_str(&json).unwrap();
    println!("   ✓ Execute request serialization");
    
    // Test Subscribe request
    let request = Request::Subscribe {
        events: vec!["window_created".to_string(), "workspace_changed".to_string()],
    };
    let json = serde_json::to_string(&request).unwrap();
    let _deserialized: Request = serde_json::from_str(&json).unwrap();
    println!("   ✓ Subscribe request serialization");
    
    // Test all simple requests
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
    println!("   ✓ All simple request types serialization");
}

fn test_responses() {
    // Test Success response
    let response = Response::success();
    let json = serde_json::to_string(&response).unwrap();
    let _deserialized: Response = serde_json::from_str(&json).unwrap();
    println!("   ✓ Success response serialization");
    
    // Test Success with data
    let data = serde_json::json!({"test": "value"});
    let response = Response::success_with_data(data);
    let json = serde_json::to_string(&response).unwrap();
    let _deserialized: Response = serde_json::from_str(&json).unwrap();
    println!("   ✓ Success with data response serialization");
    
    // Test Error response
    let response = Response::error("Test error");
    let json = serde_json::to_string(&response).unwrap();
    let _deserialized: Response = serde_json::from_str(&json).unwrap();
    println!("   ✓ Error response serialization");
    
    // Test Error with code
    let response = Response::error_with_code("Test error", "ERR_TEST");
    let json = serde_json::to_string(&response).unwrap();
    let _deserialized: Response = serde_json::from_str(&json).unwrap();
    println!("   ✓ Error with code response serialization");
    
    // Test Pong response
    let response = Response::Pong;
    let json = serde_json::to_string(&response).unwrap();
    let _deserialized: Response = serde_json::from_str(&json).unwrap();
    println!("   ✓ Pong response serialization");
}

fn test_data_structures() {
    // Test WindowInfo
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
    let json = serde_json::to_string(&info).unwrap();
    let _deserialized: WindowInfo = serde_json::from_str(&json).unwrap();
    println!("   ✓ WindowInfo serialization");
    
    // Test WorkspaceInfo
    let info = WorkspaceInfo {
        id: 1,
        name: "Workspace 1".to_string(),
        monitor: 0,
        window_count: 5,
        active: true,
        visible: Some(true),
    };
    let json = serde_json::to_string(&info).unwrap();
    let _deserialized: WorkspaceInfo = serde_json::from_str(&json).unwrap();
    println!("   ✓ WorkspaceInfo serialization");
    
    // Test MonitorInfo
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
    let _deserialized: MonitorInfo = serde_json::from_str(&json).unwrap();
    println!("   ✓ MonitorInfo serialization");
    
    // Test ConfigInfo
    let info = ConfigInfo {
        version: "1.0.0".to_string(),
        config_path: "/path/to/config.toml".to_string(),
        workspaces_count: 9,
        layouts: vec!["dwindle".to_string(), "master".to_string()],
        current_layout: "dwindle".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let _deserialized: ConfigInfo = serde_json::from_str(&json).unwrap();
    println!("   ✓ ConfigInfo serialization");
    
    // Test VersionInfo
    let info = VersionInfo {
        version: "0.1.0".to_string(),
        build_date: "2024-01-01".to_string(),
        git_commit: Some("abc123".to_string()),
        rustc_version: "1.70.0".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let _deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
    println!("   ✓ VersionInfo serialization");
}

fn test_events() {
    // Test EventBroadcaster
    let broadcaster = EventBroadcaster::new();
    assert_eq!(broadcaster.subscriber_count(), 0);
    println!("   ✓ EventBroadcaster creation");
    
    let _receiver = broadcaster.subscribe();
    assert_eq!(broadcaster.subscriber_count(), 1);
    println!("   ✓ EventBroadcaster subscription");
    
    // Test Event to Response conversion
    let event = Event::WindowCreated {
        hwnd: 12345,
        title: "Test Window".to_string(),
        workspace: 1,
    };
    let response = event.to_response();
    match response {
        Response::Event { name, .. } => {
            assert_eq!(name, "window_created");
        }
        _ => panic!("Expected Event response"),
    }
    println!("   ✓ Event to Response conversion");
    
    // Test all event types
    let events = vec![
        Event::WindowCreated {
            hwnd: 1,
            title: "Test".to_string(),
            workspace: 1,
        },
        Event::WindowClosed { hwnd: 1 },
        Event::WindowFocused { hwnd: 1 },
        Event::WindowMoved {
            hwnd: 1,
            from_workspace: 1,
            to_workspace: 2,
        },
        Event::WindowStateChanged {
            hwnd: 1,
            old_state: "tiled".to_string(),
            new_state: "floating".to_string(),
        },
        Event::WorkspaceChanged { from: 1, to: 2 },
        Event::WorkspaceCreated {
            id: 1,
            name: "Test".to_string(),
        },
        Event::WorkspaceDeleted { id: 1 },
        Event::MonitorChanged,
        Event::ConfigReloaded,
        Event::LayoutChanged {
            layout: "dwindle".to_string(),
        },
    ];
    
    for event in events {
        let _response = event.to_response();
        let _name = event.event_name();
    }
    println!("   ✓ All event types conversion");
}
