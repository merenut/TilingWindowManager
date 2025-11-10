//! Virtual Desktop Integration Demo
//!
//! This example demonstrates how to use the Virtual Desktop Manager to interact
//! with Windows Virtual Desktops.
//!
//! Run with: `cargo run -p tiling-wm-core --example virtual_desktop_demo`
//!
//! Note: This example only works on Windows 10/11 with Virtual Desktop support.

#[cfg(target_os = "windows")]
use tenraku_core::workspace::VirtualDesktopManager;

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    println!("=== Virtual Desktop Integration Demo ===\n");

    // Create a Virtual Desktop Manager instance
    println!("Creating Virtual Desktop Manager...");
    let manager = VirtualDesktopManager::new()?;
    println!("✓ Virtual Desktop Manager created successfully\n");

    // Check if Virtual Desktop APIs are supported
    println!("Checking Virtual Desktop support...");
    let supported = manager.is_supported();
    if supported {
        println!("✓ Virtual Desktop APIs are supported\n");
    } else {
        println!("⚠ Virtual Desktop APIs are not fully supported on this system");
        println!("  This may be expected on older Windows versions\n");
    }

    // Get the desktop count
    println!("Getting desktop count...");
    match manager.get_desktop_count() {
        Ok(count) => {
            println!("✓ Desktop count: {}\n", count);
        }
        Err(e) => {
            println!("✗ Failed to get desktop count: {}\n", e);
        }
    }

    // Get all desktop IDs
    if supported {
        println!("Getting all desktop IDs...");
        match manager.get_desktop_ids() {
            Ok(ids) => {
                println!("✓ Found {} desktop(s):", ids.len());
                for (i, id) in ids.iter().enumerate() {
                    println!("  Desktop {}: {:?}", i + 1, id);
                }
                println!();
            }
            Err(e) => {
                println!("✗ Failed to get desktop IDs: {}\n", e);
            }
        }

        // Get the current desktop ID
        println!("Getting current desktop ID...");
        match manager.get_current_desktop_id() {
            Ok(id) => {
                println!("✓ Current desktop ID: {:?}\n", id);
            }
            Err(e) => {
                println!("✗ Failed to get current desktop ID: {}\n", e);
            }
        }

        // Demonstrate desktop switching by index
        println!("Testing desktop switching by index...");
        let count = manager.get_desktop_count().unwrap_or(1);
        if count >= 2 {
            println!("Switching to desktop 1...");
            match manager.switch_desktop_by_index(1) {
                Ok(_) => {
                    println!("✓ Switched to desktop 1");
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    println!("Switching back to desktop 0...");
                    match manager.switch_desktop_by_index(0) {
                        Ok(_) => println!("✓ Switched back to desktop 0\n"),
                        Err(e) => println!("✗ Failed to switch back: {}\n", e),
                    }
                }
                Err(e) => println!("✗ Failed to switch: {}\n", e),
            }
        } else {
            println!(
                "⚠ Need at least 2 desktops for switching demo (found {})\n",
                count
            );
        }

        // Demonstrate next/previous navigation with wraparound
        println!("Testing next/previous navigation...");
        if count >= 2 {
            println!("Switching to next desktop...");
            match manager.switch_to_next() {
                Ok(_) => {
                    println!("✓ Switched to next desktop");
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    println!("Switching to previous desktop...");
                    match manager.switch_to_previous() {
                        Ok(_) => println!("✓ Switched to previous desktop\n"),
                        Err(e) => println!("✗ Failed to switch previous: {}\n", e),
                    }
                }
                Err(e) => println!("✗ Failed to switch next: {}\n", e),
            }
        }

        // Demonstrate desktop creation and removal (commented out by default as it modifies state)
        println!("Desktop creation and removal demo (skipped - would modify system state)");
        println!("Uncomment the code in the example to test this functionality\n");

        /*
        println!("Creating a new desktop...");
        match manager.create_desktop() {
            Ok(new_id) => {
                println!("✓ Created new desktop: {:?}", new_id);

                let count_after = manager.get_desktop_count()?;
                println!("  Desktop count after creation: {}", count_after);

                // Get a fallback desktop ID
                let ids = manager.get_desktop_ids()?;
                if let Some(fallback_id) = ids.iter().find(|&id| *id != new_id) {
                    println!("Removing the newly created desktop...");
                    match manager.remove_desktop(&new_id, fallback_id) {
                        Ok(_) => {
                            println!("✓ Removed desktop: {:?}", new_id);
                            let count_final = manager.get_desktop_count()?;
                            println!("  Desktop count after removal: {}\n", count_final);
                        }
                        Err(e) => println!("✗ Failed to remove desktop: {}\n", e),
                    }
                } else {
                    println!("⚠ Could not find fallback desktop for removal\n");
                }
            }
            Err(e) => println!("✗ Failed to create desktop: {}\n", e),
        }
        */
    }

    println!("=== Demo Complete ===");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("This example only works on Windows.");
    println!("Virtual Desktop integration is a Windows-specific feature.");
}
