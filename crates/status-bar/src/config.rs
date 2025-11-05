//! Configuration system for the status bar
//!
//! This module handles loading and parsing the status bar configuration from TOML files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// Main status bar configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
        r##"# Status Bar Configuration

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
"##.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    
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
    fn test_bar_position_serialization() {
        let pos = BarPosition::Top;
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#""top""#);
        
        let deserialized: BarPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, BarPosition::Top);
    }
}
