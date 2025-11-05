# Phase 6: Status Bar Implementation - Detailed Task List

**Timeline:** Weeks 21-26 (6 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 6 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 6 Goals:**
- Create standalone status bar application separate from window manager
- Implement modular widget system for extensibility
- Build core system information widgets (CPU, memory, network, battery)
- Create workspace indicator and window title display modules
- Implement IPC integration with window manager for real-time updates
- Provide CSS-like styling system for customization
- Support multiple monitors with per-monitor status bars
- Enable custom module development
- Create comprehensive configuration system
- Ensure <50MB memory usage and <1% CPU overhead

**Prerequisites:**
- Phase 1 completed successfully (project foundation, Win32 wrappers, tree structure)
- Phase 2 completed successfully (window management, layouts, focus management, commands)
- Phase 3 completed successfully (workspace system, Virtual Desktop integration, persistence)
- Phase 4 completed successfully (configuration system, window rules, keybindings, hot-reload)
- Phase 5 completed successfully (IPC server, protocol, CLI client, event system)
- All Phase 1-5 tests passing
- IPC server functional and tested
- Event broadcasting system operational

---

## Success Criteria for Phase 6 Completion

Phase 6 is considered complete when:

1. **Status bar framework fully operational:**
   - Separate application builds and runs
   - Window positioned correctly (top/bottom)
   - Always-on-top and no decorations
   - Respects DWM composition
   - Handles monitor changes gracefully

2. **Module system complete:**
   - Module trait well-defined
   - Module loading/unloading works
   - Position system (left/center/right) functions
   - Module update lifecycle correct
   - Custom modules can be created

3. **Core modules implemented:**
   - Workspaces module shows all workspaces
   - Window title module displays active window
   - Clock module with customizable format
   - CPU module with accurate usage
   - Memory module with RAM statistics
   - Battery module (if available)
   - Network module with throughput
   - Volume module (optional)

4. **IPC integration working:**
   - Connects to window manager on startup
   - Receives workspace events
   - Receives window events
   - Can send commands (workspace switch)
   - Reconnects on disconnection
   - Event handling is real-time (<100ms)

5. **Configuration system functional:**
   - TOML configuration loads correctly
   - Module positioning configurable
   - Styling options work
   - Colors and fonts apply
   - Module-specific config works
   - Hot-reload supported

6. **All tests passing:**
   - Unit tests for modules
   - Integration tests with IPC
   - UI rendering tests
   - Configuration tests
   - Manual validation successful

---

## Task Breakdown

### Week 21: Status Bar Framework and Architecture

#### Task 6.1: Create Status Bar Project Structure

**Objective:** Set up the status bar as a separate Rust application with proper dependency management.

**File:** `crates/status-bar/Cargo.toml`

**Required Implementations:**

1. **Create status bar crate structure:**
   ```bash
   mkdir -p crates/status-bar/src
   mkdir -p crates/status-bar/src/modules
   mkdir -p crates/status-bar/src/rendering
   mkdir -p crates/status-bar/src/styling
   touch crates/status-bar/Cargo.toml
   touch crates/status-bar/src/main.rs
   touch crates/status-bar/src/lib.rs
   touch crates/status-bar/src/module.rs
   touch crates/status-bar/src/config.rs
   touch crates/status-bar/src/ipc_client.rs
   ```

2. **Cargo.toml configuration:**

   ```toml
   [package]
   name = "tiling-wm-status-bar"
   version = "0.1.0"
   edition = "2021"
   
   [[bin]]
   name = "twm-bar"
   path = "src/main.rs"
   
   [dependencies]
   # UI Framework
   iced = { version = "0.12", features = ["tokio", "canvas"] }
   iced_native = "0.12"
   
   # Async runtime
   tokio = { workspace = true }
   
   # Serialization
   serde = { workspace = true }
   serde_json = { workspace = true }
   toml = { workspace = true }
   
   # System information
   sysinfo = "0.30"
   battery = "0.7"
   netstat2 = "0.9"
   
   # Date/Time
   chrono = "0.4"
   
   # Windows API
   windows = { workspace = true }
   
   # Error handling
   anyhow = { workspace = true }
   thiserror = { workspace = true }
   
   # Logging
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true }
   
   # Utilities
   dirs = "5.0"
   
   [dev-dependencies]
   mockall = "0.12"
   ```

3. **Update workspace Cargo.toml:**

   ```toml
   [workspace]
   members = [
       "crates/core",
       "crates/cli",
       "crates/status-bar",  # Add this
   ]
   ```

**Acceptance Criteria:**
- [ ] Status bar crate compiles without errors
- [ ] Dependencies resolve correctly
- [ ] Binary can be built with `cargo build -p tiling-wm-status-bar`
- [ ] No dependency conflicts
- [ ] Directory structure is organized
- [ ] Workspace recognizes new member

**Testing Requirements:**

```bash
# Test compilation
cargo check -p tiling-wm-status-bar
cargo build -p tiling-wm-status-bar
cargo build -p tiling-wm-status-bar --release

# Verify no warnings
cargo clippy -p tiling-wm-status-bar -- -D warnings
```

**Validation Commands:**
```bash
cargo build --workspace
cargo test --workspace
cargo tree -p tiling-wm-status-bar
```

---

#### Task 6.2: Define Module Trait and Base Types

**Objective:** Create the core module system that allows extensible widgets.

**File:** `crates/status-bar/src/module.rs`

**Required Implementations:**

```rust
use iced::widget::Container;
use iced::{Color, Element, Length};
use serde::{Deserialize, Serialize};

/// Trait that all status bar modules must implement
pub trait Module: Send + Sync {
    /// Get the module's current view
    fn view(&self) -> Element<'_, Message>;
    
    /// Update the module with a message
    fn update(&mut self, message: Message) -> Option<Command>;
    
    /// Get the position where this module should be displayed
    fn position(&self) -> Position;
    
    /// Get the module's unique identifier
    fn name(&self) -> &str;
    
    /// Get the module's configuration
    fn config(&self) -> &ModuleConfig;
    
    /// Initialize the module (called once at startup)
    fn init(&mut self) -> Option<Command> {
        None
    }
    
    /// Cleanup when module is removed
    fn cleanup(&mut self) {}
    
    /// Get update interval in seconds (0 = no periodic updates)
    fn update_interval(&self) -> u64 {
        0
    }
}

/// Position of a module on the status bar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Center,
    Right,
}

/// Messages that modules can send and receive
#[derive(Debug, Clone)]
pub enum Message {
    /// Tick for periodic updates
    Tick,
    
    /// Module-specific message
    ModuleMessage {
        module_name: String,
        message: Box<ModuleMessage>,
    },
    
    /// IPC event received
    IpcEvent(IpcEvent),
    
    /// Request to switch workspace
    SwitchWorkspace(usize),
    
    /// Request to execute command
    ExecuteCommand(String),
}

/// Module-specific messages
#[derive(Debug, Clone)]
pub enum ModuleMessage {
    /// Workspace was clicked
    WorkspaceClicked(usize),
    
    /// Refresh the module
    Refresh,
    
    /// Custom string message
    Custom(String),
    
    /// Volume changed
    VolumeChanged(f32),
    
    /// Network interface clicked
    NetworkClicked(String),
}

/// IPC events from window manager
#[derive(Debug, Clone)]
pub enum IpcEvent {
    WorkspaceChanged { from: usize, to: usize },
    WindowFocused { hwnd: String, title: String },
    WindowCreated { hwnd: String, title: String },
    WindowClosed { hwnd: String },
    ConfigReloaded,
}

/// Base configuration for all modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Module name
    pub name: String,
    
    /// Position on bar
    pub position: Position,
    
    /// Whether module is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Custom styling
    #[serde(default)]
    pub style: ModuleStyle,
    
    /// Module-specific configuration (JSON value)
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Styling options for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStyle {
    /// Foreground color
    #[serde(default = "default_foreground")]
    pub color: String,
    
    /// Background color
    #[serde(default)]
    pub background: Option<String>,
    
    /// Font family
    #[serde(default)]
    pub font: Option<String>,
    
    /// Font size
    #[serde(default)]
    pub font_size: Option<f32>,
    
    /// Padding (pixels)
    #[serde(default = "default_padding")]
    pub padding: u16,
    
    /// Margin (pixels)
    #[serde(default)]
    pub margin: u16,
}

impl Default for ModuleStyle {
    fn default() -> Self {
        Self {
            color: default_foreground(),
            background: None,
            font: None,
            font_size: None,
            padding: default_padding(),
            margin: 0,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_foreground() -> String {
    "#cdd6f4".to_string()
}

fn default_padding() -> u16 {
    10
}

/// Helper to convert hex color string to iced Color
pub fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
    
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
    } else {
        1.0
    };
    
    Color::from_rgba(r, g, b, a)
}

/// Helper to create a styled container for a module
pub fn styled_container<'a, Message: 'a>(
    content: Element<'a, Message>,
    style: &ModuleStyle,
) -> Container<'a, Message> {
    let mut container = Container::new(content)
        .padding(style.padding);
    
    if let Some(ref bg) = style.background {
        container = container.style(|_theme| {
            iced::widget::container::Appearance {
                background: Some(iced::Background::Color(parse_color(bg))),
                ..Default::default()
            }
        });
    }
    
    container
}

use iced::Command;

/// Module registry for managing loaded modules
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
    
    pub fn register(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }
    
    pub fn get_by_name(&self, name: &str) -> Option<&dyn Module> {
        self.modules.iter().find(|m| m.name() == name).map(|m| m.as_ref())
    }
    
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Box<dyn Module>> {
        self.modules.iter_mut().find(|m| m.name() == name)
    }
    
    pub fn get_by_position(&self, position: Position) -> Vec<&dyn Module> {
        self.modules
            .iter()
            .filter(|m| m.position() == position && m.config().enabled)
            .map(|m| m.as_ref())
            .collect()
    }
    
    pub fn get_by_position_mut(&mut self, position: Position) -> Vec<&mut Box<dyn Module>> {
        self.modules
            .iter_mut()
            .filter(|m| m.position() == position && m.config().enabled)
            .collect()
    }
    
    pub fn update_all(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();
        
        for module in &mut self.modules {
            if let Some(cmd) = module.update(message.clone()) {
                commands.push(cmd);
            }
        }
        
        commands
    }
    
    pub fn count(&self) -> usize {
        self.modules.len()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Module trait compiles without errors
- [ ] All types are properly defined
- [ ] Serialization works for configuration types
- [ ] Color parsing handles all formats
- [ ] Module registry manages modules correctly
- [ ] Documentation is complete

**Testing Requirements:**

Create `crates/status-bar/src/module_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_color_rgb() {
        let color = parse_color("#ff0000");
        assert_eq!(color, Color::from_rgb(1.0, 0.0, 0.0));
    }
    
    #[test]
    fn test_parse_color_rgba() {
        let color = parse_color("#ff000080");
        assert_eq!(color, Color::from_rgba(1.0, 0.0, 0.0, 0.5));
    }
    
    #[test]
    fn test_position_serialization() {
        let pos = Position::Left;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""left""#);
        
        let deserialized: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Position::Left);
    }
    
    #[test]
    fn test_module_config_defaults() {
        let config = ModuleConfig {
            name: "test".to_string(),
            position: Position::Left,
            enabled: true,
            style: ModuleStyle::default(),
            config: serde_json::Value::Null,
        };
        
        assert!(config.enabled);
        assert_eq!(config.style.padding, 10);
    }
    
    #[test]
    fn test_module_registry() {
        let mut registry = ModuleRegistry::new();
        assert_eq!(registry.count(), 0);
        
        // Would add test modules here
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-status-bar module
cargo clippy -p tiling-wm-status-bar -- -D warnings
```

---

#### Task 6.3: Implement Status Bar Configuration System

**Objective:** Create a comprehensive configuration system for the status bar.

**File:** `crates/status-bar/src/config.rs`

**Required Implementations:**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// Main status bar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarConfig {
    #[serde(default)]
    pub bar: BarSettings,
    
    #[serde(default)]
    pub style: StyleSettings,
    
    #[serde(default)]
    pub modules: ModulesConfig,
}

/// Status bar positioning and appearance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarSettings {
    /// Bar height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
    
    /// Bar position (top or bottom)
    #[serde(default = "default_position")]
    pub position: BarPosition,
    
    /// Monitor to display on (0-based index, None = all monitors)
    #[serde(default)]
    pub monitor: Option<usize>,
    
    /// Layer (always on top)
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    
    /// Reserve screen space (no windows overlap)
    #[serde(default = "default_true")]
    pub reserve_space: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    Top,
    Bottom,
}

/// Global styling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSettings {
    /// Background color (hex)
    #[serde(default = "default_background")]
    pub background_color: String,
    
    /// Foreground/text color (hex)
    #[serde(default = "default_foreground")]
    pub foreground_color: String,
    
    /// Font family
    #[serde(default = "default_font")]
    pub font_family: String,
    
    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    
    /// Border color (hex)
    #[serde(default)]
    pub border_color: Option<String>,
    
    /// Border width (pixels)
    #[serde(default)]
    pub border_width: u32,
}

/// Module positioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    /// Modules on the left side
    #[serde(default = "default_left_modules")]
    pub left: Vec<String>,
    
    /// Modules in the center
    #[serde(default)]
    pub center: Vec<String>,
    
    /// Modules on the right side
    #[serde(default = "default_right_modules")]
    pub right: Vec<String>,
    
    /// Module-specific configurations
    #[serde(default)]
    pub module_configs: std::collections::HashMap<String, serde_json::Value>,
}

// Default values
fn default_height() -> u32 {
    30
}

fn default_position() -> BarPosition {
    BarPosition::Top
}

fn default_true() -> bool {
    true
}

fn default_background() -> String {
    "#1e1e2e".to_string()
}

fn default_foreground() -> String {
    "#cdd6f4".to_string()
}

fn default_font() -> String {
    "Segoe UI".to_string()
}

fn default_font_size() -> f32 {
    12.0
}

fn default_left_modules() -> Vec<String> {
    vec!["workspaces".to_string()]
}

fn default_right_modules() -> Vec<String> {
    vec![
        "cpu".to_string(),
        "memory".to_string(),
        "battery".to_string(),
        "clock".to_string(),
    ]
}

impl Default for BarSettings {
    fn default() -> Self {
        Self {
            height: default_height(),
            position: default_position(),
            monitor: None,
            always_on_top: default_true(),
            reserve_space: default_true(),
        }
    }
}

impl Default for StyleSettings {
    fn default() -> Self {
        Self {
            background_color: default_background(),
            foreground_color: default_foreground(),
            font_family: default_font(),
            font_size: default_font_size(),
            border_color: None,
            border_width: 0,
        }
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            left: default_left_modules(),
            center: Vec::new(),
            right: default_right_modules(),
            module_configs: std::collections::HashMap::new(),
        }
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            bar: BarSettings::default(),
            style: StyleSettings::default(),
            modules: ModulesConfig::default(),
        }
    }
}

/// Configuration loader
pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    /// Create a new config loader with default path
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("tiling-wm");
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        let config_path = config_dir.join("status-bar.toml");
        
        Ok(Self { config_path })
    }
    
    /// Create config loader with custom path
    pub fn from_path(path: PathBuf) -> Self {
        Self { config_path: path }
    }
    
    /// Load configuration from file
    pub fn load(&self) -> Result<BarConfig> {
        if !self.config_path.exists() {
            tracing::info!(
                "Configuration file not found at {:?}, creating default",
                self.config_path
            );
            self.create_default()?;
        }
        
        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config file: {:?}", self.config_path))?;
        
        let config: BarConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", self.config_path))?;
        
        tracing::info!("Loaded status bar configuration");
        Ok(config)
    }
    
    /// Create default configuration file
    pub fn create_default(&self) -> Result<()> {
        let default_config = Self::default_config_toml();
        
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        fs::write(&self.config_path, default_config)
            .with_context(|| format!("Failed to write default config to {:?}", self.config_path))?;
        
        tracing::info!("Created default configuration at {:?}", self.config_path);
        Ok(())
    }
    
    /// Get default configuration as TOML string
    fn default_config_toml() -> String {
        r#"# Status Bar Configuration

[bar]
# Height of the status bar in pixels
height = 30

# Position: "top" or "bottom"
position = "top"

# Monitor to display on (null = all monitors)
# monitor = 0

# Always keep bar on top of other windows
always_on_top = true

# Reserve screen space (prevent windows from overlapping)
reserve_space = true

[style]
# Background color (hex)
background_color = "#1e1e2e"

# Foreground/text color (hex)
foreground_color = "#cdd6f4"

# Font family
font_family = "Segoe UI"

# Font size
font_size = 12.0

# Optional border
# border_color = "#89b4fa"
# border_width = 1

[modules]
# Modules to display on the left
left = ["workspaces"]

# Modules to display in the center
center = []

# Modules to display on the right
right = ["cpu", "memory", "battery", "clock"]

# Module-specific configurations

[modules.module_configs.workspaces]
format = "{icon}"
icons = { "1" = "1", "2" = "2", "3" = "3", "4" = "4", "5" = "5" }
active_color = "#89b4fa"
inactive_color = "#585b70"

[modules.module_configs.window-title]
max_length = 50
format = "{title}"

[modules.module_configs.clock]
format = "%H:%M:%S"
format_alt = "%Y-%m-%d"

[modules.module_configs.battery]
format = "{icon} {percentage}%"
warning_level = 30
critical_level = 15

[modules.module_configs.cpu]
format = " {usage}%"
interval = 5

[modules.module_configs.memory]
format = " {percentage}%"
interval = 5

[modules.module_configs.network]
format = " {down}  {up}"
interface = "auto"
"#.to_string()
    }
    
    /// Get config path
    pub fn get_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigLoader")
    }
}
```

**Acceptance Criteria:**
- [ ] Configuration loads from TOML correctly
- [ ] Default configuration is created if missing
- [ ] All settings have sensible defaults
- [ ] Module-specific configs are supported
- [ ] Serialization/deserialization works
- [ ] Error messages are helpful

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = BarConfig::default();
        assert_eq!(config.bar.height, 30);
        assert_eq!(config.bar.position, BarPosition::Top);
        assert!(config.bar.always_on_top);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = BarConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: BarConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(deserialized.bar.height, config.bar.height);
        assert_eq!(deserialized.bar.position, config.bar.position);
    }
    
    #[test]
    fn test_create_default_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("status-bar.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        loader.create_default().unwrap();
        
        assert!(config_path.exists());
        
        let config = loader.load().unwrap();
        assert_eq!(config.bar.height, 30);
    }
    
    #[test]
    fn test_bar_position_serialization() {
        let pos = BarPosition::Top;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""top""#);
        
        let deserialized: BarPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, BarPosition::Top);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-status-bar config
