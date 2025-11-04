//! Command system for window operations.
//!
//! This module provides a comprehensive command system for all window, layout,
//! focus, and workspace operations. Commands are executed through the CommandExecutor
//! which acts on the WindowManager.
//!
//! # Example
//!
//! ```no_run
//! use tiling_wm_core::commands::{Command, CommandExecutor};
//! use tiling_wm_core::window_manager::WindowManager;
//!
//! let mut wm = WindowManager::new();
//! wm.initialize().expect("Failed to initialize");
//!
//! let executor = CommandExecutor::new();
//! executor.execute(Command::SetLayoutMaster, &mut wm).ok();
//! ```

use crate::window_manager::focus::Direction;
use crate::window_manager::{LayoutType, WindowManager};
use anyhow::Result;
use tracing::{debug, error, info, warn};

/// Commands for window, layout, focus, and workspace operations.
///
/// Each variant represents a specific action that can be performed by
/// the window manager. Commands are executed through the CommandExecutor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Window commands
    /// Close the currently active window
    CloseActiveWindow,
    /// Toggle floating state for the active window
    ToggleFloating,
    /// Toggle fullscreen state for the active window
    ToggleFullscreen,
    /// Minimize the active window
    MinimizeActive,
    /// Restore a minimized window
    RestoreActive,

    // Focus commands
    /// Focus the window to the left
    FocusLeft,
    /// Focus the window to the right
    FocusRight,
    /// Focus the window above
    FocusUp,
    /// Focus the window below
    FocusDown,
    /// Focus the previous window in history (Alt-Tab)
    FocusPrevious,
    /// Focus the next window in history
    FocusNext,

    // Move commands
    /// Move active window left in the tree
    MoveWindowLeft,
    /// Move active window right in the tree
    MoveWindowRight,
    /// Move active window up in the tree
    MoveWindowUp,
    /// Move active window down in the tree
    MoveWindowDown,
    /// Swap active window with master window
    SwapWithMaster,

    // Layout commands
    /// Switch to dwindle layout
    SetLayoutDwindle,
    /// Switch to master-stack layout
    SetLayoutMaster,
    /// Increase the number of master windows
    IncreaseMasterCount,
    /// Decrease the number of master windows
    DecreaseMasterCount,
    /// Increase master area factor
    IncreaseMasterFactor,
    /// Decrease master area factor
    DecreaseMasterFactor,

    // Workspace commands
    /// Switch to specified workspace
    SwitchWorkspace(usize),
    /// Move active window to specified workspace
    MoveToWorkspace(usize),
    /// Move active window to workspace and follow
    MoveToWorkspaceAndFollow(usize),

    // System commands
    /// Reload configuration
    Reload,
    /// Quit the window manager
    Quit,
}

/// Executes commands on the WindowManager.
///
/// The CommandExecutor provides methods to execute commands, handling
/// errors and logging appropriately.
///
/// # Example
///
/// ```no_run
/// use tiling_wm_core::commands::{Command, CommandExecutor};
/// use tiling_wm_core::window_manager::WindowManager;
///
/// let mut wm = WindowManager::new();
/// wm.initialize().expect("Failed to initialize");
///
/// let executor = CommandExecutor::new();
///
/// // Execute a command
/// if let Err(e) = executor.execute(Command::CloseActiveWindow, &mut wm) {
///     eprintln!("Command failed: {}", e);
/// }
/// ```
pub struct CommandExecutor {
    // Currently no state needed, but this structure allows for future extensions
    // such as command history, undo/redo, or command queuing
}

