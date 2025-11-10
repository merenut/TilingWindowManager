//! Core WindowManager struct and initialization.
//!
//! This module contains the WindowManager struct definition and basic
//! initialization and configuration operations.

use crate::config::Config;
use crate::rules::RuleMatcher;
use crate::window_manager::{
    DwindleLayout, FocusManager, LayoutType, MasterLayout, MonitorInfo, TreeNode, WindowRegistry,
};
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

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
/// use tenraku_core::window_manager::WindowManager;
/// use tenraku_core::utils::win32;
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
    /// Window trees for each workspace and monitor ((workspace_id, monitor_idx) -> tree)
    pub(super) trees: HashMap<(usize, usize), TreeNode>,
    /// Currently active workspace ID
    pub(super) active_workspace: usize,
    /// Information about connected monitors
    pub(super) monitors: Vec<MonitorInfo>,
    /// Registry of all managed windows
    pub(super) registry: WindowRegistry,
    /// Dwindle layout configuration
    pub(super) dwindle_layout: DwindleLayout,
    /// Master layout configuration
    pub(super) master_layout: MasterLayout,
    /// Currently active layout type
    pub(super) current_layout: LayoutType,
    /// Rule matcher for window rules
    pub(super) rule_matcher: Option<RuleMatcher>,
    /// Focus manager for focus history
    pub(super) focus_manager: FocusManager,
    /// Flag to prevent recursive retiling
    pub(super) is_tiling: bool,
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
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let wm = WindowManager::new();
    /// ```
    pub fn new() -> Self {
        WindowManager {
            trees: HashMap::new(),
            active_workspace: 1,
            monitors: Vec::new(),
            registry: WindowRegistry::new(),
            dwindle_layout: DwindleLayout::new(),
            master_layout: MasterLayout::new(),
            current_layout: LayoutType::Dwindle,
            rule_matcher: None,
            focus_manager: FocusManager::new(),
            is_tiling: false,
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
    /// use tenraku_core::window_manager::WindowManager;
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
            // Apply outer gaps to work area
            let gaps_out = 10;
            let work_area_with_gaps = crate::window_manager::Rect::new(
                monitor.work_area.x + gaps_out,
                monitor.work_area.y + gaps_out,
                monitor.work_area.width - 2 * gaps_out,
                monitor.work_area.height - 2 * gaps_out,
            );

            for workspace_id in 1..=10 {
                // We don't create trees yet - they'll be created when windows are added
                // Just reserve the workspace IDs for the primary monitor
                self.trees.insert(
                    (workspace_id, 0),
                    TreeNode::new_leaf(HWND(0), work_area_with_gaps),
                );
            }
        }

        Ok(())
    }

    /// Update configuration and rebuild rule matcher.
    ///
    /// This method updates layout settings and rebuilds the rule matcher
    /// with the new window rules from the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The new configuration to apply
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if rule compilation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    /// use tenraku_core::config::ConfigLoader;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    ///
    /// let loader = ConfigLoader::new().unwrap();
    /// let config = loader.load().unwrap();
    /// wm.update_config(&config).unwrap();
    /// ```
    pub fn update_config(&mut self, config: &Config) -> anyhow::Result<()> {
        // Update layout settings
        self.dwindle_layout.ratio = config.layouts.dwindle.split_ratio;
        self.dwindle_layout.smart_split = config.layouts.dwindle.smart_split;
        self.dwindle_layout.no_gaps_when_only = config.layouts.dwindle.no_gaps_when_only;
        self.dwindle_layout.gaps_in = config.general.gaps_in;
        self.dwindle_layout.gaps_out = config.general.gaps_out;

        self.master_layout.master_factor = config.layouts.master.master_factor;
        self.master_layout.master_count = config.layouts.master.master_count;
        self.master_layout.gaps_in = config.general.gaps_in;
        self.master_layout.gaps_out = config.general.gaps_out;

        // Rebuild rule matcher
        self.rule_matcher = Some(RuleMatcher::new(config.window_rules.clone())?);

        tracing::info!("Configuration updated successfully");
        Ok(())
    }

    /// Get the currently active layout type.
    ///
    /// # Returns
    ///
    /// The active layout type.
    pub fn get_current_layout(&self) -> LayoutType {
        self.current_layout
    }

    /// Get a reference to the focus manager.
    ///
    /// # Returns
    ///
    /// A reference to the FocusManager.
    pub fn focus_manager(&self) -> &FocusManager {
        &self.focus_manager
    }

    /// Get a mutable reference to the focus manager.
    ///
    /// # Returns
    ///
    /// A mutable reference to the FocusManager.
    pub fn focus_manager_mut(&mut self) -> &mut FocusManager {
        &mut self.focus_manager
    }

    /// Get a mutable reference to the window registry.
    ///
    /// # Returns
    ///
    /// A mutable reference to the WindowRegistry.
    pub fn registry_mut(&mut self) -> &mut WindowRegistry {
        &mut self.registry
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
    pub(super) fn get_primary_monitor_work_area(&self) -> crate::window_manager::Rect {
        self.monitors
            .first()
            .map(|m| m.work_area)
            .unwrap_or(crate::window_manager::Rect::new(0, 0, 1920, 1080))
    }

    /// Determine which monitor a window is on.
    ///
    /// Uses the Windows API to find which monitor contains the largest
    /// portion of the window.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle
    ///
    /// # Returns
    ///
    /// The monitor index (0-based). Returns 0 if monitor cannot be determined.
    pub(super) fn get_monitor_for_window(&self, hwnd: HWND) -> usize {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};

            unsafe {
                let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

                // Find this monitor in our list
                for (idx, monitor) in self.monitors.iter().enumerate() {
                    if monitor.handle == hmonitor {
                        return idx;
                    }
                }
            }
        }

        // Default to primary monitor if not found or on non-Windows
        0
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
