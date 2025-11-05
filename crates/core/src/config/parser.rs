//! Configuration file parser
//! 
//! This module provides functionality for loading and parsing TOML configuration files.

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
            // Generate default configuration
            toml::to_string_pretty(&Config::default())
                .context("Failed to serialize default config")?
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
    fn test_create_default_config_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("subdir").join("config");
        let config_path = config_dir.join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        loader.create_default_config().unwrap();
        
        assert!(config_dir.exists());
        assert!(config_path.exists());
    }
    
    #[test]
    fn test_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Write a simple config
        let config_content = concat!(
            "[general]\n",
            "gaps_in = 10\n",
            "gaps_out = 20\n",
            "border_size = 3\n",
            "active_border_color = \"#ff0000\"\n",
            "inactive_border_color = \"#00ff00\"\n",
            "auto_tile = false\n",
            "\n",
            "[decoration]\n",
            "rounding = 15\n",
            "active_opacity = 0.95\n",
            "inactive_opacity = 0.85\n",
            "shadows = false\n",
            "shadow_color = \"#000000ff\"\n",
            "\n",
            "[animations]\n",
            "enabled = false\n",
            "speed = 2.0\n",
            "curve = \"linear\"\n",
            "\n",
            "[input]\n",
            "repeat_rate = 30\n",
            "repeat_delay = 500\n",
            "follow_mouse = true\n",
            "\n",
            "[layouts]\n",
            "default = \"master\"\n",
            "\n",
            "[layouts.dwindle]\n",
            "smart_split = false\n",
            "no_gaps_when_only = true\n",
            "split_ratio = 0.6\n",
            "\n",
            "[layouts.master]\n",
            "master_factor = 0.6\n",
            "master_count = 2\n",
        );
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::from_path(config_path);
        let config = loader.load().unwrap();
        
        assert_eq!(config.general.gaps_in, 10);
        assert_eq!(config.general.gaps_out, 20);
        assert_eq!(config.general.border_size, 3);
        assert_eq!(config.general.active_border_color, "#ff0000");
        assert_eq!(config.general.inactive_border_color, "#00ff00");
        assert_eq!(config.general.auto_tile, false);
        
        assert_eq!(config.decoration.rounding, 15);
        assert_eq!(config.decoration.active_opacity, 0.95);
        assert_eq!(config.decoration.inactive_opacity, 0.85);
        assert_eq!(config.decoration.shadows, false);
        
        assert_eq!(config.animations.enabled, false);
        assert_eq!(config.animations.speed, 2.0);
        
        assert_eq!(config.input.repeat_rate, 30);
        assert_eq!(config.input.repeat_delay, 500);
        assert_eq!(config.input.follow_mouse, true);
        
        assert_eq!(config.layouts.default, "master");
        assert_eq!(config.layouts.dwindle.smart_split, false);
        assert_eq!(config.layouts.dwindle.no_gaps_when_only, true);
        assert_eq!(config.layouts.dwindle.split_ratio, 0.6);
        assert_eq!(config.layouts.master.master_factor, 0.6);
        assert_eq!(config.layouts.master.master_count, 2);
    }
    
    #[test]
    fn test_load_creates_default_if_missing() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        let config = loader.load().unwrap();
        
        assert!(config_path.exists());
        assert_eq!(config.general.gaps_in, 5); // Default value
        assert_eq!(config.general.gaps_out, 10); // Default value
    }
    
    #[test]
    fn test_save_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        
        let mut config = Config::default();
        config.general.gaps_in = 15;
        config.general.gaps_out = 25;
        
        loader.save(&config).unwrap();
        
        assert!(config_path.exists());
        
        // Load and verify
        let loaded_config = loader.load().unwrap();
        assert_eq!(loaded_config.general.gaps_in, 15);
        assert_eq!(loaded_config.general.gaps_out, 25);
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
        
        // Verify backup contains original config
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("gaps_in = 5")); // Original default value
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
        assert!(err_msg.to_lowercase().contains("parse") || err_msg.to_lowercase().contains("toml"));
    }
    
    #[test]
    fn test_with_default_config_path() {
        let temp_dir = tempdir().unwrap();
        let default_config_path = temp_dir.path().join("default.toml");
        let config_path = temp_dir.path().join("config.toml");
        
        // Create a custom default config
        fs::write(&default_config_path, "[general]\ngaps_in = 100\ngaps_out = 200\n").unwrap();
        
        let loader = ConfigLoader::from_path(config_path.clone())
            .with_default(default_config_path);
        
        loader.create_default_config().unwrap();
        
        // Verify the custom default was used
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("gaps_in = 100"));
        assert!(content.contains("gaps_out = 200"));
    }
    
    #[test]
    fn test_exists() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path.clone());
        
        // Should not exist initially
        assert!(!loader.exists());
        
        // Create the config
        loader.create_default_config().unwrap();
        
        // Should exist now
        assert!(loader.exists());
    }
    
    #[test]
    fn test_load_from_path_with_window_rules() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Write config with window rules
        let config_content = concat!(
            "[general]\n",
            "gaps_in = 5\n",
            "\n",
            "[[window_rules]]\n",
            "match_process = \"firefox\\\\.exe\"\n",
            "actions = [{ workspace = 2 }]\n",
            "\n",
            "[[window_rules]]\n",
            "match_title = \".*Steam.*\"\n",
            "actions = [\"float\"]\n",
        );
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::from_path(config_path);
        let config = loader.load().unwrap();
        
        assert_eq!(config.window_rules.len(), 2);
        assert_eq!(config.window_rules[0].match_process, Some("firefox\\.exe".to_string()));
        assert_eq!(config.window_rules[1].match_title, Some(".*Steam.*".to_string()));
    }
    
    #[test]
    fn test_load_from_path_with_keybinds() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Write config with keybinds
        let config_content = concat!(
            "[general]\n",
            "gaps_in = 5\n",
            "\n",
            "[[keybinds]]\n",
            "modifiers = [\"Win\", \"Shift\"]\n",
            "key = \"q\"\n",
            "command = \"close\"\n",
            "args = []\n",
            "\n",
            "[[keybinds]]\n",
            "modifiers = [\"Win\"]\n",
            "key = \"1\"\n",
            "command = \"workspace-1\"\n",
            "args = []\n",
        );
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::from_path(config_path);
        let config = loader.load().unwrap();
        
        assert_eq!(config.keybinds.len(), 2);
        assert_eq!(config.keybinds[0].modifiers, vec!["Win", "Shift"]);
        assert_eq!(config.keybinds[0].key, "q");
        assert_eq!(config.keybinds[0].command, "close");
        assert_eq!(config.keybinds[1].key, "1");
    }
    
    #[test]
    fn test_config_serialization_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let loader = ConfigLoader::from_path(config_path);
        
        // Create a config with custom values
        let mut config = Config::default();
        config.general.gaps_in = 42;
        config.general.border_size = 5;
        config.decoration.rounding = 20;
        config.animations.speed = 1.5;
        
        // Save and reload
        loader.save(&config).unwrap();
        let loaded_config = loader.load().unwrap();
        
        // Verify all values match
        assert_eq!(loaded_config.general.gaps_in, 42);
        assert_eq!(loaded_config.general.border_size, 5);
        assert_eq!(loaded_config.decoration.rounding, 20);
        assert_eq!(loaded_config.animations.speed, 1.5);
    }
}
