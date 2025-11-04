//! Configuration schema definitions
//! 
//! This module defines all configuration data structures with serde support
//! for TOML parsing and serialization.

use serde::{Serialize, Deserialize};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General window manager settings
    #[serde(default)]
    pub general: GeneralConfig,
    
    /// Window decoration settings
    #[serde(default)]
    pub decoration: DecorationConfig,
    
    /// Animation settings
    #[serde(default)]
    pub animations: AnimationsConfig,
    
    /// Input and keyboard settings
    #[serde(default)]
    pub input: InputConfig,
    
    /// Layout-specific settings
    #[serde(default)]
    pub layouts: LayoutsConfig,
    
    /// Window rules for automatic window management
    #[serde(default)]
    pub window_rules: Vec<WindowRule>,
    
    /// Workspace assignment rules
    #[serde(default)]
    pub workspace_rules: Vec<WorkspaceRule>,
    
    /// Keybinding configuration
    #[serde(default)]
    pub keybinds: Vec<Keybind>,
    
    /// Monitor configuration
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

/// Animation curve types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationCurve {
    /// Linear interpolation
    Linear,
    /// Ease in curve
    EaseIn,
    /// Ease out curve
    EaseOut,
    /// Ease in and out curve
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

/// Layout-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutsConfig {
    /// Default layout for new workspaces
    #[serde(default = "default_layout")]
    pub default: String,
    
    /// Dwindle layout configuration
    #[serde(default)]
    pub dwindle: DwindleConfig,
    
    /// Master layout configuration
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
