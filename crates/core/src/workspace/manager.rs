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
                // TODO: Make gap values configurable via WorkspaceConfig
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

    /// Get sorted list of workspace IDs on a specific monitor
    fn get_monitor_workspaces(&self, monitor: usize) -> Vec<usize> {
        let mut workspaces: Vec<usize> = self
            .workspaces
            .values()
            .filter(|ws| ws.monitor == monitor)
            .map(|ws| ws.id)
            .collect();
        workspaces.sort();
        workspaces
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

    /// Add a window to a workspace
    pub fn add_window_to_workspace(&mut self, hwnd: isize, workspace_id: usize) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.add_window(hwnd);
            self.window_to_workspace.insert(hwnd, workspace_id);
            
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::Win32::Foundation::HWND;
                
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
            use windows::Win32::UI::WindowsAndMessaging::*;
            use windows::Win32::Foundation::HWND;
            
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
                    vd_manager.move_window_to_desktop(
                        windows::Win32::Foundation::HWND(hwnd),
                        &vd_id,
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Move the currently focused window to a different workspace
    pub fn move_active_window_to_workspace(&mut self, target_workspace: usize) -> anyhow::Result<()> {
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

    /// Assign workspaces to monitors.
    ///
    /// This updates the monitor manager to track which workspaces are assigned
    /// to each monitor and which workspace is currently active on each monitor.
    ///
    /// # Arguments
    ///
    /// * `monitor_manager` - Mutable reference to the monitor manager
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
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
    ///
    /// # Arguments
    ///
    /// * `monitor_id` - Monitor ID to query
    ///
    /// # Returns
    ///
    /// `Some(workspace_id)` if there is an active workspace on the monitor,
    /// `None` otherwise.
    pub fn get_active_workspace_for_monitor(&self, monitor_id: usize) -> Option<usize> {
        self.workspaces
            .values()
            .find(|ws| ws.monitor == monitor_id && ws.visible)
            .map(|ws| ws.id)
    }
    
    /// Switch workspace on a specific monitor.
    ///
    /// This hides all currently visible workspaces on the specified monitor
    /// and shows the target workspace. Windows in the hidden workspaces are
    /// hidden, and windows in the shown workspace are displayed.
    ///
    /// # Arguments
    ///
    /// * `monitor_id` - Monitor ID where the workspace switch should occur
    /// * `workspace_id` - Target workspace ID to switch to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The workspace doesn't exist
    /// - The workspace is not on the specified monitor
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
        let current_visible = self.workspaces
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
                            use windows::Win32::UI::WindowsAndMessaging::*;
                            use windows::Win32::Foundation::HWND;
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
                        use windows::Win32::UI::WindowsAndMessaging::*;
                        use windows::Win32::Foundation::HWND;
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
    
    /// Redistribute workspaces when monitors change.
    ///
    /// This handles monitor configuration changes by:
    /// - Moving workspaces from disconnected monitors to monitor 0
    /// - Updating workspace tree rectangles to match new monitor dimensions
    ///
    /// # Arguments
    ///
    /// * `monitor_manager` - Reference to the monitor manager with current monitor state
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
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
    ///
    /// This method iterates through all workspaces and updates their geometry
    /// to match the DPI-aware coordinates from the monitor manager. After updating
    /// the workspace area, it re-applies the geometry to all windows in the workspace.
    ///
    /// The method assumes that `monitor.work_area` already contains the correct
    /// physical pixel coordinates from the Windows API, which automatically accounts
    /// for the monitor's DPI scaling. The `apply_dpi_scaling` helper function is
    /// available for cases where manual DPI scaling of rectangles is needed.
    ///
    /// # Arguments
    ///
    /// * `monitor_manager` - Reference to the monitor manager with current monitor DPI information
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // After detecting a DPI change event
    /// workspace_manager.update_dpi_scaling(&monitor_manager)?;
    /// ```
    pub fn update_dpi_scaling(
        &mut self,
        monitor_manager: &crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        for workspace in self.workspaces.values_mut() {
            if let Some(monitor) = monitor_manager.get_by_id(workspace.monitor) {
                // Update workspace area with DPI-aware coordinates
                // The monitor.work_area is assumed to already be in physical pixels
                // from the Windows API, accounting for the monitor's DPI scale
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
    ///
    /// This is a utility method that scales the coordinates and dimensions of a 
    /// rectangle based on a DPI scale factor. It only applies scaling if the scale 
    /// factor is significantly different from 1.0 (threshold: 0.01).
    ///
    /// This function is provided as a utility and is not used by `update_dpi_scaling`
    /// because the MonitorManager is expected to provide already-scaled coordinates
    /// from the Windows API. This function can be used in other contexts where manual
    /// DPI scaling is needed.
    ///
    /// # Arguments
    ///
    /// * `rect` - Mutable reference to the rectangle to scale
    /// * `dpi_scale` - DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%)
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::workspace::WorkspaceManager;
    /// use tiling_wm_core::window_manager::tree::Rect;
    ///
    /// let mut rect = Rect::new(0, 0, 1920, 1080);
    /// WorkspaceManager::apply_dpi_scaling(&mut rect, 1.5);
    /// assert_eq!(rect.width, 2880);
    /// assert_eq!(rect.height, 1620);
    /// ```
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
    
    /// Save current workspace state to disk
    pub fn save_state(&self, persistence: &crate::workspace::persistence::PersistenceManager) -> anyhow::Result<()> {
        if !self.config.persist_state {
            return Ok(());
        }
        
        let mut state = crate::workspace::persistence::SessionState::default();
        state.active_workspace = self.active_workspace;
        
        for workspace in self.workspaces.values() {
            let ws_state = crate::workspace::persistence::WorkspaceState {
                id: workspace.id,
                name: workspace.name.clone(),
                monitor: workspace.monitor,
                windows: workspace.windows
                    .iter()
                    .filter_map(|&hwnd| {
                        #[cfg(target_os = "windows")]
                        {
                            let handle = crate::utils::win32::WindowHandle::from_hwnd(
                                windows::Win32::Foundation::HWND(hwnd)
                            );
                            
                            if let (Ok(title), Ok(class), Ok(process)) = (
                                handle.get_title(),
                                handle.get_class_name(),
                                handle.get_process_name(),
                            ) {
                                Some(crate::workspace::persistence::WindowState {
                                    hwnd: format!("{}", hwnd),
                                    process_name: process,
                                    title,
                                    class_name: class,
                                    workspace: workspace.id,
                                })
                            } else {
                                None
                            }
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            // On non-Windows platforms, store minimal window info
                            Some(crate::workspace::persistence::WindowState {
                                hwnd: format!("{}", hwnd),
                                process_name: String::new(),
                                title: String::new(),
                                class_name: String::new(),
                                workspace: workspace.id,
                            })
                        }
                    })
                    .collect(),
                virtual_desktop_id: workspace.virtual_desktop_id
                    .map(|guid| format!("{:?}", guid)),
            };
            
            state.workspaces.push(ws_state);
        }
        
        for (&hwnd, &workspace_id) in &self.window_to_workspace {
            state.window_to_workspace.insert(format!("{}", hwnd), workspace_id);
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
            
            let mut workspace = Workspace::new(
                ws_state.id,
                ws_state.name,
                ws_state.monitor,
                area,
            );
            
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
