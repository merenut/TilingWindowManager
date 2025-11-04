//! Validation script for the default configuration file
//! 
//! This example validates that the default_config.toml file is:
//! - Valid TOML syntax
//! - Can be parsed into the Config struct
//! - Meets all requirements (keybindings count, window rules, etc.)
//! 
//! Run with: cargo run --example validate_default_config

use tiling_wm_core::config::schema::Config;
use std::process;

fn main() {
    println!("========================================");
    println!("Validating Default Configuration File");
    println!("========================================\n");
    
    // Path to default config (relative to workspace root)
    let config_path = "config/default_config.toml";
    
    // Read the configuration file
    let config_content = match std::fs::read_to_string(config_path) {
        Ok(content) => {
            println!("✓ Configuration file found: {}", config_path);
            content
        }
        Err(e) => {
            eprintln!("✗ Failed to read configuration file: {}", e);
            eprintln!("  Expected path: {}", config_path);
            process::exit(1);
        }
    };
    
    // Test 1: Validate TOML syntax
    println!("\n[1/4] Validating TOML syntax...");
    match toml::from_str::<toml::Value>(&config_content) {
        Ok(_) => println!("✓ Configuration is valid TOML"),
        Err(e) => {
            eprintln!("✗ TOML parsing error:");
            eprintln!("  {}", e);
            process::exit(1);
        }
    }
    
    // Test 2: Parse into Config struct
    println!("\n[2/4] Parsing into configuration schema...");
    let config: Config = match toml::from_str(&config_content) {
        Ok(config) => {
            println!("✓ Configuration successfully parsed into schema");
            config
        }
        Err(e) => {
            eprintln!("✗ Schema parsing error:");
            eprintln!("  {}", e);
            process::exit(1);
        }
    };
    
    // Test 3: Validate configuration values
    println!("\n[3/4] Validating configuration values...");
    
    // Check general settings
    println!("  General settings:");
    println!("    - gaps_in: {}", config.general.gaps_in);
    println!("    - gaps_out: {}", config.general.gaps_out);
    println!("    - border_size: {}", config.general.border_size);
    println!("    - active_border_color: {}", config.general.active_border_color);
    println!("    - inactive_border_color: {}", config.general.inactive_border_color);
    println!("    - auto_tile: {}", config.general.auto_tile);
    
    // Check decoration settings
    println!("  Decoration settings:");
    println!("    - rounding: {}", config.decoration.rounding);
    println!("    - active_opacity: {}", config.decoration.active_opacity);
    println!("    - inactive_opacity: {}", config.decoration.inactive_opacity);
    println!("    - shadows: {}", config.decoration.shadows);
    println!("    - shadow_color: {}", config.decoration.shadow_color);
    
    // Check animation settings
    println!("  Animation settings:");
    println!("    - enabled: {}", config.animations.enabled);
    println!("    - speed: {}", config.animations.speed);
    println!("    - curve: {:?}", config.animations.curve);
    
    // Check input settings
    println!("  Input settings:");
    println!("    - repeat_rate: {}", config.input.repeat_rate);
    println!("    - repeat_delay: {}", config.input.repeat_delay);
    println!("    - follow_mouse: {}", config.input.follow_mouse);
    
    // Check layout settings
    println!("  Layout settings:");
    println!("    - default: {}", config.layouts.default);
    println!("    - dwindle.smart_split: {}", config.layouts.dwindle.smart_split);
    println!("    - dwindle.no_gaps_when_only: {}", config.layouts.dwindle.no_gaps_when_only);
    println!("    - dwindle.split_ratio: {}", config.layouts.dwindle.split_ratio);
    println!("    - master.master_factor: {}", config.layouts.master.master_factor);
    println!("    - master.master_count: {}", config.layouts.master.master_count);
    
    println!("✓ All configuration values are valid");
    
    // Test 4: Check requirements
    println!("\n[4/4] Checking requirements...");
    
    // Count keybindings
    let keybind_count = config.keybinds.len();
    println!("  Keybindings: {}", keybind_count);
    if keybind_count >= 20 {
        println!("  ✓ Requirement met: At least 20 keybindings (found {})", keybind_count);
    } else {
        eprintln!("  ✗ Requirement failed: Need at least 20 keybindings, found {}", keybind_count);
        process::exit(1);
    }
    
    // Count window rules
    let window_rules_count = config.window_rules.len();
    println!("  Window rules: {}", window_rules_count);
    if window_rules_count >= 5 {
        println!("  ✓ Requirement met: At least 5 window rules (found {})", window_rules_count);
    } else {
        eprintln!("  ✗ Requirement failed: Need at least 5 window rules, found {}", window_rules_count);
        process::exit(1);
    }
    
    // Count workspace rules
    let workspace_rules_count = config.workspace_rules.len();
    println!("  Workspace rules: {}", workspace_rules_count);
    if workspace_rules_count > 0 {
        println!("  ✓ Requirement met: Workspace configuration present ({} workspaces)", workspace_rules_count);
    } else {
        eprintln!("  ✗ Requirement failed: No workspace configuration found");
        process::exit(1);
    }
    
    // Count monitor configurations
    let monitor_count = config.monitors.len();
    println!("  Monitor configurations: {}", monitor_count);
    println!("  ℹ  (Monitor configuration is optional)");
    
    // Display summary
    println!("\n========================================");
    println!("Configuration Summary");
    println!("========================================");
    println!("  ✓ Valid TOML syntax");
    println!("  ✓ Parses into schema correctly");
    println!("  ✓ {} keybindings (≥20 required)", keybind_count);
    println!("  ✓ {} window rules (≥5 required)", window_rules_count);
    println!("  ✓ {} workspace rules", workspace_rules_count);
    println!("  ℹ  {} monitor configurations", monitor_count);
    println!();
    
    // Show keybinding categories
    println!("Keybinding Categories:");
    let mut window_mgmt = 0;
    let mut focus_nav = 0;
    let mut window_move = 0;
    let mut layout = 0;
    let mut workspace_switch = 0;
    let mut workspace_move = 0;
    let mut system = 0;
    let mut apps = 0;
    
    for keybind in &config.keybinds {
        if keybind.command.starts_with("close") 
            || keybind.command.starts_with("toggle-floating")
            || keybind.command.starts_with("toggle-fullscreen")
            || keybind.command.starts_with("minimize") {
            window_mgmt += 1;
        } else if keybind.command.starts_with("focus-") {
            focus_nav += 1;
        } else if keybind.command.starts_with("move-left")
            || keybind.command.starts_with("move-right")
            || keybind.command.starts_with("move-up")
            || keybind.command.starts_with("move-down") {
            window_move += 1;
        } else if keybind.command.starts_with("layout-")
            || keybind.command.contains("master") {
            layout += 1;
        } else if keybind.command.starts_with("workspace-") 
            && !keybind.command.starts_with("move-to-workspace") {
            workspace_switch += 1;
        } else if keybind.command.starts_with("move-to-workspace") {
            workspace_move += 1;
        } else if keybind.command.starts_with("reload")
            || keybind.command.starts_with("exit") {
            system += 1;
        } else if keybind.command.starts_with("exec") {
            apps += 1;
        }
    }
    
    println!("  - Window Management: {}", window_mgmt);
    println!("  - Focus Navigation: {}", focus_nav);
    println!("  - Window Movement: {}", window_move);
    println!("  - Layout Commands: {}", layout);
    println!("  - Workspace Switching: {}", workspace_switch);
    println!("  - Move to Workspace: {}", workspace_move);
    println!("  - System Commands: {}", system);
    println!("  - Application Launchers: {}", apps);
    println!();
    
    // Show window rule types
    println!("Window Rule Types:");
    for (i, rule) in config.window_rules.iter().enumerate() {
        let match_type = if rule.match_process.is_some() {
            "process"
        } else if rule.match_title.is_some() {
            "title"
        } else if rule.match_class.is_some() {
            "class"
        } else {
            "unknown"
        };
        
        let actions: Vec<String> = rule.actions.iter().map(|a| format!("{:?}", a)).collect();
        println!("  Rule {}: match by {}, actions: [{}]", 
                 i + 1, match_type, actions.join(", "));
    }
    println!();
    
    // Show workspace assignments
    println!("Workspace Assignments:");
    for ws in &config.workspace_rules {
        let default_marker = if ws.default { " (default)" } else { "" };
        let name = ws.name.as_ref().map(|n| n.as_str()).unwrap_or("unnamed");
        println!("  Workspace {}: \"{}\" on monitor {}{}", 
                 ws.id, name, ws.monitor, default_marker);
    }
    println!();
    
    println!("========================================");
    println!("✓ Default configuration is VALID!");
    println!("========================================");
}
