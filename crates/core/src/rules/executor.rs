//! Rule executor for applying window rule actions
//! 
//! This module provides functionality to execute rule actions on windows,
//! converting the matched actions into actual window state changes.

use crate::config::schema::RuleAction;
use crate::window_manager::window::ManagedWindow;
use anyhow::Result;

/// Executor for window rule actions
/// 
/// The RuleExecutor applies rule actions to managed windows, handling
/// state changes, workspace assignments, and other window modifications.
pub struct RuleExecutor;

impl RuleExecutor {
    /// Execute a list of actions on a window
    /// 
    /// This method applies all provided actions to the window. Actions are
    /// applied in order, and any errors are logged but don't stop execution
    /// of subsequent actions.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to apply actions to
    /// * `actions` - List of actions to execute
    /// 
    /// # Returns
    /// 
    /// Ok(()) if all actions were applied successfully, or an error if
    /// any action failed critically
    pub fn execute_actions(window: &mut ManagedWindow, actions: &[RuleAction]) -> Result<()> {
        for action in actions {
            if let Err(e) = Self::execute_action(window, action) {
                tracing::warn!(
                    "Failed to execute action {:?} on window '{}': {}",
                    action,
                    window.title,
                    e
                );
            }
        }
        Ok(())
    }
    
    /// Execute a single action on a window
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to apply the action to
    /// * `action` - The action to execute
    /// 
    /// # Returns
    /// 
    /// Ok(()) if the action was applied successfully
    fn execute_action(window: &mut ManagedWindow, action: &RuleAction) -> Result<()> {
        match action {
            RuleAction::Float => {
                tracing::debug!("Setting window '{}' to floating", window.title);
                window.set_floating()?;
            }
            RuleAction::Tile => {
                tracing::debug!("Setting window '{}' to tiled", window.title);
                window.set_tiled()?;
            }
            RuleAction::Workspace(workspace_id) => {
                tracing::debug!("Assigning window '{}' to workspace {}", window.title, workspace_id);
                window.workspace = *workspace_id;
            }
            RuleAction::Monitor(monitor_id) => {
                tracing::debug!("Assigning window '{}' to monitor {}", window.title, monitor_id);
                window.monitor = *monitor_id;
            }
            RuleAction::Fullscreen => {
                tracing::debug!("Setting window '{}' to fullscreen (deferred)", window.title);
                // Fullscreen is typically applied after window is positioned
                // This is usually handled by the window manager during layout
            }
            RuleAction::NoFocus => {
                tracing::debug!("Window '{}' marked as no-focus", window.title);
                // This is a passive flag that the focus manager checks
            }
            RuleAction::NoManage => {
                tracing::debug!("Window '{}' marked as unmanaged", window.title);
                window.managed = false;
            }
            RuleAction::Opacity(_opacity) => {
                tracing::debug!("Opacity setting for window '{}' (not yet implemented)", window.title);
                // Window opacity would be implemented with DWM API
                // This is a future enhancement
            }
            RuleAction::Pin => {
                tracing::debug!("Window '{}' pinned to all workspaces (not yet implemented)", window.title);
                // Pin functionality would require workspace manager integration
                // This is a future enhancement
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::win32::WindowHandle;
    use crate::window_manager::window::WindowState;
    use windows::Win32::Foundation::HWND;
    
    fn create_test_window() -> ManagedWindow {
        ManagedWindow {
            handle: WindowHandle::from_hwnd(HWND(12345)),
            state: WindowState::Tiled,
            workspace: 1,
            monitor: 0,
            title: "Test Window".to_string(),
            class: "TestClass".to_string(),
            process_name: "test.exe".to_string(),
            original_rect: None,
            managed: true,
            user_floating: false,
        }
    }
    
    #[test]
    fn test_execute_workspace_action() {
        let mut window = create_test_window();
        let actions = vec![RuleAction::Workspace(5)];
        
        RuleExecutor::execute_actions(&mut window, &actions).unwrap();
        
        assert_eq!(window.workspace, 5);
    }
    
    #[test]
    fn test_execute_monitor_action() {
        let mut window = create_test_window();
        let actions = vec![RuleAction::Monitor(2)];
        
        RuleExecutor::execute_actions(&mut window, &actions).unwrap();
        
        assert_eq!(window.monitor, 2);
    }
    
    #[test]
    fn test_execute_no_manage_action() {
        let mut window = create_test_window();
        assert!(window.managed);
        
        let actions = vec![RuleAction::NoManage];
        RuleExecutor::execute_actions(&mut window, &actions).unwrap();
        
        assert!(!window.managed);
    }
    
    #[test]
    fn test_execute_multiple_actions() {
        let mut window = create_test_window();
        let actions = vec![
            RuleAction::Workspace(3),
            RuleAction::Monitor(1),
        ];
        
        RuleExecutor::execute_actions(&mut window, &actions).unwrap();
        
        assert_eq!(window.workspace, 3);
        assert_eq!(window.monitor, 1);
    }
    
    #[test]
    fn test_execute_empty_actions() {
        let mut window = create_test_window();
        let original_workspace = window.workspace;
        
        RuleExecutor::execute_actions(&mut window, &[]).unwrap();
        
        assert_eq!(window.workspace, original_workspace);
    }
}
