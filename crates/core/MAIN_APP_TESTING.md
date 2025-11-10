# Main Application Testing Guide

This document provides guidance on testing the main application entry point (`main.rs`) of the Tiling Window Manager.

## Overview

The main application integrates:
- Event loop for monitoring Windows events
- Window manager for organizing and tiling windows
- Signal handling for clean shutdown (Ctrl+C)
- Automatic detection and management of existing windows
- Comprehensive logging for debugging

## Building and Running

### Prerequisites

- Windows 10/11 (required for full functionality)
- Rust 1.75+ with cargo
- Visual Studio Build Tools

### Building

```bash
# Debug build
cargo build -p tiling-wm-core

# Release build (optimized)
cargo build -p tiling-wm-core --release
```

### Running

```bash
# Run in debug mode (more verbose logging)
cargo run -p tiling-wm-core

# Run release build
cargo run -p tiling-wm-core --release

# Run with custom log level
RUST_LOG=tenraku_core=trace cargo run -p tiling-wm-core
```

## Expected Behavior

### Startup Sequence

When the application starts, you should see:

```
[INFO] Starting Tiling Window Manager v0.1.0
[INFO] Initializing window manager...
[INFO] Window manager initialized successfully
[INFO] Starting event loop...
[INFO] Event loop started successfully
[INFO] Scanning for existing windows...
[INFO] Found X existing windows
[INFO] Managing Y windows
[INFO] Tiling Window Manager is now running. Press Ctrl+C to exit.
```

### Runtime Behavior

1. **Existing Windows**: The application scans for existing windows and manages those that qualify
2. **New Windows**: When you open a new application, it should be detected and tiled automatically
3. **Window Events**: The application logs debug messages for all window events (creation, destruction, focus, etc.)
4. **Tiling**: Windows are automatically arranged in a binary tree layout with gaps

### Clean Shutdown

When you press Ctrl+C:

```
[INFO] Received Ctrl+C signal, initiating shutdown...
[INFO] Stopping event loop...
[INFO] Tiling Window Manager stopped successfully
```

## Manual Testing Procedure

### Test 1: Application Startup

1. Run the application: `cargo run -p tiling-wm-core`
2. Verify:
   - [ ] Application starts without errors
   - [ ] Logging is initialized
   - [ ] Event loop starts successfully
   - [ ] Window manager initializes
   - [ ] Existing windows are scanned

### Test 2: Window Detection

1. With the application running, open a new window (e.g., Notepad)
2. Verify:
   - [ ] `WindowCreated` event is logged
   - [ ] Window is automatically managed
   - [ ] Window title appears in logs
   - [ ] Window is tiled (positioned and sized)

### Test 3: Multiple Windows

1. Open 3-4 different applications (Notepad, Calculator, Explorer, etc.)
2. Verify:
   - [ ] Each window is detected
   - [ ] All windows are tiled in a tree layout
   - [ ] Windows don't overlap (unless intentionally)
   - [ ] Gap spacing is visible between windows

### Test 4: Window Closure

1. Close one or more managed windows
2. Verify:
   - [ ] `WindowDestroyed` event is logged
   - [ ] Window is unmanaged
   - [ ] Remaining windows are re-tiled
   - [ ] No errors in logs

### Test 5: Window Focus

1. Click between different managed windows
2. Verify:
   - [ ] `WindowFocused` events are logged
   - [ ] Focus changes are tracked
   - [ ] No errors occur

### Test 6: Window Minimize/Restore

1. Minimize a managed window
2. Restore the window
3. Verify:
   - [ ] `WindowMinimized` event is logged
   - [ ] `WindowRestored` event is logged
   - [ ] Window is re-tiled after restore
   - [ ] No layout corruption

### Test 7: Clean Shutdown

1. Press Ctrl+C while application is running
2. Verify:
   - [ ] Shutdown message appears
   - [ ] Event loop stops cleanly
   - [ ] No panic or crash
   - [ ] Application exits with code 0

### Test 8: Long-Running Stability

1. Run the application for 10+ minutes
2. During this time:
   - Open and close multiple windows
   - Switch between applications
   - Minimize/restore windows
3. Verify:
   - [ ] Memory usage stays stable (check Task Manager)
   - [ ] CPU usage is reasonable (<5% when idle)
   - [ ] No slowdown over time
   - [ ] No errors accumulate in logs

