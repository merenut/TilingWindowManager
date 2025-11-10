//! Integration tests for IPC handler with window manager.
//!
//! This test suite validates that the IPC handler correctly integrates
//! with the window manager, workspace manager, and command executor.

use std::sync::Arc;
use tokio::sync::Mutex;
use tenraku_core::commands::CommandExecutor;
use tenraku_core::ipc::handler::RequestHandler;
use tenraku_core::ipc::protocol::{Request, Response};
use tenraku_core::window_manager::WindowManager;
use tenraku_core::workspace::{WorkspaceConfig, WorkspaceManager};

#[tokio::main]
async fn main() {
    println!("==============================================");
    println!("Testing IPC Handler Integration");
    println!("==============================================\n");

    let mut passed = 0;
    let mut failed = 0;

    // Setup
    println!("Setting up test environment...");
    let wm = Arc::new(Mutex::new(WindowManager::new()));
    let config = WorkspaceConfig::default();
    let wsm = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    let executor = Arc::new(CommandExecutor::new());
    let handler = RequestHandler::new(wm, wsm, executor);
    println!("✓ Test environment ready\n");

    // Test 1: Ping
    print!("Test 1: Ping request... ");
    let response = handler.handle_request(Request::Ping).await;
    match response {
        Response::Pong => {
            println!("✓ PASS");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL: Expected Pong");
            failed += 1;
        }
    }

    // Test 2: GetVersion
    print!("Test 2: GetVersion request... ");
    let response = handler.handle_request(Request::GetVersion).await;
    match response {
        Response::Success { data } => {
            if let Some(data) = data {
                if data.get("version").is_some() {
                    println!("✓ PASS");
                    passed += 1;
                } else {
                    println!("✗ FAIL: Missing version field");
                    failed += 1;
                }
            } else {
                println!("✗ FAIL: No data in response");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Success response");
            failed += 1;
        }
    }

    // Test 3: GetWorkspaces
    print!("Test 3: GetWorkspaces request... ");
    let response = handler.handle_request(Request::GetWorkspaces).await;
    match response {
        Response::Success { data } => {
            if let Some(data) = data {
                if data.is_array() {
                    println!("✓ PASS");
                    passed += 1;
                } else {
                    println!("✗ FAIL: Data is not an array");
                    failed += 1;
                }
            } else {
                println!("✗ FAIL: No data in response");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Success response");
            failed += 1;
        }
    }

    // Test 4: GetMonitors
    print!("Test 4: GetMonitors request... ");
    let response = handler.handle_request(Request::GetMonitors).await;
    match response {
        Response::Success { data } => {
            if let Some(data) = data {
                if data.is_array() {
                    println!("✓ PASS");
                    passed += 1;
                } else {
                    println!("✗ FAIL: Data is not an array");
                    failed += 1;
                }
            } else {
                println!("✗ FAIL: No data in response");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Success response");
            failed += 1;
        }
    }

    // Test 5: GetConfig
    print!("Test 5: GetConfig request... ");
    let response = handler.handle_request(Request::GetConfig).await;
    match response {
        Response::Success { data } => {
            if let Some(data) = data {
                if data.get("version").is_some() && data.get("layouts").is_some() {
                    println!("✓ PASS");
                    passed += 1;
                } else {
                    println!("✗ FAIL: Missing required fields");
                    failed += 1;
                }
            } else {
                println!("✗ FAIL: No data in response");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Success response");
            failed += 1;
        }
    }

    // Test 6: SetLayout (dwindle)
    print!("Test 6: SetLayout to dwindle... ");
    let response = handler
        .handle_request(Request::SetLayout {
            layout: "dwindle".to_string(),
        })
        .await;
    match response {
        Response::Success { .. } => {
            println!("✓ PASS");
            passed += 1;
        }
        Response::Error { message, .. } => {
            println!("✗ FAIL: {}", message);
            failed += 1;
        }
        _ => {
            println!("✗ FAIL: Unexpected response type");
            failed += 1;
        }
    }

    // Test 7: SetLayout (master)
    print!("Test 7: SetLayout to master... ");
    let response = handler
        .handle_request(Request::SetLayout {
            layout: "master".to_string(),
        })
        .await;
    match response {
        Response::Success { .. } => {
            println!("✓ PASS");
            passed += 1;
        }
        Response::Error { message, .. } => {
            println!("✗ FAIL: {}", message);
            failed += 1;
        }
        _ => {
            println!("✗ FAIL: Unexpected response type");
            failed += 1;
        }
    }

    // Test 8: SetLayout (invalid)
    print!("Test 8: SetLayout with invalid layout... ");
    let response = handler
        .handle_request(Request::SetLayout {
            layout: "invalid".to_string(),
        })
        .await;
    match response {
        Response::Error { message, .. } => {
            if message.contains("Unknown layout") {
                println!("✓ PASS");
                passed += 1;
            } else {
                println!("✗ FAIL: Wrong error message");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Error response");
            failed += 1;
        }
    }

    // Test 9: Execute command
    print!("Test 9: Execute command (layout_dwindle)... ");
    let response = handler
        .handle_request(Request::Execute {
            command: "layout_dwindle".to_string(),
            args: vec![],
        })
        .await;
    match response {
        Response::Success { .. } => {
            println!("✓ PASS");
            passed += 1;
        }
        Response::Error { message, .. } => {
            println!("✗ FAIL: {}", message);
            failed += 1;
        }
        _ => {
            println!("✗ FAIL: Unexpected response type");
            failed += 1;
        }
    }

    // Test 10: Execute unknown command
    print!("Test 10: Execute unknown command... ");
    let response = handler
        .handle_request(Request::Execute {
            command: "nonexistent_command".to_string(),
            args: vec![],
        })
        .await;
    match response {
        Response::Error { message, .. } => {
            if message.contains("Unknown command") {
                println!("✓ PASS");
                passed += 1;
            } else {
                println!("✗ FAIL: Wrong error message");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Error response");
            failed += 1;
        }
    }

    // Test 11: Subscribe (should error - handled by server)
    print!("Test 11: Subscribe request (should be server-handled)... ");
    let response = handler
        .handle_request(Request::Subscribe {
            events: vec!["window_created".to_string()],
        })
        .await;
    match response {
        Response::Error { message, .. } => {
            if message.contains("handled by IPC server") {
                println!("✓ PASS");
                passed += 1;
            } else {
                println!("✗ FAIL: Wrong error message");
                failed += 1;
            }
        }
        _ => {
            println!("✗ FAIL: Expected Error response");
            failed += 1;
        }
    }

    // Test 12: Quit
    print!("Test 12: Quit request... ");
    let response = handler.handle_request(Request::Quit).await;
    match response {
        Response::Success { .. } => {
            println!("✓ PASS");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL: Expected Success response");
            failed += 1;
        }
    }

    // Summary
    println!("\n==============================================");
    println!("Test Results:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Total:  {}", passed + failed);
    println!("==============================================");

    if failed == 0 {
        println!("\n✓ All tests passed!");
        std::process::exit(0);
    } else {
        println!("\n✗ Some tests failed!");
        std::process::exit(1);
    }
}