```

---

### Week 22: Main Application and Window Management

#### Task 6.4: Implement Main Status Bar Application

**Objective:** Create the main application using iced framework.

**File:** `crates/status-bar/src/main.rs`

**Required Implementations:**

```rust
use iced::{Application, Command, Element, Settings, Theme, window};
use iced::widget::{container, row, Column, Row};
use iced::executor;
use std::time::Duration;
use tracing::{info, error};

mod module;
mod config;
mod ipc_client;
mod modules;

use module::{Module, Message, ModuleRegistry, Position};
use config::{BarConfig, ConfigLoader};

struct StatusBar {
    /// Module registry
    modules: ModuleRegistry,
    
    /// Configuration
    config: BarConfig,
    
    /// IPC client
    ipc_client: Option<ipc_client::IpcClient>,
    
    /// Window ID
    window_id: Option<window::Id>,
}

impl Application for StatusBar {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Load configuration
        let config_loader = ConfigLoader::new()
            .expect("Failed to create config loader");
        
        let config = config_loader.load()
            .expect("Failed to load configuration");
        
        info!("Status bar starting...");
        info!("Height: {}, Position: {:?}", config.bar.height, config.bar.position);
        
        // Create module registry and load modules
        let mut modules = ModuleRegistry::new();
        
        // Register modules based on configuration
        modules.register(Box::new(modules::workspaces::WorkspacesModule::new()));
        modules.register(Box::new(modules::window_title::WindowTitleModule::new()));
        modules.register(Box::new(modules::clock::ClockModule::new()));
        modules.register(Box::new(modules::cpu::CpuModule::new()));
        modules.register(Box::new(modules::memory::MemoryModule::new()));
        
