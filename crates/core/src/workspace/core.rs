//! Core workspace data structures and WorkspaceManager implementation.
//!
//! This module contains the core Workspace and WorkspaceManager structs,
//! along with basic initialization and CRUD operations.

use crate::window_manager::tree::{Rect, TreeNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::core::GUID;

// On non-Windows platforms, define GUID as a type alias for byte array
#[cfg(not(target_os = "windows"))]
#[allow(clippy::upper_case_acronyms)]
type GUID = [u8; 16];

/// Represents a single workspace with its windows and layout.
///
/// A workspace is a virtual desktop that contains a set of windows arranged
/// in a binary tree layout. Each workspace can be assigned to a monitor and
/// optionally integrated with Windows Virtual Desktops.
#[derive(Debug)]
pub struct Workspace {
    /// Unique workspace ID
    pub id: usize,

    /// Human-readable workspace name
    pub name: String,

    /// Monitor this workspace is assigned to
    pub monitor: usize,

    /// Layout tree for this workspace (None if empty)
    pub tree: Option<TreeNode>,

    /// Windows in this workspace (HWND values as isize)
    pub windows: Vec<isize>,

    /// Virtual Desktop ID (if using Virtual Desktop integration)
    pub virtual_desktop_id: Option<GUID>,

    /// Whether this workspace is currently visible
    pub visible: bool,

    /// Last time this workspace was active
    pub last_active: std::time::Instant,
}

impl Workspace {
    /// Create a new empty workspace.
    pub fn new(id: usize, name: String, monitor: usize, _area: Rect) -> Self {
        Self {
            id,
            name,
            monitor,
            tree: None,
            windows: Vec::new(),
            virtual_desktop_id: None,
            visible: false,
            last_active: std::time::Instant::now(),
        }
    }

    /// Add a window to this workspace.
    pub fn add_window(&mut self, hwnd: isize) {
        if !self.windows.contains(&hwnd) {
            self.windows.push(hwnd);
        }
    }

    /// Remove a window from this workspace.
    pub fn remove_window(&mut self, hwnd: isize) -> bool {
        if let Some(pos) = self.windows.iter().position(|&w| w == hwnd) {
            self.windows.remove(pos);
            true
        } else {
            false
        }
    }

    /// Mark this workspace as active.
    pub fn mark_active(&mut self) {
        self.visible = true;
        self.last_active = std::time::Instant::now();
    }

    /// Mark this workspace as inactive.
    pub fn mark_inactive(&mut self) {
        self.visible = false;
    }
}

/// Configuration for the workspace management system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Default number of workspaces to create
    pub default_count: usize,

    /// Workspace names (index = workspace number - 1)
    pub names: Vec<String>,

    /// Whether to persist workspace state across restarts
    pub persist_state: bool,

    /// Whether to create workspaces on demand when accessed
    pub create_on_demand: bool,

    /// Whether to use Windows Virtual Desktop integration
    pub use_virtual_desktops: bool,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            default_count: 10,
            names: (1..=10).map(|i| i.to_string()).collect(),
            persist_state: true,
            create_on_demand: false,
            use_virtual_desktops: false,
        }
    }
}

/// Manages all workspaces and window-to-workspace mappings.
pub struct WorkspaceManager {
    /// All workspaces by ID
    pub(super) workspaces: HashMap<usize, Workspace>,

    /// Currently active workspace ID
    pub(super) active_workspace: usize,

    /// Next workspace ID to assign
    pub(super) next_id: usize,

    /// Configuration
    pub(super) config: WorkspaceConfig,

    /// Virtual Desktop manager (if enabled)
    pub(super) vd_manager: Option<crate::workspace::virtual_desktop::VirtualDesktopManager>,

    /// Map of windows to their workspaces
    pub(super) window_to_workspace: HashMap<isize, usize>,
}

impl WorkspaceManager {
    /// Create a new workspace manager with the given configuration.
    pub fn new(config: WorkspaceConfig) -> Self {
        Self {
            workspaces: HashMap::new(),
            active_workspace: 1,
            next_id: 1,
            config,
            vd_manager: None,
            window_to_workspace: HashMap::new(),
        }
    }

    /// Get the currently active workspace ID.
    pub fn active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// Get the workspace with the given ID, if it exists.
    pub fn get_workspace(&self, id: usize) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    /// Get the workspace ID for a given window, if it exists.
    pub fn get_window_workspace(&self, hwnd: isize) -> Option<usize> {
        self.window_to_workspace.get(&hwnd).copied()
    }

