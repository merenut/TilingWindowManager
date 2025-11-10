//! Window-to-workspace management operations.
//!
//! This module contains operations for adding, removing, and moving windows between workspaces.

use crate::workspace::WorkspaceManager;

impl WorkspaceManager {
    /// Add a window to a workspace
    pub fn add_window_to_workspace(
        &mut self,
        hwnd: isize,
        workspace_id: usize,
    ) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.add_window(hwnd);
            self.window_to_workspace.insert(hwnd, workspace_id);

            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::*;

                if workspace_id == self.active_workspace {
                    ShowWindow(HWND(hwnd), SW_SHOW);
                } else {
                    ShowWindow(HWND(hwnd), SW_HIDE);
                }
            }

            Ok(())
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
    }

    /// Remove a window from its workspace
    pub fn remove_window(&mut self, hwnd: isize) -> anyhow::Result<Option<usize>> {
        if let Some(&workspace_id) = self.window_to_workspace.get(&hwnd) {
            if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
                workspace.remove_window(hwnd);
            }
            self.window_to_workspace.remove(&hwnd);
            Ok(Some(workspace_id))
        } else {
            Ok(None)
        }
    }

    /// Move a window from one workspace to another
    pub fn move_window_to_workspace(
        &mut self,
        hwnd: isize,
        from_workspace: usize,
        to_workspace: usize,
    ) -> anyhow::Result<()> {
        if from_workspace == to_workspace {
            return Ok(());
        }

        if let Some(from_ws) = self.workspaces.get_mut(&from_workspace) {
            from_ws.remove_window(hwnd);
        }

        if let Some(to_ws) = self.workspaces.get_mut(&to_workspace) {
            to_ws.add_window(hwnd);
        } else {
            anyhow::bail!("Target workspace {} does not exist", to_workspace);
        }

        self.window_to_workspace.insert(hwnd, to_workspace);

        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::*;

            if to_workspace == self.active_workspace {
                ShowWindow(HWND(hwnd), SW_SHOW);
            } else {
                ShowWindow(HWND(hwnd), SW_HIDE);
            }
        }

        #[cfg(target_os = "windows")]
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(to_ws) = self.workspaces.get(&to_workspace) {
                if let Some(vd_id) = to_ws.virtual_desktop_id {
                    vd_manager
                        .move_window_to_desktop(windows::Win32::Foundation::HWND(hwnd), &vd_id)?;
                }
            }
        }

        Ok(())
    }

    /// Move the currently focused window to a different workspace
    pub fn move_active_window_to_workspace(
        &mut self,
        target_workspace: usize,
    ) -> anyhow::Result<()> {
        let fg_window = crate::utils::win32::get_foreground_window()
            .ok_or_else(|| anyhow::anyhow!("No foreground window"))?;

        let hwnd = fg_window.0 .0;

        if let Some(current_workspace) = self.window_to_workspace.get(&hwnd).copied() {
            self.move_window_to_workspace(hwnd, current_workspace, target_workspace)?;
        }

        Ok(())
    }

    /// Move the active window to a workspace and follow it
    pub fn move_active_window_and_follow(&mut self, target_workspace: usize) -> anyhow::Result<()> {
        self.move_active_window_to_workspace(target_workspace)?;
        self.switch_to(target_workspace)?;
        Ok(())
    }
}
