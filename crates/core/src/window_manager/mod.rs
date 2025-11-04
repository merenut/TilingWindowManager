//! Window manager module for managing window trees, workspaces, and monitors.
//!
//! This module provides the core WindowManager struct that orchestrates:
//! - Window trees for each workspace
//! - Workspace management and switching
//! - Monitor information and DPI handling
//! - Window filtering and management rules
//!
//! # Example
//!
//! ```no_run
//! use tiling_wm_core::window_manager::WindowManager;
//!
//! let mut wm = WindowManager::new();
//! wm.initialize().expect("Failed to initialize window manager");
//!
//! // The window manager is now ready to manage windows
//! ```

pub mod tree;

#[cfg(test)]
mod tree_tests;

pub use tree::{TreeNode, Rect, Split};

use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use crate::utils::win32::WindowHandle;

/// Information about a display monitor.
///
/// Contains details about a monitor's position, size, and DPI scaling.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Unique identifier for the monitor (internal)
    pub id: usize,
    /// Monitor name (e.g., "\\.\DISPLAY1")
    pub name: String,
    /// Work area rectangle (excludes taskbar)
    pub work_area: Rect,
    /// DPI scale factor (1.0 = 100%, 1.5 = 150%, etc.)
    pub dpi_scale: f32,
}

/// Central window manager that coordinates windows, workspaces, and monitors.
///
/// The WindowManager maintains:
/// - A collection of window trees, one per workspace
/// - The currently active workspace
/// - Information about all connected monitors
/// - A registry of managed windows
///
/// # Example
///
/// ```no_run
/// use tiling_wm_core::window_manager::WindowManager;
/// use tiling_wm_core::utils::win32;
///
/// let mut wm = WindowManager::new();
/// wm.initialize().expect("Failed to initialize");
///
/// // Enumerate windows and manage them
/// let windows = win32::enumerate_app_windows().unwrap();
/// for window in windows {
///     if wm.should_manage_window(&window).unwrap_or(false) {
///         wm.manage_window(window).ok();
///     }
/// }
/// ```
pub struct WindowManager {
    /// Window trees for each workspace (workspace_id -> tree)
    trees: HashMap<usize, TreeNode>,
    /// Currently active workspace ID
    active_workspace: usize,
    /// Information about connected monitors
    monitors: Vec<MonitorInfo>,
    /// Map of window handles to workspace IDs (hwnd.0 -> workspace_id)
    managed_windows: HashMap<isize, usize>,
}

impl WindowManager {
    /// Create a new WindowManager instance.
    ///
    /// The window manager starts with no workspaces or monitors.
    /// Call `initialize()` to set up initial state.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let wm = WindowManager::new();
    /// ```
    pub fn new() -> Self {
        WindowManager {
            trees: HashMap::new(),
            active_workspace: 1,
            monitors: Vec::new(),
            managed_windows: HashMap::new(),
        }
    }

    /// Initialize the window manager by enumerating monitors and creating workspaces.
    ///
    /// This should be called once after creating the WindowManager.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if monitor enumeration fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().expect("Failed to initialize window manager");
    /// ```
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        // Enumerate monitors
        self.refresh_monitors()?;

        // Create initial workspace trees for the first few workspaces
        // Use the first monitor's work area for initial tree rectangles
        if let Some(monitor) = self.monitors.first() {
            for workspace_id in 1..=10 {
                // We don't create trees yet - they'll be created when windows are added
                // Just reserve the workspace IDs
                self.trees.insert(workspace_id, TreeNode::new_leaf(HWND(0), monitor.work_area));
            }
        }

