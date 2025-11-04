//! Workspace management core data structures.
//!
//! This module provides the core data structures for managing workspaces:
//! - `Workspace`: Represents a single workspace with its windows and layout tree
//! - `WorkspaceConfig`: Configuration for the workspace system
//! - `WorkspaceManager`: Manages all workspaces and window-to-workspace mappings

use crate::window_manager::tree::{Rect, TreeNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::core::GUID;

// On non-Windows platforms, define GUID as a type alias for byte array
#[cfg(not(target_os = "windows"))]
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
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this workspace
    /// * `name` - Human-readable name for the workspace
    /// * `monitor` - Monitor index this workspace is assigned to
    /// * `area` - Screen area available for this workspace
    ///
    /// # Returns
    ///
    /// A new workspace with no windows.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::Workspace;
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let workspace = Workspace::new(1, "Main".to_string(), 0, rect);
    /// assert_eq!(workspace.id, 1);
    /// assert_eq!(workspace.name, "Main");
    /// assert!(workspace.windows.is_empty());
    /// ```
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
    ///
    /// If the window is not already in the workspace, it is added to the windows list.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - Window handle as isize
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::Workspace;
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);
    /// workspace.add_window(12345);
    /// assert_eq!(workspace.windows.len(), 1);
    /// assert!(workspace.windows.contains(&12345));
    /// ```
    pub fn add_window(&mut self, hwnd: isize) {
        if !self.windows.contains(&hwnd) {
            self.windows.push(hwnd);
        }
    }

    /// Remove a window from this workspace.
    ///
    /// If the window exists in the workspace, it is removed from the windows list.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - Window handle as isize
    ///
    /// # Returns
    ///
    /// `true` if the window was found and removed, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::Workspace;
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);
    /// workspace.add_window(12345);
    /// assert!(workspace.remove_window(12345));
    /// assert!(!workspace.windows.contains(&12345));
    /// assert!(!workspace.remove_window(12345)); // Already removed
    /// ```
    pub fn remove_window(&mut self, hwnd: isize) -> bool {
        if let Some(pos) = self.windows.iter().position(|&w| w == hwnd) {
            self.windows.remove(pos);
            true
        } else {
            false
        }
    }

    /// Mark this workspace as active.
    ///
    /// This sets the workspace as visible and updates the last active timestamp.
    pub fn mark_active(&mut self) {
        self.visible = true;
        self.last_active = std::time::Instant::now();
    }

    /// Mark this workspace as inactive.
    ///
    /// This sets the workspace as not visible.
    pub fn mark_inactive(&mut self) {
        self.visible = false;
    }
}

/// Configuration for the workspace management system.
///
/// This structure defines how workspaces are created and managed,
/// including default counts, naming, and integration features.
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
    /// Create default workspace configuration.
    ///
    /// Defaults:
    /// - 10 workspaces numbered "1" through "10"
    /// - State persistence enabled
    /// - No on-demand creation
    /// - No Virtual Desktop integration
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::WorkspaceConfig;
    ///
    /// let config = WorkspaceConfig::default();
    /// assert_eq!(config.default_count, 10);
    /// assert_eq!(config.names.len(), 10);
    /// assert_eq!(config.names[0], "1");
    /// assert!(config.persist_state);
    /// assert!(!config.create_on_demand);
    /// assert!(!config.use_virtual_desktops);
    /// ```
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
///
/// The WorkspaceManager is responsible for:
/// - Creating and tracking workspaces
/// - Managing which workspace is currently active
/// - Maintaining bidirectional mapping between windows and workspaces
/// - Optional integration with Windows Virtual Desktops
pub struct WorkspaceManager {
    /// All workspaces by ID
    workspaces: HashMap<usize, Workspace>,

    /// Currently active workspace ID
    active_workspace: usize,

    /// Next workspace ID to assign
    next_id: usize,

    /// Configuration
    config: WorkspaceConfig,

    /// Virtual Desktop manager (if enabled)
    vd_manager: Option<crate::workspace::virtual_desktop::VirtualDesktopManager>,

    /// Map of windows to their workspaces
    window_to_workspace: HashMap<isize, usize>,
}

impl WorkspaceManager {
    /// Create a new workspace manager with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for workspace management
    ///
    /// # Returns
    ///
    /// A new WorkspaceManager with no workspaces created yet.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::{WorkspaceManager, WorkspaceConfig};
    ///
    /// let config = WorkspaceConfig::default();
    /// let manager = WorkspaceManager::new(config);
    /// ```
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
                    tree.apply_layout(0, 0)?;
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

        let mut monitor_workspaces: Vec<usize> = self
            .workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();

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

        let mut monitor_workspaces: Vec<usize> = self
            .workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();

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

        let mut monitor_workspaces: Vec<usize> = self
            .workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();

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

    /// Move a window from one workspace to another
    fn move_window_to_workspace(
        &mut self,
        hwnd: isize,
        from_workspace: usize,
        to_workspace: usize,
    ) -> anyhow::Result<()> {
        // Remove from source workspace
        if let Some(workspace) = self.workspaces.get_mut(&from_workspace) {
            workspace.remove_window(hwnd);
        }

        // Add to destination workspace
        if let Some(workspace) = self.workspaces.get_mut(&to_workspace) {
            workspace.add_window(hwnd);
        }

        // Update window-to-workspace mapping
        self.window_to_workspace.insert(hwnd, to_workspace);

        Ok(())
    }

    /// Sync workspaces with virtual desktops
    fn sync_with_virtual_desktops(&mut self) -> anyhow::Result<()> {
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