## Performance Expectations

- **CPU Usage (Idle)**: < 1-2%
- **CPU Usage (Active)**: 2-5% with frequent window operations
- **Memory Usage**: < 50 MB total
- **Event Processing Latency**: < 100ms from window creation to tiling
- **Startup Time**: < 500ms

## Troubleshooting

### Issue: Application fails to start

**Error**: "EventLoop is only supported on Windows"

**Solution**: This is expected on non-Windows platforms. The application requires Windows 10/11.

---

**Error**: "Failed to set event hook"

**Solution**: 
- Run as administrator if needed
- Check Windows Event Viewer for system errors
- Verify all Win32 features are enabled in Cargo.toml

### Issue: Windows not being managed

**Symptoms**: Windows open but aren't tiled

**Possible causes**:
1. Windows are filtered out (popups, tool windows, etc.)
2. Window filtering logic is too strict
3. Errors in window management (check logs)

**Solution**:
- Check debug logs for "should_manage_window" results
- Verify window has a title and is visible
- Test with simple applications (Notepad, Calculator)

### Issue: High CPU usage

**Symptoms**: CPU usage > 10% when idle

**Possible causes**:
1. Event loop running too fast
2. Too many events being processed
3. Inefficient window operations

**Solution**:
- Check the sleep duration in main event loop (should be 50ms)
- Review logs for excessive event churn
- Profile with Windows Performance Analyzer

### Issue: Memory leaks

**Symptoms**: Memory usage grows over time

**Possible causes**:
1. Windows not being unmanaged properly
2. Event queue backing up
3. Resource handles not released

**Solution**:
- Monitor with Task Manager or Process Explorer
- Check for accumulating managed windows
- Verify event loop cleanup on stop

## Known Limitations

1. **Single Workspace**: Currently only workspace 1 is active
2. **No Configuration**: Uses hardcoded settings (gaps, ratios, etc.)
3. **No Keybindings**: No keyboard shortcuts implemented yet
4. **Windows-Only**: Core functionality requires Windows platform
5. **No Status Bar**: No visual feedback yet
6. **No IPC**: No external control interface

## Future Enhancements

Planned improvements for future phases:

1. **Keybindings**: Add hotkeys for window management
2. **Configuration**: TOML-based configuration file
3. **Multiple Workspaces**: Support for workspace switching
4. **Floating Windows**: Allow some windows to float
5. **Window Rules**: Custom rules for specific applications
6. **Status Bar**: Visual status and workspace indicators
7. **IPC Server**: External control via CLI/scripts

## Acceptance Criteria

The main application is considered working when:

- [x] Application compiles without errors
- [x] Logging is initialized and working
- [x] Event loop starts and detects events
- [x] Window manager initializes with monitor detection
- [x] Existing windows are scanned on startup
- [ ] New windows are automatically detected and tiled (requires Windows testing)
- [ ] Window closure is detected and handled (requires Windows testing)
- [ ] Ctrl+C triggers clean shutdown (requires Windows testing)
- [ ] CPU usage is reasonable (<5% idle) (requires Windows testing)
- [ ] Memory usage is stable over time (requires Windows testing)

## Validation Commands

```bash
# Build the application
cargo build -p tiling-wm-core --release

# Run the application
cargo run -p tiling-wm-core

# Run tests
cargo test -p tiling-wm-core

# Check code quality
cargo clippy -p tiling-wm-core -- -D warnings

# Format code
cargo fmt -p tiling-wm-core --check
```

## Success Criteria

✅ **Complete** when:
- Application builds and runs without errors
- All manual tests pass on Windows
- Performance is within acceptable ranges
- Clean shutdown works reliably
- Memory/CPU usage is stable

## References

- [EVENT_LOOP_TESTING.md](EVENT_LOOP_TESTING.md) - Event loop testing guide
- [PHASE_1_TASKS.md](../../PHASE_1_TASKS.md) - Phase 1 task specifications
- [DETAILED_ROADMAP.md](../../DETAILED_ROADMAP.md) - Complete project roadmap

## Contact

For issues or questions, please open an issue on GitHub.

---

**Note**: Full testing requires a Windows 10/11 environment. On non-Windows platforms, the application will compile but fail at runtime with an appropriate error message.
