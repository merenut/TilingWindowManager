# IPC Protocol Documentation

## Overview

The Tiling Window Manager provides an IPC (Inter-Process Communication) interface for external control and automation. The IPC uses a JSON-based protocol over Windows named pipes, enabling external applications, scripts, and tools to interact with the window manager programmatically.

## Table of Contents

- [Connection](#connection)
- [Protocol Format](#protocol-format)
- [Request Types](#request-types)
  - [Query Requests](#query-requests)
  - [Command Requests](#command-requests)
  - [Event Subscription](#event-subscription)
- [Response Types](#response-types)
- [Event Types](#event-types)
- [Error Handling](#error-handling)
- [Security Considerations](#security-considerations)
- [Protocol Version](#protocol-version)

## Connection

### Named Pipe

**Default Path:** `\\.\pipe\tiling-wm`

The window manager creates a Windows named pipe server that listens for incoming connections. Multiple clients can connect simultaneously.

### Connecting from PowerShell

```powershell
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "tiling-wm", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect()
$reader = New-Object System.IO.StreamReader($pipe)
$writer = New-Object System.IO.StreamWriter($pipe)
```

### Connecting from Python

```python
import win32pipe
import win32file

pipe = win32file.CreateFile(
    r'\\.\pipe\tiling-wm',
    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
    0, None,
    win32file.OPEN_EXISTING,
    0, None
)
```

### Connecting from Rust

```rust
use tokio::net::windows::named_pipe::ClientOptions;

let client = ClientOptions::new()
    .open(r"\\.\pipe\tiling-wm")?;
```

## Protocol Format

Messages are framed with a 4-byte length prefix (little-endian uint32) followed by JSON data.

```
[4 bytes: length] [N bytes: JSON payload]
```

### Message Structure

1. **Length Prefix**: 4 bytes, little-endian uint32, specifies the length of the JSON payload
2. **JSON Payload**: UTF-8 encoded JSON object

### Example

For the JSON `{"type":"ping"}`:
- Length: 15 bytes
- Prefix: `0x0F 0x00 0x00 0x00`
- Payload: `{"type":"ping"}`

## Request Types

All requests are JSON objects with a `type` field that specifies the request type. Additional fields depend on the specific request.

### Query Requests

#### Get Active Window

Returns information about the currently focused window.

**Request:**
```json
{
  "type": "get_active_window"
}
```

**Response:**
```json
{
  "type": "success",
  "data": {
    "hwnd": "12345",
    "title": "Window Title",
    "class": "WindowClass",
    "process_name": "process.exe",
    "workspace": 1,
    "monitor": 0,
    "state": "tiled",
    "rect": {
      "x": 0,
      "y": 0,
      "width": 1920,
      "height": 1080
    },
    "focused": true
  }
}
```

#### Get Windows

Returns a list of all windows, optionally filtered by workspace.

**Request:**
```json
{
  "type": "get_windows",
  "workspace": 1  // optional
}
```

**Response:**
```json
{
  "type": "success",
  "data": [
    {
      "hwnd": "12345",
      "title": "Window Title",
      "class": "WindowClass",
      "process_name": "process.exe",
      "workspace": 1,
      "monitor": 0,
      "state": "tiled",
      "rect": {
        "x": 0,
        "y": 0,
        "width": 960,
        "height": 1080
      },
      "focused": false
    }
  ]
}
```

#### Get Workspaces

Returns information about all workspaces.

**Request:**
```json
{
  "type": "get_workspaces"
}
```

**Response:**
```json
{
  "type": "success",
  "data": [
    {
      "id": 1,
      "name": "Workspace 1",
      "monitor": 0,
      "window_count": 3,
      "active": true,
      "visible": true
    },
    {
      "id": 2,
      "name": "Workspace 2",
      "monitor": 0,
      "window_count": 1,
      "active": false,
      "visible": false
    }
  ]
}
```

#### Get Monitors

Returns information about all monitors.

**Request:**
```json
{
  "type": "get_monitors"
}
```

**Response:**
```json
{
  "type": "success",
  "data": [
    {
      "id": 0,
      "name": "Monitor 1",
      "width": 1920,
      "height": 1080,
      "x": 0,
      "y": 0,
      "scale": 1.0,
      "primary": true,
      "active_workspace": 1
    }
  ]
}
```

#### Get Config

Returns current configuration summary.

**Request:**
```json
{
  "type": "get_config"
}
```

**Response:**
```json
{
  "type": "success",
  "data": {
    "version": "0.1.0",
    "config_path": "C:\\Users\\...\\config.toml",
    "workspaces_count": 10,
    "layouts": ["dwindle", "master"],
    "current_layout": "dwindle"
  }
}
```

#### Get Version

Returns version and build information.

**Request:**
```json
{
  "type": "get_version"
}
```

**Response:**
```json
{
  "type": "success",
  "data": {
    "version": "0.1.0",
    "build_date": "2024-11-05",
    "git_commit": "abc123",
    "rustc_version": "1.75.0"
  }
}
```

### Command Requests

#### Switch Workspace

Switch to a different workspace.

**Request:**
```json
{
  "type": "switch_workspace",
  "id": 2
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Close Window

Close the active window or a specific window.

**Request:**
```json
{
  "type": "close_window",
  "hwnd": "12345"  // optional, defaults to active window
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Focus Window

Set focus to a specific window.

**Request:**
```json
{
  "type": "focus_window",
  "hwnd": "12345"
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Move Window

Move a window to a different workspace.

**Request:**
```json
{
  "type": "move_window",
  "hwnd": "12345",
  "workspace": 2
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Toggle Floating

Toggle floating state for a window.

**Request:**
```json
{
  "type": "toggle_floating",
  "hwnd": "12345"  // optional, defaults to active window
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Toggle Fullscreen

Toggle fullscreen state for a window.

**Request:**
```json
{
  "type": "toggle_fullscreen",
  "hwnd": "12345"  // optional, defaults to active window
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Create Workspace

Create a new workspace.

**Request:**
```json
{
  "type": "create_workspace",
  "name": "Development",
  "monitor": 0
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Delete Workspace

Delete a workspace.

**Request:**
```json
{
  "type": "delete_workspace",
  "id": 5
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Rename Workspace

Rename a workspace.

**Request:**
```json
{
  "type": "rename_workspace",
  "id": 1,
  "name": "Main"
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Set Layout

Change the tiling layout.

**Request:**
```json
{
  "type": "set_layout",
  "layout": "dwindle"  // or "master"
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Adjust Master Factor

Adjust the master area size in master layout.

**Request:**
```json
{
  "type": "adjust_master_factor",
  "delta": 0.05  // positive or negative
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Increase/Decrease Master Count

Change the number of windows in the master area.

**Request:**
```json
{
  "type": "increase_master_count"
}
// or
{
  "type": "decrease_master_count"
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Reload Config

Reload the configuration file.

**Request:**
```json
{
  "type": "reload_config"
}
```

**Response:**
```json
{
  "type": "success"
}
```

#### Ping

Health check request.

**Request:**
```json
{
  "type": "ping"
}
```

**Response:**
```json
{
  "type": "pong"
}
```

#### Quit

Signal the window manager to quit.

**Request:**
```json
{
  "type": "quit"
}
```

**Response:**
```json
{
  "type": "success"
}
```

### Event Subscription

#### Subscribe

Subscribe to specific events.

**Request:**
```json
{
  "type": "subscribe",
  "events": [
    "window_created",
    "workspace_changed"
  ]
}
```

**Response:**
```json
{
  "type": "success",
  "data": {
    "subscribed": [
      "window_created",
      "workspace_changed"
    ]
  }
}
```

After subscribing, the client will receive event messages for the subscribed events.

#### Unsubscribe

Unsubscribe from events.

**Request:**
```json
{
  "type": "unsubscribe"
}
```

**Response:**
```json
{
  "type": "success"
}
```

## Response Types

### Success Response

Indicates successful execution of a request.

```json
{
  "type": "success",
  "data": { ... }  // optional, depends on request
}
```

### Error Response

Indicates an error occurred.

```json
{
  "type": "error",
  "message": "Error description",
  "code": "ERROR_CODE"  // optional
}
```

### Event Response

Sent to subscribed clients when an event occurs.

```json
{
  "type": "event",
  "name": "window_created",
  "data": {
    "hwnd": "12345",
    "title": "New Window",
    "workspace": 1
  }
}
```

### Pong Response

Response to ping request.

```json
{
  "type": "pong"
}
```

## Event Types

When subscribed, the server sends events in this format:

```json
{
  "type": "event",
  "name": "<event_name>",
  "data": { ... }
}
```

### Available Events

#### window_created

Fired when a new window is created.

```json
{
  "type": "event",
  "name": "window_created",
  "data": {
    "hwnd": "12345",
    "title": "New Window",
    "workspace": 1
  }
}
```

#### window_closed

Fired when a window is closed.

```json
{
  "type": "event",
  "name": "window_closed",
  "data": {
    "hwnd": "12345"
  }
}
```

#### window_focused

Fired when a window gains focus.

```json
{
  "type": "event",
  "name": "window_focused",
  "data": {
    "hwnd": "12345"
  }
}
```

#### window_moved

Fired when a window is moved to a different workspace.

```json
{
  "type": "event",
  "name": "window_moved",
  "data": {
    "hwnd": "12345",
    "from_workspace": 1,
    "to_workspace": 2
  }
}
```

#### window_state_changed

Fired when a window's state changes.

```json
{
  "type": "event",
  "name": "window_state_changed",
  "data": {
    "hwnd": "12345",
    "old_state": "tiled",
    "new_state": "floating"
  }
}
```

#### workspace_changed

Fired when the active workspace changes.

```json
{
  "type": "event",
  "name": "workspace_changed",
  "data": {
    "from": 1,
    "to": 2
  }
}
```

#### workspace_created

Fired when a new workspace is created.

```json
{
  "type": "event",
  "name": "workspace_created",
  "data": {
    "id": 11,
    "name": "New Workspace"
  }
}
```

#### workspace_deleted

Fired when a workspace is deleted.

```json
{
  "type": "event",
  "name": "workspace_deleted",
  "data": {
    "id": 5
  }
}
```

#### monitor_changed

Fired when monitor configuration changes.

```json
{
  "type": "event",
  "name": "monitor_changed",
  "data": {}
}
```

#### config_reloaded

Fired when configuration is reloaded.

```json
{
  "type": "event",
  "name": "config_reloaded",
  "data": {}
}
```

#### layout_changed

Fired when the layout changes.

```json
{
  "type": "event",
  "name": "layout_changed",
  "data": {
    "layout": "master"
  }
}
```

## Error Handling

Errors are returned in this format:

```json
{
  "type": "error",
  "message": "Error description",
  "code": "ERROR_CODE"  // optional
}
```

### Common Errors

- **Invalid Request**: Malformed JSON or unknown request type
- **Not Found**: Requested window, workspace, or resource not found
- **Invalid Parameter**: Invalid parameter value
- **Not Implemented**: Feature not yet implemented
- **Internal Error**: Unexpected internal error

### Error Codes

While not all errors include error codes, some common ones include:

- `INVALID_REQUEST`: Request format is invalid
- `NOT_FOUND`: Resource not found
- `INVALID_PARAMETER`: Parameter value is invalid
- `NOT_IMPLEMENTED`: Feature not implemented
- `INTERNAL_ERROR`: Internal error occurred

## Security Considerations

### Local-Only Access

Named pipes on Windows are local-only by default and cannot be accessed remotely. This provides a baseline level of security.

### No Authentication

Currently, the IPC system does not require authentication. Any process running under the same user account can connect to the named pipe and send commands.

### Privilege Level

Commands execute with the same privileges as the window manager. This means:

- If the window manager runs as a regular user, IPC commands also run as that user
- If the window manager runs as administrator, IPC commands also have administrative privileges

### Best Practices

1. **Run as Non-Admin**: Run the window manager as a regular user when possible
2. **Access Control**: Consider implementing access control for production use
3. **Input Validation**: Always validate and sanitize input from IPC clients
4. **Audit Logging**: Consider logging IPC commands for audit purposes
5. **Rate Limiting**: Implement rate limiting to prevent abuse

### Future Enhancements

Planned security enhancements:

- Optional authentication mechanism
- Access control lists for specific commands
- Audit logging of IPC operations
- Rate limiting per client
- Sandboxing of command execution

## Protocol Version

**Current Protocol Version:** 1.0.0

The protocol version is included in the `GetVersion` response and can be used to ensure compatibility between clients and the server.

### Version Format

The protocol follows semantic versioning:

- **Major version**: Incompatible API changes
- **Minor version**: Backwards-compatible functionality additions
- **Patch version**: Backwards-compatible bug fixes

### Checking Version

```json
{
  "type": "get_version"
}
```

Response includes protocol version information.

## Implementation Notes

### Concurrency

- The IPC server supports multiple concurrent connections
- Each connection is handled in a separate async task
- Event subscriptions are per-connection
- No global state is shared between connections

### Performance

- Minimal overhead for IPC communication
- Async I/O for non-blocking operation
- Efficient JSON serialization
- No impact on window manager performance

### Reliability

- Graceful handling of client disconnections
- Error recovery mechanisms
- No crashes from invalid requests
- Connection limits to prevent resource exhaustion

## Examples

See the [examples/ipc](../../examples/ipc/) directory for complete examples in PowerShell and Python.

## Further Reading

- [CLI Documentation](CLI.md) - Using the command-line interface
- [Example Scripts](../../examples/ipc/README.md) - PowerShell and Python examples
- [Phase 5 Tasks](../PHASE_5_TASKS.md) - Implementation details

## Support

For issues, questions, or contributions, please visit the project repository.
