# Workspace Module

This module provides workspace management functionality, including integration with Windows Virtual Desktops.

## Virtual Desktop Integration

The Virtual Desktop integration allows the tiling window manager to interact with Windows 10/11 Virtual Desktops through COM interfaces.

### Features

- **Desktop Enumeration**: List all virtual desktops and their IDs
- **Desktop Detection**: Detect which desktop is currently active
- **Window Tracking**: Determine which desktop a window belongs to
- **Multi-Desktop Support**: Check if a window is on the current virtual desktop

### Usage Example

```rust
use tiling_wm_core::workspace::VirtualDesktopManager;

fn main() -> anyhow::Result<()> {
    // Create a Virtual Desktop Manager
    let manager = VirtualDesktopManager::new()?;
    
    // Check if Virtual Desktop APIs are supported
    if manager.is_supported() {
        println!("Virtual Desktop APIs are available!");
        
        // Get the number of desktops
        let count = manager.get_desktop_count()?;
        println!("Desktop count: {}", count);
        
        // Get all desktop IDs
        let ids = manager.get_desktop_ids()?;
        for (i, id) in ids.iter().enumerate() {
            println!("Desktop {}: {:?}", i + 1, id);
        }
        
        // Get current desktop ID
        let current = manager.get_current_desktop_id()?;
        println!("Current desktop: {:?}", current);
    } else {
        println!("Virtual Desktop APIs not supported on this system");
    }
    
    Ok(())
}
```

### Platform Support

This module is **Windows-only**. On non-Windows platforms:
- The module compiles but all operations return errors
- The code is properly gated with `#[cfg(target_os = "windows")]`
- No Windows-specific dependencies are required on other platforms

### COM Interfaces

The implementation uses both documented and undocumented Windows COM interfaces:

#### Documented Interfaces
- `IVirtualDesktopManager` - Official Windows API for basic virtual desktop operations

#### Undocumented Interfaces (Windows 10/11)
- `IVirtualDesktopManagerInternal` - Internal API for advanced operations
- `IVirtualDesktop` - Represents a single virtual desktop
- `IApplicationView` - Represents a window in the virtual desktop system

⚠️ **Warning**: The undocumented interfaces may change in future Windows versions. The implementation includes graceful fallback behavior for systems where these interfaces are not available.

### Testing

Tests are marked with `#[ignore]` because they require:
- Windows 10 or 11
- Virtual Desktop support enabled
- May modify system state (creating/switching desktops)

To run tests on Windows:

```bash
cargo test -p tiling-wm-core virtual_desktop -- --ignored --nocapture
```

### Running the Example

To see the Virtual Desktop integration in action:

```bash
cargo run -p tiling-wm-core --example virtual_desktop_demo
```

On non-Windows platforms, the example will display a message indicating that Virtual Desktop integration is Windows-only.

### Safety

All unsafe COM operations are encapsulated in safe wrapper functions with proper error handling. The implementation:

- ✅ Properly initializes and uninitializes COM
- ✅ Checks all COM return values (HRESULTs)
- ✅ Validates all pointers before dereferencing
- ✅ Uses Rust's type system to prevent memory leaks
- ✅ Handles errors gracefully with Result<T>

### API Reference

#### `VirtualDesktopManager::new()`
Creates a new Virtual Desktop manager instance. Initializes COM and creates the necessary COM objects.

**Returns**: `anyhow::Result<VirtualDesktopManager>`

**Errors**: Returns an error if COM initialization fails or if the Virtual Desktop Manager cannot be created.

#### `VirtualDesktopManager::is_supported()`
Checks if the system supports Virtual Desktop undocumented APIs.

**Returns**: `bool` - `true` if supported, `false` otherwise

#### `VirtualDesktopManager::get_desktop_count()`
Gets the number of virtual desktops.

**Returns**: `anyhow::Result<usize>`

**Errors**: Returns an error if the COM calls fail. Returns `Ok(1)` if internal manager is not available.

#### `VirtualDesktopManager::get_desktop_ids()`
Gets all virtual desktop IDs.

**Returns**: `anyhow::Result<Vec<GUID>>`

**Errors**: Returns an error if the COM calls fail. Returns `Ok(vec![])` if internal manager is not available.

#### `VirtualDesktopManager::get_current_desktop_id()`
Gets the ID of the currently active virtual desktop.

**Returns**: `anyhow::Result<GUID>`

**Errors**: Returns an error if the Virtual Desktop API is not available or if COM calls fail.

#### `VirtualDesktopManager::is_window_on_current_desktop(hwnd: HWND)`
Checks if a window is on the current virtual desktop.

**Parameters**:
- `hwnd` - Handle to the window to check

**Returns**: `anyhow::Result<bool>`

**Errors**: Returns an error if the COM call fails.

#### `VirtualDesktopManager::get_window_desktop_id(hwnd: HWND)`
Gets the virtual desktop ID that a window is on.

**Parameters**:
- `hwnd` - Handle to the window to query

**Returns**: `anyhow::Result<GUID>`

**Errors**: Returns an error if the COM call fails or if the window is not on any desktop.

### Known Limitations

1. **Windows Version Compatibility**: The undocumented COM interface GUIDs are specific to Windows 10 Build 10240+ and Windows 11. Different versions may require different GUIDs.

2. **Graceful Degradation**: If the undocumented APIs are not available, the manager will still work but with limited functionality (only documented API features).

3. **COM Initialization**: Each `VirtualDesktopManager` instance initializes COM. Multiple instances should be used with caution in multi-threaded environments.

### Future Enhancements

Potential future improvements include:
- Desktop switching functionality
- Desktop creation and deletion
- Window movement between desktops
- Desktop event notifications
- Windows version detection for correct GUID selection

## License

This module is part of the Tiling Window Manager project and follows the same license as the main project.
