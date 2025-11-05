# IPC Examples

This directory contains example scripts demonstrating how to interact with the Tiling Window Manager via IPC.

## Prerequisites

- Window manager running with IPC enabled
- `twm` CLI tool installed and in PATH
- For PowerShell scripts: PowerShell 5.1+ or PowerShell Core
- For Python scripts: Python 3.6+

## Installing the CLI Tool

```bash
# Build and install the CLI tool
cargo install --path crates/cli

# Or build in debug mode
cargo build --package tiling-wm-cli

# The binary will be named 'twm' (Tiling Window Manager)
```

## PowerShell Examples

### 1. Monitor Window Events (`monitor-windows.ps1`)

Monitor window creation, closing, and focus events in real-time.

```powershell
.\examples\ipc\powershell\monitor-windows.ps1
```

**Features:**
- Real-time event monitoring
- Color-coded output
- Timestamps for each event
- Displays window title and workspace information

### 2. Switch Workspace (`switch-workspace.ps1`)

Quickly switch to a workspace by ID.

```powershell
.\examples\ipc\powershell\switch-workspace.ps1 3
```

**Parameters:**
- `WorkspaceId` (required): The ID of the workspace to switch to

### 3. Workspace Status (`workspace-status.ps1`)

Display current status of all workspaces.

```powershell
.\examples\ipc\powershell\workspace-status.ps1
```

**Features:**
- Shows all workspaces with their details
- Highlights active workspace
- Displays window count per workspace
- Shows monitor assignment

### 4. Toggle Layout (`toggle-layout.ps1`)

Toggle between dwindle and master layouts.

```powershell
.\examples\ipc\powershell\toggle-layout.ps1
```

**Features:**
- Automatically detects current layout
- Switches to the alternative layout
- Provides feedback on the switch

## Python Examples

### 1. Window Monitor (`window_monitor.py`)

Monitor window events with Python.

```bash
python examples/ipc/python/window_monitor.py
```

**Features:**
- Real-time event monitoring
- JSON parsing of IPC responses
- Clean output with timestamps
- Handles keyboard interrupt gracefully

### 2. Workspace Status (`workspace_status.py`)

Display workspace information using Python.

```bash
python examples/ipc/python/workspace_status.py
```

**Features:**
- Lists all workspaces
- Shows window count and monitor
- Indicates active workspace
- Error handling and validation

### 3. Window Info (`window_info.py`)

Display detailed information about the active window.

```bash
python examples/ipc/python/window_info.py
```

**Features:**
- Complete window metadata
- Position and size information
- Process information
- State information (tiled/floating/fullscreen)

### 4. Auto-Tiler (`auto_tiler.py`)

Automatically process new windows as they are created.

```bash
python examples/ipc/python/auto_tiler.py
```

**Features:**
- Monitors window creation events
- Template for custom automation rules
- Examples of conditional logic based on window properties
- Can be extended to implement custom tiling rules

## CLI Usage Examples

### Query Commands

```bash
# Get all windows
twm windows

# Get windows in specific workspace
twm windows --workspace 1

# Get active window
twm active-window

# List all workspaces
twm workspaces

# List monitors
twm monitors

# Get configuration info
twm config

# Get version information
twm version
```

### Window Operations

```bash
# Close active window
twm close

# Close specific window
twm close --window 12345

# Focus a window
twm focus 12345

# Move window to workspace 2
twm move 12345 2

# Toggle floating for active window
twm toggle-float

# Toggle fullscreen for active window
twm toggle-fullscreen
```

### Workspace Operations

```bash
# Switch to workspace 3
twm workspace 3

# Create new workspace
twm create-workspace "Development" --monitor 0

# Rename workspace
twm rename-workspace 1 "Main"

# Delete workspace
twm delete-workspace 5
```

### Layout Commands

```bash
# Set layout
twm layout dwindle
twm layout master

# Adjust master factor
twm exec master-factor 0.05
twm exec master-factor -0.05

# Change master count
twm exec increase-master
twm exec decrease-master
```