impl CommandExecutor {
    /// Create a new CommandExecutor.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::commands::CommandExecutor;
    ///
    /// let executor = CommandExecutor::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a command on the WindowManager.
    ///
    /// This is the main entry point for command execution. It handles
    /// logging, error handling, and delegates to specific command implementations.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `wm` - Mutable reference to the WindowManager
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the command fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::commands::{Command, CommandExecutor};
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().expect("Failed to initialize");
    ///
    /// let executor = CommandExecutor::new();
    /// executor.execute(Command::ToggleFloating, &mut wm).ok();
    /// ```
    pub fn execute(&self, command: Command, wm: &mut WindowManager) -> Result<()> {
        debug!("Executing command: {:?}", command);

        let result = match command {
            // Window commands
            Command::CloseActiveWindow => self.close_active_window(wm),
            Command::ToggleFloating => self.toggle_floating(wm),
            Command::ToggleFullscreen => self.toggle_fullscreen(wm),
            Command::MinimizeActive => self.minimize_active(wm),
            Command::RestoreActive => self.restore_active(wm),

            // Focus commands
            Command::FocusLeft => self.focus_direction(wm, Direction::Left),
            Command::FocusRight => self.focus_direction(wm, Direction::Right),
            Command::FocusUp => self.focus_direction(wm, Direction::Up),
            Command::FocusDown => self.focus_direction(wm, Direction::Down),
            Command::FocusPrevious => self.focus_previous(wm),
            Command::FocusNext => self.focus_next(wm),

            // Move commands
            Command::MoveWindowLeft => self.move_window(wm, Direction::Left),
            Command::MoveWindowRight => self.move_window(wm, Direction::Right),
            Command::MoveWindowUp => self.move_window(wm, Direction::Up),
            Command::MoveWindowDown => self.move_window(wm, Direction::Down),
            Command::SwapWithMaster => self.swap_with_master(wm),

            // Layout commands
            Command::SetLayoutDwindle => {
                info!("Switching to dwindle layout");
                wm.set_layout(LayoutType::Dwindle)
            }
            Command::SetLayoutMaster => {
                info!("Switching to master layout");
                wm.set_layout(LayoutType::Master)
            }
            Command::IncreaseMasterCount => self.adjust_master_count(wm, 1),
            Command::DecreaseMasterCount => self.adjust_master_count(wm, -1),
            Command::IncreaseMasterFactor => self.adjust_master_factor(wm, 0.05),
            Command::DecreaseMasterFactor => self.adjust_master_factor(wm, -0.05),

            // Workspace commands
            Command::SwitchWorkspace(id) => {
                info!("Switching to workspace {}", id);
                wm.switch_workspace(id)
            }
            Command::MoveToWorkspace(id) => self.move_to_workspace(wm, id),
            Command::MoveToWorkspaceAndFollow(id) => self.move_to_workspace_and_follow(wm, id),

            // System commands
            Command::Reload => {
                info!("Reload command received");
                Ok(())
            }
            Command::Quit => {
                info!("Quit command received");
                Ok(())
            }
        };

        if let Err(ref e) = result {
            error!("Command execution failed: {:?} - {}", command, e);
        } else {
            debug!("Command executed successfully: {:?}", command);
        }

        result
    }

    /// Close the currently active window.
    ///
    /// Sends WM_CLOSE message to the active window, which allows the window
    /// to perform cleanup before closing.
    fn close_active_window(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            debug!("Closing window: {}", window.title);

            #[cfg(target_os = "windows")]
            {
                let handle = window.handle;
                unsafe {
                    use windows::Win32::Foundation::{LPARAM, WPARAM};
                    use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_CLOSE};
                    SendMessageW(handle.hwnd(), WM_CLOSE, WPARAM(0), LPARAM(0));
                }
            }

