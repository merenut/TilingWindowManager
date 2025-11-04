//! Tests for Virtual Desktop COM interfaces and management
//!
//! These tests verify the Virtual Desktop functionality. Most tests are marked
//! with `#[ignore]` because they require actual Windows Virtual Desktop support
//! and may modify the system state.
//!
//! To run these tests on Windows:
//! ```
//! cargo test -p tiling-wm-core virtual_desktop -- --ignored --nocapture
//! ```

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::super::VirtualDesktopManager;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn test_create_manager() {
        let result = VirtualDesktopManager::new();
        assert!(
            result.is_ok(),
            "Failed to create VirtualDesktopManager: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_is_supported() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");
        // Just verify the method doesn't panic - support depends on Windows version
        let _supported = manager.is_supported();
        // Note: We can't assert true or false here since it depends on the Windows version
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_get_desktop_count() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");
        println!("Desktop count: {}", count);
        assert!(count > 0, "Desktop count should be at least 1");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_get_desktop_ids() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");
        println!("Found {} desktop(s)", ids.len());
        assert!(!ids.is_empty(), "Should have at least one desktop");

        // Print the desktop GUIDs for debugging
        for (i, id) in ids.iter().enumerate() {
            println!("Desktop {}: {:?}", i + 1, id);
        }
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_get_current_desktop_id() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let current_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");
        println!("Current desktop ID: {:?}", current_id);

        // Verify the current desktop is in the list of all desktops
        let all_ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");
        assert!(
            all_ids.contains(&current_id),
            "Current desktop should be in the list of all desktops"
        );
    }

    #[test]
    #[ignore] // Requires a valid window handle
    fn test_is_window_on_current_desktop() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        // Create a test HWND (Note: In real tests, you'd use GetForegroundWindow or similar)
        let hwnd = HWND(1 as _);

        // This might fail if the window doesn't exist, which is expected
        match manager.is_window_on_current_desktop(hwnd) {
            Ok(result) => {
                println!("Window on current desktop: {}", result);
            }
            Err(e) => {
                println!("Expected error for invalid HWND: {}", e);
            }
        }
    }

    #[test]
    #[ignore] // Requires a valid window handle
    fn test_get_window_desktop_id() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        // Create a test HWND (Note: In real tests, you'd use GetForegroundWindow or similar)
        let hwnd = HWND(1 as _);

        // This might fail if the window doesn't exist, which is expected
        match manager.get_window_desktop_id(hwnd) {
            Ok(id) => {
                println!("Window desktop ID: {:?}", id);
            }
            Err(e) => {
                println!("Expected error for invalid HWND: {}", e);
            }
        }
    }

    #[test]
    fn test_desktop_count_without_support() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        // get_desktop_count should return 1 even if internal manager is not available
        let count = manager
            .get_desktop_count()
            .expect("get_desktop_count should not fail");
        assert!(
            count >= 1,
            "Desktop count should be at least 1 even without internal support"
        );
    }

    #[test]
    fn test_desktop_ids_without_support() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        // get_desktop_ids should return empty vec if internal manager is not available
        let ids = manager
            .get_desktop_ids()
            .expect("get_desktop_ids should not fail");

        if manager.is_supported() {
            assert!(!ids.is_empty(), "Should have desktops if supported");
        } else {
            assert_eq!(ids.len(), 0, "Should have no desktops if not supported");
        }
    }
}

#[cfg(test)]
#[cfg(not(target_os = "windows"))]
mod non_windows_tests {
    use super::super::VirtualDesktopManager;

    #[test]
    fn test_new_fails_on_non_windows() {
        let result = VirtualDesktopManager::new();
        assert!(
            result.is_err(),
            "VirtualDesktopManager::new should fail on non-Windows platforms"
        );
    }
}
