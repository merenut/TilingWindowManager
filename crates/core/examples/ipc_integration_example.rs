//! Example of how to integrate the IPC server with the window manager.
//!
//! This example demonstrates:
//! - Creating the IPC server with event broadcaster
//! - Creating the request handler with window manager and workspace manager
//! - Starting the IPC server in a background task
//! - Emitting events when window manager state changes
//!
//! # Note
//!
//! This example requires Windows and cannot be run on other platforms.
//! It serves as documentation for how to integrate IPC in the main application.

use std::sync::Arc;
use tokio::sync::Mutex;
use tiling_wm_core::commands::CommandExecutor;
use tiling_wm_core::ipc::{Event, EventBroadcaster, IpcServer, RequestHandler};
use tiling_wm_core::window_manager::WindowManager;
use tiling_wm_core::workspace::{WorkspaceConfig, WorkspaceManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("==============================================");
    println!("IPC Server Integration Example");
    println!("==============================================\n");
    
    // Step 1: Create the event broadcaster (shared between WM and IPC server)
    println!("1. Creating event broadcaster...");
    let event_broadcaster = Arc::new(EventBroadcaster::new());
    println!("   ✓ Event broadcaster created\n");
    
    // Step 2: Create window manager and workspace manager
    println!("2. Creating window manager and workspace manager...");
    let mut wm = WindowManager::new();
    wm.initialize()?;
    let wm = Arc::new(Mutex::new(wm));
    
    let config = WorkspaceConfig::default();
    let wsm = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    
    let executor = Arc::new(CommandExecutor::new());
    println!("   ✓ Window manager initialized\n");
    
    // Step 3: Create the request handler
    println!("3. Creating IPC request handler...");
    let handler = Arc::new(RequestHandler::new(
        Arc::clone(&wm),
        Arc::clone(&wsm),
        Arc::clone(&executor),
    ));
    println!("   ✓ Request handler created\n");
    
    // Step 4: Create and configure the IPC server
    println!("4. Creating IPC server...");
    let ipc_server = Arc::new(
        IpcServer::new(Arc::clone(&event_broadcaster))
            .with_handler(handler)
    );
    println!("   ✓ IPC server created\n");
    
    // Step 5: Start the IPC server in a background task
    println!("5. Starting IPC server...");
    let server_clone = Arc::clone(&ipc_server);
    let server_task = tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            eprintln!("IPC server error: {}", e);
        }
    });
    println!("   ✓ IPC server started on {}\n", ipc_server.pipe_name());
    
    // Step 6: Demonstrate event emission
    println!("6. Demonstrating event emission...");
    println!("   When window manager state changes, emit events like:");
    println!("   - event_broadcaster.emit(Event::WindowCreated {{ hwnd, title, workspace }})");
    println!("   - event_broadcaster.emit(Event::WorkspaceChanged {{ from, to }})");
    println!("   - event_broadcaster.emit(Event::LayoutChanged {{ layout }})");
    println!();
    
    // Example: Emit a test event
    event_broadcaster.emit(Event::WorkspaceChanged {
        from: 1,
        to: 2,
    });
    println!("   ✓ Test event emitted (WorkspaceChanged)\n");
    
    // Step 7: Show where to integrate in main application
    println!("7. Integration points in main.rs:");
    println!("   a) Create event_broadcaster at application startup");
    println!("   b) Pass event_broadcaster to window manager initialization");
    println!("   c) Create and start IPC server after window manager is ready");
    println!("   d) Emit events whenever window manager state changes:\n");
    
    println!("      // In window creation handler:");
    println!("      event_broadcaster.emit(Event::WindowCreated {{");
    println!("          hwnd: window.handle.0 .0,");
    println!("          title: window.title.clone(),");
    println!("          workspace: current_workspace,");
    println!("      }});\n");
    
    println!("      // In workspace switch handler:");
    println!("      event_broadcaster.emit(Event::WorkspaceChanged {{");
    println!("          from: old_workspace,");
    println!("          to: new_workspace,");
    println!("      }});\n");
    
    println!("      // In config reload handler:");
    println!("      event_broadcaster.emit(Event::ConfigReloaded);\n");
    
    println!("8. Shutdown handling:");
    println!("   a) Call ipc_server.stop().await when application exits");
    println!("   b) Wait for server_task to complete");
    println!();
    
    // Simulate running for a short time
    println!("Running for 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Shutdown
    println!("\nShutting down...");
    ipc_server.stop().await;
    
    // Wait for server to stop
    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        server_task
    ).await;
    
    println!("✓ Shutdown complete\n");
    println!("==============================================");
    println!("Example complete!");
    println!("==============================================");
    
    Ok(())
}
