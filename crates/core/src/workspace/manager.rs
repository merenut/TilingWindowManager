//! Workspace management core data structures.
//!
//! This module provides the core data structures for managing workspaces:
//! - `Workspace`: Represents a single workspace with its windows and layout tree
//! - `WorkspaceConfig`: Configuration for the workspace system
//! - `WorkspaceManager`: Manages all workspaces and window-to-workspace mappings

use crate::window_manager::tree::{Rect, Split, TreeNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub virtual_desktop_id: Option<windows::core::GUID>,

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
}