### System Commands

```bash
# Reload configuration
twm reload

# Ping the server (health check)
twm ping
```

### Event Subscription

```bash
# Listen to all events (JSON format)
twm --format json listen --events window_created,workspace_changed

# Monitor specific events
twm listen --events window_created,window_closed,window_focused

# Monitor workspace changes
twm listen --events workspace_changed,workspace_created,workspace_deleted

# Monitor configuration changes
twm listen --events config_reloaded,layout_changed
```

## Output Formats

The CLI supports three output formats:

### 1. Table Format (Default)

```bash
twm workspaces
```

Produces a formatted table with borders and colored output.

### 2. JSON Format

```bash
twm --format json workspaces
```

Produces machine-readable JSON output, perfect for scripting.

### 3. Compact Format

```bash
twm --format compact workspaces
```

Produces minimal output for piping or simple scripts.

## Event Types

Available event types for subscription:

- `window_created` - New window opened
- `window_closed` - Window closed
- `window_focused` - Window gained focus
- `window_moved` - Window moved to different workspace
- `window_state_changed` - Window state changed (tiled/floating/fullscreen)
- `workspace_changed` - Active workspace changed
- `workspace_created` - New workspace created
- `workspace_deleted` - Workspace deleted
- `monitor_changed` - Monitor configuration changed
- `config_reloaded` - Configuration reloaded
- `layout_changed` - Layout changed

## Creating Custom Scripts

### PowerShell Template

```powershell
#!/usr/bin/env pwsh

try {
    # Get data in JSON format
    $result = & twm --format json <command> 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: $result" -ForegroundColor Red
        exit 1
    }
    
    $data = ($result | ConvertFrom-Json).data
    
    # Process data
    # ...
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
```

### Python Template

```python
#!/usr/bin/env python3
import subprocess
import json
import sys

try:
    result = subprocess.run(
        ['twm', '--format', 'json', '<command>'],
        capture_output=True,
        text=True,
        check=False
    )
    
    if result.returncode != 0:
        print(f"Error: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    
    response = json.loads(result.stdout)
    data = response.get('data', {})
    
    # Process data
    # ...
    
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
```

## Troubleshooting

### CLI Tool Not Found

```bash
# Make sure the CLI tool is built and in PATH
cargo build --package tiling-wm-cli

# Or install it globally
cargo install --path crates/cli
```

### Connection Failed

```bash
# Verify the window manager is running
twm ping

# Check if the named pipe exists (Windows)
# The default pipe is: \\.\pipe\tiling-wm

# If using a custom pipe name, specify it:
twm --pipe \\.\pipe\custom-name <command>
```

### JSON Parsing Errors

```bash
# Ensure you're using JSON format for scripting
twm --format json <command>

# Check the output manually first
twm --format json workspaces | jq .
```

## Security Considerations

- Named pipes are local-only (cannot be accessed remotely on Windows)
- No authentication required (running as same user)
- Commands execute with same privileges as window manager
- Be cautious when running scripts from untrusted sources

## Advanced Usage

### Combining Commands

```bash
# Get active window and move it to workspace 2
$hwnd = (twm --format json active-window | ConvertFrom-Json).data.hwnd
twm move $hwnd 2
```

### Monitoring and Automation

```bash
# Monitor events and trigger actions
twm listen --events workspace_changed | ForEach-Object {
    $event = $_ | ConvertFrom-Json
    Write-Host "Switched from workspace $($event.data.from) to $($event.data.to)"
    # Trigger custom action...
}
```

### Integration with Other Tools

```bash
# Use with jq for JSON processing
twm --format json workspaces | jq '.data[] | select(.active == true)'

# Use with fzf for interactive selection
twm --format json windows | jq -r '.data[] | "\(.hwnd): \(.title)"' | fzf
```

## Contributing

To add new examples:

1. Create your script in the appropriate directory (powershell/ or python/)
2. Add documentation to this README
3. Test the script thoroughly
4. Submit a pull request

## License

These examples are part of the Tiling Window Manager project and follow the same license.
