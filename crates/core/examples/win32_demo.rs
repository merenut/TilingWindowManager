//! Windows API Wrapper Demo
//!
//! This example demonstrates the functionality of the Windows API wrapper utilities.
//! It can only be run on Windows platforms.
//!
//! Run with: cargo run -p tiling-wm-core --example win32_demo

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use tiling_wm_core::utils::win32::*;

    println!("=== Windows API Wrapper Demo ===\n");
    
    // Test 1: Get foreground window
    println!("1. Getting foreground window...");
    if let Some(window) = get_foreground_window() {
        let title = window.get_title()?;
        let class = window.get_class_name()?;
        let pid = window.get_process_id();
        let rect = window.get_rect()?;
        println!("   Active: '{}' [{}] (PID: {})", title, class, pid);
        println!("   Position: ({}, {}), Size: {}x{}", 
            rect.left, rect.top, 
            rect.right - rect.left, 
            rect.bottom - rect.top
        );
    } else {
        println!("   No foreground window");
    }
    
    // Test 2: Enumerate all windows
    println!("\n2. Enumerating all windows...");
    let all_windows = enumerate_windows()?;
    println!("   Total: {} windows", all_windows.len());
    
    // Test 3: Enumerate visible windows
    println!("\n3. Enumerating visible windows...");
    let visible = enumerate_visible_windows()?;
    println!("   Visible: {} windows", visible.len());
    
    // Test 4: Enumerate app windows
    println!("\n4. Enumerating application windows...");
    let apps = enumerate_app_windows()?;
    println!("   Applications: {} windows", apps.len());
    
    // Test 5: Show some app windows
    println!("\n5. Sample application windows:");
    for (i, window) in apps.iter().take(5).enumerate() {
        let title = window.get_title().unwrap_or_default();
        let class = window.get_class_name().unwrap_or_default();
        let pid = window.get_process_id();
        let minimized = window.is_minimized();
        let maximized = window.is_maximized();
        
        println!("   {}. '{}'", i + 1, title);
        println!("      Class: {}", class);
        println!("      PID: {}", pid);
        println!("      State: {}{}",
            if minimized { "minimized " } else { "" },
            if maximized { "maximized " } else { "" }
        );
    }
    
    // Test 6: Filter by pattern
    println!("\n6. Searching for windows with 'code' in title...");
    let code_windows = filter_by_title_pattern(&apps, "code");
    println!("   Found: {} matching windows", code_windows.len());
    for window in code_windows.iter().take(3) {
        let title = window.get_title().unwrap_or_default();
        println!("      - {}", title);
    }
    
    // Test 7: Group windows by process
    println!("\n7. Grouping windows by process (showing top 3 processes)...");
    let mut process_windows: std::collections::HashMap<u32, Vec<_>> = std::collections::HashMap::new();
    for window in &apps {
        let pid = window.get_process_id();
        process_windows.entry(pid).or_default().push(window);
    }
    
    let mut process_counts: Vec<_> = process_windows.iter().collect();
    process_counts.sort_by_key(|(_, windows)| std::cmp::Reverse(windows.len()));
    
    for (pid, windows) in process_counts.iter().take(3) {
        println!("   PID {}: {} windows", pid, windows.len());
        for window in windows.iter().take(2) {
            let title = window.get_title().unwrap_or_default();
            if !title.is_empty() {
                println!("      - {}", title);
            }
        }
    }
    
    println!("\n=== Demo Complete ===");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This example can only be run on Windows");
    std::process::exit(1);
}
