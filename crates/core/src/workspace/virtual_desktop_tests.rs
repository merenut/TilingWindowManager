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

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_desktop_by_index() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        if count < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Get the current desktop
        let original_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Switch to desktop 1
        manager
            .switch_desktop_by_index(1)
            .expect("Failed to switch to desktop 1");

        // Verify we switched
        let new_id = manager
            .get_current_desktop_id()
            .expect("Failed to get new desktop ID");

        println!("Switched from {:?} to {:?}", original_id, new_id);
        assert_ne!(original_id, new_id, "Desktop should have changed");

        // Switch back to desktop 0
        manager
            .switch_desktop_by_index(0)
            .expect("Failed to switch back to desktop 0");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_desktop_by_index_out_of_range() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        // Try to switch to an out-of-range index
        let result = manager.switch_desktop_by_index(count + 10);
        assert!(
            result.is_err(),
            "Should fail when switching to out-of-range desktop"
        );
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_desktop_by_id() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");

        if ids.len() < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Get the current desktop
        let original_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Find a different desktop to switch to
        let target_id = ids.iter().find(|&id| *id != original_id).unwrap();

        // Switch to the target desktop
        manager
            .switch_desktop_by_id(target_id)
            .expect("Failed to switch desktop by ID");

        // Verify we switched
        let new_id = manager
            .get_current_desktop_id()
            .expect("Failed to get new desktop ID");

        println!("Switched from {:?} to {:?}", original_id, new_id);
        assert_eq!(*target_id, new_id, "Should be on the target desktop");

        // Switch back
        manager
            .switch_desktop_by_id(&original_id)
            .expect("Failed to switch back");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support and modifies system state
    fn test_create_and_remove_desktop() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let initial_count = manager
            .get_desktop_count()
            .expect("Failed to get initial desktop count");
        println!("Initial desktop count: {}", initial_count);

        // Create a new desktop
        let new_desktop_id = manager.create_desktop().expect("Failed to create desktop");
        println!("Created new desktop: {:?}", new_desktop_id);

        // Verify the count increased
        let new_count = manager
            .get_desktop_count()
            .expect("Failed to get new desktop count");
        assert_eq!(
            new_count,
            initial_count + 1,
            "Desktop count should increase by 1"
        );

        // Verify the new desktop is in the list
        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");
        assert!(
            ids.contains(&new_desktop_id),
            "New desktop should be in the list"
        );

        // Get a fallback desktop (the first one that's not the new one)
        let fallback_id = ids.iter().find(|&id| *id != new_desktop_id).unwrap();

        // Remove the desktop
        manager
            .remove_desktop(&new_desktop_id, fallback_id)
            .expect("Failed to remove desktop");
        println!("Removed desktop: {:?}", new_desktop_id);

        // Verify the count decreased
        let final_count = manager
            .get_desktop_count()
            .expect("Failed to get final desktop count");
        assert_eq!(
            final_count, initial_count,
            "Desktop count should return to initial value"
        );
    }

    #[test]
    #[ignore] // Requires a valid window handle
    fn test_move_window_to_desktop() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

        // Get the current foreground window
        let hwnd = unsafe { GetForegroundWindow() };

        if hwnd.0 == 0 {
            println!("No foreground window available, skipping test");
            return;
        }

        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");

        if ids.len() < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Get the window's current desktop
        let original_desktop = manager
            .get_window_desktop_id(hwnd)
            .expect("Failed to get window desktop ID");

        // Find a different desktop to move to
        let target_desktop = ids.iter().find(|&id| *id != original_desktop).unwrap();

        // Move the window
        manager
            .move_window_to_desktop(hwnd, target_desktop)
            .expect("Failed to move window to desktop");

        // Verify the window moved
        let new_desktop = manager
            .get_window_desktop_id(hwnd)
            .expect("Failed to get window desktop ID after move");

        assert_eq!(
            *target_desktop, new_desktop,
            "Window should be on the target desktop"
        );

        // Move it back
        manager
            .move_window_to_desktop(hwnd, &original_desktop)
            .expect("Failed to move window back");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_to_next() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        if count < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Get the current desktop
        let original_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Switch to next
        manager
            .switch_to_next()
            .expect("Failed to switch to next desktop");

        // Verify we switched
        let new_id = manager
            .get_current_desktop_id()
            .expect("Failed to get new desktop ID");

        println!("Switched from {:?} to {:?}", original_id, new_id);
        assert_ne!(original_id, new_id, "Desktop should have changed");

        // Switch back to original by going previous
        manager
            .switch_to_previous()
            .expect("Failed to switch to previous desktop");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_to_next_wraparound() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        if count < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Switch to the last desktop
        manager
            .switch_desktop_by_index(count - 1)
            .expect("Failed to switch to last desktop");

        let last_desktop_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Switch to next (should wrap around to first)
        manager
            .switch_to_next()
            .expect("Failed to switch to next (wraparound)");

        let first_desktop_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID after wraparound");

        println!(
            "Wrapped from last desktop {:?} to first {:?}",
            last_desktop_id, first_desktop_id
        );

        // Verify we're on the first desktop
        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");
        assert_eq!(
            ids[0], first_desktop_id,
            "Should be on the first desktop after wraparound"
        );
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_to_previous() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        if count < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Switch to desktop 1 (not the first)
        manager
            .switch_desktop_by_index(1)
            .expect("Failed to switch to desktop 1");

        let desktop1_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Switch to previous
        manager
            .switch_to_previous()
            .expect("Failed to switch to previous desktop");

        // Verify we switched
        let new_id = manager
            .get_current_desktop_id()
            .expect("Failed to get new desktop ID");

        println!("Switched from {:?} to {:?}", desktop1_id, new_id);
        assert_ne!(desktop1_id, new_id, "Desktop should have changed");
    }

    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_switch_to_previous_wraparound() {
        let manager = VirtualDesktopManager::new().expect("Failed to create manager");

        if !manager.is_supported() {
            println!("Virtual Desktop API not supported on this system, skipping test");
            return;
        }

        let count = manager
            .get_desktop_count()
            .expect("Failed to get desktop count");

        if count < 2 {
            println!("Need at least 2 desktops for this test, skipping");
            return;
        }

        // Switch to the first desktop
        manager
            .switch_desktop_by_index(0)
            .expect("Failed to switch to first desktop");

        let first_desktop_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID");

        // Switch to previous (should wrap around to last)
        manager
            .switch_to_previous()
            .expect("Failed to switch to previous (wraparound)");

        let last_desktop_id = manager
            .get_current_desktop_id()
            .expect("Failed to get current desktop ID after wraparound");

        println!(
            "Wrapped from first desktop {:?} to last {:?}",
            first_desktop_id, last_desktop_id
        );

        // Verify we're on the last desktop
        let ids = manager
            .get_desktop_ids()
            .expect("Failed to get desktop IDs");
        assert_eq!(
            ids[count - 1],
            last_desktop_id,
            "Should be on the last desktop after wraparound"
        );
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
