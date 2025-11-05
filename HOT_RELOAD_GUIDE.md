# Configuration Hot-Reload Guide

## Overview

The Tiling Window Manager now supports **configuration hot-reload**, allowing you to modify the configuration file and see changes applied immediately without restarting the application.

## Features

- **Automatic Detection**: File changes are detected automatically using file system watchers
- **Debouncing**: Rapid edits are debounced to prevent reload storms (500ms default)
- **Validation**: Configurations are validated before being applied
- **Error Handling**: Invalid configurations are rejected, keeping the previous valid config active
- **Performance**: Reload completes in <100ms for valid configurations
- **User Feedback**: Success and failure messages are logged

## How It Works

1. The window manager watches the configuration file for changes
2. When a change is detected (after debounce period), the file is reloaded
3. The new configuration is validated
4. If valid, it's applied to the window manager:
   - Layout settings (gaps, ratios, etc.) are updated
   - Window rules are recompiled
   - All changes take effect immediately
5. If invalid, an error is logged and the previous configuration remains active

## Usage

### Enabling Hot-Reload

Hot-reload is enabled automatically when the window manager starts. You'll see:

```
Configuration hot-reload enabled
Configuration hot-reload is active
```

### Editing Configuration

1. Open your configuration file: `%APPDATA%\tiling-wm\config.toml`
2. Make your changes
3. Save the file
4. Changes are applied automatically within 1 second

### Monitoring Reload Status

Watch the log output for reload notifications:

```
Configuration change detected, reloading...
✓ Configuration reloaded successfully
Configuration reload completed in 45ms
```

### Handling Errors

If you save an invalid configuration, you'll see:

```
✗ Failed to reload configuration: <error details>
Continuing with previous configuration
```

The window manager continues running with the last valid configuration, so your workflow isn't interrupted.

## Debouncing Behavior

The hot-reload feature includes intelligent debouncing:

- **Initial change**: Detected immediately
- **Rapid edits**: Additional changes within 500ms are batched
- **Editor patterns**: Handles atomic writes, temp files, and other save patterns

This prevents reload storms when you're actively editing the file.

## What Gets Reloaded

When configuration is reloaded, the following are updated:

### Layout Settings
- Gap sizes (inner and outer)
- Split ratios
- Master layout factors
- Smart split behavior

### Window Rules
- Process name patterns
- Title patterns
- Class patterns
- Rule actions (float, tile, workspace assignment, etc.)

### Future Support
The following will be added in future updates:
- Keybindings (requires re-registration with Windows)
- Decoration settings
- Animation settings
- Monitor configurations

## Performance

Configuration reload is designed to be fast:

- **Target**: <100ms for valid configurations
- **Typical**: 20-50ms on modern systems
- **Warning**: If reload exceeds 100ms, a warning is logged

## Technical Details

### File System Events

The watcher listens for these file system events:
- `Modify::Data::Content` - Direct file content changes
- `Modify::Data::Any` - Any data modification
- `Create` - New file creation (handles atomic writes)
- `Remove` - File deletion (handles editor patterns)

### Debounce Algorithm

1. When an event is received, check if within debounce window
2. If yes, buffer the event without triggering reload
3. If no, trigger reload and reset debounce timer
4. This ensures one reload per edit session, not one per keystroke

### Error Recovery

The reload process is transactional:

```
1. Load new config from disk
2. Parse TOML
3. Validate all settings
4. Apply to window manager
   └─> If any step fails, rollback and keep previous config
```

## Troubleshooting

### Hot-Reload Not Working

**Symptom**: Configuration changes don't apply

**Solutions**:
1. Check if hot-reload is enabled at startup
2. Verify the configuration file path is correct
3. Look for error messages in the logs
4. Try restarting the window manager

### Configuration Rejected

**Symptom**: "Failed to reload configuration" errors

**Solutions**:
1. Check the error message for specific validation failures
2. Verify TOML syntax (use a TOML validator)
3. Check value ranges (e.g., opacity must be 0.0-1.0)
4. Ensure all required fields are present

### Slow Reload

**Symptom**: Reload takes >100ms

**Possible Causes**:
1. Very large configuration file
2. Many window rules (100+)
3. Complex regex patterns in rules
4. Disk I/O issues

**Solutions**:
1. Simplify configuration if possible
2. Optimize regex patterns
3. Check disk health

## Best Practices

1. **Test Changes Incrementally**: Make small changes and verify they work
2. **Keep Backup**: Keep a copy of working configuration
3. **Use Comments**: Document your changes for future reference
4. **Validate First**: Use a TOML validator before saving if unsure
5. **Watch Logs**: Keep log window visible when testing changes

## Examples

### Example 1: Adjusting Gaps

```toml
[general]
gaps_in = 10  # Change from 5 to 10
gaps_out = 20 # Change from 10 to 20
```

Save the file, and windows will immediately reposition with new gaps.

### Example 2: Adding a Window Rule

```toml
[[window_rules]]
match_process = "code\\.exe"
actions = ["float", { workspace = 3 }]
```

Save the file, and new VS Code windows will automatically float and go to workspace 3.

### Example 3: Updating Layout Ratios

```toml
[layouts.dwindle]
split_ratio = 0.6  # Change from 0.5 to 0.6
```

Save the file, and the next window split will use the new ratio.

## Integration with Development Workflow

For developers working on the configuration:

1. Open config file in your editor
2. Open window manager with visible logs
3. Make changes and save
4. Observe immediate feedback
5. Iterate quickly without restarts

This makes experimentation and tuning much faster!

## Limitations

### Current Limitations

1. **Keybindings**: Not hot-reloadable (requires Windows hotkey re-registration)
2. **Monitor Configs**: Requires restart to apply
3. **Existing Windows**: Changes don't affect already-managed windows
4. **Decoration**: Some decoration changes may require restart

### Workarounds

- For keybindings: Restart window manager
- For monitor configs: Restart window manager
- For existing windows: Close and reopen to apply new rules

## Future Enhancements

Planned improvements:

1. **Keybinding Hot-Reload**: Auto-unregister and re-register hotkeys
2. **Partial Reload**: Only reload changed sections
3. **Dry-Run Mode**: Test config without applying
4. **Config Diff**: Show what changed in reload logs
5. **Rollback Command**: CLI command to revert to previous config
6. **Config History**: Keep last N valid configurations

## See Also

- [Configuration Schema Documentation](CONFIGURATION.md)
- [Window Rules Guide](WINDOW_RULES.md)
- [Phase 4 Tasks](PHASE_4_TASKS.md)

## Support

If you encounter issues with hot-reload:

1. Check this guide for troubleshooting steps
2. Review log messages for error details
3. Report issues on GitHub with:
   - Log output
   - Configuration file (sanitized)
   - Steps to reproduce