        // Only add battery module if battery is available
        if modules::battery::BatteryModule::is_available() {
            modules.register(Box::new(modules::battery::BatteryModule::new()));
        }
        
        info!("Loaded {} modules", modules.count());
        
        // Initialize IPC client
        let ipc_client = ipc_client::IpcClient::new();
        
        (
            Self {
                modules,
                config,
                ipc_client: Some(ipc_client),
                window_id: None,
            },
            Command::batch(vec![
                // Subscribe to time ticks
                Command::perform(async {}, |_| Message::Tick),
            ]),
        )
    }
    
    fn title(&self) -> String {
        "Tiling WM Status Bar".to_string()
    }
    
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                // Update all modules that need periodic updates
                let commands: Vec<Command<Message>> = self.modules
                    .update_all(Message::Tick);
                
                // Schedule next tick (1 second)
                Command::batch(commands)
            }
            
            Message::ModuleMessage { module_name, message } => {
                // Route message to specific module
                if let Some(module) = self.modules.get_by_name_mut(&module_name) {
                    if let Some(cmd) = module.update(Message::ModuleMessage {
                        module_name: module_name.clone(),
                        message,
                    }) {
                        return cmd;
                    }
                }
                Command::none()
            }
            
            Message::IpcEvent(event) => {
                // Broadcast event to all modules
                let commands = self.modules.update_all(Message::IpcEvent(event));
                Command::batch(commands)
            }
            
            Message::SwitchWorkspace(id) => {
                // Send workspace switch command via IPC
                if let Some(ref mut ipc) = self.ipc_client {
                    // TODO: Implement IPC command sending
                }
                Command::none()
            }
            
            Message::ExecuteCommand(cmd) => {
                // Execute arbitrary command via IPC
                if let Some(ref mut ipc) = self.ipc_client {
                    // TODO: Implement IPC command execution
                }
                Command::none()
            }
        }
    }
    
    fn view(&self) -> Element<Message> {
        // Create rows for each position
        let left_modules = self.create_module_row(Position::Left);
        let center_modules = self.create_module_row(Position::Center);
        let right_modules = self.create_module_row(Position::Right);
        
        // Create main row with spacing
        let main_row = Row::new()
            .push(left_modules)
            .push(iced::widget::Space::with_width(iced::Length::Fill))
            .push(center_modules)
            .push(iced::widget::Space::with_width(iced::Length::Fill))
            .push(right_modules);
        
        // Wrap in container with styling
        let background_color = module::parse_color(&self.config.style.background_color);
        
        container(main_row)
            .width(iced::Length::Fill)
            .height(iced::Length::Fixed(self.config.bar.height as f32))
            .style(move |_theme| {
                iced::widget::container::Appearance {
                    background: Some(iced::Background::Color(background_color)),
                    ..Default::default()
                }
            })
            .into()
    }
    
    fn subscription(&self) -> iced::Subscription<Message> {
        // Subscribe to time ticks every second
        iced::time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    }
}

