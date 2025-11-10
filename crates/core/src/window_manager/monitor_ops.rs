//! Monitor enumeration and management operations.
//!
//! This module handles monitor detection, enumeration, and refresh operations.

use crate::window_manager::{MonitorInfo, Rect, WindowManager};

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

impl WindowManager {
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
    /// use tenraku_core::window_manager::WindowManager;
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
            let rect = Rect::new(0, 0, 1920, 1080);
            self.monitors.push(MonitorInfo::new(
                0,
                0,
                "Primary Monitor".to_string(),
                rect,
                rect,
                1.0,
            ));
        }

        // Sort monitors by position for consistent ordering
        self.monitors
            .sort_by_key(|m| (m.work_area.x, m.work_area.y));

        // Assign sequential IDs
        for (idx, monitor) in self.monitors.iter_mut().enumerate() {
            monitor.id = idx;
        }

        Ok(())
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

    if let Some(monitor_info) = get_monitor_info(hmonitor) {
        monitors.push(monitor_info);
    }

    true.into()
}

#[cfg(target_os = "windows")]
fn get_monitor_info(hmonitor: HMONITOR) -> Option<MonitorInfo> {
    let mut monitor_info_ex = MONITORINFOEXW {
        monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    unsafe {
        if !GetMonitorInfoW(
            hmonitor,
            &mut monitor_info_ex.monitorInfo as *mut _ as *mut _,
        )
        .as_bool()
        {
            return None;
        }
    }

    let work_rect = rect_from_win32(&monitor_info_ex.monitorInfo.rcWork);
    let full_rect = rect_from_win32(&monitor_info_ex.monitorInfo.rcMonitor);
    let device_name = String::from_utf16_lossy(&monitor_info_ex.szDevice)
        .trim_end_matches('\0')
        .to_string();

    Some(MonitorInfo::new(
        0, // Will be assigned later
        hmonitor,
        device_name,
        work_rect,
        full_rect,
        1.0, // DPI scaling - could be enhanced with GetDpiForMonitor API
    ))
}

#[cfg(target_os = "windows")]
fn rect_from_win32(win32_rect: &windows::Win32::Foundation::RECT) -> Rect {
    Rect::new(
        win32_rect.left,
        win32_rect.top,
        win32_rect.right - win32_rect.left,
        win32_rect.bottom - win32_rect.top,
    )
}
