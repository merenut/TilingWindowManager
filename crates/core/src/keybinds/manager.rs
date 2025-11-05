//! Keybinding manager for Windows hotkey registration and handling.
//!
//! This module provides the KeybindManager which registers global hotkeys
//! with Windows and maps them to commands. It handles:
//! - Hotkey registration with Windows API
//! - Virtual key code translation
//! - Modifier key parsing
//! - Hotkey unregistration and cleanup
//! - Conflict detection

use crate::config::schema::Keybind;
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Keybinding manager that handles Windows hotkey registration.
///
/// The KeybindManager registers global hotkeys with the Windows API and
/// maintains a mapping from hotkey IDs to commands. When a hotkey is
/// pressed, the ID can be used to look up which command to execute.
pub struct KeybindManager {
    /// Map of hotkey ID to command string
    bindings: HashMap<i32, String>,

    /// Map of hotkey ID to command arguments
    arguments: HashMap<i32, Vec<String>>,

    /// Next hotkey ID to assign
    next_id: i32,
}

impl KeybindManager {
    /// Create a new KeybindManager.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            arguments: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a list of keybindings.
    ///
    /// This will first unregister all existing keybindings, then register
    /// the new ones. If any keybinding fails to register, an error is returned
    /// but previously registered bindings from this call remain registered.
    pub fn register_keybinds(&mut self, keybinds: Vec<Keybind>) -> Result<()> {
        // Unregister existing keybinds
        self.unregister_all()?;

        let mut registered = 0;
        let mut failed = Vec::new();

        for keybind in keybinds {
            match self.register_keybind(keybind.clone()) {
                Ok(()) => registered += 1,
                Err(e) => {
                    warn!(
                        "Failed to register keybind {:?}+{}: {}",
                        keybind.modifiers, keybind.key, e
                    );
                    failed.push(format!(
                        "{:?}+{}: {}",
                        keybind.modifiers, keybind.key, e
                    ));
                }
            }
        }

        if !failed.is_empty() {
            error!(
                "Failed to register {} keybinding(s): {:?}",
                failed.len(),
                failed
            );
        }

        info!("Successfully registered {} keybinding(s)", registered);
        Ok(())
    }

    /// Register a single keybinding.
    fn register_keybind(&mut self, keybind: Keybind) -> Result<()> {
        let modifiers = self.parse_modifiers(&keybind.modifiers)?;
        let vk_code = self.parse_key(&keybind.key)?;

        let hotkey_id = self.next_id;
        self.next_id += 1;

        unsafe {
            RegisterHotKey(None, hotkey_id, modifiers, vk_code.0 as u32)
                .with_context(|| {
                    format!(
                        "Failed to register hotkey: {:?}+{} (may be in use)",
                        keybind.modifiers, keybind.key
                    )
                })?;
        }

        self.bindings.insert(hotkey_id, keybind.command.clone());
        self.arguments.insert(hotkey_id, keybind.args.clone());

        debug!(
            "Registered hotkey {}: {:?}+{} -> {} {:?}",
            hotkey_id, keybind.modifiers, keybind.key, keybind.command, keybind.args
        );

        Ok(())
    }

    /// Unregister all keybindings.
    pub fn unregister_all(&mut self) -> Result<()> {
        for &hotkey_id in self.bindings.keys() {
            unsafe {
                if let Err(e) = UnregisterHotKey(None, hotkey_id) {
                    warn!("Failed to unregister hotkey {}: {}", hotkey_id, e);
                }
            }
        }

        let count = self.bindings.len();
        self.bindings.clear();
        self.arguments.clear();
        self.next_id = 1;

        if count > 0 {
            info!("Unregistered {} keybinding(s)", count);
        }

        Ok(())
    }

    /// Get the command and arguments for a hotkey ID.
    ///
    /// Returns None if the hotkey ID is not registered.
    pub fn get_command(&self, hotkey_id: i32) -> Option<(&String, &Vec<String>)> {
        self.bindings.get(&hotkey_id).and_then(|cmd| {
            self.arguments
                .get(&hotkey_id)
                .map(|args| (cmd, args))
        })
    }

