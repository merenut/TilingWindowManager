//! Command system for window operations.
//!
//! This module provides a comprehensive command system for all window, layout,
//! focus, and workspace operations. Commands are executed through the CommandExecutor
//! which acts on the WindowManager.
//!
//! # Example
//!
//! ```no_run
//! use tenraku_core::commands::{Command, CommandExecutor};
//! use tenraku_core::window_manager::WindowManager;
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
    /// Show command palette overlay
    ShowCommandPalette,
}

/// Executes commands on the WindowManager.
///
/// The CommandExecutor provides methods to execute commands, handling
/// errors and logging appropriately.
///
/// # Example
///
/// ```no_run
/// use tenraku_core::commands::{Command, CommandExecutor};
/// use tenraku_core::window_manager::WindowManager;
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
    /// use tenraku_core::commands::CommandExecutor;
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
    /// use tenraku_core::commands::{Command, CommandExecutor};
    /// use tenraku_core::window_manager::WindowManager;
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
            Command::ShowCommandPalette => self.show_command_palette(wm),
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
    /// Uses the DirectionalFocus helper to find adjacent windows based on
    /// their spatial relationships.
    fn focus_direction(&self, wm: &mut WindowManager, direction: Direction) -> Result<()> {
        use crate::utils::win32;
        use crate::window_manager::focus::DirectionalFocus;

        debug!("Focus direction: {:?}", direction);

        let (current_window, current_rect) = match self.get_current_window_and_rect() {
            Some(data) => data,
            None => return Ok(()),
        };

        let candidates = self.get_workspace_window_candidates(wm, current_window.hwnd());

        if candidates.is_empty() {
            debug!("No other windows to focus");
            return Ok(());
        }

        if let Some(target_hwnd) =
            DirectionalFocus::find_window_in_direction(&current_rect, direction, &candidates)
        {
            debug!(
                "Focusing window {:?} in direction {:?}",
                target_hwnd, direction
            );
            let target_window =
                win32::WindowHandle::from_hwnd(windows::Win32::Foundation::HWND(target_hwnd as _));
            wm.focus_manager_mut().focus_window(&target_window)?;
        } else {
            debug!("No window found in direction {:?}", direction);
        }

        Ok(())
    }

    fn get_current_window_and_rect(
        &self,
    ) -> Option<(
        crate::utils::win32::WindowHandle,
        crate::window_manager::Rect,
    )> {
        use crate::utils::win32;

        let current_window = win32::get_foreground_window()?;

        let rect_raw = match current_window.get_rect() {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to get current window rect: {}", e);
                return None;
            }
        };

        let rect = crate::window_manager::Rect::new(
            rect_raw.left,
            rect_raw.top,
            rect_raw.right - rect_raw.left,
            rect_raw.bottom - rect_raw.top,
        );

        Some((current_window, rect))
    }

    fn get_workspace_window_candidates(
        &self,
        wm: &WindowManager,
        exclude_hwnd: windows::Win32::Foundation::HWND,
    ) -> Vec<(isize, crate::window_manager::Rect)> {
        let workspace_id = wm.get_active_workspace();
        let workspace_trees = wm.get_workspace_trees(workspace_id);

        if workspace_trees.is_empty() {
            warn!("No trees found for workspace {}", workspace_id);
            return Vec::new();
        }

        let mut windows = Vec::new();
        for (_, tree) in workspace_trees {
            windows.extend(tree.collect());
        }

        windows
            .into_iter()
            .filter(|(hwnd, _)| hwnd.0 != 0 && *hwnd != exclude_hwnd)
            .map(|(hwnd, rect)| (hwnd.0 as isize, rect))
            .collect()
    }

    /// Focus the previous window in focus history.
    fn focus_previous(&self, wm: &mut WindowManager) -> Result<()> {
        use crate::utils::win32::WindowHandle;
        use windows::Win32::Foundation::HWND;

        debug!("Focus previous window");

        let focus_mgr = wm.focus_manager_mut();
        if let Some(hwnd_val) = focus_mgr.focus_previous() {
            let window = WindowHandle::from_hwnd(HWND(hwnd_val as _));
            if let Err(e) = focus_mgr.focus_window(&window) {
                warn!("Failed to focus previous window: {}", e);
            }
        } else {
            debug!("No previous window in focus history");
        }

        Ok(())
    }

    /// Focus the next window in focus history.
    fn focus_next(&self, wm: &mut WindowManager) -> Result<()> {
        use crate::utils::win32::WindowHandle;
        use windows::Win32::Foundation::HWND;

        debug!("Focus next window");

        let focus_mgr = wm.focus_manager_mut();
        if let Some(hwnd_val) = focus_mgr.focus_next() {
            let window = WindowHandle::from_hwnd(HWND(hwnd_val as _));
            if let Err(e) = focus_mgr.focus_window(&window) {
                warn!("Failed to focus next window: {}", e);
            }
        } else {
            debug!("No next window in focus history");
        }

        Ok(())
    }

    /// Move a window in a specific direction within the tree.
    ///
    /// Swaps the active window with an adjacent window in the specified direction.
    fn move_window(&self, wm: &mut WindowManager, direction: Direction) -> Result<()> {
        use crate::window_manager::focus::DirectionalFocus;

        debug!("Move window: {:?}", direction);

        let (current_window, current_rect) = match self.get_current_window_and_rect() {
            Some(data) => data,
            None => {
                warn!("No active window to move");
                return Ok(());
            }
        };

        let current_hwnd = current_window.hwnd();
        let candidates = self.get_workspace_window_candidates(wm, current_hwnd);

        if candidates.is_empty() {
            debug!("No other windows to swap with");
            return Ok(());
        }

        if let Some(target_hwnd_val) =
            DirectionalFocus::find_window_in_direction(&current_rect, direction, &candidates)
        {
            debug!(
                "Swapping window {:?} with {:?}",
                current_hwnd.0, target_hwnd_val
            );

            // Note: This simplified implementation just retiles the workspace
            // A more sophisticated approach would manipulate the tree structure directly
            wm.retile_workspace(wm.get_active_workspace())?;
        } else {
            debug!("No window found in direction {:?} to swap with", direction);
        }

        Ok(())
    }

    /// Swap the active window with the master window.
    fn swap_with_master(&self, wm: &mut WindowManager) -> Result<()> {
        use crate::utils::win32;

        debug!("Swap with master");

        let current_window = match win32::get_foreground_window() {
            Some(w) => w,
            None => {
                warn!("No active window to swap");
                return Ok(());
            }
        };

        let current_hwnd = current_window.hwnd();
        let workspace_id = wm.get_active_workspace();

        let windows = self.collect_all_workspace_windows(wm, workspace_id);

        if windows.is_empty() {
            debug!("Workspace is empty");
            return Ok(());
        }

        let master_hwnd = windows[0].0;

        if master_hwnd == current_hwnd {
            debug!("Window is already master");
            return Ok(());
        }

        if master_hwnd.0 == 0 {
            debug!("No valid master window");
            return Ok(());
        }

        if !windows.iter().any(|(h, _)| *h == current_hwnd) {
            warn!("Active window is not in current workspace");
            return Ok(());
        }

        debug!(
            "Swapping window {:?} with master {:?}",
            current_hwnd.0, master_hwnd.0
        );

        // Note: This simplified implementation just retiles the workspace
        wm.retile_workspace(workspace_id)?;
        current_window.set_foreground()?;

        Ok(())
    }

    fn collect_all_workspace_windows(
        &self,
        wm: &WindowManager,
        workspace_id: usize,
    ) -> Vec<(
        windows::Win32::Foundation::HWND,
        crate::window_manager::Rect,
    )> {
        let workspace_trees = wm.get_workspace_trees(workspace_id);

        if workspace_trees.is_empty() {
            warn!("No trees found for workspace {}", workspace_id);
            return Vec::new();
        }

        let mut windows = Vec::new();
        for (_, tree) in workspace_trees {
            windows.extend(tree.collect());
        }
        windows
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
    fn move_to_workspace(&self, wm: &mut WindowManager, workspace_id: usize) -> Result<()> {
        use crate::utils::win32;

        debug!("Moving window to workspace {}", workspace_id);

        // Validate workspace ID
        if workspace_id < 1 || workspace_id > 10 {
            warn!("Invalid workspace ID: {}", workspace_id);
            return Ok(());
        }

        // Get current active window
        let current_window = match win32::get_foreground_window() {
            Some(window) => window,
            None => {
                warn!("No active window to move");
                return Ok(());
            }
        };
        let current_hwnd = current_window.hwnd();

        let current_workspace = wm.get_active_workspace();

        // Check if already on target workspace
        if current_workspace == workspace_id {
            debug!("Window is already on workspace {}", workspace_id);
            return Ok(());
        }

        // Update the window's workspace in the registry
        if let Some(managed_window) = wm.registry_mut().get_mut(current_hwnd.0) {
            let old_workspace = managed_window.workspace;
            managed_window.workspace = workspace_id;

            debug!(
                "Moved window {:?} from workspace {} to {}",
                current_hwnd.0, old_workspace, workspace_id
            );

            // Hide the window since it's no longer on the current workspace
            #[cfg(target_os = "windows")]
            current_window.hide();

            // Retile both workspaces
            wm.retile_workspace(old_workspace)?;
            wm.retile_workspace(workspace_id)?;
        } else {
            warn!("Active window is not managed");
        }

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

    /// Show the command palette overlay.
    fn show_command_palette(&self, _wm: &mut WindowManager) -> Result<()> {
        use crate::utils::win32;
        use std::process::Command;

        println!("=== SHOWING COMMAND PALETTE ===");
        info!("Showing command palette");

        // Get the current window handle to use as parent
        let parent_hwnd = match win32::get_foreground_window() {
            Some(window) => {
                println!("Parent window: {}", window.hwnd().0);
                window.hwnd().0
            }
            None => {
                println!("No parent window");
                0 // No parent if no window is focused
            }
        };

        // Get the path to the command palette executable
        let palette_exe = if cfg!(debug_assertions) {
            // In debug mode, use target/debug
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .map(|dir| dir.join("tenraku-palette.exe"))
                .unwrap_or_else(|| "target/debug/tenraku-palette.exe".into())
        } else {
            // In release mode, use target/release or same directory
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .map(|dir| dir.join("tenraku-palette.exe"))
                .unwrap_or_else(|| "target/release/tenraku-palette.exe".into())
        };

        println!("Palette executable path: {}", palette_exe.display());

        // Spawn the command palette process
        match Command::new(&palette_exe)
            .arg("--parent-hwnd")
            .arg(parent_hwnd.to_string())
            .spawn()
        {
            Ok(_) => {
                println!("Command palette spawned successfully!");
                debug!("Command palette spawned successfully");
                Ok(())
            }
            Err(e) => {
                println!("ERROR: Failed to spawn command palette: {}", e);
                error!("Failed to spawn command palette: {}", e);
                Err(e.into())
            }
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}
