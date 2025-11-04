//! Monitor management for per-monitor workspace support.
//!
//! This module provides monitor tracking and management capabilities:
//! - MonitorInfo: Information about each monitor including assigned workspaces
//! - MonitorManager: Centralized management of all connected monitors

use crate::window_manager::tree::Rect;
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::HMONITOR;

/// Information about a display monitor with workspace tracking.
///
/// Contains details about a monitor's position, size, DPI scaling,
/// and the workspaces assigned to this monitor.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Unique identifier for the monitor (internal)
    pub id: usize,

    /// Monitor handle (Windows specific)
    #[cfg(target_os = "windows")]
    pub handle: HMONITOR,

    /// Monitor handle (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub handle: usize,

    /// Monitor name (e.g., "\\.\DISPLAY1")
    pub name: String,

    /// Work area rectangle (excludes taskbar)
    pub work_area: Rect,

    /// Full area rectangle (includes taskbar)
    pub full_area: Rect,

    /// DPI scale factor (1.0 = 100%, 1.5 = 150%, etc.)
    pub dpi_scale: f32,

    /// Workspace IDs assigned to this monitor
    pub workspaces: Vec<usize>,

    /// Currently active workspace on this monitor
    pub active_workspace: Option<usize>,
}

impl MonitorInfo {
    /// Create a new MonitorInfo instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the monitor
    /// * `handle` - Platform-specific monitor handle
    /// * `name` - Monitor name
    /// * `work_area` - Work area rectangle (excludes taskbar)
    /// * `full_area` - Full area rectangle (includes taskbar)
    /// * `dpi_scale` - DPI scale factor
    ///
    /// # Returns
    ///
    /// A new MonitorInfo with empty workspace assignments.
    #[cfg(target_os = "windows")]
    pub fn new(
        id: usize,
        handle: HMONITOR,
        name: String,
        work_area: Rect,
        full_area: Rect,
        dpi_scale: f32,
    ) -> Self {
        Self {
            id,
            handle,
            name,
            work_area,
            full_area,
            dpi_scale,
            workspaces: Vec::new(),
            active_workspace: None,
        }
    }

    /// Create a new MonitorInfo instance (non-Windows).
    #[cfg(not(target_os = "windows"))]
    pub fn new(
        id: usize,
        handle: usize,
        name: String,
        work_area: Rect,
        full_area: Rect,
        dpi_scale: f32,
    ) -> Self {
        Self {
            id,
            handle,
            name,
            work_area,
            full_area,
            dpi_scale,
            workspaces: Vec::new(),
            active_workspace: None,
        }
    }
}

/// Manages all connected monitors.
///
/// The MonitorManager provides centralized management of display monitors,
/// including enumeration, tracking, and workspace-to-monitor assignments.
pub struct MonitorManager {
    /// All monitors by ID
    pub monitors: HashMap<usize, MonitorInfo>,

    /// Next monitor ID to assign
    next_id: usize,
}

impl MonitorManager {
    /// Create a new MonitorManager.
    ///
    /// # Returns
    ///
    /// A new MonitorManager with no monitors.
    pub fn new() -> Self {
        Self {
            monitors: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a monitor to the manager.
    ///
    /// # Arguments
    ///
    /// * `monitor` - MonitorInfo to add
    ///
    /// # Returns
    ///
    /// The ID assigned to the monitor.
    pub fn add_monitor(&mut self, mut monitor: MonitorInfo) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        monitor.id = id;
        self.monitors.insert(id, monitor);
        id
    }

    /// Get a monitor by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Monitor ID
    ///
    /// # Returns
    ///
    /// Reference to the MonitorInfo, or None if not found.
    pub fn get_by_id(&self, id: usize) -> Option<&MonitorInfo> {
        self.monitors.get(&id)
    }

    /// Get a mutable reference to a monitor by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Monitor ID
    ///
    /// # Returns
    ///
    /// Mutable reference to the MonitorInfo, or None if not found.
    pub fn get_by_id_mut(&mut self, id: usize) -> Option<&mut MonitorInfo> {
        self.monitors.get_mut(&id)
    }

    /// Remove a monitor by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Monitor ID to remove
    ///
    /// # Returns
    ///
    /// The removed MonitorInfo, or None if not found.
    pub fn remove_monitor(&mut self, id: usize) -> Option<MonitorInfo> {
        self.monitors.remove(&id)
    }

    /// Get the number of monitors.
    ///
    /// # Returns
    ///
    /// The number of monitors currently tracked.
    pub fn monitor_count(&self) -> usize {
        self.monitors.len()
    }

    /// Clear all monitors.
    pub fn clear(&mut self) {
        self.monitors.clear();
        self.next_id = 0;
    }

    /// Get all monitor IDs.
    ///
    /// # Returns
    ///
    /// Vector of all monitor IDs.
    pub fn get_monitor_ids(&self) -> Vec<usize> {
        self.monitors.keys().copied().collect()
    }
}

impl Default for MonitorManager {
    fn default() -> Self {
        Self::new()
    }
}
