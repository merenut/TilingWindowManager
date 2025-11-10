//! Monitor integration operations.
//!
//! This module contains operations for managing workspace-monitor relationships
//! and handling monitor changes.

use crate::workspace::WorkspaceManager;

impl WorkspaceManager {
    /// Assign workspaces to monitors.
    pub fn assign_workspaces_to_monitors(
        &mut self,
        monitor_manager: &mut crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        // Clear existing workspace assignments
        for monitor in monitor_manager.monitors.values_mut() {
            monitor.workspaces.clear();
        }

        // Assign workspaces to their respective monitors
        for workspace in self.workspaces.values() {
            if let Some(monitor) = monitor_manager.get_by_id_mut(workspace.monitor) {
                monitor.workspaces.push(workspace.id);

                // Set as active workspace if this workspace is visible
                if workspace.visible {
                    monitor.active_workspace = Some(workspace.id);
                }
            }
        }

        Ok(())
    }

    /// Get the active workspace for a specific monitor.
    pub fn get_active_workspace_for_monitor(&self, monitor_id: usize) -> Option<usize> {
        self.workspaces
            .values()
            .find(|ws| ws.monitor == monitor_id && ws.visible)
            .map(|ws| ws.id)
    }

    /// Redistribute workspaces when monitors change.
    pub fn handle_monitor_change(
        &mut self,
        monitor_manager: &crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        // Reassign workspaces from disconnected monitors and update geometries
        for workspace in self.workspaces.values_mut() {
            if monitor_manager.get_by_id(workspace.monitor).is_none() {
                // Monitor was disconnected, move workspace to primary monitor (ID 0)
                workspace.monitor = 0;

                // Update tree rectangle if it exists
                if let Some(monitor) = monitor_manager.get_by_id(0) {
                    if let Some(ref mut tree) = workspace.tree {
                        tree.set_rect(monitor.work_area);
                    }
                }
            } else {
                // Monitor still exists, update tree rectangle to match current dimensions
                if let Some(monitor) = monitor_manager.get_by_id(workspace.monitor) {
                    if let Some(ref mut tree) = workspace.tree {
                        tree.set_rect(monitor.work_area);
                    }
                }
            }
        }

        Ok(())
    }

    /// Update workspace geometries based on DPI scaling.
    pub fn update_dpi_scaling(
        &mut self,
        monitor_manager: &crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        for workspace in self.workspaces.values_mut() {
            if let Some(monitor) = monitor_manager.get_by_id(workspace.monitor) {
                // Update workspace area with DPI-aware coordinates
                if let Some(ref mut tree) = workspace.tree {
                    tree.set_rect(monitor.work_area);

                    // Re-apply geometry to all windows
                    tree.apply_layout(0, 0)?;
                }
            }
        }

        Ok(())
    }

    /// Apply DPI scaling to a rect.
    pub fn apply_dpi_scaling(rect: &mut crate::window_manager::tree::Rect, dpi_scale: f32) {
        // Clamp dpi_scale to a safe range to prevent overflow and nonsensical geometry
        let scale = dpi_scale.clamp(0.5, 5.0);
        if (scale - 1.0).abs() > 0.01 {
            rect.x = (rect.x as f32 * scale) as i32;
            rect.y = (rect.y as f32 * scale) as i32;
            rect.width = (rect.width as f32 * scale) as i32;
            rect.height = (rect.height as f32 * scale) as i32;
        }
    }
}
