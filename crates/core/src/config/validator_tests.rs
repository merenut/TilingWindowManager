//! Comprehensive tests for configuration validator
//! 
//! Tests cover:
//! - Valid configurations
//! - Invalid numeric ranges
//! - Invalid color formats
//! - Invalid regex patterns
//! - Duplicate workspace IDs
//! - Duplicate keybindings
//! - Edge cases and boundary conditions

#[cfg(test)]
mod tests {
    use crate::config::validator::ConfigValidator;
    use crate::config::schema::*;
    
    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // General Configuration Tests
    // ========================================
    
    #[test]
    fn test_negative_gaps_in() {
        let mut config = Config::default();
        config.general.gaps_in = -5;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("gaps_in"));
        assert!(err_msg.contains("non-negative"));
    }
    
    #[test]
    fn test_negative_gaps_out() {
        let mut config = Config::default();
        config.general.gaps_out = -10;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("gaps_out"));
    }
    
    #[test]
    fn test_negative_border_size() {
        let mut config = Config::default();
        config.general.border_size = -1;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("border_size"));
    }
    
    #[test]
    fn test_zero_values_allowed() {
        let mut config = Config::default();
        config.general.gaps_in = 0;
        config.general.gaps_out = 0;
        config.general.border_size = 0;
        
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Decoration Configuration Tests
    // ========================================
    
    #[test]
    fn test_invalid_active_opacity_too_high() {
        let mut config = Config::default();
        config.decoration.active_opacity = 1.5;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("opacity"));
        assert!(err_msg.contains("0.0") || err_msg.contains("1.0"));
    }
    
    #[test]
    fn test_invalid_active_opacity_negative() {
        let mut config = Config::default();
        config.decoration.active_opacity = -0.1;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_inactive_opacity() {
        let mut config = Config::default();
        config.decoration.inactive_opacity = 2.0;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("opacity"));
    }
    
    #[test]
    fn test_opacity_boundary_values() {
        let mut config = Config::default();
        
        // Test 0.0
        config.decoration.active_opacity = 0.0;
        config.decoration.inactive_opacity = 0.0;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test 1.0
        config.decoration.active_opacity = 1.0;
        config.decoration.inactive_opacity = 1.0;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test mid-range
        config.decoration.active_opacity = 0.5;
        config.decoration.inactive_opacity = 0.75;
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    #[test]
    fn test_negative_rounding() {
        let mut config = Config::default();
        config.decoration.rounding = -5;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rounding"));
    }
    
    // ========================================
    // Color Validation Tests
    // ========================================
    
    #[test]
    fn test_valid_color_formats() {
        // Test #RGB format
        assert!(ConfigValidator::validate_color("#fff").is_ok());
        assert!(ConfigValidator::validate_color("#000").is_ok());
        assert!(ConfigValidator::validate_color("#abc").is_ok());
        
        // Test #RRGGBB format
        assert!(ConfigValidator::validate_color("#ffffff").is_ok());
        assert!(ConfigValidator::validate_color("#000000").is_ok());
        assert!(ConfigValidator::validate_color("#89b4fa").is_ok());
        assert!(ConfigValidator::validate_color("#AABBCC").is_ok());
        
        // Test #RRGGBBAA format
        assert!(ConfigValidator::validate_color("#ffffffff").is_ok());
        assert!(ConfigValidator::validate_color("#00000080").is_ok());
        assert!(ConfigValidator::validate_color("#89b4faff").is_ok());
    }
    
    #[test]
    fn test_invalid_color_formats() {
        // Missing # prefix
        assert!(ConfigValidator::validate_color("fff").is_err());
        assert!(ConfigValidator::validate_color("ffffff").is_err());
        
        // Wrong length
        assert!(ConfigValidator::validate_color("#ff").is_err());
        assert!(ConfigValidator::validate_color("#ffff").is_err());
        assert!(ConfigValidator::validate_color("#fffff").is_err());
        assert!(ConfigValidator::validate_color("#fffffff").is_err());
        assert!(ConfigValidator::validate_color("#fffffffff").is_err());
        
        // Invalid hex characters
        assert!(ConfigValidator::validate_color("#gggggg").is_err());
        assert!(ConfigValidator::validate_color("#12345g").is_err());
        assert!(ConfigValidator::validate_color("#xyz").is_err());
        
        // Empty or just #
        assert!(ConfigValidator::validate_color("#").is_err());
        assert!(ConfigValidator::validate_color("").is_err());
    }
    
    #[test]
    fn test_invalid_border_colors() {
        let mut config = Config::default();
        config.general.active_border_color = "not a color".to_string();
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("active_border_color") || err_msg.contains("Color"));
    }
    
    #[test]
    fn test_invalid_shadow_color() {
        let mut config = Config::default();
        config.decoration.shadow_color = "invalid".to_string();
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("shadow_color") || err_msg.contains("Color"));
    }
    
    // ========================================
    // Animation Configuration Tests
    // ========================================
    
    #[test]
    fn test_zero_animation_speed() {
        let mut config = Config::default();
        config.animations.speed = 0.0;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("speed"));
    }
    
    #[test]
    fn test_negative_animation_speed() {
        let mut config = Config::default();
        config.animations.speed = -1.0;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_excessive_animation_speed() {
        let mut config = Config::default();
        config.animations.speed = 11.0;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("10.0"));
    }
    
    #[test]
    fn test_valid_animation_speeds() {
        let mut config = Config::default();
        
        // Test minimum valid
        config.animations.speed = 0.1;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test normal
        config.animations.speed = 1.0;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test fast
        config.animations.speed = 5.0;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test maximum
        config.animations.speed = 10.0;
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Layout Configuration Tests
    // ========================================
    
    #[test]
    fn test_invalid_default_layout() {
        let mut config = Config::default();
        config.layouts.default = "invalid".to_string();
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("layout"));
        assert!(err_msg.contains("dwindle") || err_msg.contains("master"));
    }
    
    #[test]
    fn test_invalid_split_ratio_too_low() {
        let mut config = Config::default();
        config.layouts.dwindle.split_ratio = 0.05;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("split_ratio"));
    }
    
    #[test]
    fn test_invalid_split_ratio_too_high() {
        let mut config = Config::default();
        config.layouts.dwindle.split_ratio = 0.95;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_master_factor_too_low() {
        let mut config = Config::default();
        config.layouts.master.master_factor = 0.05;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("master_factor"));
    }
    
    #[test]
    fn test_invalid_master_factor_too_high() {
        let mut config = Config::default();
        config.layouts.master.master_factor = 0.95;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_zero_master_count() {
        let mut config = Config::default();
        config.layouts.master.master_count = 0;
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("master_count"));
        assert!(err_msg.contains("1") || err_msg.contains("at least"));
    }
    
    #[test]
    fn test_valid_layout_ranges() {
        let mut config = Config::default();
        
        // Test boundary values
        config.layouts.dwindle.split_ratio = 0.1;
        config.layouts.master.master_factor = 0.1;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        config.layouts.dwindle.split_ratio = 0.9;
        config.layouts.master.master_factor = 0.9;
        assert!(ConfigValidator::validate(&config).is_ok());
        
        // Test normal values
        config.layouts.dwindle.split_ratio = 0.5;
        config.layouts.master.master_factor = 0.55;
        config.layouts.master.master_count = 2;
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Window Rule Tests
    // ========================================
    
    #[test]
    fn test_window_rule_no_match_condition() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("match condition"));
    }
    
    #[test]
    fn test_window_rule_no_actions() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: Some("test.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("action"));
    }
    
    #[test]
    fn test_window_rule_invalid_regex_process() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: Some("[invalid regex".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("regex") || err_msg.contains("Invalid"));
        assert!(err_msg.contains("match_process"));
    }
    
    #[test]
    fn test_window_rule_invalid_regex_title() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: Some("(unclosed group".to_string()),
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("regex") || err_msg.contains("Invalid"));
        assert!(err_msg.contains("match_title"));
    }
    
    #[test]
    fn test_window_rule_invalid_regex_class() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: None,
            match_class: Some("*invalid*".to_string()),
            actions: vec![RuleAction::Float],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("regex") || err_msg.contains("Invalid"));
        assert!(err_msg.contains("match_class"));
    }
    
    #[test]
    fn test_window_rule_valid_regex_patterns() {
        let mut config = Config::default();
        
        // Valid literal match
        config.window_rules.push(WindowRule {
            match_process: Some("firefox.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Workspace(2)],
        });
        
        // Valid regex with escaping
        config.window_rules.push(WindowRule {
            match_process: Some(r"notepad\.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        // Valid wildcard pattern
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: Some(".*Steam.*".to_string()),
            match_class: None,
            actions: vec![RuleAction::Float],
        });
        
        // Valid character class
        config.window_rules.push(WindowRule {
            match_process: None,
            match_title: None,
            match_class: Some("[A-Z][a-z]+Window".to_string()),
            actions: vec![RuleAction::NoManage],
        });
        
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    #[test]
    fn test_rule_action_invalid_opacity() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: Some("test.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Opacity(1.5)],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("opacity"));
    }
    
    #[test]
    fn test_rule_action_invalid_workspace_id() {
        let mut config = Config::default();
        
        config.window_rules.push(WindowRule {
            match_process: Some("test.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Workspace(0)],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("workspace") || err_msg.contains("ID"));
    }
    
    // ========================================
    // Workspace Rule Tests
    // ========================================
    
    #[test]
    fn test_workspace_rule_zero_id() {
        let mut config = Config::default();
        
        config.workspace_rules.push(WorkspaceRule {
            id: 0,
            monitor: 0,
            default: true,
            name: Some("Invalid".to_string()),
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Workspace ID"));
        assert!(err_msg.contains("1") || err_msg.contains("at least"));
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
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Duplicate"));
        assert!(err_msg.contains("1"));
    }
    
    #[test]
    fn test_valid_workspace_rules() {
        let mut config = Config::default();
        
        config.workspace_rules.push(WorkspaceRule {
            id: 1,
            monitor: 0,
            default: true,
            name: Some("Main".to_string()),
        });
        
        config.workspace_rules.push(WorkspaceRule {
            id: 2,
            monitor: 0,
            default: false,
            name: Some("Web".to_string()),
        });
        
        config.workspace_rules.push(WorkspaceRule {
            id: 3,
            monitor: 1,
            default: true,
            name: None,
        });
        
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Keybinding Tests
    // ========================================
    
    #[test]
    fn test_keybind_invalid_modifier() {
        let mut config = Config::default();
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Super".to_string()], // Invalid modifier
            key: "q".to_string(),
            command: "close".to_string(),
            args: vec![],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("modifier"));
        assert!(err_msg.contains("Super"));
    }
    
    #[test]
    fn test_keybind_empty_command() {
        let mut config = Config::default();
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Win".to_string()],
            key: "q".to_string(),
            command: "".to_string(),
            args: vec![],
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("command"));
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
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Duplicate"));
        assert!(err_msg.contains("keybind"));
    }
    
    #[test]
    fn test_valid_keybindings() {
        let mut config = Config::default();
        
        // Single modifier
        config.keybinds.push(Keybind {
            modifiers: vec!["Win".to_string()],
            key: "q".to_string(),
            command: "close".to_string(),
            args: vec![],
        });
        
        // Multiple modifiers
        config.keybinds.push(Keybind {
            modifiers: vec!["Win".to_string(), "Shift".to_string()],
            key: "q".to_string(),
            command: "quit".to_string(),
            args: vec![],
        });
        
        // All valid modifiers
        config.keybinds.push(Keybind {
            modifiers: vec!["Ctrl".to_string()],
            key: "a".to_string(),
            command: "focus".to_string(),
            args: vec![],
        });
        
        config.keybinds.push(Keybind {
            modifiers: vec!["Alt".to_string()],
            key: "b".to_string(),
            command: "focus".to_string(),
            args: vec![],
        });
        
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Monitor Configuration Tests
    // ========================================
    
    #[test]
    fn test_invalid_monitor_resolution() {
        let mut config = Config::default();
        
        config.monitors.push(MonitorConfig {
            name: "Test".to_string(),
            resolution: Some("invalid".to_string()),
            position: None,
            scale: None,
            refresh_rate: None,
            rotation: None,
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("resolution"));
    }
    
    #[test]
    fn test_invalid_monitor_position() {
        let mut config = Config::default();
        
        config.monitors.push(MonitorConfig {
            name: "Test".to_string(),
            resolution: None,
            position: Some("notvalid".to_string()),
            scale: None,
            refresh_rate: None,
            rotation: None,
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("position"));
    }
    
    #[test]
    fn test_invalid_monitor_scale_too_low() {
        let mut config = Config::default();
        
        config.monitors.push(MonitorConfig {
            name: "Test".to_string(),
            resolution: None,
            position: None,
            scale: Some(0.0),
            refresh_rate: None,
            rotation: None,
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("scale"));
    }
    
    #[test]
    fn test_invalid_monitor_scale_too_high() {
        let mut config = Config::default();
        
        config.monitors.push(MonitorConfig {
            name: "Test".to_string(),
            resolution: None,
            position: None,
            scale: Some(5.0),
            refresh_rate: None,
            rotation: None,
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("scale"));
    }
    
    #[test]
    fn test_invalid_monitor_rotation() {
        let mut config = Config::default();
        
        config.monitors.push(MonitorConfig {
            name: "Test".to_string(),
            resolution: None,
            position: None,
            scale: None,
            refresh_rate: None,
            rotation: Some(45),
        });
        
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("rotation"));
        assert!(err_msg.contains("90") || err_msg.contains("180") || err_msg.contains("270"));
    }
    
    #[test]
    fn test_valid_monitor_configurations() {
        let mut config = Config::default();
        
        // Valid resolution formats
        config.monitors.push(MonitorConfig {
            name: "Monitor1".to_string(),
            resolution: Some("1920x1080".to_string()),
            position: Some("0x0".to_string()),
            scale: Some(1.0),
            refresh_rate: Some(60),
            rotation: Some(0),
        });
        
        config.monitors.push(MonitorConfig {
            name: "Monitor2".to_string(),
            resolution: Some("2560x1440".to_string()),
            position: Some("1920x0".to_string()),
            scale: Some(1.25),
            refresh_rate: Some(144),
            rotation: Some(90),
        });
        
        // Auto position
        config.monitors.push(MonitorConfig {
            name: "Monitor3".to_string(),
            resolution: Some("3840x2160".to_string()),
            position: Some("auto".to_string()),
            scale: Some(2.0),
            refresh_rate: Some(60),
            rotation: Some(180),
        });
        
        // Valid rotation values
        config.monitors.push(MonitorConfig {
            name: "Monitor4".to_string(),
            resolution: None,
            position: None,
            scale: None,
            refresh_rate: None,
            rotation: Some(270),
        });
        
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    // ========================================
    // Resolution and Position Format Tests
    // ========================================
    
    #[test]
    fn test_valid_resolution_formats() {
        assert!(ConfigValidator::is_valid_resolution("1920x1080"));
        assert!(ConfigValidator::is_valid_resolution("2560x1440"));
        assert!(ConfigValidator::is_valid_resolution("3840x2160"));
        assert!(ConfigValidator::is_valid_resolution("1024x768"));
        assert!(ConfigValidator::is_valid_resolution("800x600"));
    }
    
    #[test]
    fn test_invalid_resolution_formats() {
        assert!(!ConfigValidator::is_valid_resolution("1920"));
        assert!(!ConfigValidator::is_valid_resolution("invalid"));
        assert!(!ConfigValidator::is_valid_resolution("1920x"));
        assert!(!ConfigValidator::is_valid_resolution("x1080"));
        assert!(!ConfigValidator::is_valid_resolution("1920-1080"));
        assert!(!ConfigValidator::is_valid_resolution("1920 x 1080"));
    }
    
    #[test]
    fn test_valid_position_formats() {
        assert!(ConfigValidator::is_valid_position("0x0"));
        assert!(ConfigValidator::is_valid_position("1920x0"));
        assert!(ConfigValidator::is_valid_position("-100x-50"));
        assert!(ConfigValidator::is_valid_position("100x200"));
    }
    
    #[test]
    fn test_invalid_position_formats() {
        assert!(!ConfigValidator::is_valid_position("100"));
        assert!(!ConfigValidator::is_valid_position("invalid"));
        assert!(!ConfigValidator::is_valid_position("100x"));
        assert!(!ConfigValidator::is_valid_position("x200"));
        assert!(!ConfigValidator::is_valid_position("100-200"));
    }
}