impl StatusBar {
    /// Create a row of modules for a specific position
    fn create_module_row(&self, position: Position) -> Row<Message> {
        let mut row = Row::new().spacing(10);
        
        for module in self.modules.get_by_position(position) {
            if module.config().enabled {
                row = row.push(module.view());
            }
        }
        
        row
    }
}

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_status_bar=debug,info")
        .with_target(false)
        .init();
    
    info!("Starting Tiling Window Manager Status Bar");
    
    // Load config to get window settings
    let config = ConfigLoader::new()
        .and_then(|loader| loader.load())
        .expect("Failed to load configuration");
    
    // Determine window position
    let (x, y) = match config.bar.position {
        config::BarPosition::Top => (0, 0),
        config::BarPosition::Bottom => {
            // Get screen height and position at bottom
            // TODO: Get actual screen dimensions
            (0, 1050) // Placeholder
        }
    };
    
    // Run application
    StatusBar::run(Settings {
        window: window::Settings {
            size: (1920, config.bar.height), // TODO: Get screen width
            position: window::Position::Specific(x, y),
            decorations: false,
            transparent: false,
            always_on_top: config.bar.always_on_top,
            level: if config.bar.always_on_top {
                window::Level::AlwaysOnTop
            } else {
                window::Level::Normal
            },
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
```

**Acceptance Criteria:**
- [ ] Application compiles and runs
- [ ] Window appears at correct position
- [ ] Always-on-top works
- [ ] No window decorations
- [ ] Modules are displayed
- [ ] Configuration is loaded
- [ ] Logging works correctly

**Testing Requirements:**
- Manual testing required (UI application)
- Verify window positioning
- Verify module loading
- Check memory usage (<50MB)

**Validation Commands:**
```bash
cargo run -p tiling-wm-status-bar
cargo build -p tiling-wm-status-bar --release
```

---

### Week 23: Core Modules Implementation

#### Task 6.5: Implement Workspaces Module

**Objective:** Create the workspaces indicator module that shows all workspaces and allows switching.

**File:** `crates/status-bar/src/modules/workspaces.rs`

**Required Implementations:**

```rust
use crate::module::{Module, Message, ModuleMessage, ModuleConfig, Position, IpcEvent};
use iced::{Command, Element, Length};
use iced::widget::{button, row, text, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WorkspacesModule {
    config: ModuleConfig,
    workspaces: Vec<WorkspaceInfo>,
    active_workspace: usize,
    workspace_config: WorkspacesConfig,
}

#[derive(Debug, Clone)]
struct WorkspaceInfo {
    id: usize,
    name: String,
    window_count: usize,
    visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspacesConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default)]
    icons: std::collections::HashMap<String, String>,
    
    #[serde(default = "default_active_color")]
    active_color: String,
    
    #[serde(default = "default_inactive_color")]
    inactive_color: String,
    
    #[serde(default = "default_urgent_color")]
    urgent_color: String,
}

fn default_format() -> String {
    "{icon}".to_string()
}

fn default_active_color() -> String {
    "#89b4fa".to_string()
}

fn default_inactive_color() -> String {
    "#585b70".to_string()
}

fn default_urgent_color() -> String {
    "#f38ba8".to_string()
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            icons: Default::default(),
            active_color: default_active_color(),
            inactive_color: default_inactive_color(),
            urgent_color: default_urgent_color(),
        }
    }
}

impl WorkspacesModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "workspaces".to_string(),
            position: Position::Left,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        // Create initial workspaces (will be updated from IPC)
        let workspaces = (1..=10)
            .map(|i| WorkspaceInfo {
                id: i,
                name: i.to_string(),
                window_count: 0,
                visible: i == 1,
            })
            .collect();
        
        Self {
            config,
            workspaces,
            active_workspace: 1,
            workspace_config: WorkspacesConfig::default(),
        }
    }
    
    fn format_workspace(&self, ws: &WorkspaceInfo) -> String {
        // Use icon if available, otherwise use name
        let icon = self.workspace_config.icons
            .get(&ws.id.to_string())
            .cloned()
            .unwrap_or_else(|| ws.name.clone());
        
        self.workspace_config.format
            .replace("{icon}", &icon)
            .replace("{name}", &ws.name)
            .replace("{windows}", &ws.window_count.to_string())
    }
    
    fn get_workspace_color(&self, ws: &WorkspaceInfo) -> iced::Color {
        if ws.visible {
            crate::module::parse_color(&self.workspace_config.active_color)
        } else {
            crate::module::parse_color(&self.workspace_config.inactive_color)
        }
    }
}

impl Module for WorkspacesModule {
    fn view(&self) -> Element<'_, Message> {
        let mut workspace_row = Row::new().spacing(5);
        
        for ws in &self.workspaces {
            let label = self.format_workspace(ws);
            let color = self.get_workspace_color(ws);
            
            let btn = button(
                text(label)
                    .style(move |_theme| {
                        iced::widget::text::Appearance {
                            color: Some(color),
                        }
                    })
                    .size(self.config.style.font_size.unwrap_or(12.0))
            )
            .on_press(Message::ModuleMessage {
                module_name: self.name().to_string(),
                message: Box::new(ModuleMessage::WorkspaceClicked(ws.id)),
            })
            .style(|_theme, _status| {
                iced::widget::button::Appearance {
                    background: None,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            });
            
            workspace_row = workspace_row.push(btn);
        }
        
        crate::module::styled_container(workspace_row.into(), &self.config.style).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        match message {
            Message::ModuleMessage { ref message, .. } => {
                if let ModuleMessage::WorkspaceClicked(id) = **message {
                    // Send workspace switch command
                    return Some(Command::perform(
                        async move { id },
                        Message::SwitchWorkspace,
                    ));
                }
            }
            Message::IpcEvent(IpcEvent::WorkspaceChanged { to, .. }) => {
                // Update active workspace
                self.active_workspace = to;
                for ws in &mut self.workspaces {
                    ws.visible = ws.id == to;
                }
            }
            _ => {}
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
}

impl Default for WorkspacesModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Module compiles without errors
- [ ] Workspaces are displayed
- [ ] Active workspace is highlighted
- [ ] Clicking workspace switches to it
- [ ] IPC events update workspace state
- [ ] Custom icons work
- [ ] Colors are configurable

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_module_creation() {
        let module = WorkspacesModule::new();
        assert_eq!(module.name(), "workspaces");
        assert_eq!(module.position(), Position::Left);
        assert_eq!(module.workspaces.len(), 10);
    }
    
    #[test]
    fn test_workspace_formatting() {
        let module = WorkspacesModule::new();
        let ws = WorkspaceInfo {
            id: 1,
            name: "Main".to_string(),
            window_count: 3,
            visible: true,
        };
        
        let formatted = module.format_workspace(&ws);
        assert!(!formatted.is_empty());
    }
    
    #[test]
    fn test_active_workspace_update() {
        let mut module = WorkspacesModule::new();
        assert_eq!(module.active_workspace, 1);
        
        module.update(Message::IpcEvent(IpcEvent::WorkspaceChanged {
            from: 1,
            to: 2,
        }));
        
        assert_eq!(module.active_workspace, 2);
    }
}
```

---

(The document continues with more tasks for other modules, IPC client implementation, styling system, and completion checklist. Due to length constraints, I'll create the document in multiple writes.)

Would you like me to continue writing the rest of the Phase 6 tasks document?

#### Task 6.6: Implement Clock Module

**Objective:** Create a clock module with customizable time/date format.

**File:** `crates/status-bar/src/modules/clock.rs`

**Required Implementations:**

```rust
use crate::module::{Module, Message, ModuleConfig, Position};
use iced::{Command, Element};
use iced::widget::text;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ClockModule {
    config: ModuleConfig,
    current_time: String,
    clock_config: ClockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClockConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default)]
    format_alt: Option<String>,
}

