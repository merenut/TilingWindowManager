//! Window enumeration functions.
//!
//! This module provides functions for enumerating Windows windows.

use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM},
    Win32::UI::WindowsAndMessaging::{EnumWindows, GetForegroundWindow},
};

use super::WindowHandle;

/// Enumerate all top-level windows in the system.
///
/// This function calls the Windows `EnumWindows` API to retrieve all top-level windows,
/// including both visible and hidden windows.
///
/// # Returns
///
/// A vector of `WindowHandle` objects representing all enumerated windows.
///
/// # Errors
///
/// Returns an error if the Windows API call fails.
///
/// # Examples
///
/// ```no_run
/// # use tenraku_core::utils::win32::enumerate_windows;
/// let windows = enumerate_windows().unwrap();
/// for window in windows {
///     if let Ok(title) = window.get_title() {
///         println!("Window: {}", title);
///     }
/// }
/// ```
pub fn enumerate_windows() -> anyhow::Result<Vec<WindowHandle>> {
    let mut windows = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<WindowHandle> as isize),
        )?;
    }

    Ok(windows)
}

/// Callback function for EnumWindows.
///
/// This is an internal callback that gets called for each window during enumeration.
/// It safely converts the LPARAM back to a mutable reference to our vector and adds
/// the window handle to it.
///
/// # Safety
///
/// This function is marked as unsafe because it dereferences a raw pointer.
/// However, it's safe in this context because:
/// - The pointer is created from a valid mutable reference in `enumerate_windows`
/// - The lifetime of the reference is controlled by the `enumerate_windows` function
/// - Windows guarantees that the callback will not be called after `EnumWindows` returns
///
/// ## Safety Requirements for Callers
///
/// This function must only be called by Windows' `EnumWindows` with an LPARAM
/// that points to a valid `Vec<WindowHandle>` for the duration of enumeration.
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowHandle>);
    windows.push(WindowHandle::from_hwnd(hwnd));
    true.into()
}

/// Enumerate only visible windows.
///
/// This is a convenience function that filters the results of `enumerate_windows`
/// to include only windows that are currently visible.
///
/// # Returns
///
/// A vector of `WindowHandle` objects representing visible windows.
///
/// # Errors
///
/// Returns an error if the Windows API call fails.
///
/// # Examples
///
/// ```no_run
/// # use tenraku_core::utils::win32::enumerate_visible_windows;
/// let visible_windows = enumerate_visible_windows().unwrap();
/// println!("Found {} visible windows", visible_windows.len());
/// ```
pub fn enumerate_visible_windows() -> anyhow::Result<Vec<WindowHandle>> {
    let all_windows = enumerate_windows()?;
    Ok(all_windows.into_iter().filter(|w| w.is_visible()).collect())
}

/// Enumerate only application windows.
///
/// This function filters windows to include only those that appear to be
/// standard application windows (visible, with title, no owner).
///
/// # Returns
///
/// A vector of `WindowHandle` objects representing application windows.
///
/// # Errors
///
/// Returns an error if the Windows API call fails.
///
/// # Examples
///
/// ```no_run
/// # use tenraku_core::utils::win32::enumerate_app_windows;
/// let app_windows = enumerate_app_windows().unwrap();
/// for window in app_windows {
///     if let Ok(title) = window.get_title() {
///         println!("Application: {}", title);
///     }
/// }
/// ```
pub fn enumerate_app_windows() -> anyhow::Result<Vec<WindowHandle>> {
    let all_windows = enumerate_windows()?;
    Ok(all_windows
        .into_iter()
        .filter(|w| w.is_app_window())
        .collect())
}

/// Get the currently focused foreground window.
///
/// # Returns
///
/// Some(WindowHandle) if there is a foreground window, None otherwise.
///
/// # Examples
///
/// ```no_run
/// # use tenraku_core::utils::win32::get_foreground_window;
/// if let Some(window) = get_foreground_window() {
///     let title = window.get_title().unwrap_or_default();
///     println!("Active window: {}", title);
/// }
/// ```
pub fn get_foreground_window() -> Option<WindowHandle> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 != 0 {
            Some(WindowHandle::from_hwnd(hwnd))
        } else {
            None
        }
    }
}
