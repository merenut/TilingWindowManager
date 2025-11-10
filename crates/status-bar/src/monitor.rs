//! Multi-monitor support for the status bar.
//!
//! This module provides functionality for enumerating and managing multiple monitors,
//! including monitor information, position calculation, and primary monitor detection.

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO, HDC,
};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};

#[cfg(target_os = "windows")]
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use once_cell::sync::Lazy;

/// Global storage for enumerated monitors during enumeration callback
#[cfg(target_os = "windows")]
static MONITORS: Lazy<Mutex<Vec<MonitorInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Information about a monitor/display
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Handle to the monitor
    #[cfg(target_os = "windows")]
    pub handle: HMONITOR,
    /// Placeholder handle for non-Windows systems
    #[cfg(not(target_os = "windows"))]
    pub handle: usize,
    /// Work area of the monitor (x, y, width, height)
    /// This is the area excluding taskbars and other system UI
    pub work_area: (i32, i32, u32, u32),
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

/// Enumerate all monitors connected to the system
///
/// # Returns
/// A vector of `MonitorInfo` for each connected monitor
///
/// # Examples
/// ```no_run
/// use tenraku_bar::monitor::enumerate_monitors;
/// let monitors = enumerate_monitors();
/// for monitor in monitors {
///     println!("Monitor: {:?}", monitor);
/// }
/// ```
#[cfg(target_os = "windows")]
pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    tracing::debug!("Starting monitor enumeration");
    
    // Clear any previous monitors
    {
        let mut monitors = MONITORS.lock().unwrap();
        monitors.clear();
        tracing::debug!("Cleared monitor list");
    }
    
    unsafe {
        tracing::debug!("Calling EnumDisplayMonitors");
        let result = EnumDisplayMonitors(
            HDC(0),
            None,
            Some(monitor_enum_proc),
            LPARAM(0),
        );
        tracing::debug!("EnumDisplayMonitors returned: {:?}", result);
    }
    
    let monitors = MONITORS.lock().unwrap();
    tracing::debug!("Found {} monitors", monitors.len());
    monitors.clone()
}

/// Non-Windows implementation returns a single default monitor
#[cfg(not(target_os = "windows"))]
pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    vec![MonitorInfo {
        handle: 0,
        work_area: (0, 0, 1920, 1080),
        is_primary: true,
    }]
}

/// Callback function for EnumDisplayMonitors
///
/// This function is called by Windows for each monitor during enumeration.
#[cfg(target_os = "windows")]
extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    unsafe {
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            let work_area = monitor_info.rcWork;
            // MONITORINFOF_PRIMARY = 1
            let is_primary = (monitor_info.dwFlags & 1) != 0;
            
            let info = MonitorInfo {
                handle: hmonitor,
                work_area: (
                    work_area.left,
                    work_area.top,
                    (work_area.right - work_area.left) as u32,
                    (work_area.bottom - work_area.top) as u32,
                ),
                is_primary,
            };
            
            MONITORS.lock().unwrap().push(info);
        }
    }
    
    BOOL(1) // Continue enumeration
}

/// Get the primary monitor
///
/// # Returns
/// The `MonitorInfo` for the primary monitor, or `None` if no monitors are found
///
/// # Examples
/// ```no_run
/// use tenraku_bar::monitor::get_primary_monitor;
/// if let Some(primary) = get_primary_monitor() {
///     println!("Primary monitor work area: {:?}", primary.work_area);
/// }
/// ```
pub fn get_primary_monitor() -> Option<MonitorInfo> {
    enumerate_monitors()
        .into_iter()
        .find(|m| m.is_primary)
}

/// Get the number of connected monitors
///
/// # Returns
/// The count of monitors
///
/// # Examples
/// ```no_run
/// use tenraku_bar::monitor::get_monitor_count;
/// println!("Number of monitors: {}", get_monitor_count());
/// ```
pub fn get_monitor_count() -> usize {
    enumerate_monitors().len()
}

/// Get a specific monitor by index
///
/// # Arguments
/// * `index` - Zero-based index of the monitor
///
/// # Returns
/// The `MonitorInfo` for the requested monitor, or `None` if the index is out of bounds
///
/// # Examples
/// ```no_run
/// use tenraku_bar::monitor::get_monitor_by_index;
/// if let Some(monitor) = get_monitor_by_index(0) {
///     println!("Monitor 0: {:?}", monitor);
/// }
/// ```
pub fn get_monitor_by_index(index: usize) -> Option<MonitorInfo> {
    enumerate_monitors().get(index).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_monitors_returns_vec() {
        let monitors = enumerate_monitors();
        // Should return a vector (always has at least one monitor in the mock)
        #[cfg(not(target_os = "windows"))]
        assert_eq!(monitors.len(), 1);
        // On Windows, just verify we get a Vec (count may vary)
    }

    #[test]
    fn test_get_monitor_count() {
        let count = get_monitor_count();
        // Should return a positive count (at least 1 in mock)
        #[cfg(not(target_os = "windows"))]
        assert_eq!(count, 1);
        // On Windows, just verify the function works (count may vary)
    }

    #[test]
    fn test_get_primary_monitor_on_system_with_monitors() {
        // On a system with monitors, this should return Some
        // In headless environments, it may return None
        let primary = get_primary_monitor();
        if primary.is_some() {
            let monitor = primary.unwrap();
            assert!(monitor.is_primary);
            // Work area should have non-zero dimensions
            assert!(monitor.work_area.2 > 0);
            assert!(monitor.work_area.3 > 0);
        }
    }

    #[test]
    fn test_get_monitor_by_index_zero() {
        // Attempt to get the first monitor
        let monitor = get_monitor_by_index(0);
        // May be None in headless environments
        if let Some(m) = monitor {
            // If we get a monitor, it should have valid dimensions
            assert!(m.work_area.2 > 0);
            assert!(m.work_area.3 > 0);
        }
    }

    #[test]
    fn test_get_monitor_by_index_out_of_bounds() {
        // Requesting a very high index should return None
        let monitor = get_monitor_by_index(999);
        assert!(monitor.is_none());
    }

    #[test]
    fn test_monitor_info_work_area_dimensions() {
        let monitors = enumerate_monitors();
        for monitor in monitors {
            // Width and height should be positive
            assert!(monitor.work_area.2 > 0, "Monitor width should be positive");
            assert!(monitor.work_area.3 > 0, "Monitor height should be positive");
        }
    }

    #[test]
    fn test_at_most_one_primary_monitor() {
        let monitors = enumerate_monitors();
        let primary_count = monitors.iter().filter(|m| m.is_primary).count();
        // There should be at most one primary monitor
        assert!(primary_count <= 1, "There should be at most one primary monitor");
    }

    #[test]
    fn test_monitor_enumeration_is_consistent() {
        // Enumerate twice and check we get the same count
        let count1 = get_monitor_count();
        let count2 = get_monitor_count();
        assert_eq!(count1, count2, "Monitor count should be consistent");
    }
}