    /// Get the number of workspaces.
    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    /// Get the number of windows tracked.
    pub fn window_count(&self) -> usize {
        self.window_to_workspace.len()
    }

    /// Initialize the workspace manager with default workspaces
    pub fn initialize(&mut self, monitor_areas: &[(usize, Rect)]) -> anyhow::Result<()> {
        for (monitor_id, area) in monitor_areas {
            for i in 0..self.config.default_count {
                let ws_id = self.next_id;
                self.next_id += 1;

                let name = if i < self.config.names.len() {
                    self.config.names[i].clone()
                } else {
                    ws_id.to_string()
                };

                let workspace = Workspace::new(ws_id, name, *monitor_id, *area);
                self.workspaces.insert(ws_id, workspace);

                if *monitor_id == 0 && i == 0 {
                    self.active_workspace = ws_id;
                    if let Some(ws) = self.workspaces.get_mut(&ws_id) {
                        ws.mark_active();
                    }
                }
            }
        }

        if self.vd_manager.is_some() {
            self.sync_with_virtual_desktops()?;
        }

        Ok(())
    }

    /// Create a new workspace
    pub fn create_workspace(&mut self, name: String, monitor: usize, area: Rect) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let mut workspace = Workspace::new(id, name, monitor, area);

        if let Some(ref vd_manager) = self.vd_manager {
            if let Ok(vd_id) = vd_manager.create_desktop() {
                workspace.virtual_desktop_id = Some(vd_id);
            }
        }

        self.workspaces.insert(id, workspace);
        id
    }

    /// Delete a workspace (moves windows to fallback workspace)
    pub fn delete_workspace(
        &mut self,
        workspace_id: usize,
        fallback_id: usize,
    ) -> anyhow::Result<()> {
        if workspace_id == fallback_id {
            anyhow::bail!("Cannot delete workspace into itself");
        }

        if self.workspaces.len() <= 1 {
            anyhow::bail!("Cannot delete the last workspace");
        }

        if !self.workspaces.contains_key(&workspace_id) {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }

        if !self.workspaces.contains_key(&fallback_id) {
            anyhow::bail!("Fallback workspace {} does not exist", fallback_id);
        }

        // Move all windows to fallback workspace
        let windows_to_move: Vec<isize> = self
            .workspaces
            .get(&workspace_id)
            .expect("Workspace should exist after validation")
            .windows
            .clone();

        for hwnd in windows_to_move {
            self.move_window_to_workspace(hwnd, workspace_id, fallback_id)?;
        }

        // Remove Virtual Desktop if using VD integration
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(workspace) = self.workspaces.get(&workspace_id) {
                if let Some(vd_id) = workspace.virtual_desktop_id {
                    if let Some(fallback_ws) = self.workspaces.get(&fallback_id) {
                        if let Some(fallback_vd_id) = fallback_ws.virtual_desktop_id {
                            let _ = vd_manager.remove_desktop(&vd_id, &fallback_vd_id);
                        }
                    }
                }
            }
        }

        // Switch to fallback if deleting active workspace
        if self.active_workspace == workspace_id {
            self.switch_to(fallback_id)?;
        }

        self.workspaces.remove(&workspace_id);
        Ok(())
    }

    /// Rename a workspace
    pub fn rename_workspace(
        &mut self,
        workspace_id: usize,
        new_name: String,
    ) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.name = new_name;
            Ok(())
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
    }

    /// Get sorted list of workspace IDs on a specific monitor
    pub(super) fn get_monitor_workspaces(&self, monitor: usize) -> Vec<usize> {
        let mut workspaces: Vec<usize> = self
            .workspaces
            .values()
            .filter(|ws| ws.monitor == monitor)
            .map(|ws| ws.id)
            .collect();
        workspaces.sort();
        workspaces
    }

    /// Sync workspaces with virtual desktops
    pub(super) fn sync_with_virtual_desktops(&mut self) -> anyhow::Result<()> {
        if let Some(ref vd_manager) = self.vd_manager {
            let vd_ids = vd_manager.get_desktop_ids()?;

            // Assign virtual desktop IDs to workspaces that don't have them
            let mut vd_index = 0;
            for workspace in self.workspaces.values_mut() {
                if workspace.virtual_desktop_id.is_none() && vd_index < vd_ids.len() {
                    workspace.virtual_desktop_id = Some(vd_ids[vd_index]);
                    vd_index += 1;
                }
            }
        }

        Ok(())
    }
}
