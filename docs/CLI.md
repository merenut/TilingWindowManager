# CLI Documentation

## Overview

The `twm` (Tiling Window Manager) CLI is a command-line interface for controlling and querying the Tiling Window Manager via IPC. It provides a convenient way to interact with the window manager from the command line, scripts, or other automation tools.

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Global Options](#global-options)
- [Commands](#commands)
  - [Query Commands](#query-commands)
  - [Window Commands](#window-commands)
  - [Workspace Commands](#workspace-commands)
  - [Layout Commands](#layout-commands)
  - [System Commands](#system-commands)
- [Output Formats](#output-formats)
- [Examples](#examples)
- [Scripting](#scripting)
- [Troubleshooting](#troubleshooting)

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/merenut/TilingWindowManager.git
cd TilingWindowManager

# Build the CLI
cargo build --package tiling-wm-cli --release

# The binary will be at: target/release/twm.exe
```

### Installing

```bash
# Install globally
cargo install --path crates/cli

# Or add to PATH manually
# The binary is named 'twm' (or 'twm.exe' on Windows)
```

### Verifying Installation

```bash
twm --version
```

## Basic Usage

```bash
twm [OPTIONS] <COMMAND>
```

### Getting Help

```bash
# General help
twm --help

# Command-specific help
twm <command> --help

# Example
twm workspace --help
```

## Global Options

### Output Format

Control the output format of responses:

```bash
twm --format <FORMAT> <command>
```

Options: `json`, `table`, `compact`

Default: `table`

Examples:
```bash
twm --format json workspaces
twm --format table workspaces
twm --format compact workspaces
```

### Named Pipe Path

Specify a custom named pipe path:

```bash
twm --pipe <PATH> <command>
```

Default: `\\.\pipe\tiling-wm`

Example:
```bash
twm --pipe \\.\pipe\custom-wm windows
```

## Commands

### Query Commands

#### windows

Get a list of all windows or windows in a specific workspace.

```bash
twm windows [OPTIONS]
```

**Options:**
- `-w, --workspace <ID>` - Filter by workspace ID

**Examples:**
```bash
# Get all windows
twm windows

# Get windows in workspace 1
twm windows --workspace 1

# Get windows in JSON format
twm --format json windows
```

**Output (table format):**
```
┌──────┬──────────────────────────────────────────┬───────────┬──────────┬─────────┐
│ HWND │ Title                                    │ Workspace │ State    │ Focused │
├──────┼──────────────────────────────────────────┼───────────┼──────────┼─────────┤
│ 1234 │ Terminal - PowerShell                    │ 1         │ tiled    │ ✓       │
│ 5678 │ Visual Studio Code                       │ 1         │ tiled    │         │
│ 9012 │ Firefox                                  │ 2         │ floating │         │
└──────┴──────────────────────────────────────────┴───────────┴──────────┴─────────┘
```

#### active-window

Get information about the currently focused window.

```bash
twm active-window
```

**Examples:**
```bash
# Get active window info
twm active-window

# Get active window in JSON format
twm --format json active-window
```

**Output (table format):**
```
Success
  hwnd: 12345
  title: Terminal - PowerShell
  class: ConsoleWindowClass
  process_name: powershell.exe
  workspace: 1
  monitor: 0
  state: tiled
```

#### workspaces

List all workspaces with their metadata.

```bash
twm workspaces
```

**Examples:**
```bash
# List all workspaces
twm workspaces

# List workspaces in JSON format
twm --format json workspaces
```

**Output (table format):**
```
┌────┬──────────────┬─────────┬─────────┬────────┐
│ ID │ Name         │ Monitor │ Windows │ Active │
├────┼──────────────┼─────────┼─────────┼────────┤
│ 1  │ Workspace 1  │ 0       │ 3       │ ✓      │
│ 2  │ Workspace 2  │ 0       │ 1       │        │
│ 3  │ Workspace 3  │ 1       │ 0       │        │
└────┴──────────────┴─────────┴─────────┴────────┘
```

#### monitors

List all monitors with their properties.

```bash
twm monitors
```

**Examples:**
```bash
# List all monitors
twm monitors

# List monitors in JSON format
twm --format json monitors
```

**Output (table format):**
```
┌────┬───────────┬────────────┬──────────┬───────┬─────────┐
│ ID │ Name      │ Resolution │ Position │ Scale │ Primary │
├────┼───────────┼────────────┼──────────┼───────┼─────────┤
│ 0  │ Monitor 1 │ 1920x1080  │ 0,0      │ 1.00  │ ✓       │
│ 1  │ Monitor 2 │ 2560x1440  │ 1920,0   │ 1.25  │         │
└────┴───────────┴────────────┴──────────┴───────┴─────────┘
```

#### config

Get current configuration information.

```bash
twm config
```

**Examples:**
```bash
# Get config info
twm config

# Get config in JSON format
twm --format json config
```

**Output (table format):**
```
Success
  version: 0.1.0
  config_path: C:\Users\...\config.toml
  workspaces_count: 10
  layouts: ["dwindle", "master"]
  current_layout: dwindle
```

#### version

Get version and build information.

```bash
twm version
```

**Examples:**
```bash
# Get version info
twm version

# Get version in JSON format
twm --format json version
```

**Output (table format):**
```
Success
  version: 0.1.0
  build_date: 2024-11-05
  git_commit: abc123
  rustc_version: 1.75.0
```

### Window Commands

#### close

Close the active window or a specific window.

```bash
twm close [OPTIONS]
```

**Options:**
- `-w, --window <HWND>` - Window handle to close (defaults to active window)

**Examples:**
```bash
# Close active window
twm close

# Close specific window
twm close --window 12345
```

#### focus

Set focus to a specific window.

```bash
twm focus <HWND>
```

**Arguments:**
- `<HWND>` - Window handle to focus

**Examples:**
```bash
# Focus window with HWND 12345
twm focus 12345
```

#### move

Move a window to a different workspace.

```bash
twm move <HWND> <WORKSPACE>
```

**Arguments:**
- `<HWND>` - Window handle to move
- `<WORKSPACE>` - Target workspace ID

**Examples:**
```bash
# Move window 12345 to workspace 2
twm move 12345 2
```

#### toggle-float

Toggle floating state for the active window or a specific window.

```bash
twm toggle-float [OPTIONS]
```

**Options:**
- `-w, --window <HWND>` - Window handle (defaults to active window)

**Examples:**
```bash
# Toggle floating for active window
twm toggle-float

# Toggle floating for specific window
twm toggle-float --window 12345
```

#### toggle-fullscreen

Toggle fullscreen state for the active window or a specific window.

```bash
twm toggle-fullscreen [OPTIONS]
```

**Options:**
- `-w, --window <HWND>` - Window handle (defaults to active window)

**Examples:**
```bash
# Toggle fullscreen for active window
twm toggle-fullscreen

# Toggle fullscreen for specific window
twm toggle-fullscreen --window 12345
```

### Workspace Commands

#### workspace

Switch to a different workspace.

```bash
twm workspace <ID>
```

**Arguments:**
- `<ID>` - Workspace ID to switch to

**Examples:**
```bash
# Switch to workspace 3
twm workspace 3
```

#### create-workspace

Create a new workspace.

```bash
twm create-workspace <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name for the new workspace

**Options:**
- `-m, --monitor <ID>` - Monitor ID (default: 0)

**Examples:**
```bash
# Create workspace on default monitor
twm create-workspace "Development"

# Create workspace on monitor 1
twm create-workspace "Development" --monitor 1
```

#### delete-workspace

Delete a workspace.

```bash
twm delete-workspace <ID>
```

**Arguments:**
- `<ID>` - Workspace ID to delete

**Examples:**
```bash
# Delete workspace 5
twm delete-workspace 5
```

#### rename-workspace

Rename a workspace.

```bash
twm rename-workspace <ID> <NAME>
```

**Arguments:**
- `<ID>` - Workspace ID to rename
- `<NAME>` - New name for the workspace

**Examples:**
```bash
# Rename workspace 1 to "Main"
twm rename-workspace 1 "Main"
```

### Layout Commands

#### layout

Set the tiling layout.

```bash
twm layout <NAME>
```

**Arguments:**
- `<NAME>` - Layout name (`dwindle` or `master`)

**Examples:**
```bash
# Set dwindle layout
twm layout dwindle

# Set master layout
twm layout master
```

#### exec

Execute layout-specific commands.

```bash
twm exec <SUBCOMMAND>
```

**Subcommands:**

##### master-factor

Adjust the master area size.

```bash
twm exec master-factor <DELTA>
```

**Arguments:**
- `<DELTA>` - Amount to adjust (positive or negative float)

**Examples:**
```bash
# Increase master size by 5%
twm exec master-factor 0.05

# Decrease master size by 5%
twm exec master-factor -0.05
```

##### increase-master

Increase the number of windows in the master area.

```bash
twm exec increase-master
```

**Examples:**
```bash
twm exec increase-master
```

##### decrease-master

Decrease the number of windows in the master area.

```bash
twm exec decrease-master
```

**Examples:**
```bash
twm exec decrease-master
```

### System Commands

#### reload

Reload the configuration file.

```bash
twm reload
```

**Examples:**
```bash
# Reload configuration
twm reload
```

#### listen

Subscribe to events and listen for real-time updates.

```bash
twm listen [OPTIONS]
```

**Options:**
- `-e, --events <EVENTS>` - Comma-separated list of event names

**Event Types:**
- `window_created` - Window creation events
- `window_closed` - Window closing events
- `window_focused` - Window focus events
- `window_moved` - Window movement events
- `window_state_changed` - Window state changes
- `workspace_changed` - Workspace switching events
- `workspace_created` - Workspace creation events
- `workspace_deleted` - Workspace deletion events
- `monitor_changed` - Monitor configuration changes
- `config_reloaded` - Configuration reload events
- `layout_changed` - Layout change events

**Examples:**
```bash
# Listen to window events
twm listen --events window_created,window_closed

# Listen to workspace events
twm listen --events workspace_changed

# Listen to all events (JSON format)
twm --format json listen --events window_created,workspace_changed,config_reloaded
```

**Output:**
```
Event window_created: {"hwnd":"12345","title":"New Window","workspace":1}
Event workspace_changed: {"from":1,"to":2}
```

#### ping

Send a ping to the server (health check).

```bash
twm ping
```

**Examples:**
```bash
# Ping the server
twm ping
```

**Output:**
```
Pong
```

## Output Formats

The CLI supports three output formats, controlled by the `--format` option.

### Table Format (Default)

Human-readable table format with borders and colors.

```bash
twm workspaces
```

### JSON Format

Machine-readable JSON format, ideal for scripting.

```bash
twm --format json workspaces
```

Example output:
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
    }
  ]
}
```

### Compact Format

Minimal output for simple scripts and piping.

```bash
twm --format compact workspaces
```

Example output:
```
[{"id":1,"name":"Workspace 1","monitor":0,"window_count":3,"active":true,"visible":true}]
```

## Examples

### Basic Operations

```bash
# Check if window manager is running
twm ping

# Get version information
twm version

# List all workspaces
twm workspaces

# Switch to workspace 3
twm workspace 3

# Get active window
twm active-window

# Close active window
twm close
```

### Window Management

```bash
# List all windows
twm windows

# List windows in workspace 2
twm windows --workspace 2

# Get active window information
twm active-window

# Close specific window
twm close --window 12345

# Focus a window
twm focus 12345

# Move window to workspace 3
twm move 12345 3

# Toggle floating for active window
twm toggle-float

# Toggle fullscreen for active window
twm toggle-fullscreen
```

### Workspace Management

```bash
# List all workspaces
twm workspaces

# Switch to workspace 2
twm workspace 2

# Create a new workspace
twm create-workspace "Development"

# Rename workspace 1
twm rename-workspace 1 "Main"

# Delete workspace 5
twm delete-workspace 5
```

### Layout Management

```bash
# Get current layout
twm config | grep current_layout

# Switch to dwindle layout
twm layout dwindle

# Switch to master layout
twm layout master

# Adjust master area size
twm exec master-factor 0.05

# Increase master count
twm exec increase-master

# Decrease master count
twm exec decrease-master
```

### Event Monitoring

```bash
# Monitor window events
twm listen --events window_created,window_closed,window_focused

# Monitor workspace changes
twm listen --events workspace_changed

# Monitor all events in JSON format
twm --format json listen --events window_created,workspace_changed,layout_changed
```

## Scripting

### PowerShell

```powershell
# Get workspace data
$workspaces = (twm --format json workspaces | ConvertFrom-Json).data

# Find active workspace
$active = $workspaces | Where-Object { $_.active -eq $true }
Write-Host "Active workspace: $($active.name)"

# Switch to next workspace
$nextId = $active.id + 1
twm workspace $nextId

# Monitor events and take action
twm listen --events workspace_changed | ForEach-Object {
    $event = $_ | ConvertFrom-Json
    Write-Host "Switched from workspace $($event.data.from) to $($event.data.to)"
}
```

### Python

```python
import subprocess
import json

# Get workspace data
result = subprocess.run(
    ['twm', '--format', 'json', 'workspaces'],
    capture_output=True,
    text=True
)
workspaces = json.loads(result.stdout)['data']

# Find active workspace
active = next(ws for ws in workspaces if ws['active'])
print(f"Active workspace: {active['name']}")

# Switch to next workspace
next_id = active['id'] + 1
subprocess.run(['twm', 'workspace', str(next_id)])

# Monitor events
proc = subprocess.Popen(
    ['twm', '--format', 'json', 'listen', '--events', 'workspace_changed'],
    stdout=subprocess.PIPE,
    text=True
)

for line in proc.stdout:
    event = json.loads(line)
    print(f"Workspace changed: {event['data']}")
```

### Bash (WSL)

```bash
# Get active window
ACTIVE=$(twm --format json active-window | jq -r '.data.hwnd')

# Move to workspace 2
twm move "$ACTIVE" 2

# Monitor events
twm listen --events window_created | while read -r line; do
    echo "New window: $line"
done
```

## Troubleshooting

### CLI Tool Not Found

```bash
# Check if twm is in PATH
where twm

# If not, add it to PATH or use full path
C:\path\to\twm.exe --help
```

### Connection Failed

```bash
# Verify window manager is running
twm ping

# If it fails, check if the window manager is running
# Check the named pipe exists: \\.\pipe\tiling-wm

# If using custom pipe, specify it:
twm --pipe \\.\pipe\custom-name ping
```

### Invalid Output

```bash
# Try JSON format for debugging
twm --format json <command>

# Check for errors in stderr
twm <command> 2>&1
```

### Permission Issues

If you encounter permission errors:

1. Ensure the window manager is running
2. Run CLI with same user as window manager
3. Check named pipe permissions

### Platform Issues

The CLI is Windows-only. On other platforms:

```bash
# On Linux/macOS
twm --help
# Output: Error: This CLI tool only works on Windows.
```

### Response Timeout Issues

If commands hang or timeout:

1. **Check server responsiveness**
   ```bash
   # Test basic connectivity
   twm ping
   ```

2. **Verify window manager is not overloaded**
   - Check CPU usage of window manager process
   - Close unnecessary applications
   - Restart window manager if needed

3. **Try simpler commands first**
   ```bash
   # Start with quick queries
   twm version
   twm workspaces
   ```

4. **Check for blocking operations**
   - Some commands may take longer (e.g., with many windows)
   - Use JSON format to see if data is being returned

### Event Subscription Problems

If event listening doesn't work:

1. **Verify event names are correct**
   ```bash
   # Event names are case-sensitive and use underscores
   twm listen --events window_created,workspace_changed
   ```

2. **Test with single event first**
   ```bash
   # Start with one event to isolate issues
   twm listen --events window_created
   ```

3. **Check if events are actually firing**
   - Create a new window while listening
   - Switch workspaces while listening
   - Verify the action triggers an event

4. **Connection may have dropped**
   - Restart the listener
   - Check window manager is still running

### JSON Parsing Errors in Scripts

When using JSON output in scripts:

**PowerShell:**
```powershell
# Ensure proper error handling
try {
    $result = twm --format json workspaces | ConvertFrom-Json
    if ($result.type -eq "success") {
        # Process data
    }
} catch {
    Write-Error "Failed to parse JSON: $_"
}
```

**Python:**
```python
import json
import subprocess

try:
    result = subprocess.run(
        ['twm', '--format', 'json', 'workspaces'],
        capture_output=True,
        text=True,
        check=True
    )
    data = json.loads(result.stdout)
except subprocess.CalledProcessError as e:
    print(f"Command failed: {e}")
except json.JSONDecodeError as e:
    print(f"JSON parsing failed: {e}")
```

### Command Not Found After Installation

If `twm` command is not found after installation:

1. **Verify installation**
   ```bash
   cargo install --path crates/cli --force
   ```

2. **Check cargo bin directory is in PATH**
   ```bash
   # Windows PowerShell
   $env:PATH -split ';' | Select-String cargo
   
   # Add to PATH if needed
   $env:PATH += ";$env:USERPROFILE\.cargo\bin"
   ```

3. **Use full path as workaround**
   ```bash
   C:\Users\<username>\.cargo\bin\twm.exe --help
   ```

4. **Restart terminal**
   - Close and reopen terminal after installation
   - PATH changes may require new session

## Advanced Usage

### Combining Commands

```bash
# Get active window and move it
$hwnd = (twm --format json active-window | ConvertFrom-Json).data.hwnd
twm move $hwnd 2
```

### Conditional Logic

```bash
# Only switch if workspace has no windows
$ws = (twm --format json workspaces | ConvertFrom-Json).data | Where-Object { $_.id -eq 3 }
if ($ws.window_count -eq 0) {
    twm workspace 3
}
```

### Integration with Other Tools

```bash
# Use with jq
twm --format json workspaces | jq '.data[] | select(.active == true)'

# Use with fzf for interactive selection
twm --format json windows | jq -r '.data[] | "\(.hwnd): \(.title)"' | fzf
```

## See Also

- [IPC Protocol Documentation](IPC.md) - Detailed protocol specification
- [Example Scripts](../examples/ipc/README.md) - PowerShell and Python examples
- [GitHub Repository](https://github.com/merenut/TilingWindowManager) - Source code and issues

## Support

For issues, questions, or contributions:

- Open an issue on GitHub
- Check existing documentation
- Review example scripts
