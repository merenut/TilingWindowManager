# Keybinding System Guide

This document provides comprehensive information about the keybinding system in the Tiling Window Manager.

## Overview

The keybinding system allows you to register global hotkeys that execute commands when pressed. It supports:

- All modifier keys (Win, Ctrl, Alt, Shift)
- All letter and number keys (A-Z, 0-9)
- Arrow keys (Left, Right, Up, Down)
- Function keys (F1-F12)
- Special keys (Space, Enter, Escape, Tab, etc.)
- Symbol keys (brackets, semicolon, comma, etc.)

## Configuration

Keybindings are defined in your `config.toml` file under the `[[keybinds]]` section.

### Basic Syntax

```toml
[[keybinds]]
modifiers = ["Win"]
key = "Q"
command = "close"
```

### With Multiple Modifiers

```toml
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Q"
command = "exit"
```

### With Command Arguments

```toml
[[keybinds]]
modifiers = ["Win"]
key = "Return"
command = "exec"
args = ["cmd.exe"]
```

## Supported Modifiers

- `Win` - Windows key
- `Ctrl` - Control key
- `Alt` - Alt key
- `Shift` - Shift key

You can combine multiple modifiers in any order.

## Supported Keys

### Letters
`A`, `B`, `C`, ..., `Z` (case insensitive)

### Numbers
`0`, `1`, `2`, ..., `9`

### Arrow Keys
`Left`, `Right`, `Up`, `Down`

### Function Keys
`F1`, `F2`, `F3`, ..., `F12`

### Special Keys
- `Space` - Spacebar
- `Enter` or `Return` - Enter key
- `Escape` or `ESC` - Escape key
- `Tab` - Tab key
- `Backspace` or `Back` - Backspace key
- `Delete` or `Del` - Delete key
- `Home` - Home key
- `End` - End key
- `PageUp` or `PgUp` - Page Up key
- `PageDown` or `PgDn` - Page Down key
- `Insert` or `Ins` - Insert key

### Symbol Keys
- `BracketLeft` or `[` - Left bracket
- `BracketRight` or `]` - Right bracket
- `Semicolon` or `;` - Semicolon
- `Quote` or `'` - Single quote
- `Comma` or `,` - Comma
- `Period` or `.` - Period
- `Slash` or `/` - Forward slash
- `Backslash` or `\` - Backslash
- `Minus` or `-` - Minus/hyphen
- `Equals` or `=` - Equals sign
- `Grave` or `` ` `` - Backtick/grave accent

## Supported Commands

### Window Commands
- `close` - Close the active window
- `toggle-floating` - Toggle floating/tiled state
- `toggle-fullscreen` - Toggle fullscreen mode
- `minimize` - Minimize the active window
- `restore` - Restore a minimized window

### Focus Commands
- `focus-left` - Focus the window to the left
- `focus-right` - Focus the window to the right
- `focus-up` - Focus the window above
- `focus-down` - Focus the window below
- `focus-previous` - Focus the previous window (Alt-Tab)
- `focus-next` - Focus the next window

### Move Commands
- `move-left` - Move active window left
- `move-right` - Move active window right
- `move-up` - Move active window up
- `move-down` - Move active window down
- `swap-master` - Swap active window with master

### Layout Commands
- `layout-dwindle` - Switch to dwindle layout
- `layout-master` - Switch to master-stack layout
- `increase-master` - Increase master window count
- `decrease-master` - Decrease master window count
- `increase-master-factor` - Increase master area size
- `decrease-master-factor` - Decrease master area size

### Workspace Commands
- `workspace-1` through `workspace-10` - Switch to workspace N
- `move-to-workspace-1` through `move-to-workspace-5` - Move active window to workspace N

### System Commands
- `reload-config` - Reload configuration from disk
- `exit` or `quit` - Exit the window manager

## Example Configuration

Here's a complete example with common keybindings:

