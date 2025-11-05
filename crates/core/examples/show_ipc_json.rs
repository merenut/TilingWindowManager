//! Example showing JSON output for IPC protocol types.

use tiling_wm_core::ipc::protocol::*;
use tiling_wm_core::ipc::events::*;

fn main() {
    println!("=== IPC Protocol JSON Examples ===\n");
    
    // Request examples
    println!("## Request Examples:\n");
    
    println!("### GetWindows Request (with workspace filter):");
    let request = Request::GetWindows { workspace: Some(1) };
    println!("{}\n", serde_json::to_string_pretty(&request).unwrap());
    
    println!("### Execute Request:");
    let request = Request::Execute {
        command: "workspace".to_string(),
        args: vec!["3".to_string()],
    };
    println!("{}\n", serde_json::to_string_pretty(&request).unwrap());
    
    println!("### Subscribe Request:");
    let request = Request::Subscribe {
        events: vec![
            "window_created".to_string(),
            "workspace_changed".to_string(),
        ],
    };
    println!("{}\n", serde_json::to_string_pretty(&request).unwrap());
    
    // Response examples
    println!("## Response Examples:\n");
    
    println!("### Success Response (no data):");
    let response = Response::success();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());
    
    println!("### Success Response (with data):");
    let window_info = WindowInfo {
        hwnd: "12345".to_string(),
        title: "Terminal".to_string(),
        class: "WindowsTerminal".to_string(),
        process_name: "WindowsTerminal.exe".to_string(),
        workspace: 1,
        monitor: 0,
        state: WindowState::Tiled,
        rect: RectInfo {
            x: 100,
            y: 100,
            width: 1600,
            height: 900,
        },
        focused: Some(true),
    };
    let response = Response::success_with_data(serde_json::to_value(&window_info).unwrap());
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());
    
    println!("### Error Response:");
    let response = Response::error_with_code("Window not found", "ERR_NOT_FOUND");
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());
    
    // Data structure examples
    println!("## Data Structure Examples:\n");
    
    println!("### WorkspaceInfo:");
    let workspace_info = WorkspaceInfo {
        id: 1,
        name: "Main".to_string(),
        monitor: 0,
        window_count: 3,
        active: true,
        visible: Some(true),
    };
    println!("{}\n", serde_json::to_string_pretty(&workspace_info).unwrap());
    
    println!("### MonitorInfo:");
    let monitor_info = MonitorInfo {
        id: 0,
        name: "Primary Monitor".to_string(),
        width: 1920,
        height: 1080,
        x: 0,
        y: 0,
        scale: 1.0,
        primary: Some(true),
        active_workspace: Some(1),
    };
    println!("{}\n", serde_json::to_string_pretty(&monitor_info).unwrap());
    
    // Event examples
    println!("## Event Examples:\n");
    
    println!("### WindowCreated Event:");
    let event = Event::WindowCreated {
        hwnd: 12345,
        title: "New Window".to_string(),
        workspace: 1,
    };
    let response = event.to_response();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());
    
    println!("### WorkspaceChanged Event:");
    let event = Event::WorkspaceChanged { from: 1, to: 2 };
    let response = event.to_response();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());
}
