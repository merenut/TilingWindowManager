//! State persistence operations.
//!
//! This module contains operations for saving and restoring workspace state.

use crate::workspace::{core::Workspace, WorkspaceManager};

impl WorkspaceManager {
    /// Save current workspace state to disk
    pub fn save_state(
        &self,
        persistence: &crate::workspace::persistence::PersistenceManager,
    ) -> anyhow::Result<()> {
        if !self.config.persist_state {
            return Ok(());
        }

        let mut state = crate::workspace::persistence::SessionState {
            active_workspace: self.active_workspace,
            ..Default::default()
        };

        for workspace in self.workspaces.values() {
            let ws_state = crate::workspace::persistence::WorkspaceState {
                id: workspace.id,
                name: workspace.name.clone(),
                monitor: workspace.monitor,
                windows: {
                    #[cfg(target_os = "windows")]
                    {
                        workspace
                            .windows
                            .iter()
                            .filter_map(|&hwnd| {
                                let handle = crate::utils::win32::WindowHandle::from_hwnd(
                                    windows::Win32::Foundation::HWND(hwnd),
                                );

                                if let (Ok(title), Ok(class), Ok(process)) = (
                                    handle.get_title(),
                                    handle.get_class_name(),
                                    handle.get_process_name(),
                                ) {
                                    Some(crate::workspace::persistence::WindowState {
                                        // HWND is stored as a string for cross-session persistence.
                                        // While HWNDs are not guaranteed to be the same across restarts,
                                        // this metadata helps track which windows belonged to which workspace.
                                        hwnd: format!("{}", hwnd),
                                        process_name: process,
                                        title,
                                        class_name: class,
                                        workspace: workspace.id,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        // On non-Windows platforms, store minimal window info
                        workspace
                            .windows
                            .iter()
                            .map(|&hwnd| crate::workspace::persistence::WindowState {
                                hwnd: format!("{}", hwnd),
                                process_name: String::new(),
                                title: String::new(),
                                class_name: String::new(),
                                workspace: workspace.id,
                            })
                            .collect()
                    }
                },
                virtual_desktop_id: workspace
                    .virtual_desktop_id
                    .map(|guid| format!("{:?}", guid)),
            };

            state.workspaces.push(ws_state);
        }

        // Save window-to-workspace mappings. HWNDs are converted to strings for serialization.
        // Note: These mappings are primarily for state tracking and analysis, as HWNDs
        // typically change across sessions.
        for (&hwnd, &workspace_id) in &self.window_to_workspace {
            state
                .window_to_workspace
                .insert(format!("{}", hwnd), workspace_id);
        }

        persistence.save_state(&state)?;
        Ok(())
    }

    /// Restore workspace state from disk
    pub fn restore_state(
        &mut self,
        persistence: &crate::workspace::persistence::PersistenceManager,
        monitor_areas: &[(usize, crate::window_manager::tree::Rect)],
    ) -> anyhow::Result<()> {
        if !self.config.persist_state {
            return Ok(());
        }

        let state = persistence.load_state_with_fallback()?;

        tracing::info!("Restoring workspace state (version: {})", state.version);

        self.workspaces.clear();
        self.window_to_workspace.clear();

        for ws_state in state.workspaces {
            let area = monitor_areas
                .iter()
                .find(|(id, _)| *id == ws_state.monitor)
                .map(|(_, area)| *area)
                .unwrap_or_else(|| monitor_areas[0].1);

            let mut workspace = Workspace::new(ws_state.id, ws_state.name, ws_state.monitor, area);

            // Virtual Desktop IDs are not restored immediately because they need to be
            // reassigned by the Virtual Desktop manager during sync_with_virtual_desktops().
            // The saved IDs are preserved in the state file for reference but may not be
            // valid after a system restart.
            workspace.virtual_desktop_id = None;

            self.workspaces.insert(workspace.id, workspace);

            if ws_state.id >= self.next_id {
                self.next_id = ws_state.id + 1;
            }
        }

        if self.workspaces.contains_key(&state.active_workspace) {
            self.active_workspace = state.active_workspace;

            if let Some(ws) = self.workspaces.get_mut(&state.active_workspace) {
                ws.mark_active();
            }
        }

        tracing::info!("Restored {} workspaces", self.workspaces.len());
        Ok(())
    }

    /// Auto-save workspace state (called periodically)
    pub fn auto_save(&self, persistence: &crate::workspace::persistence::PersistenceManager) {
        if let Err(e) = self.save_state(persistence) {
            tracing::error!("Failed to auto-save workspace state: {}", e);
        }
    }
}
