//! Integration tests for the window manager core.
//!
//! These tests verify the full interaction between components including:
//! - WindowManager initialization and state management
//! - Window management lifecycle (add, remove, switch workspaces)
//! - Monitor enumeration and refresh
//! - Integration with Windows API (on Windows only)

use tiling_wm_core::window_manager::WindowManager;

#[test]
fn test_full_initialization() {
    let mut wm = WindowManager::new();
    assert_eq!(wm.get_active_workspace(), 1);

    let result = wm.initialize();
    assert!(
        result.is_ok(),
        "WindowManager initialization should succeed"
    );

    // After initialization, we should have at least one monitor
    assert!(
        !wm.get_monitors().is_empty(),
        "Should have at least one monitor after initialization"
    );
}

#[test]
fn test_monitor_enumeration() {
    let mut wm = WindowManager::new();

    let result = wm.refresh_monitors();
    assert!(result.is_ok(), "Monitor refresh should succeed");

    let monitors = wm.get_monitors();
    assert!(!monitors.is_empty(), "Should detect at least one monitor");

    // Verify monitor IDs are sequential
    for (idx, monitor) in monitors.iter().enumerate() {
        assert_eq!(monitor.id, idx, "Monitor IDs should be sequential");
    }

    // Verify monitor has valid properties
    for monitor in monitors {
        assert!(!monitor.name.is_empty(), "Monitor should have a name");
        assert!(
            monitor.work_area.width > 0,
            "Monitor work area width should be positive"
        );
        assert!(
            monitor.work_area.height > 0,
            "Monitor work area height should be positive"
        );
        assert!(
            monitor.dpi_scale > 0.0,
            "Monitor DPI scale should be positive"
        );
    }
}

#[test]
fn test_workspace_management() {
    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    // Test workspace switching
    assert_eq!(wm.get_active_workspace(), 1);

    wm.switch_workspace(2).expect("Failed to switch workspace");
    assert_eq!(wm.get_active_workspace(), 2);

    wm.switch_workspace(5).expect("Failed to switch workspace");
    assert_eq!(wm.get_active_workspace(), 5);

    // Switch back to workspace 1
    wm.switch_workspace(1).expect("Failed to switch workspace");
    assert_eq!(wm.get_active_workspace(), 1);

    // Switching to the same workspace should be a no-op
    wm.switch_workspace(1)
        .expect("Failed to switch to same workspace");
    assert_eq!(wm.get_active_workspace(), 1);
}

