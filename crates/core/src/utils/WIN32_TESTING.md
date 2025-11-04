# Windows API Wrapper Testing Guide

This document explains how to test the Windows API wrapper utilities in `win32.rs`.

## Automated Tests

The module includes comprehensive automated tests that verify all functionality. These tests only run on Windows.

### Running Tests on Windows

```bash
# Run all win32 tests
cargo test -p tiling-wm-core win32

# Run specific test
cargo test -p tiling-wm-core test_enumerate_windows

# Run with output
cargo test -p tiling-wm-core win32 -- --nocapture
```

### Test Coverage

The test suite includes:

1. **Unit Tests**
   - `test_window_handle_creation` - WindowHandle creation and validation
   - `test_window_handle_invalid` - Null handle detection
   - `test_window_handle_equality` - Handle comparison

2. **Enumeration Tests**
   - `test_enumerate_windows` - All window enumeration
   - `test_enumerate_visible_windows` - Visible window filtering
   - `test_enumerate_app_windows` - App window filtering

3. **Property Tests**
   - `test_window_properties` - Title, class, PID, TID retrieval
   - `test_window_state_queries` - Minimize/maximize/parent/owner queries
   - `test_get_foreground_window` - Active window retrieval

4. **Filter Tests**
   - `test_filter_by_process_id` - PID-based filtering
   - `test_filter_by_class_name` - Class name filtering
   - `test_filter_by_title` - Exact title matching
   - `test_filter_by_title_pattern` - Pattern-based title search

5. **Control Tests**
   - `test_window_control_methods_exist` - Verify control methods compile
   - `test_is_app_window` - App window filter logic

6. **Integration Tests**
   - `test_integration_enumerate_and_filter` - Complete workflow example

## Manual Testing

For manual verification on Windows, you can create a simple example program:

```rust
use tiling_wm_core::utils::win32::*;

fn main() -> anyhow::Result<()> {
    println!("=== Windows API Wrapper Demo ===\n");
    
    // Test 1: Get foreground window
    println!("1. Getting foreground window...");
    if let Some(window) = get_foreground_window() {
        let title = window.get_title()?;
        let class = window.get_class_name()?;
        let pid = window.get_process_id();
        println!("   Active: '{}' [{}] (PID: {})", title, class, pid);
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
    
    Ok(())
}
```

### Running the Manual Test

1. Create a new example file in `crates/core/examples/win32_demo.rs` with the above code
2. Run with: `cargo run -p tiling-wm-core --example win32_demo`

## Validation Checklist

- [x] All WindowHandle methods compile and work correctly
- [x] Window property retrieval (title, class, PID, TID, rect) works
- [x] Window state queries (visible, minimized, maximized) work
- [x] Window control methods exist and compile
- [x] Window hierarchy queries (parent, owner) work
- [x] Enumeration functions return correct results
- [x] Filter functions work correctly
- [x] Comprehensive doc comments on all public APIs
- [x] Proper error handling throughout
- [x] No memory leaks (proper buffer management)
- [x] Tests compile and run on Windows
- [x] `cargo clippy -p tiling-wm-core --lib -- -D warnings` passes

## Memory Safety

The implementation ensures no memory leaks through:

1. **String Operations**: All string buffers are properly managed
   - `Vec<u16>` for dynamic strings (get_title)
   - Fixed arrays for class names with known max length
   - `String::from_utf16_lossy` for safe UTF-16 conversion

2. **No Pointer Storage**: Window handles (HWND) are copied values, not stored pointers

3. **Proper Lifetimes**: All Win32 API calls are within properly scoped unsafe blocks

4. **Error Handling**: All API calls check return values and handle errors

5. **No Manual Memory Management**: Rust's RAII handles all memory cleanup

## Known Limitations

1. Tests only run on Windows (`#[cfg(target_os = "windows")]`)
2. Some control methods (minimize, maximize, etc.) are provided but not heavily tested to avoid side effects
3. The binary crate shows dead code warnings because the library APIs aren't used yet - this is expected

## Future Enhancements

Potential additions for future iterations:

- Window movement and resizing functions
- Window style and extended style queries
- DWM (Desktop Window Manager) integration
- Monitor information and multi-monitor support
- Keyboard and mouse input simulation
- Window message sending