fn default_format() -> String {
    "%H:%M:%S".to_string()
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            format_alt: Some("%Y-%m-%d".to_string()),
        }
    }
}

impl ClockModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "clock".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            current_time: String::new(),
            clock_config: ClockConfig::default(),
        }
    }
    
    fn update_time(&mut self) {
        self.current_time = Local::now()
            .format(&self.clock_config.format)
            .to_string();
    }
}

impl Module for ClockModule {
    fn view(&self) -> Element<'_, Message> {
        let color = crate::module::parse_color(&self.config.style.color);
        
        crate::module::styled_container(
            text(&self.current_time)
                .style(move |_theme| {
                    iced::widget::text::Appearance {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        if matches!(message, Message::Tick) {
            self.update_time();
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
    
    fn init(&mut self) -> Option<Command<Message>> {
        self.update_time();
        None
    }
    
    fn update_interval(&self) -> u64 {
        1 // Update every second
    }
}

impl Default for ClockModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Clock displays current time
- [ ] Format is customizable
- [ ] Updates every second
- [ ] Style settings apply
- [ ] Time format is validated

---

#### Task 6.7: Implement System Information Modules (CPU, Memory)

**Objective:** Create modules for displaying CPU and memory usage.

**File:** `crates/status-bar/src/modules/cpu.rs`

**Required Implementations:**

```rust
use crate::module::{Module, Message, ModuleConfig, Position};
use iced::{Command, Element};
use iced::widget::text;
use sysinfo::{System, SystemExt, CpuExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CpuModule {
    config: ModuleConfig,
    system: System,
    usage: f32,
    cpu_config: CpuConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpuConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_format() -> String {
    " {usage}%".to_string()
}

fn default_interval() -> u64 {
    5
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            interval: default_interval(),
        }
    }
}

impl CpuModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "cpu".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        let mut system = System::new_all();
        system.refresh_cpu();
        
        Self {
            config,
            system,
            usage: 0.0,
            cpu_config: CpuConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        self.system.refresh_cpu();
        self.usage = self.system.global_cpu_info().cpu_usage();
    }
    
    fn format_text(&self) -> String {
        self.cpu_config.format
            .replace("{usage}", &format!("{:.1}", self.usage))
    }
}

impl Module for CpuModule {
    fn view(&self) -> Element<'_, Message> {
        let color = crate::module::parse_color(&self.config.style.color);
        
        crate::module::styled_container(
            text(self.format_text())
                .style(move |_theme| {
                    iced::widget::text::Appearance {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        if matches!(message, Message::Tick) {
            self.update_usage();
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
    
    fn init(&mut self) -> Option<Command<Message>> {
        self.update_usage();
        None
    }
    
    fn update_interval(&self) -> u64 {
        self.cpu_config.interval
    }
}

impl Default for CpuModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**File:** `crates/status-bar/src/modules/memory.rs`

```rust
use crate::module::{Module, Message, ModuleConfig, Position};
use iced::{Command, Element};
use iced::widget::text;
use sysinfo::{System, SystemExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MemoryModule {
    config: ModuleConfig,
    system: System,
    usage_percent: f32,
    memory_config: MemoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_format() -> String {
    " {percentage}%".to_string()
}

fn default_interval() -> u64 {
    5
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            interval: default_interval(),
        }
    }
}

impl MemoryModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "memory".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        let mut system = System::new_all();
        system.refresh_memory();
        
        Self {
            config,
            system,
            usage_percent: 0.0,
            memory_config: MemoryConfig::default(),
        }
    }
    
    fn update_usage(&mut self) {
        self.system.refresh_memory();
        let total = self.system.total_memory() as f32;
        let used = self.system.used_memory() as f32;
        self.usage_percent = (used / total) * 100.0;
    }
    
    fn format_text(&self) -> String {
        let used_gb = (self.system.used_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
        let total_gb = (self.system.total_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
        
        self.memory_config.format
            .replace("{percentage}", &format!("{:.1}", self.usage_percent))
            .replace("{used}", &format!("{:.1}", used_gb))
            .replace("{total}", &format!("{:.1}", total_gb))
    }
}

impl Module for MemoryModule {
    fn view(&self) -> Element<'_, Message> {
        let color = crate::module::parse_color(&self.config.style.color);
        
        crate::module::styled_container(
            text(self.format_text())
                .style(move |_theme| {
                    iced::widget::text::Appearance {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        if matches!(message, Message::Tick) {
            self.update_usage();
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
    
    fn init(&mut self) -> Option<Command<Message>> {
        self.update_usage();
        None
    }
    
    fn update_interval(&self) -> u64 {
        self.memory_config.interval
    }
}

impl Default for MemoryModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] CPU usage displays correctly
- [ ] Memory usage displays correctly
- [ ] Updates at configured interval
- [ ] Format strings work
- [ ] System information is accurate
- [ ] Performance impact is minimal

---

#### Task 6.8: Implement Battery Module

**Objective:** Create a battery status module for laptops.

**File:** `crates/status-bar/src/modules/battery.rs`

**Required Implementations:**

```rust
use crate::module::{Module, Message, ModuleConfig, Position};
use iced::{Command, Element};
use iced::widget::text;
use battery::{Manager, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BatteryModule {
    config: ModuleConfig,
    manager: Manager,
    percentage: f32,
    state: BatteryState,
    battery_config: BatteryConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatteryConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_warning_level")]
    warning_level: u32,
    
    #[serde(default = "default_critical_level")]
    critical_level: u32,
}

fn default_format() -> String {
    "{icon} {percentage}%".to_string()
}

fn default_warning_level() -> u32 {
    30
}

fn default_critical_level() -> u32 {
    15
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            warning_level: default_warning_level(),
            critical_level: default_critical_level(),
        }
    }
}

impl BatteryModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "battery".to_string(),
            position: Position::Right,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        let manager = Manager::new().expect("Failed to create battery manager");
        
        Self {
            config,
            manager,
            percentage: 0.0,
            state: BatteryState::Unknown,
            battery_config: BatteryConfig::default(),
        }
    }
    
    /// Check if battery is available on this system
    pub fn is_available() -> bool {
        Manager::new()
            .ok()
            .and_then(|manager| manager.batteries().ok())
            .map(|mut batteries| batteries.next().is_some())
            .unwrap_or(false)
    }
    
    fn update_status(&mut self) {
        if let Ok(batteries) = self.manager.batteries() {
            if let Some(Ok(battery)) = batteries.into_iter().next() {
                self.percentage = battery.state_of_charge().value * 100.0;
                self.state = match battery.state() {
                    State::Charging => BatteryState::Charging,
                    State::Discharging => BatteryState::Discharging,
                    State::Full => BatteryState::Full,
                    _ => BatteryState::Unknown,
                };
            }
        }
    }
    
    fn get_icon(&self) -> &str {
        match self.state {
            BatteryState::Charging => "",
            BatteryState::Full => "",
            BatteryState::Discharging => {
                if self.percentage > 75.0 {
                    ""
                } else if self.percentage > 50.0 {
                    ""
                } else if self.percentage > 25.0 {
                    ""
                } else {
                    ""
                }
            }
            BatteryState::Unknown => "",
        }
    }
    
    fn format_text(&self) -> String {
        self.battery_config.format
            .replace("{icon}", self.get_icon())
            .replace("{percentage}", &format!("{:.0}", self.percentage))
            .replace("{state}", match self.state {
                BatteryState::Charging => "Charging",
                BatteryState::Discharging => "Discharging",
                BatteryState::Full => "Full",
                BatteryState::Unknown => "Unknown",
            })
    }
    
    fn get_color(&self) -> String {
        if self.percentage <= self.battery_config.critical_level as f32 {
            "#f38ba8".to_string() // Critical - red
        } else if self.percentage <= self.battery_config.warning_level as f32 {
            "#f9e2af".to_string() // Warning - yellow
        } else {
            self.config.style.color.clone()
        }
    }
}

impl Module for BatteryModule {
    fn view(&self) -> Element<'_, Message> {
        let color = crate::module::parse_color(&self.get_color());
        
        crate::module::styled_container(
            text(self.format_text())
                .style(move |_theme| {
                    iced::widget::text::Appearance {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        if matches!(message, Message::Tick) {
            self.update_status();
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
    
    fn init(&mut self) -> Option<Command<Message>> {
        self.update_status();
        None
    }
    
    fn update_interval(&self) -> u64 {
        30 // Update every 30 seconds
    }
}

impl Default for BatteryModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Battery status displays correctly
- [ ] Charging state is shown
- [ ] Icon changes based on level
- [ ] Warning colors work
- [ ] Handles systems without battery
- [ ] Updates periodically

---

#### Task 6.9: Implement Window Title Module

**Objective:** Create a module that displays the active window's title.

**File:** `crates/status-bar/src/modules/window_title.rs`

**Required Implementations:**

```rust
use crate::module::{Module, Message, ModuleConfig, Position, IpcEvent};
use iced::{Command, Element};
use iced::widget::text;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WindowTitleModule {
    config: ModuleConfig,
    window_title: String,
    window_config: WindowTitleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowTitleConfig {
    #[serde(default = "default_format")]
    format: String,
    
    #[serde(default = "default_max_length")]
    max_length: usize,
}

fn default_format() -> String {
    "{title}".to_string()
}

fn default_max_length() -> usize {
    50
}

impl Default for WindowTitleConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            max_length: default_max_length(),
        }
    }
}

impl WindowTitleModule {
    pub fn new() -> Self {
        let config = ModuleConfig {
            name: "window-title".to_string(),
            position: Position::Center,
            enabled: true,
            style: Default::default(),
            config: serde_json::Value::Null,
        };
        
        Self {
            config,
            window_title: String::new(),
            window_config: WindowTitleConfig::default(),
        }
    }
    
    fn truncate_title(&self, title: &str) -> String {
        if title.len() > self.window_config.max_length {
            format!("{}...", &title[..self.window_config.max_length - 3])
        } else {
            title.to_string()
        }
    }
    
    fn format_text(&self) -> String {
        let truncated = self.truncate_title(&self.window_title);
        self.window_config.format
            .replace("{title}", &truncated)
    }
}

impl Module for WindowTitleModule {
    fn view(&self) -> Element<'_, Message> {
        let color = crate::module::parse_color(&self.config.style.color);
        
        crate::module::styled_container(
            text(self.format_text())
                .style(move |_theme| {
                    iced::widget::text::Appearance {
                        color: Some(color),
                    }
                })
                .size(self.config.style.font_size.unwrap_or(12.0))
                .into(),
            &self.config.style
        ).into()
    }
    
    fn update(&mut self, message: Message) -> Option<Command<Message>> {
        match message {
            Message::IpcEvent(IpcEvent::WindowFocused { title, .. }) => {
                self.window_title = title;
            }
            Message::IpcEvent(IpcEvent::WindowClosed { .. }) => {
                // Clear title if no window focused
                self.window_title.clear();
            }
            _ => {}
        }
        None
    }
    
    fn position(&self) -> Position {
        self.config.position
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config(&self) -> &ModuleConfig {
        &self.config
    }
}

impl Default for WindowTitleModule {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Displays active window title
- [ ] Updates on window focus change
- [ ] Title truncation works
- [ ] Format string is applied
- [ ] IPC events are handled

---

### Week 24: IPC Integration

#### Task 6.10: Implement IPC Client for Status Bar

**Objective:** Create an IPC client that connects to the window manager and receives events.

**File:** `crates/status-bar/src/ipc_client.rs`

**Required Implementations:**

```rust
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use crate::module::IpcEvent;

#[derive(Clone)]
pub struct IpcClient {
    pipe_name: String,
    event_sender: Option<mpsc::UnboundedSender<IpcEvent>>,
    connected: Arc<Mutex<bool>>,
}

impl IpcClient {
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
            event_sender: None,
            connected: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Set event sender for receiving IPC events
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<IpcEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Connect to window manager and start event listener
    pub async fn connect(&self) -> Result<()> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)
            .context("Failed to connect to window manager IPC")?;
        
        *self.connected.lock().await = true;
        
        tracing::info!("Connected to window manager IPC");
        
        // Subscribe to events
        let subscribe_request = serde_json::json!({
            "type": "subscribe",
            "events": [
                "workspace_changed",
                "window_focused",
                "window_created",
                "window_closed",
                "config_reloaded"
            ]
        });
        
        self.send_request_on_connection(&mut client, &subscribe_request).await?;
        
        Ok(())
    }
    
    /// Get list of workspaces
    pub async fn get_workspaces(&self) -> Result<Vec<WorkspaceData>> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)?;
        
        let request = serde_json::json!({
            "type": "get_workspaces"
        });
        
        let response = self.send_request_on_connection(&mut client, &request).await?;
        
        if let Some(data) = response.get("data") {
            let workspaces = serde_json::from_value(data.clone())?;
            Ok(workspaces)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get active window information
    pub async fn get_active_window(&self) -> Result<Option<WindowData>> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)?;
        
        let request = serde_json::json!({
            "type": "get_active_window"
        });
        
        let response = self.send_request_on_connection(&mut client, &request).await?;
        
        if let Some(data) = response.get("data") {
            if data.is_null() {
                Ok(None)
            } else {
                let window = serde_json::from_value(data.clone())?;
                Ok(Some(window))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Switch to a workspace
    pub async fn switch_workspace(&self, id: usize) -> Result<()> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)?;
        
        let request = serde_json::json!({
            "type": "switch_workspace",
            "id": id
        });
        
        let _response = self.send_request_on_connection(&mut client, &request).await?;
        
        Ok(())
    }
    
    /// Send request and receive response
    async fn send_request_on_connection(
        &self,
        client: &mut tokio::net::windows::named_pipe::NamedPipeClient,
        request: &Value,
    ) -> Result<Value> {
        // Serialize request
        let request_data = serde_json::to_vec(request)?;
        let len = request_data.len() as u32;
        
        // Write length prefix
        client.write_all(&len.to_le_bytes()).await?;
        
        // Write request data
        client.write_all(&request_data).await?;
        
        client.flush().await?;
        
        // Read response length prefix
        let mut len_buf = [0u8; 4];
        client.read_exact(&mut len_buf).await?;
        let response_len = u32::from_le_bytes(len_buf) as usize;
        
        // Read response data
        let mut response_data = vec![0u8; response_len];
        client.read_exact(&mut response_data).await?;
        
        // Parse response
        let response: Value = serde_json::from_slice(&response_data)?;
        
        Ok(response)
    }
    
    /// Start event listener (runs in background)
    pub async fn start_event_listener(&self) -> Result<()> {
        if self.event_sender.is_none() {
            anyhow::bail!("Event sender not set");
        }
        
        let sender = self.event_sender.as_ref().unwrap().clone();
        let pipe_name = self.pipe_name.clone();
        
        tokio::spawn(async move {
            loop {
                match Self::listen_for_events(&pipe_name, &sender).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Event listener error: {}", e);
                        // Retry after delay
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn listen_for_events(
        pipe_name: &str,
        sender: &mpsc::UnboundedSender<IpcEvent>,
    ) -> Result<()> {
        let mut client = ClientOptions::new()
            .open(pipe_name)?;
        
        // Already subscribed in connect(), just listen for events
        loop {
            // Read event length prefix
            let mut len_buf = [0u8; 4];
            match client.read_exact(&mut len_buf).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Connection lost: {}", e);
                    return Err(e.into());
                }
            }
            
            let event_len = u32::from_le_bytes(len_buf) as usize;
            
            // Read event data
            let mut event_data = vec![0u8; event_len];
            client.read_exact(&mut event_data).await?;
            
            // Parse event
            let event_value: Value = serde_json::from_slice(&event_data)?;
            
            // Convert to IpcEvent
            if let Some(ipc_event) = Self::parse_event(&event_value) {
                let _ = sender.send(ipc_event);
            }
        }
    }
    
    fn parse_event(value: &Value) -> Option<IpcEvent> {
        let event_type = value.get("type")?.as_str()?;
        
        match event_type {
            "event" => {
                let name = value.get("name")?.as_str()?;
                let data = value.get("data")?;
                
                match name {
                    "workspace_changed" => {
                        Some(IpcEvent::WorkspaceChanged {
                            from: data.get("from")?.as_u64()? as usize,
                            to: data.get("to")?.as_u64()? as usize,
                        })
                    }
                    "window_focused" => {
                        Some(IpcEvent::WindowFocused {
                            hwnd: data.get("hwnd")?.as_str()?.to_string(),
                            title: data.get("title")?.as_str()?.to_string(),
                        })
                    }
                    "window_created" => {
                        Some(IpcEvent::WindowCreated {
                            hwnd: data.get("hwnd")?.as_str()?.to_string(),
                            title: data.get("title")?.as_str()?.to_string(),
                        })
                    }
                    "window_closed" => {
                        Some(IpcEvent::WindowClosed {
                            hwnd: data.get("hwnd")?.as_str()?.to_string(),
                        })
                    }
                    "config_reloaded" => {
                        Some(IpcEvent::ConfigReloaded)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected.try_lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkspaceData {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub window_count: usize,
    pub active: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowData {
    pub hwnd: String,
    pub title: String,
    pub class: String,
    pub process_name: String,
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}
```

**Acceptance Criteria:**
- [ ] Connects to window manager IPC
- [ ] Subscribes to events
- [ ] Receives events in real-time
- [ ] Can query workspace information
- [ ] Can switch workspaces
- [ ] Reconnects on disconnection
- [ ] Error handling is robust

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ipc_client_creation() {
        let client = IpcClient::new();
        assert_eq!(client.pipe_name, r"\\.\pipe\tiling-wm");
    }
    
    #[tokio::test]
    #[ignore] // Requires running window manager
    async fn test_ipc_connection() {
        let client = IpcClient::new();
        let result = client.connect().await;
        
        // May fail if window manager not running
        if result.is_ok() {
            assert!(client.is_connected());
        }
    }
}
```

---

### Week 25-26: Polish and Multi-Monitor Support

#### Task 6.11: Implement Multi-Monitor Support

**Objective:** Enable status bar to work across multiple monitors.

**File:** `crates/status-bar/src/monitor.rs`

**Required Implementations:**

```rust
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO, HDC,
};
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static MONITORS: Lazy<Mutex<Vec<MonitorInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub handle: HMONITOR,
    pub work_area: (i32, i32, u32, u32), // (x, y, width, height)
    pub is_primary: bool,
}

pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors = MONITORS.lock().unwrap();
    monitors.clear();
    
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC(0),
            None,
            Some(monitor_enum_proc),
            LPARAM(0),
        );
    }
    
    monitors.clone()
}

extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    unsafe {
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            let work_area = monitor_info.rcWork;
            let is_primary = monitor_info.dwFlags == 1;
            
            let info = MonitorInfo {
                handle: hmonitor,
                work_area: (
                    work_area.left,
                    work_area.top,
                    (work_area.right - work_area.left) as u32,
                    (work_area.bottom - work_area.top) as u32,
                ),
                is_primary,
            };
            
            MONITORS.lock().unwrap().push(info);
        }
    }
    
