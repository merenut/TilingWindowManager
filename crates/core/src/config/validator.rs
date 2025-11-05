//! Configuration validator
//! 
//! This module provides validation functionality for configuration values
//! with helpful error messages.

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
                    .with_context(|| format!("Invalid regex in rule {} match_process: '{}'", i, pattern))?;
            }
            
            if let Some(ref pattern) = rule.match_title {
                regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_title: '{}'", i, pattern))?;
            }
            
            if let Some(ref pattern) = rule.match_class {
                regex::Regex::new(pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_class: '{}'", i, pattern))?;
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
            RuleAction::Monitor(_id) => {
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
    pub fn validate_color(color: &str) -> Result<()> {
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
    pub fn is_valid_resolution(res: &str) -> bool {
        let parts: Vec<&str> = res.split('x').collect();
        if parts.len() != 2 {
            return false;
        }
        
        parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
    }
    
    /// Check if position format is valid (e.g., "0x0")
    pub fn is_valid_position(pos: &str) -> bool {
        let parts: Vec<&str> = pos.split('x').collect();
        if parts.len() != 2 {
            return false;
        }
        
        parts[0].parse::<i32>().is_ok() && parts[1].parse::<i32>().is_ok()
    }
}

#[cfg(test)]
#[path = "validator_tests.rs"]
mod validator_tests;
