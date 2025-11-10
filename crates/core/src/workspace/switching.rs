//! Workspace switching operations.
//!
//! This module contains all operations related to switching between workspaces.

use crate::workspace::WorkspaceManager;

impl WorkspaceManager {
    /// Switch to a different workspace
    pub fn switch_to(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if !self.workspaces.contains_key(&workspace_id) {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }

        if self.active_workspace == workspace_id {
            return Ok(());
        }

        tracing::info!(
            "Switching from workspace {} to {}",
            self.active_workspace,
            workspace_id
        );

        // Hide windows from current workspace
        if let Some(current) = self.workspaces.get_mut(&self.active_workspace) {
            current.mark_inactive();

            #[cfg(target_os = "windows")]
            {
                for &hwnd in &current.windows {
                    unsafe {
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::UI::WindowsAndMessaging::*;
                        ShowWindow(HWND(hwnd), SW_HIDE);
                    }
                }
            }
        }

        // Show windows from target workspace
        if let Some(target) = self.workspaces.get_mut(&workspace_id) {
            target.mark_active();

            #[cfg(target_os = "windows")]
            {
                for &hwnd in &target.windows {
                    unsafe {
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::UI::WindowsAndMessaging::*;
                        ShowWindow(HWND(hwnd), SW_SHOW);
                    }
                }

                // Apply layout geometry if tree exists
                if let Some(ref tree) = target.tree {
                    tree.apply_layout(5, 10)?;
                }
            }
        }

        // Switch Virtual Desktop if enabled
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(workspace) = self.workspaces.get(&workspace_id) {
                if let Some(vd_id) = workspace.virtual_desktop_id {
                    vd_manager.switch_desktop_by_id(&vd_id)?;
                }
            }
        }

        self.active_workspace = workspace_id;
        Ok(())
    }

    /// Switch to the next workspace
    pub fn switch_to_next(&mut self) -> anyhow::Result<()> {
        let current_monitor = self
            .workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);

        let monitor_workspaces = self.get_monitor_workspaces(current_monitor);

        if let Some(current_idx) = monitor_workspaces
            .iter()
            .position(|&id| id == self.active_workspace)
        {
            let next_idx = (current_idx + 1) % monitor_workspaces.len();
            let next_id = monitor_workspaces[next_idx];
            self.switch_to(next_id)?;
        }

        Ok(())
    }

    /// Switch to the previous workspace
    pub fn switch_to_previous(&mut self) -> anyhow::Result<()> {
        let current_monitor = self
            .workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);

        let monitor_workspaces = self.get_monitor_workspaces(current_monitor);

        if let Some(current_idx) = monitor_workspaces
            .iter()
            .position(|&id| id == self.active_workspace)
        {
            let prev_idx = if current_idx == 0 {
                monitor_workspaces.len() - 1
            } else {
                current_idx - 1
            };
            let prev_id = monitor_workspaces[prev_idx];
            self.switch_to(prev_id)?;
        }

        Ok(())
    }

    /// Switch to a workspace by index on the current monitor (1-based)
    pub fn switch_to_index(&mut self, index: usize) -> anyhow::Result<()> {
        if index == 0 {
            anyhow::bail!("Workspace index must be >= 1");
        }

        let current_monitor = self
            .workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);

        let monitor_workspaces = self.get_monitor_workspaces(current_monitor);

        if index > monitor_workspaces.len() {
            anyhow::bail!(
                "Workspace index {} out of range (max: {})",
                index,
                monitor_workspaces.len()
            );
        }

        let target_id = monitor_workspaces[index - 1];
        self.switch_to(target_id)
    }

    /// Switch workspace on a specific monitor.
    pub fn switch_workspace_on_monitor(
        &mut self,
        monitor_id: usize,
        workspace_id: usize,
    ) -> anyhow::Result<()> {
        // Validate that the workspace exists and is on the correct monitor
        if let Some(workspace) = self.workspaces.get(&workspace_id) {
            if workspace.monitor != monitor_id {
                anyhow::bail!(
                    "Workspace {} is on monitor {}, not monitor {}",
                    workspace_id,
                    workspace.monitor,
                    monitor_id
                );
            }
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }

        // Find and hide the currently visible workspace on this monitor (if any)
        let current_visible = self
            .workspaces
            .values()
            .find(|ws| ws.monitor == monitor_id && ws.visible)
            .map(|ws| ws.id);

        // Hide the currently visible workspace on this monitor
        if let Some(ws_id) = current_visible {
            if let Some(ws) = self.workspaces.get_mut(&ws_id) {
                ws.mark_inactive();

                // Hide all windows in this workspace
                #[cfg(target_os = "windows")]
                {
                    for &hwnd in &ws.windows {
                        unsafe {
                            use windows::Win32::Foundation::HWND;
                            use windows::Win32::UI::WindowsAndMessaging::*;
                            ShowWindow(HWND(hwnd), SW_HIDE);
                        }
                    }
                }
            }
        }

        // Show the target workspace
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.mark_active();

            // Show all windows in the target workspace
            #[cfg(target_os = "windows")]
            {
                for &hwnd in &workspace.windows {
                    unsafe {
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::UI::WindowsAndMessaging::*;
                        ShowWindow(HWND(hwnd), SW_SHOW);
                    }
                }

                // Apply layout geometry if tree exists
                if let Some(ref tree) = workspace.tree {
                    tree.apply_layout(0, 0)?;
                }
            }
        }

        Ok(())
    }
}