```toml
# Window Management
[[keybinds]]
modifiers = ["Win"]
key = "q"
command = "close"

[[keybinds]]
modifiers = ["Win"]
key = "v"
command = "toggle-floating"

[[keybinds]]
modifiers = ["Win"]
key = "f"
command = "toggle-fullscreen"

[[keybinds]]
modifiers = ["Win"]
key = "m"
command = "minimize"

# Focus Navigation
[[keybinds]]
modifiers = ["Win"]
key = "Left"
command = "focus-left"

[[keybinds]]
modifiers = ["Win"]
key = "Right"
command = "focus-right"

[[keybinds]]
modifiers = ["Win"]
key = "Up"
command = "focus-up"

[[keybinds]]
modifiers = ["Win"]
key = "Down"
command = "focus-down"

[[keybinds]]
modifiers = ["Win"]
key = "Tab"
command = "focus-next"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Tab"
command = "focus-previous"

# Window Movement
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Left"
command = "move-left"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Right"
command = "move-right"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Up"
command = "move-up"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "Down"
command = "move-down"

# Layout Commands
[[keybinds]]
modifiers = ["Win"]
key = "d"
command = "layout-dwindle"

[[keybinds]]
modifiers = ["Win"]
key = "t"
command = "layout-master"

[[keybinds]]
modifiers = ["Win"]
key = "bracketleft"
command = "decrease-master"

[[keybinds]]
modifiers = ["Win"]
key = "bracketright"
command = "increase-master"

# Workspace Switching
[[keybinds]]
modifiers = ["Win"]
key = "1"
command = "workspace-1"

[[keybinds]]
modifiers = ["Win"]
key = "2"
command = "workspace-2"

[[keybinds]]
modifiers = ["Win"]
key = "3"
command = "workspace-3"

[[keybinds]]
modifiers = ["Win"]
key = "4"
command = "workspace-4"

[[keybinds]]
modifiers = ["Win"]
key = "5"
command = "workspace-5"

# Move Window to Workspace
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "1"
command = "move-to-workspace-1"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "2"
command = "move-to-workspace-2"

# System Commands
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "r"
command = "reload-config"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "e"
command = "exit"

# Application Launchers (requires exec command)
[[keybinds]]
modifiers = ["Win"]
key = "Return"
command = "exec"
args = ["cmd.exe"]
```

## Conflict Detection

The configuration validator automatically detects duplicate keybindings during config load. If you have conflicting keybindings, the validator will report an error and the config will not load.

## Hot Reload

Keybindings are automatically reloaded when you save changes to your `config.toml` file. The window manager will:

1. Unregister all existing hotkeys
2. Load the new configuration
3. Validate the keybindings
4. Register the new hotkeys

If validation fails, the previous keybindings remain active.

## Troubleshooting

### Hotkey Not Registering

If a hotkey fails to register, possible reasons include:

1. **Already in use** - Another application has registered the same hotkey
2. **System reserved** - Some key combinations are reserved by Windows
3. **Invalid syntax** - Check your config.toml for typos

The window manager logs warnings for any keybindings that fail to register.

### Common Reserved Combinations

These key combinations are typically reserved by Windows:

- `Win+L` - Lock screen
- `Win+D` - Show desktop
- `Win+E` - Open Explorer
- `Win+R` - Run dialog
- `Win+Tab` - Task view
- `Ctrl+Alt+Del` - Security options

## Implementation Details

### Windows API

The keybinding system uses the Windows `RegisterHotKey` API to register global hotkeys. When a hotkey is pressed:

1. Windows sends a `WM_HOTKEY` message
2. The event loop detects the message
3. The hotkey ID is used to look up the command
4. The command is executed through the CommandExecutor

### Performance

- Hotkey registration: < 1ms per keybinding
- Hotkey response time: < 10ms from keypress to command execution
- Memory overhead: ~100 bytes per keybinding

### Limitations

- Maximum 1000 keybindings per application (Windows limitation)
- Global hotkeys only (cannot be window-specific)
- Some key combinations may conflict with system shortcuts

## See Also

- [Configuration Guide](config/default_config.toml) - Full configuration documentation
- [Command System](COMMAND_SYSTEM_COMPLETE.md) - Available commands
- [Phase 4 Tasks](PHASE_4_TASKS.md) - Implementation details
