//! Rule matching engine for window rules
//! 
//! This module implements efficient pattern matching for window rules using
//! compiled regex patterns. It supports matching on process name, window title,
//! and window class name, with multiple simultaneous rule matches.

use crate::config::schema::{WindowRule, RuleAction};
use crate::window_manager::window::ManagedWindow;
use anyhow::Context;
use regex::Regex;
use std::sync::Arc;

/// Compiled window rule for efficient matching
/// 
/// This structure holds pre-compiled regex patterns for fast matching
/// against window properties. All regex patterns are compiled once during
/// initialization to optimize matching performance.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    /// Compiled regex for process name matching
    pub process_regex: Option<Regex>,
    
    /// Compiled regex for window title matching
    pub title_regex: Option<Regex>,
    
    /// Compiled regex for window class matching
    pub class_regex: Option<Regex>,
    
    /// Actions to apply when rule matches
    pub actions: Vec<RuleAction>,
}

/// Rule matcher that efficiently matches windows against rules
/// 
/// The RuleMatcher compiles window rules into efficient regex patterns
/// and provides methods to match windows and extract relevant actions.
/// It supports multiple rule matches per window, allowing complex
/// rule combinations.
pub struct RuleMatcher {
    /// List of compiled rules
    rules: Vec<Arc<CompiledRule>>,
}

/// Summary of all rule actions for a window
/// 
/// This struct provides a consolidated view of all matching rule actions,
/// allowing efficient access to all rule properties without repeated matching.
#[derive(Debug, Clone, Default)]
pub struct RuleMatch {
    /// All actions from matching rules
    pub actions: Vec<RuleAction>,
    /// Whether window should be managed (false if NoManage present)
    pub should_manage: bool,
    /// Initial workspace (if specified)
    pub workspace: Option<usize>,
    /// Initial monitor (if specified)
    pub monitor: Option<usize>,
    /// Whether window should float
    pub should_float: bool,
    /// Whether window should start fullscreen
    pub should_fullscreen: bool,
    /// Whether window should be pinned
    pub should_pin: bool,
    /// Whether window should not be focused
    pub should_not_focus: bool,
    /// Opacity setting (if specified)
    pub opacity: Option<f32>,
}

