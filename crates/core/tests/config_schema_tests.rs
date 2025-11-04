/// Unit tests for configuration schema
/// 
/// Tests default values, TOML serialization/deserialization, and schema validation.

use tiling_wm_core::config::*;

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
    assert_eq!(config.active_border_color, deserialized.active_border_color);
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
    // Test as part of AnimationsConfig since enums need table context for TOML
    let config = AnimationsConfig {
        enabled: true,
        speed: 1.0,
        curve: AnimationCurve::EaseInOut,
    };
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("easeinout"));
}

#[test]
fn test_layout_config_defaults() {
    let layouts = LayoutsConfig::default();
    
    assert_eq!(layouts.default, "dwindle");
    assert!(layouts.dwindle.smart_split);
    assert!(!layouts.dwindle.no_gaps_when_only);
    assert_eq!(layouts.master.master_factor, 0.55);
}

#[test]
fn test_full_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml_str).unwrap();
    
    assert_eq!(config.general.gaps_in, deserialized.general.gaps_in);
    assert_eq!(config.decoration.rounding, deserialized.decoration.rounding);
    assert_eq!(config.animations.speed, deserialized.animations.speed);
}

#[test]
fn test_config_with_rules() {
    let toml_str = r##"
        [general]
        gaps_in = 5
        gaps_out = 10
        border_size = 2
        active_border_color = "#89b4fa"
        inactive_border_color = "#585b70"
        auto_tile = true
        
        [decoration]
        rounding = 10
        active_opacity = 1.0
        inactive_opacity = 0.9
        shadows = true
        shadow_color = "#00000080"
        
        [animations]
        enabled = true
        speed = 1.0
        curve = "easeout"
        
        [input]
        repeat_rate = 25
        repeat_delay = 600
        follow_mouse = false
        
        [layouts]
        default = "dwindle"
        
        [layouts.dwindle]
        smart_split = true
        no_gaps_when_only = false
        split_ratio = 0.5
        
        [layouts.master]
        master_factor = 0.55
        master_count = 1
        
        [[window_rules]]
        match_process = "firefox\\.exe"
        actions = ["float"]
        
        [[workspace_rules]]
        id = 1
        monitor = 0
        default = true
        name = "Main"
        
        [[keybinds]]
        modifiers = ["Win"]
        key = "q"
        command = "close"
        args = []
    "##;
    
    let config: Config = toml::from_str(toml_str).unwrap();
    
    assert_eq!(config.window_rules.len(), 1);
    assert_eq!(config.workspace_rules.len(), 1);
    assert_eq!(config.keybinds.len(), 1);
}

#[test]
fn test_decoration_config_defaults() {
    let decoration = DecorationConfig::default();
    
    assert_eq!(decoration.rounding, 10);
    assert_eq!(decoration.active_opacity, 1.0);
    assert_eq!(decoration.inactive_opacity, 0.9);
    assert!(decoration.shadows);
}

#[test]
fn test_input_config_defaults() {
    let input = InputConfig::default();
    
    assert_eq!(input.repeat_rate, 25);
    assert_eq!(input.repeat_delay, 600);
    assert!(!input.follow_mouse);
}

#[test]
fn test_dwindle_config_defaults() {
    let dwindle = DwindleConfig::default();
    
    assert!(dwindle.smart_split);
    assert!(!dwindle.no_gaps_when_only);
    assert_eq!(dwindle.split_ratio, 0.5);
}

#[test]
fn test_master_config_defaults() {
    let master = MasterConfig::default();
    
    assert_eq!(master.master_factor, 0.55);
    assert_eq!(master.master_count, 1);
}

#[test]
fn test_animations_config_defaults() {
    let animations = AnimationsConfig::default();
    
    assert!(animations.enabled);
    assert_eq!(animations.speed, 1.0);
    matches!(animations.curve, AnimationCurve::EaseOut);
}

#[test]
fn test_rule_action_workspace() {
    // Test as part of a window rule since actions are in an array context
    let toml_str = r##"
        match_process = "test.exe"
        actions = [{ workspace = 3 }]
    "##;
    let rule: WindowRule = toml::from_str(toml_str).unwrap();
    
    match &rule.actions[0] {
        RuleAction::Workspace(id) => assert_eq!(*id, 3),
        _ => panic!("Expected Workspace action"),
    }
}

#[test]
fn test_rule_action_float() {
    // Test as part of a window rule since actions are in an array context
    let toml_str = r##"
        match_process = "test.exe"
        actions = ["float"]
    "##;
    let rule: WindowRule = toml::from_str(toml_str).unwrap();
    
    assert!(matches!(rule.actions[0], RuleAction::Float));
}

#[test]
fn test_workspace_rule_deserialization() {
    let toml_str = r#"
        id = 1
        monitor = 0
        default = true
        name = "Main"
    "#;
    
    let rule: WorkspaceRule = toml::from_str(toml_str).unwrap();
    
    assert_eq!(rule.id, 1);
    assert_eq!(rule.monitor, 0);
    assert!(rule.default);
    assert_eq!(rule.name, Some("Main".to_string()));
}

#[test]
fn test_monitor_config_deserialization() {
    let toml_str = r#"
        name = "Monitor1"
        resolution = "1920x1080"
        position = "0x0"
        scale = 1.0
        refresh_rate = 60
        rotation = 0
    "#;
    
    let monitor: MonitorConfig = toml::from_str(toml_str).unwrap();
    
    assert_eq!(monitor.name, "Monitor1");
    assert_eq!(monitor.resolution, Some("1920x1080".to_string()));
    assert_eq!(monitor.position, Some("0x0".to_string()));
    assert_eq!(monitor.scale, Some(1.0));
    assert_eq!(monitor.refresh_rate, Some(60));
    assert_eq!(monitor.rotation, Some(0));
}

#[test]
fn test_empty_config_uses_defaults() {
    let toml_str = r#""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    
    // Should use all default values
    assert_eq!(config.general.gaps_in, 5);
    assert_eq!(config.decoration.rounding, 10);
    assert!(config.animations.enabled);
    assert_eq!(config.layouts.default, "dwindle");
}

#[test]
fn test_partial_config_uses_defaults() {
    let toml_str = r#"
        [general]
        gaps_in = 10
    "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    
    // Specified value
    assert_eq!(config.general.gaps_in, 10);
    
    // Default values for unspecified fields
    assert_eq!(config.general.gaps_out, 10);
    assert_eq!(config.general.border_size, 2);
    assert_eq!(config.decoration.rounding, 10);
}

#[test]
fn test_keybind_with_args() {
    let toml_str = r#"
        modifiers = ["Win"]
        key = "Return"
        command = "exec"
        args = ["cmd.exe"]
    "#;
    
    let keybind: Keybind = toml::from_str(toml_str).unwrap();
    
    assert_eq!(keybind.args.len(), 1);
    assert_eq!(keybind.args[0], "cmd.exe");
}

#[test]
fn test_multiple_window_rules() {
    let toml_str = r##"
        [[window_rules]]
        match_process = "firefox\\.exe"
        actions = ["float"]
        
        [[window_rules]]
        match_title = ".*Steam.*"
        actions = ["tile", { workspace = 2 }]
        
        [[window_rules]]
        match_class = "Calculator"
        actions = [{ opacity = 0.95 }]
    "##;
    
    let config: Config = toml::from_str(&format!("{}", toml_str)).unwrap();
    
    assert_eq!(config.window_rules.len(), 3);
    assert!(config.window_rules[0].match_process.is_some());
    assert!(config.window_rules[1].match_title.is_some());
    assert!(config.window_rules[2].match_class.is_some());
}