    BOOL(1) // Continue enumeration
}

pub fn get_primary_monitor() -> Option<MonitorInfo> {
    enumerate_monitors()
        .into_iter()
        .find(|m| m.is_primary)
}

pub fn get_monitor_count() -> usize {
    enumerate_monitors().len()
}
```

**Update main.rs to support multiple monitors:**

```rust
// In main.rs, add multi-monitor window creation
fn create_status_bars(config: &BarConfig) -> Vec<(window::Id, iced::window::Settings)> {
    let monitors = if let Some(monitor_id) = config.bar.monitor {
        // Single monitor specified
        vec![monitor_id]
    } else {
        // All monitors
        (0..monitor::get_monitor_count()).collect()
    };
    
    monitors
        .into_iter()
        .map(|monitor_id| {
            let monitor_info = monitor::enumerate_monitors()
                .get(monitor_id)
                .cloned()
                .unwrap_or_else(|| monitor::get_primary_monitor().unwrap());
            
            let (x, y) = match config.bar.position {
                config::BarPosition::Top => (monitor_info.work_area.0, monitor_info.work_area.1),
                config::BarPosition::Bottom => {
                    let bottom_y = monitor_info.work_area.1 
                        + monitor_info.work_area.3 as i32 
                        - config.bar.height as i32;
                    (monitor_info.work_area.0, bottom_y)
                }
            };
            
            let settings = window::Settings {
                size: (monitor_info.work_area.2, config.bar.height),
                position: window::Position::Specific(x, y),
                decorations: false,
                transparent: false,
                always_on_top: config.bar.always_on_top,
                level: if config.bar.always_on_top {
                    window::Level::AlwaysOnTop
                } else {
                    window::Level::Normal
                },
                resizable: false,
                ..Default::default()
            };
            
            (window::Id::unique(), settings)
        })
        .collect()
}
```

**Acceptance Criteria:**
- [ ] Monitors are enumerated correctly
- [ ] Status bar appears on all monitors (if configured)
- [ ] Status bar can be limited to specific monitor
- [ ] Position calculation works for all monitors
- [ ] Primary monitor is detected
- [ ] Multi-monitor hotplug is handled

---

#### Task 6.12: Add Module Loading System

**Objective:** Create a system to dynamically load and configure modules.

**File:** `crates/status-bar/src/modules/mod.rs`

**Required Implementations:**

```rust
pub mod workspaces;
pub mod window_title;
pub mod clock;
pub mod cpu;
pub mod memory;
pub mod battery;
// pub mod network;  // Optional
// pub mod volume;   // Optional

