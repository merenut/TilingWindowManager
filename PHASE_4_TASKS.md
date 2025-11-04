# Phase 4: Configuration & Rules - Detailed Task List

**Timeline:** Weeks 13-16 (4 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 4 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 4 Goals:**
- Implement complete TOML configuration system with schema validation
- Create comprehensive window rules engine with regex pattern matching
- Build configuration hot-reload capability for live updates
- Implement keybinding system for user-defined shortcuts
- Create workspace assignment rules
- Add monitor configuration support
- Provide default configuration with extensive documentation
- Enable rule-based automatic window management

**Prerequisites:**
- Phase 1 completed successfully (project foundation, Win32 wrappers, tree structure)
- Phase 2 completed successfully (window management, layouts, focus management, commands)
- Phase 3 completed successfully (workspace system, Virtual Desktop integration, persistence)
- All Phase 1-3 tests passing
- Window manager can manage windows across workspaces
- Command system is functional

---

## Success Criteria for Phase 4 Completion

Phase 4 is considered complete when:

1. **Configuration system fully functional:**
   - Can parse TOML configuration files
   - Schema validation catches errors
   - Default configuration is comprehensive
   - All config sections load correctly
   - Configuration paths work on all systems

2. **Window rules engine operational:**
   - Can match windows by process name
   - Can match windows by title (regex)
   - Can match windows by class name
   - Multiple rules can apply to one window
   - Actions are applied in correct order
   - NoManage action prevents window management

3. **Hot-reload working:**
   - Configuration changes detected automatically
   - Changes apply without restart
   - Invalid configs don't crash application
   - Reload completes within 100ms
   - User is notified of reload success/failure

4. **Keybinding system complete:**
   - Can parse keybind configurations
   - Supports modifier keys (Win, Ctrl, Alt, Shift)
   - Hotkey registration with Windows works
   - Commands execute on key press
   - Conflicts are detected and reported

5. **Workspace and monitor rules working:**
   - Workspace assignment rules apply
   - Monitor-specific configurations work
   - Per-monitor DPI settings apply
   - Multi-monitor configs are validated

6. **All tests passing:**
   - Unit tests for config parsing
   - Unit tests for rule matching
   - Integration tests for hot-reload
   - Keybinding tests
   - Manual validation successful

---

## Task Breakdown

### Week 13: Configuration Schema and Data Structures

#### Task 4.1: Create Configuration Schema

**Objective:** Define comprehensive configuration data structures with serde support for TOML parsing.

**File:** `crates/core/src/config/schema.rs`

**Required Implementations:**

1. **Create config module structure:**
   ```bash
   mkdir -p crates/core/src/config
   touch crates/core/src/config/mod.rs
   touch crates/core/src/config/schema.rs
   touch crates/core/src/config/parser.rs
   touch crates/core/src/config/validator.rs
   touch crates/core/src/config/watcher.rs
   ```

2. **Main Config struct:**

   ```rust
   use serde::{Serialize, Deserialize};
   use std::collections::HashMap;
   
   /// Root configuration structure
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Config {
       #[serde(default)]
       pub general: GeneralConfig,
       
       #[serde(default)]
       pub decoration: DecorationConfig,
       
       #[serde(default)]
       pub animations: AnimationsConfig,
       
       #[serde(default)]
       pub input: InputConfig,
       
       #[serde(default)]
       pub layouts: LayoutsConfig,
       
       #[serde(default)]
       pub window_rules: Vec<WindowRule>,
       
       #[serde(default)]
       pub workspace_rules: Vec<WorkspaceRule>,
       
       #[serde(default)]
       pub keybinds: Vec<Keybind>,
       
       #[serde(default)]
       pub monitors: Vec<MonitorConfig>,
   }
   
   impl Default for Config {
       fn default() -> Self {
           Self {
               general: GeneralConfig::default(),
               decoration: DecorationConfig::default(),
               animations: AnimationsConfig::default(),
               input: InputConfig::default(),
               layouts: LayoutsConfig::default(),
               window_rules: Vec::new(),
               workspace_rules: Vec::new(),
               keybinds: Vec::new(),
               monitors: Vec::new(),
           }
       }
   }
   ```

3. **GeneralConfig struct:**

   ```rust
   /// General window manager settings
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct GeneralConfig {
       /// Gap size between windows (pixels)
       #[serde(default = "default_gaps_in")]
       pub gaps_in: i32,
       
       /// Gap size around screen edges (pixels)
       #[serde(default = "default_gaps_out")]
       pub gaps_out: i32,
       
       /// Border size around windows (pixels)
       #[serde(default = "default_border_size")]
       pub border_size: i32,
       
       /// Active window border color (hex)
       #[serde(default = "default_active_border_color")]
       pub active_border_color: String,
       
       /// Inactive window border color (hex)
       #[serde(default = "default_inactive_border_color")]
       pub inactive_border_color: String,
       
       /// Enable auto-tiling for new windows
       #[serde(default = "default_true")]
       pub auto_tile: bool,
   }
   
   fn default_gaps_in() -> i32 { 5 }
   fn default_gaps_out() -> i32 { 10 }
   fn default_border_size() -> i32 { 2 }
   fn default_active_border_color() -> String { "#89b4fa".to_string() }
   fn default_inactive_border_color() -> String { "#585b70".to_string() }
   fn default_true() -> bool { true }
   
   impl Default for GeneralConfig {
       fn default() -> Self {
           Self {
               gaps_in: default_gaps_in(),
               gaps_out: default_gaps_out(),
               border_size: default_border_size(),
               active_border_color: default_active_border_color(),
               inactive_border_color: default_inactive_border_color(),
               auto_tile: default_true(),
           }
       }
   }
   ```

4. **DecorationConfig struct:**

   ```rust
   /// Window decoration settings
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DecorationConfig {
       /// Corner rounding radius (pixels)
       #[serde(default = "default_rounding")]
       pub rounding: i32,
       
       /// Active window opacity (0.0 - 1.0)
       #[serde(default = "default_opacity")]
       pub active_opacity: f32,
       
       /// Inactive window opacity (0.0 - 1.0)
       #[serde(default = "default_inactive_opacity")]
       pub inactive_opacity: f32,
       
       /// Enable window shadows
       #[serde(default = "default_true")]
       pub shadows: bool,
       
       /// Shadow color (hex with alpha)
       #[serde(default = "default_shadow_color")]
       pub shadow_color: String,
   }
   
   fn default_rounding() -> i32 { 10 }
   fn default_opacity() -> f32 { 1.0 }
   fn default_inactive_opacity() -> f32 { 0.9 }
   fn default_shadow_color() -> String { "#00000080".to_string() }
   
   impl Default for DecorationConfig {
       fn default() -> Self {
           Self {
               rounding: default_rounding(),
               active_opacity: default_opacity(),
               inactive_opacity: default_inactive_opacity(),
               shadows: default_true(),
               shadow_color: default_shadow_color(),
           }
       }
   }
   ```

5. **AnimationsConfig struct:**

   ```rust
   /// Animation settings
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AnimationsConfig {
       /// Enable animations globally
       #[serde(default = "default_true")]
       pub enabled: bool,
       
       /// Animation speed multiplier (1.0 = normal)
       #[serde(default = "default_animation_speed")]
       pub speed: f32,
       
       /// Animation curve type
       #[serde(default = "default_curve")]
       pub curve: AnimationCurve,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum AnimationCurve {
       Linear,
       EaseIn,
       EaseOut,
       EaseInOut,
   }
   
   fn default_animation_speed() -> f32 { 1.0 }
   fn default_curve() -> AnimationCurve { AnimationCurve::EaseOut }
   
   impl Default for AnimationsConfig {
       fn default() -> Self {
           Self {
               enabled: default_true(),
               speed: default_animation_speed(),
               curve: default_curve(),
           }
       }
   }
   ```

6. **InputConfig struct:**

   ```rust
   /// Input and keyboard settings
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct InputConfig {
       /// Keyboard repeat rate (characters per second)
       #[serde(default = "default_repeat_rate")]
       pub repeat_rate: u32,
       
       /// Keyboard repeat delay (milliseconds)
       #[serde(default = "default_repeat_delay")]
       pub repeat_delay: u32,
       
       /// Follow mouse focus
       #[serde(default = "default_false")]
       pub follow_mouse: bool,
   }
   
   fn default_repeat_rate() -> u32 { 25 }
   fn default_repeat_delay() -> u32 { 600 }
   fn default_false() -> bool { false }
   
   impl Default for InputConfig {
       fn default() -> Self {
           Self {
               repeat_rate: default_repeat_rate(),
               repeat_delay: default_repeat_delay(),
               follow_mouse: default_false(),
           }
       }
   }
   ```

7. **LayoutsConfig struct:**

   ```rust
   /// Layout-specific settings
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct LayoutsConfig {
       /// Default layout for new workspaces
       #[serde(default = "default_layout")]
       pub default: String,
       
       #[serde(default)]
       pub dwindle: DwindleConfig,
       
       #[serde(default)]
       pub master: MasterConfig,
   }
   
   fn default_layout() -> String { "dwindle".to_string() }
   
   impl Default for LayoutsConfig {
       fn default() -> Self {
           Self {
               default: default_layout(),
               dwindle: DwindleConfig::default(),
               master: MasterConfig::default(),
           }
       }
   }
   
   /// Dwindle layout configuration
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DwindleConfig {
       /// Automatically choose split direction based on window dimensions
       #[serde(default = "default_true")]
       pub smart_split: bool,
       
       /// Remove gaps when only one window
       #[serde(default = "default_false")]
       pub no_gaps_when_only: bool,
       
       /// Split ratio
       #[serde(default = "default_split_ratio")]
       pub split_ratio: f32,
   }
   
   fn default_split_ratio() -> f32 { 0.5 }
   
   impl Default for DwindleConfig {
       fn default() -> Self {
           Self {
               smart_split: default_true(),
               no_gaps_when_only: default_false(),
               split_ratio: default_split_ratio(),
           }
       }
   }
   
   /// Master layout configuration
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct MasterConfig {
       /// Size ratio for master window (0.0 - 1.0)
       #[serde(default = "default_master_factor")]
       pub master_factor: f32,
       
       /// Number of windows in master area
       #[serde(default = "default_master_count")]
       pub master_count: usize,
   }
   
   fn default_master_factor() -> f32 { 0.55 }
   fn default_master_count() -> usize { 1 }
   
   impl Default for MasterConfig {
       fn default() -> Self {
           Self {
               master_factor: default_master_factor(),
               master_count: default_master_count(),
           }
       }
   }
   ```

8. **WindowRule struct:**

   ```rust
   /// Window rule for automatic window management
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WindowRule {
       /// Match window by process name (regex)
       pub match_process: Option<String>,
       
       /// Match window by title (regex)
       pub match_title: Option<String>,
       
       /// Match window by class name (regex)
       pub match_class: Option<String>,
       
       /// Actions to apply when window matches
       pub actions: Vec<RuleAction>,
   }
   
   /// Actions that can be applied by window rules
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum RuleAction {
       /// Make window floating
       Float,
       
       /// Make window tiled
       Tile,
       
       /// Assign to specific workspace
       Workspace(usize),
       
       /// Assign to specific monitor
       Monitor(usize),
       
       /// Start in fullscreen
       Fullscreen,
       
       /// Don't focus this window automatically
       NoFocus,
       
       /// Don't manage this window at all
       NoManage,
       
       /// Set opacity
       Opacity(f32),
       
       /// Pin window (show on all workspaces)
       Pin,
   }
   ```

9. **WorkspaceRule struct:**

   ```rust
   /// Workspace assignment rule
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WorkspaceRule {
       /// Workspace ID
       pub id: usize,
       
       /// Monitor to assign workspace to
       pub monitor: usize,
       
       /// Make this the default workspace for the monitor
       #[serde(default)]
       pub default: bool,
       
       /// Custom name for workspace
       pub name: Option<String>,
   }
   ```

10. **Keybind struct:**

    ```rust
    /// Keybinding configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Keybind {
        /// Modifier keys (Win, Ctrl, Alt, Shift)
        pub modifiers: Vec<String>,
        
        /// Key to bind
        pub key: String,
        
        /// Command to execute
        pub command: String,
        
        /// Optional arguments for command
        #[serde(default)]
        pub args: Vec<String>,
    }
    ```

11. **MonitorConfig struct:**

    ```rust
    /// Monitor configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MonitorConfig {
        /// Monitor name or identifier
        pub name: String,
        
        /// Resolution (e.g., "1920x1080")
        pub resolution: Option<String>,
        
        /// Position (e.g., "0x0" or "auto")
        pub position: Option<String>,
        
        /// DPI scale factor
        pub scale: Option<f32>,
        
        /// Refresh rate (Hz)
        pub refresh_rate: Option<u32>,
        
        /// Rotation (0, 90, 180, 270)
        pub rotation: Option<u32>,
    }
    ```

**Acceptance Criteria:**
- [ ] All configuration structs compile without errors
- [ ] Structs derive Serialize and Deserialize correctly
- [ ] Default implementations are sensible
- [ ] Default functions are type-safe
- [ ] Documentation comments explain each field
- [ ] Enums use proper serde rename attributes
- [ ] Optional fields use Option<T>
- [ ] All numeric types have appropriate ranges

**Testing Requirements:**

Create `crates/core/src/config/schema_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert_eq!(config.general.gaps_in, 5);
        assert_eq!(config.general.gaps_out, 10);
        assert_eq!(config.general.border_size, 2);
        assert!(config.general.auto_tile);
        
        assert_eq!(config.decoration.rounding, 10);
        assert_eq!(config.decoration.active_opacity, 1.0);
        
        assert!(config.animations.enabled);
        assert_eq!(config.animations.speed, 1.0);
    }
    
    #[test]
    fn test_general_config_serialization() {
        let config = GeneralConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: GeneralConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.gaps_in, deserialized.gaps_in);
        assert_eq!(config.border_size, deserialized.border_size);
    }
    
    #[test]
    fn test_window_rule_deserialization() {
        let toml_str = r#"
            match_process = "firefox.exe"
            actions = ["float", { workspace = 2 }]
        "#;
        
        let rule: WindowRule = toml::from_str(toml_str).unwrap();
        
        assert_eq!(rule.match_process, Some("firefox.exe".to_string()));
        assert_eq!(rule.actions.len(), 2);
    }
    
    #[test]
    fn test_keybind_deserialization() {
        let toml_str = r#"
            modifiers = ["Win", "Shift"]
            key = "q"
            command = "close"
        "#;
        
        let keybind: Keybind = toml::from_str(toml_str).unwrap();
        
        assert_eq!(keybind.modifiers, vec!["Win", "Shift"]);
        assert_eq!(keybind.key, "q");
        assert_eq!(keybind.command, "close");
    }
    
    #[test]
    fn test_animation_curve_serialization() {
        let curve = AnimationCurve::EaseInOut;
        let toml_str = toml::to_string(&curve).unwrap();
        assert_eq!(toml_str.trim(), r#""easeinout""#);
    }
    
    #[test]
    fn test_layout_config_defaults() {
        let layouts = LayoutsConfig::default();
        
        assert_eq!(layouts.default, "dwindle");
        assert!(layouts.dwindle.smart_split);
        assert!(!layouts.dwindle.no_gaps_when_only);
        assert_eq!(layouts.master.master_factor, 0.55);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core config::schema
cargo clippy -p tiling-wm-core -- -D warnings
cargo doc --no-deps --document-private-items
```

**Expected Output:**
- All tests pass
- No clippy warnings
- Documentation builds successfully
- Struct layouts are logical and well-organized

---

#### Task 4.2: Create Default Configuration File

**Objective:** Create a comprehensive default TOML configuration file with extensive documentation.

**File:** `config/default_config.toml`

**Required Content:**

```toml
# ========================================
# Tiling Window Manager Configuration
# ========================================
# 
# This is the default configuration file for the Tiling Window Manager.
# Copy this file to your config directory to customize settings.
#
# Config Location (Windows):
#   %APPDATA%\tiling-wm\config.toml
#
# Documentation:
#   https://github.com/yourusername/tiling-wm/docs
#
# ========================================

# ========================================
# General Settings
# ========================================
[general]
# Gap size between windows (pixels)
gaps_in = 5

# Gap size from screen edges (pixels)
gaps_out = 10

# Border size around windows (pixels)
border_size = 2

# Active window border color (hex)
# Colors use standard hex format: #RRGGBB
active_border_color = "#89b4fa"

# Inactive window border color (hex)
inactive_border_color = "#585b70"

# Automatically tile new windows
auto_tile = true

# ========================================
# Decoration Settings
# ========================================
[decoration]
# Corner rounding radius (pixels)
# Set to 0 for square corners
rounding = 10

# Active window opacity (0.0 - 1.0)
# 1.0 = fully opaque, 0.0 = fully transparent
active_opacity = 1.0

# Inactive window opacity (0.0 - 1.0)
inactive_opacity = 0.9

# Enable window shadows
shadows = true

# Shadow color with alpha (hex)
# Format: #RRGGBBAA
shadow_color = "#00000080"

# ========================================
# Animation Settings
# ========================================
[animations]
# Enable animations globally
enabled = true

# Animation speed multiplier
# 1.0 = normal speed, 2.0 = twice as fast
speed = 1.0

# Animation curve
# Options: linear, easein, easeout, easeinout
curve = "easeout"

# ========================================
# Input Settings
# ========================================
[input]
# Keyboard repeat rate (characters per second)
repeat_rate = 25

# Keyboard repeat delay (milliseconds)
repeat_delay = 600

# Follow mouse focus
# When true, focus follows mouse cursor
follow_mouse = false

# ========================================
# Layout Settings
# ========================================
[layouts]
# Default layout for new workspaces
# Options: "dwindle", "master"
default = "dwindle"

# Dwindle Layout Settings
[layouts.dwindle]
# Automatically choose split direction based on window aspect ratio
smart_split = true

# Remove gaps when only one window is present
no_gaps_when_only = false

# Default split ratio (0.0 - 1.0)
split_ratio = 0.5

# Master Layout Settings
[layouts.master]
# Size ratio for master window (0.0 - 1.0)
master_factor = 0.55

# Number of windows in master area
master_count = 1

# ========================================
# Window Rules
# ========================================
# Window rules allow automatic actions based on window properties
# Rules are matched using regex patterns
# Multiple rules can match the same window

# Float Notepad windows
[[window_rules]]
match_process = "notepad\\.exe"
actions = ["float"]

# Send Firefox to workspace 2
[[window_rules]]
match_process = "firefox\\.exe"
actions = [{ workspace = 2 }]

# Float and send Calculator to workspace 3
[[window_rules]]
match_process = "calc\\.exe"
actions = ["float", { workspace = 3 }]

# Match by window title
[[window_rules]]
match_title = ".*Steam.*"
actions = ["float"]

# Don't manage popup windows
[[window_rules]]
match_class = ".*Popup.*"
actions = ["no_manage"]

# Set opacity for specific windows
[[window_rules]]
match_process = "discord\\.exe"
actions = [{ opacity = 0.95 }]

# Pin window to all workspaces
[[window_rules]]
match_title = "Task Manager"
actions = ["pin"]

# ========================================
# Workspace Rules
# ========================================
# Define workspace-to-monitor assignments

[[workspace_rules]]
id = 1
monitor = 0
default = true
name = "Main"

[[workspace_rules]]
id = 2
monitor = 0
name = "Web"

[[workspace_rules]]
id = 3
monitor = 0
name = "Code"

[[workspace_rules]]
id = 4
monitor = 0
name = "Media"

[[workspace_rules]]
id = 5
monitor = 1  # Second monitor
default = true
name = "Comm"

# ========================================
# Keybindings
# ========================================
# Define keyboard shortcuts
# Modifiers: Win, Ctrl, Alt, Shift
# Special keys: Left, Right, Up, Down, Space, Enter, Escape
# Number keys: 1-9, 0
# Letter keys: A-Z (case insensitive)

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

[[keybinds]]
modifiers = ["Win"]
key = "6"
command = "workspace-6"

[[keybinds]]
modifiers = ["Win"]
key = "7"
command = "workspace-7"

[[keybinds]]
modifiers = ["Win"]
key = "8"
command = "workspace-8"

[[keybinds]]
modifiers = ["Win"]
key = "9"
command = "workspace-9"

[[keybinds]]
modifiers = ["Win"]
key = "0"
command = "workspace-10"

# Move Window to Workspace
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "1"
command = "move-to-workspace-1"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "2"
command = "move-to-workspace-2"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "3"
command = "move-to-workspace-3"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "4"
command = "move-to-workspace-4"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "5"
command = "move-to-workspace-5"

# System Commands
[[keybinds]]
modifiers = ["Win", "Shift"]
key = "r"
command = "reload-config"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "e"
command = "exit"

# Application Launchers (requires external tools)
[[keybinds]]
modifiers = ["Win"]
key = "Return"
command = "exec"
args = ["cmd.exe"]

[[keybinds]]
modifiers = ["Win"]
key = "b"
command = "exec"
args = ["firefox.exe"]

# ========================================
# Monitor Configuration
# ========================================
# Configure monitor-specific settings
# Leave empty for automatic detection

# Example: Configure primary monitor
# [[monitors]]
# name = "\\\\?\\DISPLAY#DEL4098#..."
# resolution = "1920x1080"
# position = "0x0"
# scale = 1.0
# refresh_rate = 60
# rotation = 0

# Example: Configure secondary monitor
# [[monitors]]
# name = "\\\\?\\DISPLAY#DEL4099#..."
# resolution = "2560x1440"
# position = "1920x0"
# scale = 1.25
# refresh_rate = 144
# rotation = 0

# ========================================
# End of Configuration
# ========================================
```

**Acceptance Criteria:**
- [ ] Configuration file is valid TOML
- [ ] All sections are documented with comments
- [ ] Examples cover common use cases
- [ ] Default values match schema defaults
- [ ] File is well-organized and readable
- [ ] Contains at least 20 keybindings
- [ ] Includes 5+ window rules examples
- [ ] Has workspace configuration examples

**Testing Requirements:**

Create `crates/core/examples/validate_default_config.rs`:

```rust
use std::fs;

fn main() {
    println!("Validating default configuration file...");
    
    let config_content = include_str!("../../config/default_config.toml");
    
    // Try to parse as TOML
    match toml::from_str::<toml::Value>(config_content) {
        Ok(_) => println!("✓ Configuration is valid TOML"),
        Err(e) => {
            eprintln!("✗ Configuration parsing error: {}", e);
            std::process::exit(1);
        }
    }
    
    // Try to parse as Config struct
    match toml::from_str::<tiling_wm_core::config::schema::Config>(config_content) {
        Ok(config) => {
            println!("✓ Configuration successfully parsed into schema");
            println!("  - General config: gaps_in={}, gaps_out={}", 
                config.general.gaps_in, config.general.gaps_out);
            println!("  - Window rules: {}", config.window_rules.len());
            println!("  - Keybindings: {}", config.keybinds.len());
            println!("  - Workspace rules: {}", config.workspace_rules.len());
        }
        Err(e) => {
            eprintln!("✗ Schema parsing error: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n✓ Default configuration is valid!");
}
```

**Validation Commands:**
```bash
cargo run --example validate_default_config
toml-validator config/default_config.toml  # If toml-validator is installed
```

**Expected Output:**
```
Validating default configuration file...
✓ Configuration is valid TOML
✓ Configuration successfully parsed into schema
  - General config: gaps_in=5, gaps_out=10
  - Window rules: 7
  - Keybindings: 35
  - Workspace rules: 5

✓ Default configuration is valid!
```

---

### Week 14: Configuration Parser and Loader

#### Task 4.3: Implement Configuration Parser

**Objective:** Create a configuration loader that reads, parses, and validates TOML files.

**File:** `crates/core/src/config/parser.rs`

**Required Implementations:**

```rust
use super::schema::Config;
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};

/// Configuration loader and parser
pub struct ConfigLoader {
    /// Path to the configuration file
    config_path: PathBuf,
    
    /// Path to the default configuration
    default_config_path: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader with default paths
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("tiling-wm");
        
        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        let config_path = config_dir.join("config.toml");
        
        Ok(Self {
            config_path,
            default_config_path: None,
        })
    }
    
    /// Create a config loader with a custom path
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
            default_config_path: None,
        }
    }
    
    /// Set the path to the default configuration
    pub fn with_default(mut self, default_path: PathBuf) -> Self {
        self.default_config_path = Some(default_path);
        self
    }
    
    /// Load the configuration file
    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            tracing::info!(
                "Configuration file not found at {:?}, creating default",
                self.config_path
            );
            self.create_default_config()?;
        }
        
        self.load_from_path(&self.config_path)
    }
    
    /// Load configuration from a specific path
    pub fn load_from_path(&self, path: &PathBuf) -> Result<Config> {
        tracing::debug!("Loading configuration from {:?}", path);
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        
        tracing::info!("Successfully loaded configuration");
        tracing::debug!("Config: {:?}", config);
        
        Ok(config)
    }
    
    /// Create default configuration file
    pub fn create_default_config(&self) -> Result<()> {
        let default_content = if let Some(ref default_path) = self.default_config_path {
            fs::read_to_string(default_path)
                .context("Failed to read default config file")?
        } else {
            // Use embedded default config
            include_str!("../../../config/default_config.toml").to_string()
        };
        
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        fs::write(&self.config_path, default_content)
            .with_context(|| format!("Failed to write default config to {:?}", self.config_path))?;
        
        tracing::info!("Created default configuration at {:?}", self.config_path);
        Ok(())
    }
    
    /// Save configuration to disk
    pub fn save(&self, config: &Config) -> Result<()> {
        let toml_string = toml::to_string_pretty(config)
            .context("Failed to serialize configuration")?;
        
        // Backup existing config
        if self.config_path.exists() {
            let backup_path = self.config_path.with_extension("toml.bak");
            fs::copy(&self.config_path, &backup_path)
                .context("Failed to create config backup")?;
        }
        
        fs::write(&self.config_path, toml_string)
            .with_context(|| format!("Failed to write config to {:?}", self.config_path))?;
        
        tracing::info!("Saved configuration to {:?}", self.config_path);
        Ok(())
    }
    
    /// Get the configuration file path
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
    
    /// Check if configuration file exists
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigLoader")
    }
}
```

**Acceptance Criteria:**
- [ ] Can load configuration from default location
- [ ] Can load configuration from custom path
- [ ] Creates default config if not found
- [ ] Handles missing config directory
- [ ] Creates backup before saving
- [ ] Error messages are helpful
- [ ] Logging is comprehensive
- [ ] Works on Windows with proper paths

**Testing Requirements:**

Create `crates/core/src/config/parser_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_loader_from_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        assert_eq!(loader.get_config_path(), &config_path);
    }
    
    #[test]
    fn test_create_default_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        loader.create_default_config().unwrap();
        
        assert!(config_path.exists());
        
        // Verify content is valid
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[general]"));
        assert!(content.contains("[decoration]"));
    }
    
    #[test]
    fn test_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Write a simple config
        fs::write(&config_path, r#"
            [general]
            gaps_in = 10
            gaps_out = 20
        "#).unwrap();
        
        let loader = ConfigLoader::from_path(config_path);
        let config = loader.load().unwrap();
        
        assert_eq!(config.general.gaps_in, 10);
        assert_eq!(config.general.gaps_out, 20);
    }
    
    #[test]
    fn test_load_creates_default_if_missing() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        let config = loader.load().unwrap();
        
        assert!(config_path.exists());
        assert_eq!(config.general.gaps_in, 5); // Default value
    }
    
    #[test]
    fn test_save_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        
        let mut config = Config::default();
        config.general.gaps_in = 15;
        
        loader.save(&config).unwrap();
        
        assert!(config_path.exists());
        
        // Load and verify
        let loaded_config = loader.load().unwrap();
        assert_eq!(loaded_config.general.gaps_in, 15);
    }
    
    #[test]
    fn test_save_creates_backup() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        
        // Save initial config
        let config1 = Config::default();
        loader.save(&config1).unwrap();
        
        // Save modified config
        let mut config2 = Config::default();
        config2.general.gaps_in = 20;
        loader.save(&config2).unwrap();
        
        // Check backup exists
        let backup_path = config_path.with_extension("toml.bak");
        assert!(backup_path.exists());
    }
    
    #[test]
    fn test_invalid_toml_error() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Write invalid TOML
        fs::write(&config_path, "this is not valid toml {{{").unwrap();
        
        let loader = ConfigLoader::from_path(config_path);
        let result = loader.load();
        
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("parse"));
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core config::parser
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 4.4: Implement Configuration Validator

**Objective:** Create a validator to check configuration values and provide helpful error messages.

**File:** `crates/core/src/config/validator.rs`

**Required Implementations:**

```rust
use super::schema::*;
use anyhow::{Result, Context};
use std::collections::HashSet;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate an entire configuration
    pub fn validate(config: &Config) -> Result<()> {
        Self::validate_general(&config.general)?;
        Self::validate_decoration(&config.decoration)?;
        Self::validate_animations(&config.animations)?;
        Self::validate_layouts(&config.layouts)?;
        Self::validate_window_rules(&config.window_rules)?;
        Self::validate_workspace_rules(&config.workspace_rules)?;
        Self::validate_keybinds(&config.keybinds)?;
        Self::validate_monitors(&config.monitors)?;
        
        Ok(())
    }
    
    /// Validate general configuration
    fn validate_general(config: &GeneralConfig) -> Result<()> {
        if config.gaps_in < 0 {
            anyhow::bail!("gaps_in must be non-negative");
        }
        
        if config.gaps_out < 0 {
            anyhow::bail!("gaps_out must be non-negative");
        }
        
        if config.border_size < 0 {
            anyhow::bail!("border_size must be non-negative");
        }
        
        Self::validate_color(&config.active_border_color)
            .context("Invalid active_border_color")?;
        Self::validate_color(&config.inactive_border_color)
            .context("Invalid inactive_border_color")?;
        
        Ok(())
    }
    
    /// Validate decoration configuration
    fn validate_decoration(config: &DecorationConfig) -> Result<()> {
        if config.rounding < 0 {
            anyhow::bail!("rounding must be non-negative");
        }
        
        if !(0.0..=1.0).contains(&config.active_opacity) {
            anyhow::bail!("active_opacity must be between 0.0 and 1.0");
        }
        
        if !(0.0..=1.0).contains(&config.inactive_opacity) {
            anyhow::bail!("inactive_opacity must be between 0.0 and 1.0");
        }
        
        Self::validate_color(&config.shadow_color)
            .context("Invalid shadow_color")?;
        
        Ok(())
    }
    
    /// Validate animations configuration
    fn validate_animations(config: &AnimationsConfig) -> Result<()> {
        if config.speed <= 0.0 {
            anyhow::bail!("animation speed must be positive");
        }
        
        if config.speed > 10.0 {
            anyhow::bail!("animation speed should be reasonable (max 10.0)");
        }
        
        Ok(())
    }
    
    /// Validate layouts configuration
    fn validate_layouts(config: &LayoutsConfig) -> Result<()> {
        // Validate default layout
        if config.default != "dwindle" && config.default != "master" {
            anyhow::bail!("default layout must be 'dwindle' or 'master'");
        }
        
        // Validate dwindle config
        if !(0.1..=0.9).contains(&config.dwindle.split_ratio) {
            anyhow::bail!("dwindle split_ratio must be between 0.1 and 0.9");
        }
        
        // Validate master config
        if !(0.1..=0.9).contains(&config.master.master_factor) {
            anyhow::bail!("master master_factor must be between 0.1 and 0.9");
        }
        
        if config.master.master_count == 0 {
            anyhow::bail!("master master_count must be at least 1");
        }
        
        Ok(())
    }
    
    /// Validate window rules
    fn validate_window_rules(rules: &[WindowRule]) -> Result<()> {
        for (i, rule) in rules.iter().enumerate() {
            // Check that at least one match condition is specified
            if rule.match_process.is_none() 
                && rule.match_title.is_none() 
                && rule.match_class.is_none() 
            {
                anyhow::bail!(
                    "Window rule {} must have at least one match condition",
                    i
                );
            }
            
            // Validate regex patterns
            if let Some(ref pattern) = rule.match_process {
                regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_process", i))?;
            }
            
            if let Some(ref pattern) = rule.match_title {
                regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_title", i))?;
            }
            
            if let Some(ref pattern) = rule.match_class {
                regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_class", i))?;
            }
            
            // Validate actions
            if rule.actions.is_empty() {
                anyhow::bail!("Window rule {} must have at least one action", i);
            }
            
            for action in &rule.actions {
                Self::validate_rule_action(action)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate a single rule action
    fn validate_rule_action(action: &RuleAction) -> Result<()> {
        match action {
            RuleAction::Opacity(opacity) => {
                if !(0.0..=1.0).contains(opacity) {
                    anyhow::bail!("opacity must be between 0.0 and 1.0");
                }
            }
            RuleAction::Workspace(id) => {
                if *id == 0 {
                    anyhow::bail!("workspace ID must be at least 1");
                }
            }
            RuleAction::Monitor(id) => {
                // Monitor IDs are 0-based, so no validation needed
            }
            _ => {} // Other actions don't need validation
        }
        
        Ok(())
    }
    
    /// Validate workspace rules
    fn validate_workspace_rules(rules: &[WorkspaceRule]) -> Result<()> {
        let mut workspace_ids = HashSet::new();
        
        for rule in rules {
            if rule.id == 0 {
                anyhow::bail!("Workspace ID must be at least 1");
            }
            
            if workspace_ids.contains(&rule.id) {
                anyhow::bail!("Duplicate workspace ID: {}", rule.id);
            }
            
            workspace_ids.insert(rule.id);
        }
        
        Ok(())
    }
    
    /// Validate keybindings
    fn validate_keybinds(keybinds: &[Keybind]) -> Result<()> {
        let mut keybind_combinations = HashSet::new();
        
        for keybind in keybinds {
            // Validate modifiers
            for modifier in &keybind.modifiers {
                if !["Win", "Ctrl", "Alt", "Shift"].contains(&modifier.as_str()) {
                    anyhow::bail!("Invalid modifier: {}", modifier);
                }
            }
            
            // Check for duplicate keybindings
            let combination = format!(
                "{:?}+{}",
                keybind.modifiers,
                keybind.key
            );
            
            if keybind_combinations.contains(&combination) {
                anyhow::bail!("Duplicate keybinding: {}", combination);
            }
            
            keybind_combinations.insert(combination);
            
            // Validate command is not empty
            if keybind.command.is_empty() {
                anyhow::bail!("Keybind command cannot be empty");
            }
        }
        
        Ok(())
    }
    
    /// Validate monitor configurations
    fn validate_monitors(monitors: &[MonitorConfig]) -> Result<()> {
        for monitor in monitors {
            // Validate resolution format
            if let Some(ref res) = monitor.resolution {
                if !Self::is_valid_resolution(res) {
                    anyhow::bail!("Invalid resolution format: {}", res);
                }
            }
            
            // Validate position format
            if let Some(ref pos) = monitor.position {
                if pos != "auto" && !Self::is_valid_position(pos) {
                    anyhow::bail!("Invalid position format: {}", pos);
                }
            }
            
            // Validate scale
            if let Some(scale) = monitor.scale {
                if scale <= 0.0 || scale > 4.0 {
                    anyhow::bail!("Monitor scale must be between 0.0 and 4.0");
                }
            }
            
            // Validate rotation
            if let Some(rotation) = monitor.rotation {
                if ![0, 90, 180, 270].contains(&rotation) {
                    anyhow::bail!("Monitor rotation must be 0, 90, 180, or 270");
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate color format (hex)
    fn validate_color(color: &str) -> Result<()> {
        if !color.starts_with('#') {
            anyhow::bail!("Color must start with #");
        }
        
        let hex = &color[1..];
        
        // Support #RGB, #RRGGBB, #RRGGBBAA
        if ![3, 6, 8].contains(&hex.len()) {
            anyhow::bail!("Color must be #RGB, #RRGGBB, or #RRGGBBAA");
        }
        
        for ch in hex.chars() {
            if !ch.is_ascii_hexdigit() {
                anyhow::bail!("Color must contain only hex digits");
            }
        }
        
        Ok(())
    }
    
    /// Check if resolution format is valid (e.g., "1920x1080")
    fn is_valid_resolution(res: &str) -> bool {
        let parts: Vec<&str> = res.split('x').collect();
        if parts.len() != 2 {
            return false;
        }
        
        parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
    }
    
    /// Check if position format is valid (e.g., "0x0")
    fn is_valid_position(pos: &str) -> bool {
        let parts: Vec<&str> = pos.split('x').collect();
        if parts.len() != 2 {
            return false;
        }
        
        parts[0].parse::<i32>().is_ok() && parts[1].parse::<i32>().is_ok()
    }
}
```

**Acceptance Criteria:**
- [ ] All config sections have validation
- [ ] Numeric ranges are checked
- [ ] Regex patterns are validated
- [ ] Color formats are validated
- [ ] Duplicate values are detected
- [ ] Error messages are clear and helpful
- [ ] Validation is comprehensive
- [ ] Performance is acceptable

**Testing Requirements:**

Create `crates/core/src/config/validator_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    #[test]
    fn test_negative_gaps_in() {
        let mut config = Config::default();
        config.general.gaps_in = -5;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("gaps_in"));
    }
    
    #[test]
    fn test_invalid_opacity() {
        let mut config = Config::default();
        config.decoration.active_opacity = 1.5;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("opacity"));
    }
    
    #[test]
    fn test_invalid_color() {
        let mut config = Config::default();
        config.general.active_border_color = "not a color".to_string();
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_valid_color_formats() {
        assert!(ConfigValidator::validate_color("#fff").is_ok());
        assert!(ConfigValidator::validate_color("#ffffff").is_ok());
        assert!(ConfigValidator::validate_color("#ffffffff").is_ok());
        assert!(ConfigValidator::validate_color("#89b4fa").is_ok());
    }
    
    #[test]
    fn test_invalid_color_formats() {
        assert!(ConfigValidator::validate_color("fff").is_err());
        assert!(ConfigValidator::validate_color("#ff").is_err());
        assert!(ConfigValidator::validate_color("#gggggg").is_err());
    }
    
    #[test]
    fn test_window_rule_validation() {
        let mut config = Config::default();
        
        // Rule with no match conditions
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_regex_in_rule() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: Some("[invalid regex".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("regex"));
    }
    
    #[test]
    fn test_duplicate_workspace_ids() {
        let mut config = Config::default();
        
        config.workspace_rules.push(WorkspaceRule {
            id: 1,
            monitor: 0,
            default: true,
            name: None,
        });
        
        config.workspace_rules.push(WorkspaceRule {
            id: 1,
            monitor: 0,
            default: false,
            name: None,
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }
    
    #[test]
    fn test_duplicate_keybindings() {
        let mut config = Config::default();
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Win".to_string()],
            key: "q".to_string(),
            command: "close".to_string(),
            args: vec![],
        });
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Win".to_string()],
            key: "q".to_string(),
            command: "quit".to_string(),
            args: vec![],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }
    
    #[test]
    fn test_invalid_modifier() {
        let mut config = Config::default();
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Super".to_string()], // Invalid modifier
            key: "q".to_string(),
            command: "close".to_string(),
            args: vec![],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("modifier"));
    }
    
    #[test]
    fn test_monitor_resolution_validation() {
        assert!(ConfigValidator::is_valid_resolution("1920x1080"));
        assert!(ConfigValidator::is_valid_resolution("2560x1440"));
        assert!(!ConfigValidator::is_valid_resolution("1920"));
        assert!(!ConfigValidator::is_valid_resolution("invalid"));
    }
    
    #[test]
    fn test_monitor_position_validation() {
        assert!(ConfigValidator::is_valid_position("0x0"));
        assert!(ConfigValidator::is_valid_position("1920x0"));
        assert!(ConfigValidator::is_valid_position("-100x-50"));
        assert!(!ConfigValidator::is_valid_position("100"));
        assert!(!ConfigValidator::is_valid_position("invalid"));
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core config::validator
```

---

(Continuing in next message due to length...)

Would you like me to continue with the remaining weeks (Week 15: Window Rules Engine and Week 16: Configuration Hot-Reload)?
### Week 15: Window Rules Engine

#### Task 4.5: Implement Rule Matcher

**Objective:** Create a rule matching engine that applies window rules based on regex patterns.

**File:** `crates/core/src/rules/matcher.rs`

**Required Implementations:**

1. **Create rules module structure:**
   ```bash
   mkdir -p crates/core/src/rules
   touch crates/core/src/rules/mod.rs
   touch crates/core/src/rules/matcher.rs
   touch crates/core/src/rules/executor.rs
   ```

2. **RuleMatcher struct:**

   ```rust
   use crate::config::schema::{WindowRule, RuleAction};
   use crate::window_manager::window::ManagedWindow;
   use regex::Regex;
   use std::sync::Arc;
   
   /// Compiled window rule for efficient matching
   pub struct CompiledRule {
       /// Compiled regex for process name matching
       pub process_regex: Option<Regex>,
       
       /// Compiled regex for window title matching
       pub title_regex: Option<Regex>,
       
       /// Compiled regex for window class matching
       pub class_regex: Option<Regex>,
       
       /// Actions to apply when rule matches
       pub actions: Vec<RuleAction>,
   }
   
   /// Rule matcher that efficiently matches windows against rules
   pub struct RuleMatcher {
       /// List of compiled rules
       rules: Vec<Arc<CompiledRule>>,
   }
   
   impl RuleMatcher {
       /// Create a new rule matcher from window rules
       pub fn new(rules: Vec<WindowRule>) -> anyhow::Result<Self> {
           let mut compiled_rules = Vec::new();
           
           for (i, rule) in rules.into_iter().enumerate() {
               tracing::debug!("Compiling rule {}", i);
               
               let process_regex = if let Some(pattern) = rule.match_process {
                   Some(Regex::new(&pattern)
                       .with_context(|| format!("Invalid regex in rule {} match_process", i))?)
               } else {
                   None
               };
               
               let title_regex = if let Some(pattern) = rule.match_title {
                   Some(Regex::new(&pattern)
                       .with_context(|| format!("Invalid regex in rule {} match_title", i))?)
               } else {
                   None
               };
               
               let class_regex = if let Some(pattern) = rule.match_class {
                   Some(Regex::new(&pattern)
                       .with_context(|| format!("Invalid regex in rule {} match_class", i))?)
               } else {
                   None
               };
               
               compiled_rules.push(Arc::new(CompiledRule {
                   process_regex,
                   title_regex,
                   class_regex,
                   actions: rule.actions,
               }));
           }
           
           tracing::info!("Compiled {} window rules", compiled_rules.len());
           
           Ok(Self {
               rules: compiled_rules,
           })
       }
       
       /// Match a window against all rules and return matching actions
       pub fn match_window(&self, window: &ManagedWindow) -> Vec<RuleAction> {
           let mut actions = Vec::new();
           
           for rule in &self.rules {
               if self.rule_matches(rule, window) {
                   tracing::debug!(
                       "Rule matched for window '{}' (process: {})",
                       window.title,
                       window.process_name
                   );
                   actions.extend(rule.actions.clone());
               }
           }
           
           actions
       }
       
       /// Check if a rule matches a window
       fn rule_matches(&self, rule: &CompiledRule, window: &ManagedWindow) -> bool {
           let mut matches = true;
           
           // Check process name
           if let Some(ref regex) = rule.process_regex {
               if !regex.is_match(&window.process_name) {
                   matches = false;
               }
           }
           
           // Check window title
           if matches && rule.title_regex.is_some() {
               if let Some(ref regex) = rule.title_regex {
                   if !regex.is_match(&window.title) {
                       matches = false;
                   }
               }
           }
           
           // Check window class
           if matches && rule.class_regex.is_some() {
               if let Some(ref regex) = rule.class_regex {
                   if !regex.is_match(&window.class) {
                       matches = false;
                   }
               }
           }
           
           matches
       }
       
       /// Check if a window should be managed based on rules
       pub fn should_manage(&self, window: &ManagedWindow) -> bool {
           let actions = self.match_window(window);
           !actions.iter().any(|a| matches!(a, RuleAction::NoManage))
       }
       
       /// Get initial workspace for a window based on rules
       pub fn get_initial_workspace(&self, window: &ManagedWindow) -> Option<usize> {
           let actions = self.match_window(window);
           
           for action in actions {
               if let RuleAction::Workspace(id) = action {
                   return Some(id);
               }
           }
           
           None
       }
       
       /// Check if a window should start as floating based on rules
       pub fn should_float(&self, window: &ManagedWindow) -> bool {
           let actions = self.match_window(window);
           actions.iter().any(|a| matches!(a, RuleAction::Float))
       }
       
       /// Check if a window should start in fullscreen based on rules
       pub fn should_fullscreen(&self, window: &ManagedWindow) -> bool {
           let actions = self.match_window(window);
           actions.iter().any(|a| matches!(a, RuleAction::Fullscreen))
       }
       
       /// Get the number of rules
       pub fn rule_count(&self) -> usize {
           self.rules.len()
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Can compile window rules from configuration
- [ ] Regex matching works correctly
- [ ] Multiple conditions can be combined (AND logic)
- [ ] Multiple rules can match same window
- [ ] Actions are returned in order
- [ ] Helper methods work correctly
- [ ] Performance is acceptable for 100+ rules
- [ ] Error messages are clear

**Testing Requirements:**

Create `crates/core/src/rules/matcher_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_manager::window::ManagedWindow;
    use windows::Win32::Foundation::HWND;
    
    fn create_test_window(process: &str, title: &str, class: &str) -> ManagedWindow {
        ManagedWindow {
            handle: crate::utils::win32::WindowHandle::from_hwnd(HWND(12345)),
            state: crate::window_manager::window::WindowState::Tiled,
            workspace: 1,
            monitor: 0,
            title: title.to_string(),
            class: class.to_string(),
            process_name: process.to_string(),
            original_rect: None,
            managed: true,
            user_floating: false,
        }
    }
    
    #[test]
    fn test_rule_matcher_creation() {
        let rules = vec![
            WindowRule {
                match_process: Some("firefox\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(2)],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        assert_eq!(matcher.rule_count(), 1);
    }
    
    #[test]
    fn test_process_matching() {
        let rules = vec![
            WindowRule {
                match_process: Some("firefox\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("firefox.exe", "Mozilla Firefox", "MozillaWindowClass");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], RuleAction::Float));
    }
    
    #[test]
    fn test_title_regex_matching() {
        let rules = vec![
            WindowRule {
                match_process: None,
                match_title: Some(".*Steam.*".to_string()),
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("steam.exe", "Steam - News", "vguiPopupWindow");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 1);
    }
    
    #[test]
    fn test_multiple_conditions() {
        let rules = vec![
            WindowRule {
                match_process: Some("notepad\\.exe".to_string()),
                match_title: Some(".*Untitled.*".to_string()),
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        // Both conditions match
        let window1 = create_test_window("notepad.exe", "Untitled - Notepad", "Notepad");
        assert_eq!(matcher.match_window(&window1).len(), 1);
        
        // Only process matches
        let window2 = create_test_window("notepad.exe", "document.txt - Notepad", "Notepad");
        assert_eq!(matcher.match_window(&window2).len(), 0);
    }
    
    #[test]
    fn test_multiple_rules_same_window() {
        let rules = vec![
            WindowRule {
                match_process: Some("chrome\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(2)],
            },
            WindowRule {
                match_process: Some("chrome\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("chrome.exe", "Google Chrome", "Chrome_WidgetWin_1");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 2);
    }
    
    #[test]
    fn test_should_manage() {
        let rules = vec![
            WindowRule {
                match_process: Some("popup\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::NoManage],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window1 = create_test_window("popup.exe", "Popup", "PopupClass");
        assert!(!matcher.should_manage(&window1));
        
        let window2 = create_test_window("normal.exe", "Normal", "NormalClass");
        assert!(matcher.should_manage(&window2));
    }
    
    #[test]
    fn test_get_initial_workspace() {
        let rules = vec![
            WindowRule {
                match_process: Some("code\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(3)],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("code.exe", "VS Code", "Code");
        assert_eq!(matcher.get_initial_workspace(&window), Some(3));
    }
    
    #[test]
    fn test_should_float() {
        let rules = vec![
            WindowRule {
                match_process: Some("calc\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("calc.exe", "Calculator", "CalcFrame");
        assert!(matcher.should_float(&window));
    }
    
    #[test]
    fn test_invalid_regex_error() {
        let rules = vec![
            WindowRule {
                match_process: Some("[invalid".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let result = RuleMatcher::new(rules);
        assert!(result.is_err());
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core rules::matcher
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 4.6: Integrate Rules with Window Manager

**Objective:** Integrate the rule matcher with the window manager to automatically apply rules.

**File:** `crates/core/src/window_manager/mod.rs` (update)

**Required Implementations:**

Update WindowManager to include rule matcher:

```rust
use crate::rules::matcher::RuleMatcher;
use crate::config::schema::Config;

pub struct WindowManager {
    // ... existing fields ...
    rule_matcher: Option<RuleMatcher>,
}

impl WindowManager {
    /// Update configuration and rebuild rule matcher
    pub fn update_config(&mut self, config: &Config) -> anyhow::Result<()> {
        // Update layout settings
        self.dwindle_layout.ratio = config.layouts.dwindle.split_ratio;
        self.dwindle_layout.smart_split = config.layouts.dwindle.smart_split;
        self.dwindle_layout.no_gaps_when_only = config.layouts.dwindle.no_gaps_when_only;
        self.dwindle_layout.gaps_in = config.general.gaps_in;
        self.dwindle_layout.gaps_out = config.general.gaps_out;
        
        self.master_layout.master_factor = config.layouts.master.master_factor;
        self.master_layout.master_count = config.layouts.master.master_count;
        self.master_layout.gaps_in = config.general.gaps_in;
        self.master_layout.gaps_out = config.general.gaps_out;
        
        // Rebuild rule matcher
        self.rule_matcher = Some(RuleMatcher::new(config.window_rules.clone())?);
        
        tracing::info!("Configuration updated");
        Ok(())
    }
    
    /// Manage a window with rule application
    pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.0 .0;
        
        // Check if already managed
        if self.registry.get(hwnd).is_some() {
            return Ok(());
        }
        
        // Create managed window
        let mut managed = ManagedWindow::new(
            window,
            self.active_workspace,
            0, // TODO: Determine correct monitor
        )?;
        
        // Apply rules
        if let Some(ref matcher) = self.rule_matcher {
            // Check if window should be managed
            if !matcher.should_manage(&managed) {
                tracing::info!(
                    "Window '{}' excluded by NoManage rule",
                    managed.title
                );
                return Ok(());
            }
            
            // Get initial workspace from rules
            if let Some(workspace_id) = matcher.get_initial_workspace(&managed) {
                tracing::info!(
                    "Assigning window '{}' to workspace {} per rule",
                    managed.title,
                    workspace_id
                );
                managed.workspace = workspace_id;
            }
            
            // Check if should be floating
            if matcher.should_float(&managed) {
                tracing::info!("Setting window '{}' to floating per rule", managed.title);
                managed.set_floating()?;
            }
            
            // Check if should be fullscreen
            if matcher.should_fullscreen(&managed) {
                tracing::info!("Setting window '{}' to fullscreen per rule", managed.title);
                let monitor = self.monitors.get(managed.monitor)
                    .ok_or_else(|| anyhow::anyhow!("Monitor not found"))?;
                managed.set_fullscreen(&monitor.work_area)?;
            }
        }
        
        // Register the window
        self.registry.register(managed);
        
        // Retile workspace
        self.retile_workspace(managed.workspace)?;
        
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Rules are applied when windows are created
- [ ] NoManage rule prevents window management
- [ ] Workspace assignment rules work
- [ ] Float rules are applied
- [ ] Fullscreen rules are applied
- [ ] Configuration updates rebuild rule matcher
- [ ] Logging shows rule applications

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires real windows
    fn test_rule_application() {
        // Integration test for rule application
        // Would test with real windows
    }
}
```

---

### Week 16: Configuration Hot-Reload and Keybindings

#### Task 4.7: Implement Configuration Watcher

**Objective:** Create a file watcher that detects configuration changes and triggers reload.

**File:** `crates/core/src/config/watcher.rs`

**Required Implementations:**

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify::event::{ModifyKind, DataChange};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// Configuration file watcher
pub struct ConfigWatcher {
    /// File system watcher
    _watcher: Box<dyn Watcher>,
    
    /// Event receiver
    receiver: Receiver<notify::Result<Event>>,
    
    /// Last reload time (for debouncing)
    last_reload: Option<Instant>,
    
    /// Debounce duration
    debounce_duration: Duration,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = channel();
        
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        tracing::info!("Watching configuration file: {:?}", config_path);
        
        Ok(Self {
            _watcher: Box::new(watcher),
            receiver: rx,
            last_reload: None,
            debounce_duration: Duration::from_millis(500),
        })
    }
    
    /// Set debounce duration
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    /// Check if configuration file has changed
    /// Returns true if a change was detected and debounce period has passed
    pub fn check_for_changes(&mut self) -> bool {
        // Check if debounce period has passed
        if let Some(last) = self.last_reload {
            if last.elapsed() < self.debounce_duration {
                // Still in debounce period, drain events but don't report change
                self.receiver.try_iter().count();
                return false;
            }
        }
        
        // Check for relevant events
        let has_change = self.receiver
            .try_iter()
            .any(|event| {
                if let Ok(event) = event {
                    matches!(
                        event.kind,
                        EventKind::Modify(ModifyKind::Data(DataChange::Any))
                        | EventKind::Modify(ModifyKind::Data(DataChange::Content))
                    )
                } else {
                    false
                }
            });
        
        if has_change {
            self.last_reload = Some(Instant::now());
            tracing::info!("Configuration file change detected");
        }
        
        has_change
    }
}
```

**Acceptance Criteria:**
- [ ] Detects configuration file changes
- [ ] Debouncing prevents rapid reloads
- [ ] Works with text editor save patterns
- [ ] Handles file deletion/recreation
- [ ] Performance impact is minimal
- [ ] No file handle leaks

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_watcher_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();
        
        let watcher = ConfigWatcher::new(config_path);
        assert!(watcher.is_ok());
    }
    
    #[test]
    fn test_detect_file_changes() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();
        
        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();
        
        // Wait a bit for watcher to start
        std::thread::sleep(Duration::from_millis(100));
        
        // Modify file
        fs::write(&config_path, "[general]\ngaps_in = 10\n").unwrap();
        
        // Wait for change to be detected
        std::thread::sleep(Duration::from_millis(600));
        
        assert!(watcher.check_for_changes());
    }
    
    #[test]
    fn test_debouncing() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();
        
        let mut watcher = ConfigWatcher::new(config_path.clone())
            .unwrap()
            .with_debounce(Duration::from_millis(200));
        
        std::thread::sleep(Duration::from_millis(100));
        
        // First change
        fs::write(&config_path, "[general]\ngaps_in = 10\n").unwrap();
        std::thread::sleep(Duration::from_millis(300));
        assert!(watcher.check_for_changes());
        
        // Second change immediately after
        fs::write(&config_path, "[general]\ngaps_in = 20\n").unwrap();
        std::thread::sleep(Duration::from_millis(50));
        
        // Should be debounced
        assert!(!watcher.check_for_changes());
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core config::watcher
```

---

#### Task 4.8: Implement Keybinding System

**Objective:** Create a keybinding system that registers hotkeys with Windows.

**File:** `crates/core/src/keybinds/manager.rs`

**Required Implementations:**

1. **Create keybinds module:**
   ```bash
   mkdir -p crates/core/src/keybinds
   touch crates/core/src/keybinds/mod.rs
   touch crates/core/src/keybinds/manager.rs
   touch crates/core/src/keybinds/parser.rs
   ```

2. **KeybindManager struct:**

   ```rust
   use crate::config::schema::Keybind;
   use windows::Win32::UI::Input::KeyboardAndMouse::*;
   use windows::Win32::Foundation::*;
   use std::collections::HashMap;
   
   /// Keybinding manager
   pub struct KeybindManager {
       /// Map of hotkey ID to command
       bindings: HashMap<i32, String>,
       
       /// Next hotkey ID to assign
       next_id: i32,
   }
   
   impl KeybindManager {
       pub fn new() -> Self {
           Self {
               bindings: HashMap::new(),
               next_id: 1,
           }
       }
       
       /// Register keybindings
       pub fn register_keybinds(&mut self, keybinds: Vec<Keybind>) -> anyhow::Result<()> {
           // Unregister existing keybinds
           self.unregister_all()?;
           
           for keybind in keybinds {
               self.register_keybind(keybind)?;
           }
           
           tracing::info!("Registered {} keybindings", self.bindings.len());
           Ok(())
       }
       
       /// Register a single keybinding
       fn register_keybind(&mut self, keybind: Keybind) -> anyhow::Result<()> {
           let modifiers = self.parse_modifiers(&keybind.modifiers)?;
           let vk_code = self.parse_key(&keybind.key)?;
           
           let hotkey_id = self.next_id;
           self.next_id += 1;
           
           unsafe {
               let result = RegisterHotKey(
                   None,
                   hotkey_id,
                   modifiers,
                   vk_code as u32,
               );
               
               if !result.as_bool() {
                   anyhow::bail!(
                       "Failed to register hotkey: {:?}+{}",
                       keybind.modifiers,
                       keybind.key
                   );
               }
           }
           
           self.bindings.insert(hotkey_id, keybind.command);
           
           tracing::debug!(
               "Registered hotkey {}: {:?}+{} -> {}",
               hotkey_id,
               keybind.modifiers,
               keybind.key,
               keybind.command
           );
           
           Ok(())
       }
       
       /// Unregister all keybindings
       pub fn unregister_all(&mut self) -> anyhow::Result<()> {
           for &hotkey_id in self.bindings.keys() {
               unsafe {
                   UnregisterHotKey(None, hotkey_id);
               }
           }
           
           self.bindings.clear();
           self.next_id = 1;
           
           Ok(())
       }
       
       /// Get command for a hotkey ID
       pub fn get_command(&self, hotkey_id: i32) -> Option<&String> {
           self.bindings.get(&hotkey_id)
       }
       
       /// Parse modifier keys
       fn parse_modifiers(&self, modifiers: &[String]) -> anyhow::Result<HOT_KEY_MODIFIERS> {
           let mut result = HOT_KEY_MODIFIERS(0);
           
           for modifier in modifiers {
               match modifier.as_str() {
                   "Ctrl" => result |= MOD_CONTROL,
                   "Alt" => result |= MOD_ALT,
                   "Shift" => result |= MOD_SHIFT,
                   "Win" => result |= MOD_WIN,
                   _ => anyhow::bail!("Unknown modifier: {}", modifier),
               }
           }
           
           Ok(result)
       }
       
       /// Parse key to virtual key code
       fn parse_key(&self, key: &str) -> anyhow::Result<VIRTUAL_KEY> {
           let vk = match key.to_uppercase().as_str() {
               // Letters
               "A" => VK_A,
               "B" => VK_B,
               "C" => VK_C,
               "D" => VK_D,
               "E" => VK_E,
               "F" => VK_F,
               "G" => VK_G,
               "H" => VK_H,
               "I" => VK_I,
               "J" => VK_J,
               "K" => VK_K,
               "L" => VK_L,
               "M" => VK_M,
               "N" => VK_N,
               "O" => VK_O,
               "P" => VK_P,
               "Q" => VK_Q,
               "R" => VK_R,
               "S" => VK_S,
               "T" => VK_T,
               "U" => VK_U,
               "V" => VK_V,
               "W" => VK_W,
               "X" => VK_X,
               "Y" => VK_Y,
               "Z" => VK_Z,
               
               // Numbers
               "0" => VK_0,
               "1" => VK_1,
               "2" => VK_2,
               "3" => VK_3,
               "4" => VK_4,
               "5" => VK_5,
               "6" => VK_6,
               "7" => VK_7,
               "8" => VK_8,
               "9" => VK_9,
               
               // Special keys
               "LEFT" => VK_LEFT,
               "RIGHT" => VK_RIGHT,
               "UP" => VK_UP,
               "DOWN" => VK_DOWN,
               "SPACE" => VK_SPACE,
               "ENTER" | "RETURN" => VK_RETURN,
               "ESCAPE" | "ESC" => VK_ESCAPE,
               "TAB" => VK_TAB,
               "BACKSPACE" => VK_BACK,
               "DELETE" | "DEL" => VK_DELETE,
               "HOME" => VK_HOME,
               "END" => VK_END,
               "PAGEUP" | "PGUP" => VK_PRIOR,
               "PAGEDOWN" | "PGDN" => VK_NEXT,
               
               // Brackets
               "BRACKETLEFT" | "[" => VK_OEM_4,
               "BRACKETRIGHT" | "]" => VK_OEM_6,
               
               // Function keys
               "F1" => VK_F1,
               "F2" => VK_F2,
               "F3" => VK_F3,
               "F4" => VK_F4,
               "F5" => VK_F5,
               "F6" => VK_F6,
               "F7" => VK_F7,
               "F8" => VK_F8,
               "F9" => VK_F9,
               "F10" => VK_F10,
               "F11" => VK_F11,
               "F12" => VK_F12,
               
               _ => anyhow::bail!("Unknown key: {}", key),
           };
           
           Ok(vk)
       }
   }
   
   impl Drop for KeybindManager {
       fn drop(&mut self) {
           let _ = self.unregister_all();
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Can register hotkeys with Windows
- [ ] Modifier combinations work correctly
- [ ] All common keys are supported
- [ ] Conflicts are detected
- [ ] Hotkeys are unregistered on drop
- [ ] Error messages are helpful
- [ ] Performance is acceptable

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keybind_manager_creation() {
        let manager = KeybindManager::new();
        assert_eq!(manager.bindings.len(), 0);
    }
    
    #[test]
    fn test_parse_modifiers() {
        let manager = KeybindManager::new();
        
        let mods = manager.parse_modifiers(&["Win".to_string(), "Shift".to_string()]).unwrap();
        assert!(mods.0 & MOD_WIN.0 != 0);
        assert!(mods.0 & MOD_SHIFT.0 != 0);
    }
    
    #[test]
    fn test_parse_key() {
        let manager = KeybindManager::new();
        
        assert_eq!(manager.parse_key("Q").unwrap(), VK_Q);
        assert_eq!(manager.parse_key("1").unwrap(), VK_1);
        assert_eq!(manager.parse_key("Left").unwrap(), VK_LEFT);
        assert_eq!(manager.parse_key("F1").unwrap(), VK_F1);
    }
    
    #[test]
    fn test_invalid_key() {
        let manager = KeybindManager::new();
        assert!(manager.parse_key("InvalidKey").is_err());
    }
    
    #[test]
    #[ignore] // Requires Windows API
    fn test_register_hotkey() {
        let mut manager = KeybindManager::new();
        
        let keybind = Keybind {
            modifiers: vec!["Win".to_string(), "Ctrl".to_string()],
            key: "T".to_string(),
            command: "test-command".to_string(),
            args: vec![],
        };
        
        let result = manager.register_keybinds(vec![keybind]);
        assert!(result.is_ok());
        
        assert_eq!(manager.bindings.len(), 1);
    }
}
```

---

#### Task 4.9: Integrate Configuration Hot-Reload

**Objective:** Integrate configuration watcher with the main application for live reload.

**File:** `crates/core/src/main.rs` (update)

**Required Implementations:**

```rust
mod config;
mod rules;
mod keybinds;

use config::{ConfigLoader, ConfigWatcher};
use keybinds::KeybindManager;

fn main() -> Result<()> {
    initialize_logging();
    
    info!("Starting Tiling Window Manager");
    
    // Load configuration
    let config_loader = ConfigLoader::new()?;
    let config = config_loader.load()?;
    
    // Initialize window manager with config
    let mut wm = WindowManager::new();
    wm.initialize()?;
    wm.update_config(&config)?;
    
    // Set up keybindings
    let mut keybind_manager = KeybindManager::new();
    keybind_manager.register_keybinds(config.keybinds.clone())?;
    
    // Set up config watcher
    let mut config_watcher = ConfigWatcher::new(config_loader.get_config_path().clone())?;
    
    // Set up event loop
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    
    // Main event loop
    loop {
        // Check for config changes
        if config_watcher.check_for_changes() {
            info!("Reloading configuration...");
            
            match reload_config(&config_loader, &mut wm, &mut keybind_manager) {
                Ok(()) => info!("Configuration reloaded successfully"),
                Err(e) => error!("Failed to reload configuration: {}", e),
            }
        }
        
        // Process window events
        for event in event_loop.poll_events() {
            handle_window_event(&mut wm, event)?;
        }
        
        // Process hotkey events
        // TODO: Implement hotkey event processing
        
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn reload_config(
    loader: &ConfigLoader,
    wm: &mut WindowManager,
    keybind_manager: &mut KeybindManager,
) -> Result<()> {
    // Load new configuration
    let config = loader.load()?;
    
    // Validate configuration
    config::validator::ConfigValidator::validate(&config)?;
    
    // Update window manager
    wm.update_config(&config)?;
    
    // Update keybindings
    keybind_manager.register_keybinds(config.keybinds)?;
    
    Ok(())
}
```

**Acceptance Criteria:**
- [ ] Config changes trigger reload
- [ ] Invalid configs don't crash application
- [ ] Rules are updated on reload
- [ ] Keybindings are updated on reload
- [ ] User is notified of reload success/failure
- [ ] Reload completes quickly (<100ms)

---

## Phase 4 Completion Checklist

### Build & Compilation
- [ ] `cargo build --workspace` succeeds without errors
- [ ] `cargo build --workspace --release` succeeds
- [ ] No warnings from `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted with `cargo fmt --workspace --check`

### Core Functionality
- [ ] Configuration schema is complete and documented
- [ ] Default configuration file is comprehensive
- [ ] Configuration parser loads TOML correctly
- [ ] Configuration validator catches all errors
- [ ] Window rules engine matches patterns correctly
- [ ] Rule actions are applied properly
- [ ] Hot-reload detects file changes
- [ ] Hot-reload applies changes without restart
- [ ] Keybinding system registers hotkeys with Windows
- [ ] All keybinds execute correct commands

### Testing
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Configuration parsing tests pass
- [ ] Rule matching tests pass
- [ ] Validator tests pass
- [ ] Watcher tests pass
- [ ] Keybinding tests pass
- [ ] No test failures or panics

### Integration
- [ ] Window manager uses configuration
- [ ] Rules are applied when windows are created
- [ ] Hot-reload updates window manager
- [ ] Keybindings integrate with command system
- [ ] Configuration validation prevents invalid states

### Documentation
- [ ] All new public APIs have doc comments
- [ ] `cargo doc --no-deps` builds successfully
- [ ] README updated with Phase 4 features
- [ ] Configuration options are documented
- [ ] Examples show common use cases

### Manual Validation
- [ ] Can edit config file and see changes apply
- [ ] Window rules work for test applications
- [ ] Keybindings execute correct actions
- [ ] Invalid configs show helpful errors
- [ ] Config reload is smooth and fast
- [ ] Application runs stable for 15+ minutes
- [ ] CPU usage remains reasonable
- [ ] Memory usage is stable

---

## Deliverables for Phase 4

At the end of Phase 4, you should have:

1. **Complete Configuration System:**
   - Comprehensive TOML schema
   - Parser and loader
   - Validator with helpful errors
   - Default configuration file
   - Configuration path management

2. **Window Rules Engine:**
   - Regex-based pattern matching
   - Process, title, and class matching
   - Multiple actions per rule
   - Rule priority and ordering
   - Integration with window manager

3. **Hot-Reload Capability:**
   - File watcher with debouncing
   - Live configuration updates
   - Error handling for invalid configs
   - User feedback on reload status

4. **Keybinding System:**
   - Windows hotkey registration
   - Modifier key support
   - Comprehensive key mapping
   - Conflict detection
   - Integration with commands

5. **Integration:**
   - Window manager uses configuration
   - Rules apply automatically
   - Keybindings work system-wide
   - Hot-reload updates all systems

6. **Quality Assurance:**
   - Comprehensive unit tests
   - Integration tests
   - Validation tests
   - Documentation complete
   - Manual testing procedures

---

## Success Criteria Summary

Phase 4 is complete when:

1. ✅ **Configuration system is robust:**
   - Parses TOML correctly
   - Validates all values
   - Provides helpful errors
   - Default config is excellent

2. ✅ **Rules engine is powerful:**
   - Matches patterns accurately
   - Applies actions correctly
   - Handles complex scenarios
   - Performance is good

3. ✅ **Hot-reload is reliable:**
   - Detects changes quickly
   - Applies updates smoothly
   - Handles errors gracefully
   - No crashes from bad configs

4. ✅ **Keybindings work well:**
   - All keys are supported
   - Modifiers work correctly
   - No conflicts
   - Commands execute properly

5. ✅ **Integration is seamless:**
   - Window manager uses config
   - Rules apply automatically
   - Everything updates on reload
   - User experience is smooth

6. ✅ **Quality standards met:**
   - All tests passing
   - No clippy warnings
   - Stable operation
   - Good documentation

---

## Next Steps

After completing Phase 4, proceed to **Phase 5: IPC & CLI** (Weeks 17-20), which will implement:

- Named pipe IPC server
- JSON protocol for communication
- CLI client for remote control
- Event subscription system
- Scripting support
- Status queries

See DETAILED_ROADMAP.md for Phase 5 specifications.

---

## Troubleshooting

### Common Issues

**Issue: Configuration file not found**
- Solution: Check config directory path
- Verify permissions on directory
- Ensure default config is created
- Check for typos in path

**Issue: TOML parsing errors**
- Solution: Validate TOML syntax
- Check for missing quotes
- Verify array/table structure
- Use TOML linter

**Issue: Rules not matching**
- Solution: Test regex patterns
- Check process name matches executable
- Verify window class with Spy++
- Enable debug logging

**Issue: Hot-reload not working**
- Solution: Check file watcher permissions
- Verify file system notifications
- Test with manual reload
- Check debounce timing

**Issue: Keybindings not registering**
- Solution: Check for conflicts
- Verify modifier keys
- Test with simple bindings first
- Check Windows security restrictions

**Issue: Invalid config crashes app**
- Solution: Improve validation
- Add try-catch around reload
- Provide better error messages
- Test with malformed configs

---

## Notes for Autonomous Agents

When executing this task list:

1. **Follow order strictly**: Configuration must be done before rules
2. **Validate extensively**: Config errors should never crash
3. **Test thoroughly**: Rules are critical functionality
4. **Document well**: Config options need good docs
5. **Handle errors gracefully**: Invalid configs are common
6. **Performance matters**: Hot-reload should be fast
7. **Think about users**: Error messages should help
8. **Test edge cases**: Weird regex patterns, etc.
9. **Check Windows APIs**: Hotkey registration can fail
10. **Reference phases 1-3**: Build on existing code

---

**End of Phase 4 Task Document**
