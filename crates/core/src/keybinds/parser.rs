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