use crate::module::Module;
use crate::config::BarConfig;

/// Factory for creating modules
pub struct ModuleFactory;

impl ModuleFactory {
    pub fn create_module(name: &str, config: &BarConfig) -> Option<Box<dyn Module>> {
        match name {
            "workspaces" => Some(Box::new(workspaces::WorkspacesModule::new())),
            "window-title" => Some(Box::new(window_title::WindowTitleModule::new())),
            "clock" => Some(Box::new(clock::ClockModule::new())),
            "cpu" => Some(Box::new(cpu::CpuModule::new())),
            "memory" => Some(Box::new(memory::MemoryModule::new())),
            "battery" if battery::BatteryModule::is_available() => {
                Some(Box::new(battery::BatteryModule::new()))
            }
            _ => {
                tracing::warn!("Unknown module: {}", name);
                None
            }
        }
    }
    
    pub fn create_all_modules(config: &BarConfig) -> Vec<Box<dyn Module>> {
        let mut modules = Vec::new();
        
        // Load modules from left, center, and right
        for name in &config.modules.left {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        for name in &config.modules.center {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        for name in &config.modules.right {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        tracing::info!("Loaded {} modules", modules.len());
        modules
    }
}
```

**Update main application to use module factory:**

```rust
// In StatusBar::new()
let modules_list = modules::ModuleFactory::create_all_modules(&config);

for module in modules_list {
    modules.register(module);
}
```

**Acceptance Criteria:**
- [ ] Module factory creates modules correctly
- [ ] Configuration is passed to modules
- [ ] Unknown modules are logged
- [ ] Optional modules are handled
- [ ] Module creation errors are handled

---

## Phase 6 Completion Checklist

### Build & Compilation
- [ ] `cargo build -p tiling-wm-status-bar` succeeds without errors
- [ ] `cargo build -p tiling-wm-status-bar --release` succeeds
- [ ] No warnings from `cargo clippy -p tiling-wm-status-bar -- -D warnings`
- [ ] Code formatted with `cargo fmt -p tiling-wm-status-bar --check`
- [ ] All dependencies resolve correctly

### Core Functionality
- [ ] Status bar window appears correctly
- [ ] Window positioning works (top/bottom)
- [ ] Always-on-top behavior works
- [ ] Module system loads modules
- [ ] All core modules display correctly
- [ ] IPC connection works
- [ ] Events are received from window manager
- [ ] Workspace switching from bar works
- [ ] Configuration loads correctly
- [ ] Multi-monitor support works

### Module Functionality
- [ ] Workspaces module shows all workspaces
- [ ] Workspace switching works from module
- [ ] Active workspace is highlighted
- [ ] Window title module displays active window
- [ ] Window title updates on focus change
- [ ] Clock module shows correct time
- [ ] Clock updates every second
- [ ] CPU module shows usage
- [ ] Memory module shows usage
- [ ] Battery module shows status (if available)
- [ ] Battery warnings work

### IPC Integration
- [ ] Connects to window manager on startup
- [ ] Subscribes to events successfully
- [ ] Receives workspace change events
- [ ] Receives window focus events
- [ ] Receives window created/closed events
- [ ] Can send commands to window manager
- [ ] Reconnects on disconnection
- [ ] Error handling is robust

### Configuration
- [ ] TOML configuration parses correctly
- [ ] All settings are respected
- [ ] Module positioning works
- [ ] Styling applies correctly
- [ ] Colors and fonts work
- [ ] Module-specific configs work
- [ ] Default config is created if missing

### Testing
- [ ] Unit tests pass: `cargo test -p tiling-wm-status-bar`
- [ ] Module tests pass
- [ ] Configuration tests pass
- [ ] IPC client tests pass
- [ ] No test failures or panics

### Performance
- [ ] Memory usage < 50MB
- [ ] CPU usage < 1% when idle
- [ ] CPU usage < 5% during updates
- [ ] No memory leaks
- [ ] Update intervals are respected
- [ ] IPC doesn't block UI

### Documentation
- [ ] All public APIs have doc comments
- [ ] `cargo doc --no-deps -p tiling-wm-status-bar` builds successfully
- [ ] Configuration options documented
- [ ] Module creation guide written
- [ ] README updated with Phase 6 features

### Manual Validation
- [ ] Start status bar with window manager running
- [ ] Verify all modules display
- [ ] Switch workspaces and verify indicator updates
- [ ] Focus different windows and verify title updates
- [ ] Verify system info modules show correct values
- [ ] Test on multiple monitors
- [ ] Test with different configurations
- [ ] Verify reconnection after window manager restart
- [ ] Run for 15+ minutes to check stability
- [ ] Memory usage remains stable
- [ ] No UI glitches or rendering issues

---

## Deliverables for Phase 6

At the end of Phase 6, you should have:

1. **Complete Status Bar Application:**
   - Separate binary that runs independently
   - Iced-based UI framework
   - Window positioning system
   - Multi-monitor support
   - Always-on-top behavior

2. **Modular Widget System:**
   - Module trait definition
   - Module registry
   - Module factory
   - Position system (left/center/right)
   - Module lifecycle management

3. **Core Modules:**
   - Workspaces indicator module
   - Window title display module
   - Clock module with format support
   - CPU usage module
   - Memory usage module
   - Battery status module
   - All modules tested and functional

4. **IPC Integration:**
   - IPC client library
   - Event subscription
   - Real-time event handling
   - Command execution
   - Connection management
   - Reconnection logic

5. **Configuration System:**
   - TOML configuration parser
   - Comprehensive default config
   - Bar positioning settings
   - Style configuration
   - Module configuration
   - Per-module settings

6. **Multi-Monitor Support:**
   - Monitor enumeration
   - Per-monitor status bars
   - Position calculation
   - DPI awareness

7. **Quality Assurance:**
   - Comprehensive unit tests
   - Integration tests
   - Manual validation
   - Performance testing
   - Documentation complete

---

## Success Criteria Summary

Phase 6 is complete when:

1. ✅ **Status bar framework is operational:**
   - Application builds and runs
   - Window positioning is correct
   - Always-on-top works
   - Multi-monitor support functional
   - Memory and CPU usage acceptable

2. ✅ **Module system is complete:**
   - Modules load correctly
   - Position system works
   - Module updates function
   - Custom modules possible
   - Module factory operational

3. ✅ **Core modules are implemented:**
   - All specified modules working
   - Updates are timely
   - Information is accurate
   - Styling applies correctly
   - Performance is good

4. ✅ **IPC integration works:**
   - Connects to window manager
   - Receives events in real-time
   - Can send commands
   - Reconnection works
   - Error handling robust

5. ✅ **Configuration is functional:**
   - TOML parsing works
   - All settings respected
   - Default config comprehensive
   - Module configs apply
   - Validation works

6. ✅ **Quality standards met:**
   - All tests passing
   - No clippy warnings
   - Documentation complete
   - Performance acceptable
   - Stable operation

---

## Next Steps

After completing Phase 6, proceed to **Phase 7: Polish & Advanced Features** (Weeks 27-32), which will implement:

- Window animations (if feasible with DWM)
- Window groups/containers
- Scratchpad workspace
- Window swallowing
- Advanced layout options
- Performance optimizations
- Additional status bar modules
- Themes system

See DETAILED_ROADMAP.md for Phase 7 specifications.

---

## Troubleshooting

### Common Issues

**Issue: Status bar window not visible**
- Solution: Check window positioning calculations
- Verify monitor enumeration works
- Ensure always-on-top is enabled
- Check DWM composition is active
- Test with default config

**Issue: Modules not loading**
- Solution: Check module factory
- Verify module names in config
- Check for module errors in logs
- Test modules individually
- Verify dependencies available

**Issue: IPC connection fails**
- Solution: Verify window manager is running
- Check pipe name is correct
- Test with CLI tool first
- Check Windows permissions
- Verify named pipe creation

**Issue: Events not received**
- Solution: Check subscription worked
- Verify event listener is running
- Test with window manager events
- Check for parsing errors
- Enable debug logging

**Issue: High CPU usage**
- Solution: Check update intervals
- Profile module update functions
- Verify no busy loops
- Check IPC event handling
- Review rendering efficiency

**Issue: Memory leaks**
- Solution: Check module cleanup
- Verify IPC connection cleanup
- Review iced widget lifecycle
- Use memory profiler
- Check for circular references

**Issue: Multi-monitor issues**
- Solution: Test monitor enumeration
- Verify position calculations
- Check DPI scaling
- Test with different arrangements
- Handle hotplug events

**Issue: Styling not applied**
- Solution: Verify color parsing
- Check CSS-like syntax
- Test with default styles
- Verify font availability
- Check iced theme system

---

## Notes for Autonomous Agents

When executing this task list:

1. **Follow order strictly**: Status bar tasks build on each other
2. **Test UI frequently**: Visual verification is critical
3. **Validate each step**: UI bugs are hard to track down
4. **Handle IPC carefully**: Connection issues are common
5. **Test on real systems**: Multi-monitor needs real testing
6. **Check performance**: Status bar should be lightweight
7. **Think about users**: UI should be intuitive
8. **Test edge cases**: Battery missing, no windows, etc.
9. **Check iced docs**: Framework has specific patterns
10. **Reference phases 1-5**: Build on IPC foundation

---

**End of Phase 6 Task Document**
