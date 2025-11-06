//! Status bar modules
//!
//! This module contains implementations of various status bar widgets.

pub mod workspaces;
pub mod window_title;
pub mod clock;
pub mod cpu;
pub mod memory;
pub mod battery;

use crate::module::Module;
use crate::config::BarConfig;

/// Factory for creating modules based on configuration
pub struct ModuleFactory;

impl ModuleFactory {
    /// Create a single module by name
    ///
    /// # Arguments
    /// * `name` - The name of the module to create (e.g., "workspaces", "clock")
    /// * `config` - The bar configuration containing module-specific settings (currently unused, reserved for future enhancements)
    ///
    /// # Returns
    /// * `Some(Box<dyn Module>)` if the module is known and available
    /// * `None` if the module is unknown or unavailable (e.g., battery on desktop)
    pub fn create_module(name: &str, config: &BarConfig) -> Option<Box<dyn Module>> {
        let _ = config; // Acknowledge parameter for future use
        match name {
            "workspaces" => {
                tracing::debug!("Creating workspaces module");
                Some(Box::new(workspaces::WorkspacesModule::new()))
            }
            "window-title" | "window_title" => {
                tracing::debug!("Creating window-title module");
                Some(Box::new(window_title::WindowTitleModule::new()))
            }
            "clock" => {
                tracing::debug!("Creating clock module");
                Some(Box::new(clock::ClockModule::new()))
            }
            "cpu" => {
                tracing::debug!("Creating cpu module");
                Some(Box::new(cpu::CpuModule::new()))
            }
            "memory" => {
                tracing::debug!("Creating memory module");
                Some(Box::new(memory::MemoryModule::new()))
            }
            "battery" => {
                if battery::BatteryModule::is_available() {
                    tracing::debug!("Creating battery module (battery detected)");
                    Some(Box::new(battery::BatteryModule::new()))
                } else {
                    tracing::info!("Battery module requested but no battery detected, skipping");
                    None
                }
            }
            _ => {
                tracing::warn!("Unknown module requested: '{}', skipping", name);
                None
            }
        }
    }
    
    /// Create all modules specified in the configuration
    ///
    /// This method loads modules from left, center, and right positions
    /// as specified in the configuration. Unknown or unavailable modules
    /// are logged and skipped.
    ///
    /// # Arguments
    /// * `config` - The bar configuration containing module lists
    ///
    /// # Returns
    /// A vector of boxed module trait objects ready to be registered
    pub fn create_all_modules(config: &BarConfig) -> Vec<Box<dyn Module>> {
        let mut modules = Vec::new();
        
        tracing::info!("Loading modules from configuration...");
        
        // Load modules from left side
        for name in &config.modules.left {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        // Load modules from center
        for name in &config.modules.center {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        // Load modules from right side
        for name in &config.modules.right {
            if let Some(module) = Self::create_module(name, config) {
                modules.push(module);
            }
        }
        
        tracing::info!("Successfully loaded {} modules", modules.len());
        modules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_module_workspaces() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("workspaces", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "workspaces");
    }
    
    #[test]
    fn test_create_module_window_title() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("window-title", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "window-title");
    }
    
    #[test]
    fn test_create_module_window_title_underscore_variant() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("window_title", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "window-title");
    }
    