impl RuleMatcher {
    /// Create a new rule matcher from window rules
    /// 
    /// This function compiles all regex patterns in the provided rules.
    /// If any regex pattern is invalid, an error is returned.
    /// 
    /// # Arguments
    /// 
    /// * `rules` - Vector of window rules from configuration
    /// 
    /// # Returns
    /// 
    /// A new RuleMatcher instance with compiled rules, or an error if
    /// any regex pattern is invalid.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use tiling_wm_core::rules::RuleMatcher;
    /// use tiling_wm_core::config::schema::{WindowRule, RuleAction};
    /// 
    /// let rules = vec![
    ///     WindowRule {
    ///         match_process: Some("firefox\\.exe".to_string()),
    ///         match_title: None,
    ///         match_class: None,
    ///         actions: vec![RuleAction::Workspace(2)],
    ///     },
    /// ];
    /// 
    /// let matcher = RuleMatcher::new(rules).unwrap();
    /// ```
    pub fn new(rules: Vec<WindowRule>) -> anyhow::Result<Self> {
        let mut compiled_rules = Vec::new();
        
        for (i, rule) in rules.into_iter().enumerate() {
            tracing::debug!("Compiling rule {}", i);
            
            let process_regex = if let Some(pattern) = rule.match_process {
                Some(Regex::new(&pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_process", i))?)
            } else {
                None
            };
            
            let title_regex = if let Some(pattern) = rule.match_title {
                Some(Regex::new(&pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_title", i))?)
            } else {
                None
            };
            
            let class_regex = if let Some(pattern) = rule.match_class {
                Some(Regex::new(&pattern)
                    .with_context(|| format!("Invalid regex in rule {} match_class", i))?)
            } else {
                None
            };
            
            compiled_rules.push(Arc::new(CompiledRule {
                process_regex,
                title_regex,
                class_regex,
                actions: rule.actions,
            }));
        }
        
        tracing::info!("Compiled {} window rules", compiled_rules.len());
        
        Ok(Self {
            rules: compiled_rules,
        })
    }
    
    /// Match a window against all rules and return matching actions
    /// 
    /// This method checks the window against all rules and returns a vector
    /// of all actions from matching rules. Multiple rules can match the same
    /// window, and all their actions will be returned.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The managed window to match against rules
    /// 
    /// # Returns
    /// 
    /// A vector of all actions from rules that matched the window
    pub fn match_window(&self, window: &ManagedWindow) -> Vec<RuleAction> {
        let mut actions = Vec::new();
        
        for rule in &self.rules {
            if self.rule_matches(rule, window) {
                tracing::debug!(
                    "Rule matched for window '{}' (process: {})",
                    window.title,
                    window.process_name
                );
                actions.extend(rule.actions.clone());
            }
        }
        
        actions
    }
    
    /// Check if a rule matches a window
    /// 
    /// A rule matches if all specified conditions match. If a rule has
    /// multiple conditions (e.g., both process and title), ALL conditions
    /// must match for the rule to apply (AND logic).
    /// 
    /// # Arguments
    /// 
    /// * `rule` - The compiled rule to check
    /// * `window` - The window to match against
    /// 
    /// # Returns
    /// 
    /// true if all specified conditions in the rule match the window
    fn rule_matches(&self, rule: &CompiledRule, window: &ManagedWindow) -> bool {
        // Check process name
        if let Some(ref regex) = rule.process_regex {
            if !regex.is_match(&window.process_name) {
                return false;
            }
        }
        
        // Check window title
        if let Some(ref regex) = rule.title_regex {
            if !regex.is_match(&window.title) {
                return false;
            }
        }
        
        // Check window class
        if let Some(ref regex) = rule.class_regex {
            if !regex.is_match(&window.class) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a window should be managed based on rules
    /// 
    /// A window should not be managed if any matching rule has the
    /// NoManage action.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// false if any matching rule has NoManage action, true otherwise
    pub fn should_manage(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        !actions.iter().any(|a| matches!(a, RuleAction::NoManage))
    }
    
    /// Get initial workspace for a window based on rules
    /// 
    /// Returns the workspace ID from the first matching Workspace action.
    /// If multiple rules specify different workspaces, the first one wins.
    /// 
    /// # Performance Note
    /// 
    /// This method calls `match_window()` internally which processes all rules.
    /// If you need multiple rule properties (workspace, monitor, etc.), consider
    /// calling `match_window()` once and processing the actions yourself to avoid
    /// redundant rule matching.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// Some(workspace_id) if a Workspace action matched, None otherwise
    pub fn get_initial_workspace(&self, window: &ManagedWindow) -> Option<usize> {
        // Note: This could be optimized by caching match results or stopping early,
        // but current implementation prioritizes simplicity and correctness.
        let actions = self.match_window(window);
        
        for action in actions {
            if let RuleAction::Workspace(id) = action {
                return Some(id);
            }
        }
        
        None
    }
    
    /// Check if a window should start as floating based on rules
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// true if any matching rule has Float action
    pub fn should_float(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        actions.iter().any(|a| matches!(a, RuleAction::Float))
    }
    
    /// Check if a window should start in fullscreen based on rules
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// true if any matching rule has Fullscreen action
    pub fn should_fullscreen(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        actions.iter().any(|a| matches!(a, RuleAction::Fullscreen))
    }
    
    /// Check if a window should be pinned (visible on all workspaces) based on rules
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// true if any matching rule has Pin action
    pub fn should_pin(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        actions.iter().any(|a| matches!(a, RuleAction::Pin))
    }
    
    /// Get opacity setting for a window based on rules
    /// 
    /// Returns the opacity from the first matching Opacity action.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// Some(opacity) if an Opacity action matched, None otherwise
    pub fn get_opacity(&self, window: &ManagedWindow) -> Option<f32> {
        let actions = self.match_window(window);
        
        for action in actions {
            if let RuleAction::Opacity(opacity) = action {
                return Some(opacity);
            }
        }
        
        None
    }
    
    /// Get monitor assignment for a window based on rules
    /// 
    /// Returns the monitor ID from the first matching Monitor action.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// Some(monitor_id) if a Monitor action matched, None otherwise
    pub fn get_initial_monitor(&self, window: &ManagedWindow) -> Option<usize> {
        let actions = self.match_window(window);
        
        for action in actions {
            if let RuleAction::Monitor(id) = action {
                return Some(id);
            }
        }
        
        None
    }
    
    /// Check if a window should not be focused automatically based on rules
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to check
    /// 
    /// # Returns
    /// 
    /// true if any matching rule has NoFocus action
    pub fn should_not_focus(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        actions.iter().any(|a| matches!(a, RuleAction::NoFocus))
    }
    
    /// Get the number of rules
    /// 
    /// # Returns
    /// 
    /// The total number of compiled rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
    
    /// Get a consolidated summary of all matching rules for a window
    /// 
    /// This method performs rule matching once and extracts all relevant
    /// information into a RuleMatch struct. This is more efficient than
    /// calling individual helper methods when you need multiple properties.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to match against rules
    /// 
    /// # Returns
    /// 
    /// A RuleMatch struct containing all matching actions and derived properties
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use tiling_wm_core::rules::RuleMatcher;
    /// # use tiling_wm_core::window_manager::ManagedWindow;
    /// # fn example(matcher: &RuleMatcher, window: &ManagedWindow) {
    /// let rule_match = matcher.match_all(window);
    /// 
    /// if !rule_match.should_manage {
    ///     return; // Window excluded by NoManage
    /// }
    /// 
    /// if let Some(workspace_id) = rule_match.workspace {
    ///     // Assign to workspace
    /// }
    /// 
    /// if rule_match.should_float {
    ///     // Make window floating
    /// }
    /// # }
    /// ```
    pub fn match_all(&self, window: &ManagedWindow) -> RuleMatch {
        let actions = self.match_window(window);
        
        let mut result = RuleMatch {
            actions: actions.clone(),
            should_manage: true,
            workspace: None,
            monitor: None,
            should_float: false,
            should_fullscreen: false,
            should_pin: false,
            should_not_focus: false,
            opacity: None,
        };
        
        // Process actions to extract relevant properties
        for action in &actions {
            match action {
                RuleAction::NoManage => result.should_manage = false,
                RuleAction::Workspace(id) if result.workspace.is_none() => {
                    result.workspace = Some(*id);
                }
                RuleAction::Monitor(id) if result.monitor.is_none() => {
                    result.monitor = Some(*id);
                }
                RuleAction::Float => result.should_float = true,
                RuleAction::Fullscreen => result.should_fullscreen = true,
                RuleAction::Pin => result.should_pin = true,
                RuleAction::NoFocus => result.should_not_focus = true,
                RuleAction::Opacity(opacity) if result.opacity.is_none() => {
                    result.opacity = Some(*opacity);
                }
                _ => {} // Ignore Tile and already-set values
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::WindowRule;
    use crate::utils::win32::WindowHandle;
    use crate::window_manager::window::{ManagedWindow, WindowState};
    use windows::Win32::Foundation::HWND;
    
    // Helper function to create a test window
    fn create_test_window(process: &str, title: &str, class: &str) -> ManagedWindow {
        ManagedWindow {
            handle: WindowHandle::from_hwnd(HWND(12345)),
            state: WindowState::Tiled,
            workspace: 1,
            monitor: 0,
            title: title.to_string(),
            class: class.to_string(),
            process_name: process.to_string(),
            original_rect: None,
            managed: true,
            user_floating: false,
        }
    }
    
    #[test]
    fn test_rule_matcher_creation() {
        let rules = vec![
            WindowRule {
                match_process: Some("firefox\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(2)],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        assert_eq!(matcher.rule_count(), 1);
    }
    
    #[test]
    fn test_process_matching() {
        let rules = vec![
            WindowRule {
                match_process: Some("firefox\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("firefox.exe", "Mozilla Firefox", "MozillaWindowClass");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], RuleAction::Float));
    }
    
    #[test]
    fn test_title_regex_matching() {
        let rules = vec![
            WindowRule {
                match_process: None,
                match_title: Some(".*Steam.*".to_string()),
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("steam.exe", "Steam - News", "vguiPopupWindow");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 1);
    }
    
    #[test]
    fn test_class_matching() {
        let rules = vec![
            WindowRule {
                match_process: None,
                match_title: None,
                match_class: Some(".*Popup.*".to_string()),
                actions: vec![RuleAction::NoManage],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("app.exe", "Popup Window", "PopupClass");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], RuleAction::NoManage));
    }
    
    #[test]
    fn test_multiple_conditions_and_logic() {
        let rules = vec![
            WindowRule {
                match_process: Some("notepad\\.exe".to_string()),
                match_title: Some(".*Untitled.*".to_string()),
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        // Both conditions match
        let window1 = create_test_window("notepad.exe", "Untitled - Notepad", "Notepad");
        assert_eq!(matcher.match_window(&window1).len(), 1);
        
        // Only process matches
        let window2 = create_test_window("notepad.exe", "document.txt - Notepad", "Notepad");
        assert_eq!(matcher.match_window(&window2).len(), 0);
        
        // Only title matches
        let window3 = create_test_window("code.exe", "Untitled-1", "Code");
        assert_eq!(matcher.match_window(&window3).len(), 0);
    }
    
    #[test]
    fn test_multiple_rules_same_window() {
        let rules = vec![
            WindowRule {
                match_process: Some("chrome\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(2)],
            },
            WindowRule {
                match_process: Some("chrome\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("chrome.exe", "Google Chrome", "Chrome_WidgetWin_1");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 2);
    }
    
    #[test]
    fn test_should_manage() {
        let rules = vec![
            WindowRule {
                match_process: Some("popup\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::NoManage],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window1 = create_test_window("popup.exe", "Popup", "PopupClass");
        assert!(!matcher.should_manage(&window1));
        
        let window2 = create_test_window("normal.exe", "Normal", "NormalClass");
        assert!(matcher.should_manage(&window2));
    }
    
    #[test]
    fn test_get_initial_workspace() {
        let rules = vec![
            WindowRule {
                match_process: Some("code\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(3)],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("code.exe", "VS Code", "Code");
        assert_eq!(matcher.get_initial_workspace(&window), Some(3));
        
        let window2 = create_test_window("other.exe", "Other", "Other");
        assert_eq!(matcher.get_initial_workspace(&window2), None);
    }
    
    #[test]
    fn test_should_float() {
        let rules = vec![
            WindowRule {
                match_process: Some("calc\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("calc.exe", "Calculator", "CalcFrame");
        assert!(matcher.should_float(&window));
        
        let window2 = create_test_window("other.exe", "Other", "Other");
        assert!(!matcher.should_float(&window2));
    }
    
    #[test]
    fn test_should_fullscreen() {
        let rules = vec![
            WindowRule {
                match_process: Some("game\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Fullscreen],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("game.exe", "Game Window", "GameClass");
        assert!(matcher.should_fullscreen(&window));
    }
    
    #[test]
    fn test_get_opacity() {
        let rules = vec![
            WindowRule {
                match_process: Some("discord\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Opacity(0.95)],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("discord.exe", "Discord", "Discord");
        assert_eq!(matcher.get_opacity(&window), Some(0.95));
    }
    
    #[test]
    fn test_should_pin() {
        let rules = vec![
            WindowRule {
                match_process: None,
                match_title: Some("Task Manager".to_string()),
                match_class: None,
                actions: vec![RuleAction::Pin],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("taskmgr.exe", "Task Manager", "TaskManager");
        assert!(matcher.should_pin(&window));
    }
    
    #[test]
    fn test_should_not_focus() {
        let rules = vec![
            WindowRule {
                match_process: Some("background\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::NoFocus],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("background.exe", "Background", "BG");
        assert!(matcher.should_not_focus(&window));
    }
    
    #[test]
    fn test_invalid_regex_error() {
        let rules = vec![
            WindowRule {
                match_process: Some("[invalid".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Float],
            },
        ];
        
        let result = RuleMatcher::new(rules);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("regex") || err_msg.contains("Invalid"));
    }
    
    #[test]
    fn test_empty_rules() {
        let matcher = RuleMatcher::new(vec![]).unwrap();
        assert_eq!(matcher.rule_count(), 0);
        
        let window = create_test_window("any.exe", "Any", "Any");
        assert_eq!(matcher.match_window(&window).len(), 0);
        assert!(matcher.should_manage(&window));
    }
    
    #[test]
    fn test_multiple_actions_in_single_rule() {
        let rules = vec![
            WindowRule {
                match_process: Some("app\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![
                    RuleAction::Float,
                    RuleAction::Workspace(2),
                    RuleAction::Opacity(0.9),
                ],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        
        let window = create_test_window("app.exe", "App", "AppClass");
        let actions = matcher.match_window(&window);
        
        assert_eq!(actions.len(), 3);
        assert!(matcher.should_float(&window));
        assert_eq!(matcher.get_initial_workspace(&window), Some(2));
        assert_eq!(matcher.get_opacity(&window), Some(0.9));
    }
    
    #[test]
    fn test_match_all_efficiency() {
        let rules = vec![
            WindowRule {
                match_process: Some("test\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![
                    RuleAction::Float,
                    RuleAction::Workspace(3),
                    RuleAction::Monitor(1),
                    RuleAction::Opacity(0.85),
                    RuleAction::NoFocus,
                ],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        let window = create_test_window("test.exe", "Test", "TestClass");
        
        // Use match_all to get all properties at once
        let rule_match = matcher.match_all(&window);
        
        // Verify all properties are extracted correctly
        assert!(rule_match.should_manage);
        assert!(rule_match.should_float);
        assert_eq!(rule_match.workspace, Some(3));
        assert_eq!(rule_match.monitor, Some(1));
        assert_eq!(rule_match.opacity, Some(0.85));
        assert!(rule_match.should_not_focus);
        assert!(!rule_match.should_fullscreen);
        assert!(!rule_match.should_pin);
        assert_eq!(rule_match.actions.len(), 5);
    }
    
    #[test]
    fn test_match_all_no_manage() {
        let rules = vec![
            WindowRule {
                match_process: Some("blocked\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::NoManage],
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        let window = create_test_window("blocked.exe", "Blocked", "BlockedClass");
        
        let rule_match = matcher.match_all(&window);
        
        assert!(!rule_match.should_manage);
    }
    
    #[test]
    fn test_match_all_first_wins_for_workspace() {
        let rules = vec![
            WindowRule {
                match_process: Some("app\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(2)],
            },
            WindowRule {
                match_process: Some("app\\.exe".to_string()),
                match_title: None,
                match_class: None,
                actions: vec![RuleAction::Workspace(3)],  // This should be ignored
            },
        ];
        
        let matcher = RuleMatcher::new(rules).unwrap();
        let window = create_test_window("app.exe", "App", "AppClass");
        
        let rule_match = matcher.match_all(&window);
        
        // First workspace assignment should win
        assert_eq!(rule_match.workspace, Some(2));
    }
}