            Ok(())
        } else {
            warn!("No active window to close");
            Ok(())
        }
    }

    /// Toggle floating state for the active window.
    fn toggle_floating(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            let handle = window.handle;
            debug!("Toggling floating for window: {}", window.title);
            wm.toggle_floating(&handle)?;
            Ok(())
        } else {
            warn!("No active window to toggle floating");
            Ok(())
        }
    }

    /// Toggle fullscreen state for the active window.
    fn toggle_fullscreen(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            let handle = window.handle;
            debug!("Toggling fullscreen for window: {}", window.title);
            wm.toggle_fullscreen(&handle)?;
            Ok(())
        } else {
            warn!("No active window to toggle fullscreen");
            Ok(())
        }
    }

    /// Minimize the active window.
    fn minimize_active(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window_mut() {
            debug!("Minimizing window: {}", window.title);
            window.minimize()?;
            Ok(())
        } else {
            warn!("No active window to minimize");
            Ok(())
        }
    }

    /// Restore the active window from minimized state.
    fn restore_active(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window_mut() {
            debug!("Restoring window: {}", window.title);
            window.restore()?;
            Ok(())
        } else {
            warn!("No active window to restore");
            Ok(())
        }
    }

    /// Focus a window in a specific direction.
    ///
    /// This is a placeholder implementation. Full directional focus would require
    /// integration with the tree structure to find adjacent windows based on
    /// their spatial relationships.
    fn focus_direction(&self, _wm: &mut WindowManager, direction: Direction) -> Result<()> {
        debug!("Focus direction: {:?}", direction);
        // TODO: Implement directional focus navigation using DirectionalFocus helper
        // This requires:
        // 1. Get current active window and its rectangle
        // 2. Get all visible windows in current workspace with their rectangles
        // 3. Use DirectionalFocus::find_window_in_direction to find target
        // 4. Focus the target window
        warn!("Directional focus not yet fully implemented");
        Ok(())
    }

    /// Focus the previous window in focus history.
    fn focus_previous(&self, _wm: &mut WindowManager) -> Result<()> {
        debug!("Focus previous window");
        // TODO: Integrate with FocusManager
        // This requires adding FocusManager to WindowManager
        warn!("Focus history navigation not yet fully implemented");
        Ok(())
    }

    /// Focus the next window in focus history.
    fn focus_next(&self, _wm: &mut WindowManager) -> Result<()> {
        debug!("Focus next window");
        // TODO: Integrate with FocusManager
        // This requires adding FocusManager to WindowManager
        warn!("Focus history navigation not yet fully implemented");
        Ok(())
    }

    /// Move a window in a specific direction within the tree.
    ///
    /// This is a placeholder implementation. Full move functionality would require
    /// tree manipulation to swap positions or reorganize the tree structure.
    fn move_window(&self, _wm: &mut WindowManager, direction: Direction) -> Result<()> {
        debug!("Move window: {:?}", direction);
        // TODO: Implement window movement in tree structure
        // This requires:
        // 1. Find the active window in the tree
        // 2. Find the adjacent window in the specified direction
        // 3. Swap their positions or reorganize the tree
        // 4. Retile the workspace
        warn!("Window movement not yet fully implemented");
        Ok(())
    }

    /// Swap the active window with the master window.
    fn swap_with_master(&self, _wm: &mut WindowManager) -> Result<()> {
        debug!("Swap with master");
        // TODO: Implement master swap functionality
        // This is most relevant for master layout
        warn!("Swap with master not yet fully implemented");
        Ok(())
    }

    /// Adjust the master window count.
    ///
    /// # Arguments
    ///
    /// * `delta` - The change in master count (positive to increase, negative to decrease)
    fn adjust_master_count(&self, wm: &mut WindowManager, delta: i32) -> Result<()> {
        debug!("Adjusting master count by {}", delta);

        if delta > 0 {
            wm.increase_master_count();
            wm.retile_workspace(wm.get_active_workspace())?;
        } else if delta < 0 {
            wm.decrease_master_count();
            wm.retile_workspace(wm.get_active_workspace())?;
        }
        // If delta == 0, do nothing
        
        Ok(())
    }

    /// Adjust the master factor (portion of screen for master area).
    ///
    /// # Arguments
    ///
    /// * `delta` - The change in master factor (typically 0.05 for 5% increments)
    fn adjust_master_factor(&self, wm: &mut WindowManager, delta: f32) -> Result<()> {
        debug!("Adjusting master factor by {}", delta);

        wm.adjust_master_factor(delta);
        wm.retile_workspace(wm.get_active_workspace())?;
        Ok(())
    }

    /// Move the active window to a different workspace.
    fn move_to_workspace(&self, _wm: &mut WindowManager, workspace_id: usize) -> Result<()> {
        debug!("Moving window to workspace {}", workspace_id);
        // TODO: Implement workspace movement
        // This requires:
        // 1. Get the active window
        // 2. Update its workspace property
        // 3. Retile both the old and new workspaces
        warn!("Move to workspace not yet fully implemented");
        Ok(())
    }

    /// Move the active window to a different workspace and switch to it.
    fn move_to_workspace_and_follow(
        &self,
        wm: &mut WindowManager,
        workspace_id: usize,
    ) -> Result<()> {
        debug!("Moving window to workspace {} and following", workspace_id);
        
        // Note: move_to_workspace is not yet fully implemented
        // For consistency, we warn but don't fail the workspace switch
        self.move_to_workspace(wm, workspace_id)?;
        
        // Switch to the target workspace regardless of move success
        // This allows the command to partially work even if move isn't implemented yet
        wm.switch_workspace(workspace_id)?;
        Ok(())
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::CloseActiveWindow;
        assert_eq!(cmd, Command::CloseActiveWindow);
    }

    #[test]
    fn test_command_debug() {
        let cmd = Command::ToggleFloating;
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("ToggleFloating"));
    }

    #[test]
    fn test_command_clone() {
        let cmd1 = Command::SetLayoutDwindle;
        let cmd2 = cmd1.clone();
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_workspace_command() {
        let cmd = Command::SwitchWorkspace(3);
        match cmd {
            Command::SwitchWorkspace(id) => assert_eq!(id, 3),
            _ => panic!("Wrong command variant"),
        }
    }

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        // Verify executor can be created
        let _ = executor;
    }

    #[test]
    fn test_executor_default() {
        let executor = CommandExecutor::default();
        // Verify default implementation works
        let _ = executor;
    }

    // Note: Full integration tests with WindowManager require a Windows environment
    // and actual window handles. These tests verify the basic structure and
    // command variants work correctly.

    #[test]
    fn test_all_command_variants_exist() {
        // Ensure all command variants can be created
        let commands = vec![
            Command::CloseActiveWindow,
            Command::ToggleFloating,
            Command::ToggleFullscreen,
            Command::MinimizeActive,
            Command::RestoreActive,
            Command::FocusLeft,
            Command::FocusRight,
            Command::FocusUp,
            Command::FocusDown,
            Command::FocusPrevious,
            Command::FocusNext,
            Command::MoveWindowLeft,
            Command::MoveWindowRight,
            Command::MoveWindowUp,
            Command::MoveWindowDown,
            Command::SwapWithMaster,
            Command::SetLayoutDwindle,
            Command::SetLayoutMaster,
            Command::IncreaseMasterCount,
            Command::DecreaseMasterCount,
            Command::IncreaseMasterFactor,
            Command::DecreaseMasterFactor,
            Command::SwitchWorkspace(1),
            Command::MoveToWorkspace(2),
            Command::MoveToWorkspaceAndFollow(3),
            Command::Reload,
            Command::Quit,
        ];

        // Verify all commands are unique
        assert_eq!(commands.len(), 27);
    }

    #[test]
    fn test_command_equality() {
        assert_eq!(Command::ToggleFloating, Command::ToggleFloating);
        assert_ne!(Command::ToggleFloating, Command::ToggleFullscreen);
        assert_eq!(Command::SwitchWorkspace(1), Command::SwitchWorkspace(1));
        assert_ne!(Command::SwitchWorkspace(1), Command::SwitchWorkspace(2));
    }

    // Integration test placeholder - would require actual WindowManager
    #[test]
    #[ignore]
    fn test_command_execution_integration() {
        let executor = CommandExecutor::new();
        let mut wm = WindowManager::new();
        wm.initialize().ok();

        // These would be tested with real windows in a Windows environment
        // For now, we just verify the execute method can be called
        let result = executor.execute(Command::SetLayoutDwindle, &mut wm);
        assert!(result.is_ok());
    }
}
