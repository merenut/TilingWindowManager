# Event Loop Testing Guide

This document explains how to test the Windows Event Loop implementation in `event_loop.rs`.

## Overview

The event loop monitors Windows events using the Win32 API's `SetWinEventHook` function and dispatches them through a thread-safe channel. It detects window creation, destruction, focus changes, and other window state changes.

## Automated Tests

The module includes comprehensive automated tests. Most tests require Windows and are marked with `#[cfg(target_os = "windows")]`.

### Running Tests on Windows

```bash
# Run all event_loop tests
cargo test -p tiling-wm-core event_loop

# Run with output
cargo test -p tiling-wm-core event_loop -- --nocapture

# Run the manual interactive test (requires user interaction)
cargo test -p tiling-wm-core test_window_events_detection -- --ignored --nocapture
```

### Test Coverage

The test suite includes:

1. **Platform-Independent Tests** (run on all platforms)
   - `test_event_loop_creation` - EventLoop creation
   - `test_poll_events_empty` - Empty event queue behavior

2. **Windows-Only Tests** (require Windows)
   - `test_event_loop_start_stop` - Start and stop functionality
   - `test_event_loop_double_start` - Idempotent start behavior
   - `test_event_loop_double_stop` - Idempotent stop behavior
   - `test_event_loop_drop_cleanup` - Automatic cleanup on drop
   - `test_process_messages` - Message pump processing

3. **Manual/Interactive Test** (requires Windows and user interaction)
   - `test_window_events_detection` - Detects real window events over 10 seconds

## Manual Testing on Windows

For manual verification on Windows, you can create a simple example program:

### Create Example File

Create `crates/core/examples/event_loop_demo.rs`:

```rust
use tenraku_core::event_loop::EventLoop;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("=== Event Loop Demo ===\n");
    println!("Starting event loop...");
    
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    
    println!("Event loop started successfully!");
    println!("Now open, close, minimize, or focus windows to see events.");
    println!("Press Ctrl+C to stop.\n");
    
    loop {
        // Process Windows messages
        event_loop.process_messages()?;
        
        // Poll for events
        for event in event_loop.poll_events() {
            println!("[EVENT] {:?}", event);
        }
        
        // Small sleep to prevent high CPU usage
        std::thread::sleep(Duration::from_millis(50));
    }
}
```

### Running the Example

```bash
# Run the example on Windows
cargo run -p tiling-wm-core --example event_loop_demo
```

### Expected Behavior

When you run the demo and interact with windows, you should see output like:

```
=== Event Loop Demo ===

Starting event loop...
Event loop started successfully!
Now open, close, minimize, or focus windows to see events.
Press Ctrl+C to stop.

[EVENT] WindowCreated(HWND(0x12345678))
[EVENT] WindowShown(HWND(0x12345678))
[EVENT] WindowFocused(HWND(0x12345678))
[EVENT] WindowMoved(HWND(0x12345678))
[EVENT] WindowHidden(HWND(0x87654321))
[EVENT] WindowDestroyed(HWND(0x87654321))
...
```

## Validation Checklist

- [ ] Event loop starts without errors
- [ ] Window creation events are detected when opening new applications
- [ ] Window destruction events are detected when closing applications
- [ ] Window show/hide events are detected when minimizing/restoring
- [ ] Window focus events are detected when switching between applications
- [ ] Window move events are detected when moving/resizing windows
- [ ] Event loop stops cleanly without errors
- [ ] No memory leaks (monitor with Task Manager or Process Explorer)
- [ ] No handle leaks (use Handle tool from Sysinternals)
- [ ] CPU usage is reasonable when idle (<5%)
- [ ] Application can run for extended periods without issues

## Event Types Detected

| Event Constant | WindowEvent Variant | Triggered When |
|---|---|---|
| `EVENT_OBJECT_CREATE` | `WindowCreated` | New window is created |
| `EVENT_OBJECT_DESTROY` | `WindowDestroyed` | Window is destroyed |
| `EVENT_OBJECT_SHOW` | `WindowShown` | Window becomes visible |
| `EVENT_OBJECT_HIDE` | `WindowHidden` | Window becomes hidden |
| `EVENT_SYSTEM_MOVESIZEEND` | `WindowMoved` | Window finished moving/resizing |
| `EVENT_OBJECT_LOCATIONCHANGE` | `WindowMoved` | Window location changed |
| `EVENT_SYSTEM_MINIMIZEEND` | `WindowRestored` | Window restored from minimize |
| `EVENT_SYSTEM_FOREGROUND` | `WindowFocused` | Window receives focus |

