//! Virtual Desktop Integration Demo
//!
//! This example demonstrates how to use the Virtual Desktop Manager to interact
//! with Windows Virtual Desktops.
//!
//! Run with: `cargo run -p tiling-wm-core --example virtual_desktop_demo`
//!
//! Note: This example only works on Windows 10/11 with Virtual Desktop support.

#[cfg(target_os = "windows")]
use tiling_wm_core::workspace::VirtualDesktopManager;

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
    }

    println!("=== Demo Complete ===");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("This example only works on Windows.");
    println!("Virtual Desktop integration is a Windows-specific feature.");
}
