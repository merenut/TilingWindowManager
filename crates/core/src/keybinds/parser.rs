//! Key and modifier parsing utilities.
//!
//! This module provides utilities for parsing keybinding strings and
//! validating key combinations.

use anyhow::Result;

/// Parse a keybinding string in the format "Modifier+Modifier+Key".
///
/// Example: "Win+Shift+Q" -> (["Win", "Shift"], "Q")
pub fn parse_keybind_string(keybind: &str) -> Result<(Vec<String>, String)> {
    let parts: Vec<&str> = keybind.split('+').collect();

    if parts.is_empty() {
        anyhow::bail!("Empty keybinding string");
    }

    if parts.len() == 1 {
        // Just a key, no modifiers
        return Ok((Vec::new(), parts[0].to_string()));
    }

    // Last part is the key, rest are modifiers
    let key = parts[parts.len() - 1].to_string();
    let modifiers = parts[..parts.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok((modifiers, key))
}

/// Check if a key is a valid modifier.
pub fn is_valid_modifier(modifier: &str) -> bool {
    matches!(modifier, "Win" | "Ctrl" | "Alt" | "Shift")
}

/// Normalize a key string to uppercase for comparison.
pub fn normalize_key(key: &str) -> String {
    key.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_keybind() {
        let (mods, key) = parse_keybind_string("Q").unwrap();
        assert_eq!(mods.len(), 0);
        assert_eq!(key, "Q");
    }

    #[test]
    fn test_parse_keybind_with_one_modifier() {
        let (mods, key) = parse_keybind_string("Win+Q").unwrap();
        assert_eq!(mods, vec!["Win"]);
        assert_eq!(key, "Q");
    }

    #[test]
    fn test_parse_keybind_with_multiple_modifiers() {
        let (mods, key) = parse_keybind_string("Win+Shift+Q").unwrap();
        assert_eq!(mods, vec!["Win", "Shift"]);
        assert_eq!(key, "Q");
    }

    #[test]
    fn test_parse_keybind_with_all_modifiers() {
        let (mods, key) = parse_keybind_string("Win+Ctrl+Alt+Shift+Q").unwrap();
        assert_eq!(mods, vec!["Win", "Ctrl", "Alt", "Shift"]);
        assert_eq!(key, "Q");
    }

    #[test]
    fn test_parse_empty_keybind() {
        let result = parse_keybind_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_modifier() {
        assert!(is_valid_modifier("Win"));
        assert!(is_valid_modifier("Ctrl"));
        assert!(is_valid_modifier("Alt"));
        assert!(is_valid_modifier("Shift"));
        assert!(!is_valid_modifier("Super"));
        assert!(!is_valid_modifier("Meta"));
    }

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("q"), "Q");
        assert_eq!(normalize_key("Q"), "Q");
        assert_eq!(normalize_key("Left"), "LEFT");
        assert_eq!(normalize_key("f1"), "F1");
    }
}
