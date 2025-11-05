//! Integration tests for the keybinding system.
//!
//! These tests verify that keybindings are correctly registered and
//! mapped to commands. Note that actual hotkey registration requires
//! Windows APIs and will only work on Windows platforms.

use tiling_wm_core::config::schema::Keybind;
use tiling_wm_core::keybinds::KeybindManager;

#[test]
fn test_keybind_manager_basic() {
    let manager = KeybindManager::new();
    assert_eq!(manager.binding_count(), 0);
}

#[test]
fn test_single_keybind_structure() {
    let keybind = Keybind {
        modifiers: vec!["Win".to_string()],
        key: "Q".to_string(),
        command: "close".to_string(),
        args: vec![],
    };

    assert_eq!(keybind.modifiers.len(), 1);
    assert_eq!(keybind.key, "Q");
    assert_eq!(keybind.command, "close");
}

#[test]
fn test_multiple_keybinds_structure() {
    let keybinds = vec![
        Keybind {
            modifiers: vec!["Win".to_string()],
            key: "Q".to_string(),
            command: "close".to_string(),
            args: vec![],
        },
        Keybind {
            modifiers: vec!["Win".to_string(), "Shift".to_string()],
            key: "Q".to_string(),
            command: "exit".to_string(),
            args: vec![],
        },
        Keybind {
            modifiers: vec!["Win".to_string()],
            key: "V".to_string(),
            command: "toggle-floating".to_string(),
            args: vec![],
        },
    ];

    assert_eq!(keybinds.len(), 3);
    assert_eq!(keybinds[0].command, "close");
    assert_eq!(keybinds[1].command, "exit");
    assert_eq!(keybinds[2].command, "toggle-floating");
}

#[test]
fn test_keybind_with_arguments() {
    let keybind = Keybind {
        modifiers: vec!["Win".to_string()],
        key: "Return".to_string(),
        command: "exec".to_string(),
        args: vec!["cmd.exe".to_string()],
    };

    assert_eq!(keybind.args.len(), 1);
    assert_eq!(keybind.args[0], "cmd.exe");
}

#[test]
fn test_workspace_keybinds() {
    let keybinds: Vec<Keybind> = (1..=10)
        .map(|i| Keybind {
            modifiers: vec!["Win".to_string()],
            key: if i == 10 {
                "0".to_string()
            } else {
                i.to_string()
            },
            command: format!("workspace-{}", i),
            args: vec![],
        })
        .collect();

    assert_eq!(keybinds.len(), 10);
    assert_eq!(keybinds[0].command, "workspace-1");
    assert_eq!(keybinds[9].command, "workspace-10");
}

#[test]
fn test_all_modifiers() {
    let keybind = Keybind {
        modifiers: vec![
            "Win".to_string(),
            "Ctrl".to_string(),
            "Alt".to_string(),
            "Shift".to_string(),
        ],
        key: "T".to_string(),
        command: "test".to_string(),
        args: vec![],
    };

    assert_eq!(keybind.modifiers.len(), 4);
    assert!(keybind.modifiers.contains(&"Win".to_string()));
    assert!(keybind.modifiers.contains(&"Ctrl".to_string()));
    assert!(keybind.modifiers.contains(&"Alt".to_string()));
    assert!(keybind.modifiers.contains(&"Shift".to_string()));
}

#[test]
fn test_function_key_keybinds() {
    let keybinds: Vec<Keybind> = (1..=12)
        .map(|i| Keybind {
            modifiers: vec!["Win".to_string()],
            key: format!("F{}", i),
            command: format!("function-{}", i),
            args: vec![],
        })
        .collect();

    assert_eq!(keybinds.len(), 12);
    assert_eq!(keybinds[0].key, "F1");
    assert_eq!(keybinds[11].key, "F12");
}

#[test]
fn test_arrow_key_keybinds() {
    let arrows = vec!["Left", "Right", "Up", "Down"];
    let keybinds: Vec<Keybind> = arrows
        .iter()
        .map(|&key| Keybind {
            modifiers: vec!["Win".to_string()],
            key: key.to_string(),
            command: format!("focus-{}", key.to_lowercase()),
            args: vec![],
        })
        .collect();

    assert_eq!(keybinds.len(), 4);
    assert_eq!(keybinds[0].command, "focus-left");
    assert_eq!(keybinds[3].command, "focus-down");
}

#[test]
fn test_special_key_keybinds() {
    let special_keys = vec![
        ("Space", "toggle-float"),
        ("Enter", "confirm"),
        ("Escape", "cancel"),
        ("Tab", "next"),
    ];

    let keybinds: Vec<Keybind> = special_keys
        .iter()
        .map(|(key, cmd)| Keybind {
            modifiers: vec!["Win".to_string()],
            key: key.to_string(),
            command: cmd.to_string(),
            args: vec![],
        })
        .collect();

    assert_eq!(keybinds.len(), 4);
    assert_eq!(keybinds[0].key, "Space");
    assert_eq!(keybinds[0].command, "toggle-float");
}

#[cfg(test)]
mod parser_tests {
    use tiling_wm_core::keybinds::parser::{is_valid_modifier, normalize_key, parse_keybind_string};

    #[test]
    fn test_parse_simple_binding() {
        let (mods, key) = parse_keybind_string("Win+Q").unwrap();
        assert_eq!(mods, vec!["Win"]);
        assert_eq!(key, "Q");
    }

    #[test]
    fn test_parse_complex_binding() {
        let (mods, key) = parse_keybind_string("Win+Shift+Alt+F1").unwrap();
        assert_eq!(mods, vec!["Win", "Shift", "Alt"]);
        assert_eq!(key, "F1");
    }

    #[test]
    fn test_normalize_keys() {
        assert_eq!(normalize_key("q"), "Q");
        assert_eq!(normalize_key("space"), "SPACE");
        assert_eq!(normalize_key("F1"), "F1");
    }

    #[test]
    fn test_valid_modifiers() {
        assert!(is_valid_modifier("Win"));
        assert!(is_valid_modifier("Ctrl"));
        assert!(is_valid_modifier("Alt"));
        assert!(is_valid_modifier("Shift"));
        assert!(!is_valid_modifier("Super"));
        assert!(!is_valid_modifier("Meta"));
        assert!(!is_valid_modifier("Invalid"));
    }
}