#[test]
fn test_workspace_tree_access() {
    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    // All workspaces 1-10 should be accessible
    for workspace_id in 1..=10 {
        assert!(
            wm.get_workspace_tree(workspace_id).is_some(),
            "Workspace {} should be accessible",
            workspace_id
        );
        assert!(
            wm.get_workspace_tree_mut(workspace_id).is_some(),
            "Workspace {} should be mutably accessible",
            workspace_id
        );
    }

    // Non-existent workspace should return None
    assert!(
        wm.get_workspace_tree(999).is_none(),
        "Non-existent workspace should return None"
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_window_enumeration() {
    use tiling_wm_core::utils::win32;

    // Test that we can enumerate windows
    let result = win32::enumerate_windows();
    assert!(result.is_ok(), "Window enumeration should succeed");

    let windows = result.unwrap();
    // We can't guarantee the exact number, but there should be some windows
    println!("Found {} windows", windows.len());

    // Test getting window properties for the first few windows
    for window in windows.iter().take(5) {
        let title = window.get_title();
        assert!(title.is_ok(), "Should be able to get window title");

        let class = window.get_class_name();
        assert!(class.is_ok(), "Should be able to get window class");

        let pid = window.get_process_id();
        assert!(pid > 0, "Process ID should be positive");
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_window_filtering() {
    use tiling_wm_core::utils::win32;

    let wm = WindowManager::new();

    // Test with actual app windows
    let result = win32::enumerate_app_windows();
    assert!(result.is_ok(), "App window enumeration should succeed");

    let windows = result.unwrap();
    println!("Found {} app windows", windows.len());

    for window in windows.iter().take(3) {
        let should_manage = wm.should_manage_window(window);
        assert!(should_manage.is_ok(), "Window filtering should not error");

        if should_manage.unwrap() {
            let title = window.get_title().unwrap_or_default();
            println!("Would manage window: {}", title);
        }
    }
}

/// Integration test for the full window management lifecycle.
/// This test requires actual windows to be open and is marked as ignored
/// for automated testing.
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_window_management_lifecycle() {
    use tiling_wm_core::utils::win32;

    let mut wm = WindowManager::new();
    wm.initialize()
        .expect("Failed to initialize window manager");

    println!(
        "WindowManager initialized with {} monitors",
        wm.get_monitors().len()
    );

    // Enumerate application windows
    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");
    println!("Found {} application windows", windows.len());

    // Try to manage a few windows
    let mut managed_count = 0;
    for window in windows.iter().take(3) {
        if wm.should_manage_window(window).unwrap_or(false) {
            let title = window.get_title().unwrap_or_default();
            println!("Managing window: {}", title);

            let result = wm.manage_window(*window);
            assert!(result.is_ok(), "Failed to manage window: {}", title);
            managed_count += 1;
        }
    }

    println!("Successfully managed {} windows", managed_count);

    // Test workspace switching with managed windows
    if managed_count > 0 {
        println!("Testing workspace switch...");
        wm.switch_workspace(2)
            .expect("Failed to switch to workspace 2");
        assert_eq!(wm.get_active_workspace(), 2);

        wm.switch_workspace(1)
            .expect("Failed to switch back to workspace 1");
        assert_eq!(wm.get_active_workspace(), 1);
    }

    // Unmanage windows
    for window in windows.iter().take(3) {
        if let Err(e) = wm.unmanage_window(window) {
            eprintln!("Warning: Failed to unmanage window: {}", e);
        }
    }

    println!("Integration test completed successfully");
}

/// Test that the window manager can handle multiple workspaces with windows
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_multi_workspace_windows() {
    use tiling_wm_core::utils::win32;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");

    if windows.len() < 4 {
        println!("Skipping test - need at least 4 windows");
        return;
    }

    // Add windows to workspace 1
    wm.switch_workspace(1)
        .expect("Failed to switch to workspace 1");
    for window in windows.iter().take(2) {
        if wm.should_manage_window(window).unwrap_or(false) {
            wm.manage_window(*window)
                .expect("Failed to manage window in workspace 1");
        }
    }

    // Add windows to workspace 2
    wm.switch_workspace(2)
        .expect("Failed to switch to workspace 2");
    for window in windows.iter().skip(2).take(2) {
        if wm.should_manage_window(window).unwrap_or(false) {
            wm.manage_window(*window)
                .expect("Failed to manage window in workspace 2");
        }
    }

    // Switch between workspaces
    wm.switch_workspace(1)
        .expect("Failed to switch to workspace 1");
    wm.switch_workspace(2)
        .expect("Failed to switch to workspace 2");
    wm.switch_workspace(1)
        .expect("Failed to switch back to workspace 1");

    println!("Multi-workspace test completed");
}

/// Test monitor refresh functionality
#[test]
#[cfg(target_os = "windows")]
fn test_monitor_refresh() {
    let mut wm = WindowManager::new();

    // Initial refresh
    wm.refresh_monitors().expect("Failed to refresh monitors");
    let initial_count = wm.get_monitors().len();

    // Refresh again - should still work
    wm.refresh_monitors()
        .expect("Failed to refresh monitors again");
    let second_count = wm.get_monitors().len();

    // Monitor count should be consistent (unless monitors were physically added/removed)
    assert_eq!(
        initial_count, second_count,
        "Monitor count should be consistent"
    );
}

/// Test that workspace trees are properly managed
#[test]
fn test_workspace_tree_management() {
    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    // All initial workspace trees should exist
    for workspace_id in 1..=10 {
        let tree = wm.get_workspace_tree(workspace_id);
        assert!(
            tree.is_some(),
            "Workspace {} should have a tree",
            workspace_id
        );
    }
}

/// Test layout type switching
#[test]
fn test_layout_switching() {
    use tiling_wm_core::window_manager::LayoutType;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    // Default should be Dwindle
    assert_eq!(wm.get_current_layout(), LayoutType::Dwindle);

    // Switch to Master layout
    wm.set_layout(LayoutType::Master)
        .expect("Failed to set Master layout");
    assert_eq!(wm.get_current_layout(), LayoutType::Master);

    // Switch back to Dwindle
    wm.set_layout(LayoutType::Dwindle)
        .expect("Failed to set Dwindle layout");
    assert_eq!(wm.get_current_layout(), LayoutType::Dwindle);
}

/// Test window registry integration
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_window_registry() {
    use tiling_wm_core::utils::win32;
    use tiling_wm_core::window_manager::WindowState;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");

    if windows.is_empty() {
        println!("Skipping test - no windows available");
        return;
    }

    // Manage a window
    let window = &windows[0];
    if wm.should_manage_window(window).unwrap_or(false) {
        wm.manage_window(*window).expect("Failed to manage window");

        // Verify window is in registry with Tiled state
        // Note: We'd need to expose registry getter methods for this test
        println!("Window successfully managed and tracked in registry");

        // Clean up
        wm.unmanage_window(window)
            .expect("Failed to unmanage window");
    }
}

/// Test floating window state
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_floating_window() {
    use tiling_wm_core::utils::win32;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");

    if windows.is_empty() {
        println!("Skipping test - no windows available");
        return;
    }

    let window = &windows[0];
    if wm.should_manage_window(window).unwrap_or(false) {
        wm.manage_window(*window).expect("Failed to manage window");

        // Toggle floating state
        wm.toggle_floating(window)
            .expect("Failed to toggle floating");

        println!("Window toggled to floating state");

        // Toggle back to tiled
        wm.toggle_floating(window)
            .expect("Failed to toggle back to tiled");

        println!("Window toggled back to tiled state");

        // Clean up
        wm.unmanage_window(window)
            .expect("Failed to unmanage window");
    }
}

/// Test fullscreen window state
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_fullscreen_window() {
    use tiling_wm_core::utils::win32;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");

    if windows.is_empty() {
        println!("Skipping test - no windows available");
        return;
    }

    let window = &windows[0];
    if wm.should_manage_window(window).unwrap_or(false) {
        wm.manage_window(*window).expect("Failed to manage window");

        // Toggle fullscreen
        wm.toggle_fullscreen(window)
            .expect("Failed to toggle fullscreen");

        println!("Window set to fullscreen");

        // Give it a moment to apply
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Exit fullscreen
        wm.toggle_fullscreen(window)
            .expect("Failed to exit fullscreen");

        println!("Window exited fullscreen");

        // Clean up
        wm.unmanage_window(window)
            .expect("Failed to unmanage window");
    }
}

/// Test retiling after window state changes
#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_retile_after_state_change() {
    use tiling_wm_core::utils::win32;

    let mut wm = WindowManager::new();
    wm.initialize().expect("Failed to initialize");

    let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");

    if windows.len() < 2 {
        println!("Skipping test - need at least 2 windows");
        return;
    }

    // Manage multiple windows
    for window in windows.iter().take(2) {
        if wm.should_manage_window(window).unwrap_or(false) {
            wm.manage_window(*window).expect("Failed to manage window");
        }
    }

    // Toggle one to floating
    wm.toggle_floating(&windows[0])
        .expect("Failed to toggle floating");

    println!("First window toggled to floating - workspace should retile");

    // Toggle it back
    wm.toggle_floating(&windows[0])
        .expect("Failed to toggle back");

    println!("First window toggled back to tiled - workspace should retile again");

    // Clean up
    for window in windows.iter().take(2) {
        wm.unmanage_window(window).ok();
    }
}
