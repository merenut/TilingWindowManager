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