        Ok(())
    }

    /// Refresh the list of connected monitors.
    ///
    /// This enumerates all display monitors using the Windows API and updates
    /// the internal monitor information.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if enumeration fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.refresh_monitors().expect("Failed to refresh monitors");
    /// ```
    pub fn refresh_monitors(&mut self) -> anyhow::Result<()> {
        self.monitors.clear();

        #[cfg(target_os = "windows")]
        unsafe {
            // Safety: We create a raw pointer to our Vec<MonitorInfo> and pass it to
            // EnumDisplayMonitors. This is safe because:
            // - The pointer is valid for the duration of the EnumDisplayMonitors call
            // - The callback is synchronous and won't be called after the function returns
            // - We maintain exclusive access to self.monitors during the enumeration
            let monitors_ptr = &mut self.monitors as *mut Vec<MonitorInfo>;
            
            let result = EnumDisplayMonitors(
                HDC(0),
                None,
                Some(enum_monitors_callback),
                windows::Win32::Foundation::LPARAM(monitors_ptr as isize),
            );
            
            if !result.as_bool() {
                return Err(anyhow::anyhow!("Failed to enumerate display monitors"));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Stub implementation for non-Windows platforms (for testing)
            self.monitors.push(MonitorInfo {
                id: 0,
                name: "Primary Monitor".to_string(),
                work_area: Rect::new(0, 0, 1920, 1080),
                dpi_scale: 1.0,
            });
        }

        // Sort monitors by position for consistent ordering
        self.monitors.sort_by_key(|m| (m.work_area.x, m.work_area.y));

        // Assign sequential IDs
        for (idx, monitor) in self.monitors.iter_mut().enumerate() {
            monitor.id = idx;
        }

        Ok(())
    }

    /// Check if a window should be managed by the window manager.
    ///
    /// This filters out windows that should not be tiled, such as:
    /// - Invisible windows
    /// - Popup windows
    /// - Tool windows
    /// - Windows without owners or parents in special cases
    ///
    /// # Arguments
    ///
    /// * `window` - The window to check
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the window should be managed, `Ok(false)` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    /// use tiling_wm_core::utils::win32;
    ///
    /// let wm = WindowManager::new();
    /// let windows = win32::enumerate_windows().unwrap();
    /// 
    /// for window in windows {
    ///     if wm.should_manage_window(&window).unwrap_or(false) {
    ///         println!("Should manage: {}", window.get_title().unwrap_or_default());
    ///     }
    /// }
    /// ```
    pub fn should_manage_window(&self, window: &WindowHandle) -> anyhow::Result<bool> {
        // Use the is_app_window heuristic from WindowHandle
        // This already filters for visible windows with titles and no owners
        Ok(window.is_app_window())
    }

    /// Add a window to be managed by the window manager.
    ///
    /// The window is added to the current workspace's tree and tiled accordingly.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to manage
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    /// use tiling_wm_core::utils::win32;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    ///
    /// if let Some(window) = win32::get_foreground_window() {
    ///     if wm.should_manage_window(&window).unwrap_or(false) {
    ///         wm.manage_window(window).ok();
    ///     }
    /// }
    /// ```
    pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();
        
        // Check if already managed
        if self.managed_windows.contains_key(&hwnd.0) {
            return Ok(());
        }

        // Get or create the tree for the active workspace
        let workspace_id = self.active_workspace;
        let work_area = self.get_primary_monitor_work_area();

        if let Some(tree) = self.trees.get(&workspace_id) {
            // Check if this is an empty placeholder tree (HWND(0))
            if tree.hwnd() == Some(HWND(0)) {
                // Replace with the new window
                let new_tree = TreeNode::new_leaf(hwnd, work_area);
                self.trees.insert(workspace_id, new_tree);
            } else {
                // Insert into existing tree
                let tree = self.trees.remove(&workspace_id).unwrap();
                let new_tree = tree.insert(hwnd, Split::Horizontal);
                self.trees.insert(workspace_id, new_tree);
            }
        } else {
            // Create new tree for this workspace
            let new_tree = TreeNode::new_leaf(hwnd, work_area);
            self.trees.insert(workspace_id, new_tree);
        }

        // Track the managed window
        self.managed_windows.insert(hwnd.0, workspace_id);

        // Apply the layout
        self.tile_workspace(workspace_id)?;

        Ok(())
    }

    /// Remove a window from management.
    ///
    /// The window is removed from its workspace tree and the layout is re-applied.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to unmanage
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    /// use windows::Win32::Foundation::HWND;
    /// use tiling_wm_core::utils::win32::WindowHandle;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    ///
    /// let window = WindowHandle::from_hwnd(HWND(12345 as _));
    /// wm.unmanage_window(&window).ok();
    /// ```
    pub fn unmanage_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();
        
        // Find which workspace this window belongs to
        if let Some(&workspace_id) = self.managed_windows.get(&hwnd.0) {
            // Remove from the tree
            if let Some(tree) = self.trees.remove(&workspace_id) {
                if let Some(new_tree) = tree.remove(hwnd) {
                    self.trees.insert(workspace_id, new_tree);
                    // Re-tile the workspace
                    self.tile_workspace(workspace_id)?;
                } else {
                    // Tree is now empty, create an empty placeholder
                    let work_area = self.get_primary_monitor_work_area();
                    self.trees.insert(workspace_id, TreeNode::new_leaf(HWND(0), work_area));
                }
            }

            // Remove from managed windows
            self.managed_windows.remove(&hwnd.0);
        }

        Ok(())
    }

    /// Apply the tiling layout to a workspace.
    ///
    /// This recalculates and applies window positions for all windows in the workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace to tile
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if layout application fails.
    pub fn tile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if let Some(tree) = self.trees.get(&workspace_id) {
            // Don't tile empty placeholder trees
            if tree.hwnd() != Some(HWND(0)) {
                tree.apply_layout(5, 10)?;
            }
        }
        Ok(())
    }

    /// Switch to a different workspace.
    ///
    /// This hides windows in the current workspace and shows windows in the target workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace to switch to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    /// wm.switch_workspace(2).ok();
    /// ```
    pub fn switch_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if workspace_id == self.active_workspace {
            return Ok(());
        }

        // Hide windows in current workspace
        #[cfg(target_os = "windows")]
        if let Some(current_tree) = self.trees.get(&self.active_workspace) {
            for (hwnd, _) in current_tree.collect() {
                if hwnd.0 != 0 {
                    WindowHandle::from_hwnd(hwnd).hide();
                }
            }
        }

        // Show windows in target workspace
        if let Some(target_tree) = self.trees.get(&workspace_id) {
            for (hwnd, _) in target_tree.collect() {
                if hwnd.0 != 0 {
                    #[cfg(target_os = "windows")]
                    {
                        use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
                        WindowHandle::from_hwnd(hwnd).show(SW_SHOW);
                    }
                }
            }
        }

        self.active_workspace = workspace_id;

        // Re-tile the new workspace to ensure proper layout
        self.tile_workspace(workspace_id)?;

        Ok(())
    }

    /// Get the currently active workspace ID.
    ///
    /// # Returns
    ///
    /// The active workspace ID (typically 1-10).
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::WindowManager;
    ///
    /// let wm = WindowManager::new();
    /// assert_eq!(wm.get_active_workspace(), 1);
    /// ```
    pub fn get_active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// Get a reference to a workspace's tree.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// `Some(&TreeNode)` if the workspace exists, `None` otherwise.
    pub fn get_workspace_tree(&self, workspace_id: usize) -> Option<&TreeNode> {
        self.trees.get(&workspace_id)
    }

    /// Get a mutable reference to a workspace's tree.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// `Some(&mut TreeNode)` if the workspace exists, `None` otherwise.
    pub fn get_workspace_tree_mut(&mut self, workspace_id: usize) -> Option<&mut TreeNode> {
        self.trees.get_mut(&workspace_id)
    }

    /// Get the list of connected monitors.
    ///
    /// # Returns
    ///
    /// A slice of MonitorInfo structs.
    pub fn get_monitors(&self) -> &[MonitorInfo] {
        &self.monitors
    }

    /// Helper function to get the primary monitor's work area.
    fn get_primary_monitor_work_area(&self) -> Rect {
        self.monitors
            .first()
            .map(|m| m.work_area)
            .unwrap_or(Rect::new(0, 0, 1920, 1080))
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback function for EnumDisplayMonitors.
///
/// # Safety
///
/// This function must only be called by Windows' EnumDisplayMonitors with an LPARAM
/// that points to a valid Vec<MonitorInfo> for the duration of enumeration.
/// 
/// Safety invariants:
/// - lparam must be a valid pointer to a Vec<MonitorInfo>
/// - The Vec must remain valid for the entire callback execution
/// - No other code accesses the Vec during enumeration (enforced by &mut borrow)
#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut windows::Win32::Foundation::RECT,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    // Safety: lparam is guaranteed to be a valid pointer to Vec<MonitorInfo>
    // by the contract of this callback (only called by our refresh_monitors method)
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo as *mut _ as *mut _).as_bool() {
        let work_area = &monitor_info.monitorInfo.rcWork;
        let device_name = String::from_utf16_lossy(&monitor_info.szDevice)
            .trim_end_matches('\0')
            .to_string();

        monitors.push(MonitorInfo {
            id: 0, // Will be assigned later
            name: device_name,
            work_area: Rect::new(
                work_area.left,
                work_area.top,
                work_area.right - work_area.left,
                work_area.bottom - work_area.top,
            ),
            // Note: DPI scaling is set to 1.0 for now. Full DPI detection using
            // GetDpiForMonitor API can be added in a future enhancement.
            dpi_scale: 1.0,
        });
    }

    true.into()
}