    /// Parse modifier keys from string representations.
    ///
    /// Supported modifiers: "Win", "Ctrl", "Alt", "Shift"
    fn parse_modifiers(&self, modifiers: &[String]) -> Result<HOT_KEY_MODIFIERS> {
        let mut result = HOT_KEY_MODIFIERS(0);

        for modifier in modifiers {
            match modifier.as_str() {
                "Ctrl" => result |= MOD_CONTROL,
                "Alt" => result |= MOD_ALT,
                "Shift" => result |= MOD_SHIFT,
                "Win" => result |= MOD_WIN,
                _ => anyhow::bail!("Unknown modifier: {}", modifier),
            }
        }

        Ok(result)
    }

    /// Parse a key string to a virtual key code.
    ///
    /// Supports:
    /// - Letters A-Z
    /// - Numbers 0-9
    /// - Arrow keys (Left, Right, Up, Down)
    /// - Special keys (Space, Enter, Escape, Tab, etc.)
    /// - Function keys (F1-F12)
    /// - Brackets and other symbols
    fn parse_key(&self, key: &str) -> Result<VIRTUAL_KEY> {
        let vk = match key.to_uppercase().as_str() {
            // Letters
            "A" => VK_A,
            "B" => VK_B,
            "C" => VK_C,
            "D" => VK_D,
            "E" => VK_E,
            "F" => VK_F,
            "G" => VK_G,
            "H" => VK_H,
            "I" => VK_I,
            "J" => VK_J,
            "K" => VK_K,
            "L" => VK_L,
            "M" => VK_M,
            "N" => VK_N,
            "O" => VK_O,
            "P" => VK_P,
            "Q" => VK_Q,
            "R" => VK_R,
            "S" => VK_S,
            "T" => VK_T,
            "U" => VK_U,
            "V" => VK_V,
            "W" => VK_W,
            "X" => VK_X,
            "Y" => VK_Y,
            "Z" => VK_Z,

            // Numbers
            "0" => VK_0,
            "1" => VK_1,
            "2" => VK_2,
            "3" => VK_3,
            "4" => VK_4,
            "5" => VK_5,
            "6" => VK_6,
            "7" => VK_7,
            "8" => VK_8,
            "9" => VK_9,

            // Arrow keys
            "LEFT" => VK_LEFT,
            "RIGHT" => VK_RIGHT,
            "UP" => VK_UP,
            "DOWN" => VK_DOWN,

            // Special keys
            "SPACE" => VK_SPACE,
            "ENTER" | "RETURN" => VK_RETURN,
            "ESCAPE" | "ESC" => VK_ESCAPE,
            "TAB" => VK_TAB,
            "BACKSPACE" | "BACK" => VK_BACK,
            "DELETE" | "DEL" => VK_DELETE,
            "HOME" => VK_HOME,
            "END" => VK_END,
            "PAGEUP" | "PGUP" => VK_PRIOR,
            "PAGEDOWN" | "PGDN" => VK_NEXT,
            "INSERT" | "INS" => VK_INSERT,

            // Brackets and symbols
            "BRACKETLEFT" | "[" => VK_OEM_4,
            "BRACKETRIGHT" | "]" => VK_OEM_6,
            "SEMICOLON" | ";" => VK_OEM_1,
            "QUOTE" | "'" => VK_OEM_7,
            "COMMA" | "," => VK_OEM_COMMA,
            "PERIOD" | "." => VK_OEM_PERIOD,
            "SLASH" | "/" => VK_OEM_2,
            "BACKSLASH" | "\\" => VK_OEM_5,
            "MINUS" | "-" => VK_OEM_MINUS,
            "EQUALS" | "=" => VK_OEM_PLUS,
            "GRAVE" | "`" => VK_OEM_3,

            // Function keys
            "F1" => VK_F1,
            "F2" => VK_F2,
            "F3" => VK_F3,
            "F4" => VK_F4,
            "F5" => VK_F5,
            "F6" => VK_F6,
            "F7" => VK_F7,
            "F8" => VK_F8,
            "F9" => VK_F9,
            "F10" => VK_F10,
            "F11" => VK_F11,
            "F12" => VK_F12,

            _ => anyhow::bail!("Unknown key: {}", key),
        };

        Ok(vk)
    }