## Implementation Details

### Hook Registration

The event loop registers hooks using `SetWinEventHook` with:
- `WINEVENT_OUTOFCONTEXT` flag for out-of-context hooks (runs in our process)
- Separate hooks for each event type
- Process ID 0 and Thread ID 0 to monitor all processes/threads

### Thread Safety

- Uses `std::sync::mpsc` channels for thread-safe event communication
- Global static pointer for callback access (safe because lifetime is managed)
- Non-blocking `PeekMessageW` for message pump

### Memory Safety

- All hooks stored in Vec and unregistered in `stop()`
- Drop trait ensures cleanup even on panic
- No unsafe pointer manipulation outside of well-defined callback scope

## Known Limitations

1. **Platform Support**: Only works on Windows (stub implementation on other platforms)
2. **Event Filtering**: Receives all window events system-wide (filtering happens at consumer level)
3. **Performance**: High-frequency events (like mouse moves) are not monitored to avoid overhead
4. **Testing**: Full testing requires Windows environment

## Troubleshooting

### "Failed to set event hook" Error

**Cause**: SetWinEventHook failed, possibly due to permissions or system limitations.

**Solution**: 
- Run as administrator if needed
- Check Windows Event Viewer for system errors
- Verify all Win32 features are enabled in Cargo.toml

### High CPU Usage

**Cause**: Event loop running without delays.

**Solution**: 
- Ensure you call `std::thread::sleep()` in your event loop
- Use appropriate sleep duration (50-100ms recommended)
- Process messages in batches if needed

### Events Not Detected

**Cause**: Hooks may not be registered or callback not receiving events.

**Solution**:
- Verify `start()` returns Ok
- Check that windows you're testing with are manageable (not system windows)
- Enable debug logging to see if hooks are registered
- Try with different applications (e.g., Notepad, Calculator)

## Performance Benchmarks

Expected performance on a typical Windows system:

- **Hook Registration Time**: < 100ms for all 8 hooks
- **Event Processing Latency**: < 10ms from event to channel delivery
- **CPU Usage (Idle)**: < 1%
- **CPU Usage (Active)**: 2-5% with frequent window operations
- **Memory Usage**: < 1 MB additional for event loop structures

## Future Enhancements

Potential improvements for future iterations:

- **Window show/hide animations**: Leverage `AnimateWindow` with the `AW_BLEND` flag (and directional flags when needed) to fade top-level windows in and out for quick visual feedback during focus changes or window toggles.
- **Layered window fades**: Mark managed windows with `WS_EX_LAYERED` and drive opacity with `SetLayeredWindowAttributes` or per-frame alpha updates via `UpdateLayeredWindow` to enable gradual transparency transitions when hiding or minimizing tiles.
- **DWM-driven effects**: Use `DwmSetWindowAttribute` for immersive dark title bars, `DwmEnableBlurBehindWindow` for glass highlights, and `DwmTransitionOwnedWindow` to coordinate animations between tool windows owned by the manager.
- **Composition storyboards**: Adopt DirectComposition or the Win32 UI Animation Manager (available through the `windows` crate) to orchestrate time-based opacity/scale animations for focus swaps, workspace transitions, and stacked window rearrangements.
- **Resilience & fallbacks**: Watch `DwmIsCompositionEnabled`/`WM_DWMCOMPOSITIONCHANGED` and fall back to instant show/hide when composition is unavailable; call `DwmFlush` after batched updates to keep animations in sync with the compositor.

1. **Event Filtering**: Add filtering at hook level to reduce unnecessary events
2. **Batch Processing**: Group events for more efficient processing
3. **Event Priorities**: Prioritize important events (focus, create) over less critical ones (move)
4. **Performance Monitoring**: Built-in metrics for event throughput and latency
5. **Advanced Hooks**: Monitor additional events (DPI changes, display changes, etc.)
6. **Error Recovery**: Automatic hook re-registration on failure

## References

- [SetWinEventHook Documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [UnhookWinEvent Documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwinevent)
- [Event Constants](https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants)
- [windows-rs Crate](https://github.com/microsoft/windows-rs)

## Contact

For issues or questions about the event loop implementation, please open an issue on GitHub.