    #[test]
    fn test_create_module_clock() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("clock", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "clock");
    }
    
    #[test]
    fn test_create_module_cpu() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("cpu", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "cpu");
    }
    
    #[test]
    fn test_create_module_memory() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("memory", &config);
        assert!(module.is_some());
        assert_eq!(module.unwrap().name(), "memory");
    }
    
    #[test]
    fn test_create_module_battery() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("battery", &config);
        // Battery may or may not be available depending on the system
        // Just check it returns a consistent result
        let is_available = battery::BatteryModule::is_available();
        assert_eq!(module.is_some(), is_available);
    }
    
    #[test]
    fn test_create_module_unknown() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("unknown-module", &config);
        assert!(module.is_none());
    }
    
    #[test]
    fn test_create_module_empty_name() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("", &config);
        assert!(module.is_none());
    }
    
    #[test]
    fn test_create_module_with_special_characters() {
        let config = BarConfig::default();
        let module = ModuleFactory::create_module("module@#$%", &config);
        assert!(module.is_none());
    }
    
    #[test]
    fn test_create_all_modules_default_config() {
        let config = BarConfig::default();
        let modules = ModuleFactory::create_all_modules(&config);
        
        // Calculate expected modules from actual default config
        let mut expected_count = config.modules.left.len() 
            + config.modules.center.len() 
            + config.modules.right.len();
        
        // Battery might not be available on all systems
        if config.modules.left.contains(&"battery".to_string())
            || config.modules.center.contains(&"battery".to_string())
            || config.modules.right.contains(&"battery".to_string())
        {
            if !battery::BatteryModule::is_available() {
                expected_count -= 1;
            }
        }
        
        assert_eq!(modules.len(), expected_count);
        
        // Verify at least some core modules are present
        let module_names: Vec<&str> = modules.iter().map(|m| m.name()).collect();
        assert!(module_names.contains(&"workspaces"), "workspaces module should be present");
        assert!(module_names.contains(&"clock"), "clock module should be present");
    }
    
    #[test]
    fn test_create_all_modules_empty_config() {
        let mut config = BarConfig::default();
        config.modules.left.clear();
        config.modules.center.clear();
        config.modules.right.clear();
        
        let modules = ModuleFactory::create_all_modules(&config);
        assert_eq!(modules.len(), 0);
    }
    
    #[test]
    fn test_create_all_modules_with_unknown_modules() {
        let mut config = BarConfig::default();
        config.modules.left = vec!["workspaces".to_string(), "unknown1".to_string()];
        config.modules.center = vec!["unknown2".to_string()];
        config.modules.right = vec!["clock".to_string()];
        
        let modules = ModuleFactory::create_all_modules(&config);
        
        // Should only create workspaces and clock (unknown modules are skipped)
        assert_eq!(modules.len(), 2);
        
        // Verify module names
        let names: Vec<&str> = modules.iter().map(|m| m.name()).collect();
        assert!(names.contains(&"workspaces"));
        assert!(names.contains(&"clock"));
    }
    
    #[test]
    fn test_create_all_modules_preserves_order() {
        let mut config = BarConfig::default();
        config.modules.left = vec!["workspaces".to_string()];
        config.modules.center = vec!["window-title".to_string()];
        config.modules.right = vec!["cpu".to_string(), "memory".to_string(), "clock".to_string()];
        
        let modules = ModuleFactory::create_all_modules(&config);
        
        // Should maintain order: left, center, then right
        assert_eq!(modules.len(), 5);
        assert_eq!(modules[0].name(), "workspaces");
        assert_eq!(modules[1].name(), "window-title");
        assert_eq!(modules[2].name(), "cpu");
        assert_eq!(modules[3].name(), "memory");
        assert_eq!(modules[4].name(), "clock");
    }
    
    #[test]
    fn test_create_all_modules_duplicate_modules() {
        let mut config = BarConfig::default();
        config.modules.left = vec!["clock".to_string()];
        config.modules.center = vec!["clock".to_string()];
        config.modules.right = vec!["clock".to_string()];
        
        let modules = ModuleFactory::create_all_modules(&config);
        
        // Should create all three instances (duplicates are allowed)
        assert_eq!(modules.len(), 3);
        assert!(modules.iter().all(|m| m.name() == "clock"));
    }
    
    #[test]
    fn test_create_all_modules_mixed_valid_invalid() {
        let mut config = BarConfig::default();
        config.modules.left = vec![
            "workspaces".to_string(),
            "invalid1".to_string(),
            "clock".to_string(),
        ];
        config.modules.center = vec!["invalid2".to_string()];
        config.modules.right = vec![
            "cpu".to_string(),
            "invalid3".to_string(),
            "memory".to_string(),
        ];
        
        let modules = ModuleFactory::create_all_modules(&config);
        
        // Should only create valid modules: workspaces, clock, cpu, memory
        assert_eq!(modules.len(), 4);
        
        let names: Vec<&str> = modules.iter().map(|m| m.name()).collect();
        assert_eq!(names, vec!["workspaces", "clock", "cpu", "memory"]);
    }
}