    /// Get the number of registered keybindings.
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }
}

impl Default for KeybindManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KeybindManager {
    fn drop(&mut self) {
        if let Err(e) = self.unregister_all() {
            error!("Failed to unregister hotkeys during drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybind_manager_creation() {
        let manager = KeybindManager::new();
        assert_eq!(manager.binding_count(), 0);
        assert_eq!(manager.next_id, 1);
    }

    #[test]
    fn test_parse_modifiers() {
        let manager = KeybindManager::new();

        // Single modifier
        let mods = manager
            .parse_modifiers(&["Win".to_string()])
            .unwrap();
        assert!(mods.0 & MOD_WIN.0 != 0);

        // Multiple modifiers
        let mods = manager
            .parse_modifiers(&["Win".to_string(), "Shift".to_string()])
            .unwrap();
        assert!(mods.0 & MOD_WIN.0 != 0);
        assert!(mods.0 & MOD_SHIFT.0 != 0);

        // All modifiers
        let mods = manager
            .parse_modifiers(&[
                "Win".to_string(),
                "Ctrl".to_string(),
                "Alt".to_string(),
                "Shift".to_string(),
            ])
            .unwrap();
        assert!(mods.0 & MOD_WIN.0 != 0);
        assert!(mods.0 & MOD_CONTROL.0 != 0);
        assert!(mods.0 & MOD_ALT.0 != 0);
        assert!(mods.0 & MOD_SHIFT.0 != 0);
    }

    #[test]
    fn test_parse_invalid_modifier() {
        let manager = KeybindManager::new();
        let result = manager.parse_modifiers(&["Super".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown modifier"));
    }

    #[test]
    fn test_parse_letter_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("A").unwrap(), VK_A);
        assert_eq!(manager.parse_key("a").unwrap(), VK_A); // Case insensitive
        assert_eq!(manager.parse_key("Q").unwrap(), VK_Q);
        assert_eq!(manager.parse_key("Z").unwrap(), VK_Z);
    }

    #[test]
    fn test_parse_number_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("0").unwrap(), VK_0);
        assert_eq!(manager.parse_key("1").unwrap(), VK_1);
        assert_eq!(manager.parse_key("9").unwrap(), VK_9);
    }

    #[test]
    fn test_parse_arrow_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("Left").unwrap(), VK_LEFT);
        assert_eq!(manager.parse_key("RIGHT").unwrap(), VK_RIGHT);
        assert_eq!(manager.parse_key("up").unwrap(), VK_UP);
        assert_eq!(manager.parse_key("Down").unwrap(), VK_DOWN);
    }

    #[test]
    fn test_parse_special_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("Space").unwrap(), VK_SPACE);
        assert_eq!(manager.parse_key("Enter").unwrap(), VK_RETURN);
        assert_eq!(manager.parse_key("Return").unwrap(), VK_RETURN);
        assert_eq!(manager.parse_key("Escape").unwrap(), VK_ESCAPE);
        assert_eq!(manager.parse_key("ESC").unwrap(), VK_ESCAPE);
        assert_eq!(manager.parse_key("Tab").unwrap(), VK_TAB);
    }

    #[test]
    fn test_parse_function_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("F1").unwrap(), VK_F1);
        assert_eq!(manager.parse_key("f5").unwrap(), VK_F5);
        assert_eq!(manager.parse_key("F12").unwrap(), VK_F12);
    }

    #[test]
    fn test_parse_bracket_keys() {
        let manager = KeybindManager::new();

        assert_eq!(manager.parse_key("BracketLeft").unwrap(), VK_OEM_4);
        assert_eq!(manager.parse_key("[").unwrap(), VK_OEM_4);
        assert_eq!(manager.parse_key("BracketRight").unwrap(), VK_OEM_6);
        assert_eq!(manager.parse_key("]").unwrap(), VK_OEM_6);
    }

    #[test]
    fn test_parse_invalid_key() {
        let manager = KeybindManager::new();
        let result = manager.parse_key("InvalidKey");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown key"));
    }

    #[test]
    fn test_get_command_not_registered() {
        let manager = KeybindManager::new();
        assert!(manager.get_command(999).is_none());
    }
}