#[cfg(test)]
mod window_manager_tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new();
        assert_eq!(wm.get_active_workspace(), 1);
        assert_eq!(wm.get_monitors().len(), 0);
    }

    #[test]
    fn test_window_manager_default() {
        let wm = WindowManager::default();
        assert_eq!(wm.get_active_workspace(), 1);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_window_manager_initialization() {
        let mut wm = WindowManager::new();
        let result = wm.initialize();
        assert!(result.is_ok());
        // Should have at least one monitor on Windows
        assert!(!wm.get_monitors().is_empty());
    }

    #[test]
    fn test_workspace_tree_access() {
        let mut wm = WindowManager::new();
        wm.initialize().ok();
        
        // Should be able to access workspace 1
        assert!(wm.get_workspace_tree(1).is_some());
        assert!(wm.get_workspace_tree_mut(1).is_some());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_should_manage_window() {
        use crate::utils::win32;
        
        let wm = WindowManager::new();
        
        // Test with actual app windows
        if let Ok(windows) = win32::enumerate_app_windows() {
            for window in windows.iter().take(3) {
                let result = wm.should_manage_window(window);
                assert!(result.is_ok());
                // App windows should generally be manageable
                if result.unwrap() {
                    println!("Would manage: {}", window.get_title().unwrap_or_default());
                }
            }
        }
    }

    #[test]
    fn test_switch_workspace() {
        let mut wm = WindowManager::new();
        wm.initialize().ok();
        
        assert_eq!(wm.get_active_workspace(), 1);
        
        wm.switch_workspace(2).ok();
        assert_eq!(wm.get_active_workspace(), 2);
        
        wm.switch_workspace(1).ok();
        assert_eq!(wm.get_active_workspace(), 1);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_refresh_monitors() {
        let mut wm = WindowManager::new();
        let result = wm.refresh_monitors();
        assert!(result.is_ok());
        
        // Should detect at least one monitor
        assert!(!wm.get_monitors().is_empty());
        
        // Check monitor IDs are sequential
        for (idx, monitor) in wm.get_monitors().iter().enumerate() {
            assert_eq!(monitor.id, idx);
        }
    }

    // Integration test for managing and unmanaging windows
    #[test]
    #[ignore]
    #[cfg(target_os = "windows")]
    fn test_manage_and_unmanage_window() {
        use crate::utils::win32;
        
        let mut wm = WindowManager::new();
        wm.initialize().expect("Failed to initialize");
        
        // Find a manageable window
        let windows = win32::enumerate_app_windows().expect("Failed to enumerate windows");
        
        if let Some(window) = windows.first() {
            if wm.should_manage_window(window).unwrap_or(false) {
                // Manage the window
                let result = wm.manage_window(*window);
                assert!(result.is_ok());
                
                // Verify it's tracked
                assert!(wm.managed_windows.contains_key(&window.hwnd().0));
                
                // Unmanage the window
                let result = wm.unmanage_window(window);
                assert!(result.is_ok());
                
                // Verify it's no longer tracked
                assert!(!wm.managed_windows.contains_key(&window.hwnd().0));
            }
        }
    }
}
