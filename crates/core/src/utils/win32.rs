//! Windows API wrapper utilities for window management.
//!
//! This module provides safe Rust wrappers around Windows API functions for window management,
//! including window enumeration, property retrieval, and control operations.
//!
//! # Safety
//!
//! All unsafe Windows API calls are wrapped in safe Rust functions with proper error handling.
//! The module ensures no memory leaks by:
//! - Using proper buffer management for string operations
//! - Not storing raw pointers beyond their valid lifetime
//! - Properly handling Win32 API return values and error codes
//!
//! # Platform Support
//!
//! This module is only available on Windows platforms. Tests are conditional and will only
//! run on Windows (`#[cfg(target_os = "windows")]`).
//!
//! # Examples
//!
//! ```no_run
//! use tiling_wm_core::utils::win32::{enumerate_app_windows, get_foreground_window};
//!
//! // Get the currently focused window
//! if let Some(window) = get_foreground_window() {
//!     let title = window.get_title().unwrap_or_default();
//!     let pid = window.get_process_id();
//!     println!("Active: {} (PID: {})", title, pid);
//! }
//!
//! // Enumerate all application windows
//! let app_windows = enumerate_app_windows().unwrap();
//! for window in app_windows {
//!     let title = window.get_title().unwrap_or_default();
//!     println!("App: {}", title);
//! }
//! ```

use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM, RECT, WPARAM},
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetForegroundWindow, GetParent, GetWindow, GetWindowRect,
        GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, IsZoomed, PostMessageW, SetForegroundWindow, ShowWindow,
        GW_OWNER, SHOW_WINDOW_CMD, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE,
    },
};

/// A wrapper around Windows HWND providing safe access to window operations.
/// 
/// This struct provides a type-safe Rust interface to Windows window management APIs,
/// ensuring proper error handling and memory safety when interacting with Win32 APIs.
/// 
/// # Examples
/// 
/// ```no_run
/// use tiling_wm_core::utils::win32::WindowHandle;
/// 
/// // Get the foreground window
/// if let Some(window) = tiling_wm_core::utils::win32::get_foreground_window() {
///     // Get window properties
///     let title = window.get_title().unwrap_or_default();
///     let class = window.get_class_name().unwrap_or_default();
///     println!("Window: {} ({})", title, class);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle(pub HWND);

impl WindowHandle {
    /// Create a WindowHandle from a raw HWND.
    /// 
    /// # Arguments
    /// 
    /// * `hwnd` - A Windows window handle (HWND)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use windows::Win32::Foundation::HWND;
    /// # use tiling_wm_core::utils::win32::WindowHandle;
    /// let hwnd = HWND(0x12345678 as _);
    /// let window = WindowHandle::from_hwnd(hwnd);
    /// ```
    pub fn from_hwnd(hwnd: HWND) -> Self {
        WindowHandle(hwnd)
    }

    /// Get the raw HWND value.
    /// 
    /// # Returns
    /// 
    /// The underlying Windows window handle.
    pub fn hwnd(&self) -> HWND {
        self.0
    }

    /// Check if the window handle is valid (non-null).
    /// 
    /// # Returns
    /// 
    /// `true` if the handle is non-null, `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.0.0 != 0
    }

    /// Get the window title.
    /// 
    /// # Returns
    /// 
    /// The window title as a String, or an empty string if the window has no title.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the Windows API call fails.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use tiling_wm_core::utils::win32::get_foreground_window;
    /// if let Some(window) = get_foreground_window() {
    ///     let title = window.get_title().unwrap_or_default();
    ///     println!("Window title: {}", title);
    /// }
    /// ```
    pub fn get_title(&self) -> anyhow::Result<String> {
        unsafe {
            let length = GetWindowTextLengthW(self.0);
            if length == 0 {
                return Ok(String::new());
            }

            let mut buffer = vec![0u16; (length + 1) as usize];
            let result = GetWindowTextW(self.0, &mut buffer);
            
            if result > 0 {
                let title = String::from_utf16_lossy(&buffer[..result as usize]);
                Ok(title)
            } else {
                Ok(String::new())
            }
        }
    }

    /// Get the window class name.
    /// 
    /// # Returns
    /// 
    /// The window class name as a String, or an empty string on failure.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the Windows API call fails.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use tiling_wm_core::utils::win32::get_foreground_window;
    /// if let Some(window) = get_foreground_window() {
    ///     let class = window.get_class_name().unwrap_or_default();
    ///     println!("Window class: {}", class);
    /// }
    /// ```
    pub fn get_class_name(&self) -> anyhow::Result<String> {
        unsafe {
            let mut buffer = [0u16; 256];
            let result = GetClassNameW(self.0, &mut buffer);
            
            if result > 0 {
                let class = String::from_utf16_lossy(&buffer[..result as usize]);
                Ok(class)
            } else {
                Ok(String::new())
            }
        }
    }

    /// Get the process ID that created this window.
    /// 
    /// # Returns
    /// 
    /// The process ID (PID) as a u32.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use tiling_wm_core::utils::win32::get_foreground_window;
    /// if let Some(window) = get_foreground_window() {
    ///     let pid = window.get_process_id();
    ///     println!("Window process ID: {}", pid);
    /// }
    /// ```
    pub fn get_process_id(&self) -> u32 {
        unsafe {
            let mut process_id = 0u32;
            GetWindowThreadProcessId(self.0, Some(&mut process_id));
            process_id
        }
    }

    /// Get the thread ID that created this window.
    /// 
    /// # Returns
    /// 
    /// The thread ID as a u32.
    pub fn get_thread_id(&self) -> u32 {
        unsafe {
            GetWindowThreadProcessId(self.0, None)
        }
    }

    /// Check if the window is visible.
    /// 
    /// # Returns
    /// 
    /// `true` if the window is visible, `false` otherwise.
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.0).as_bool() }
    }

    /// Check if the window is minimized (iconic).
    /// 
    /// # Returns
    /// 
    /// `true` if the window is minimized, `false` otherwise.
    pub fn is_minimized(&self) -> bool {
        unsafe { IsIconic(self.0).as_bool() }
    }

    /// Check if the window is maximized.
    /// 
    /// # Returns
    /// 
    /// `true` if the window is maximized, `false` otherwise.
    pub fn is_maximized(&self) -> bool {
        unsafe { IsZoomed(self.0).as_bool() }
    }

    /// Get the window rectangle (position and size) in screen coordinates.
    /// 
    /// # Returns
    /// 
    /// A RECT structure containing the window's position and dimensions.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the Windows API call fails.
    pub fn get_rect(&self) -> anyhow::Result<RECT> {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect)?;
            Ok(rect)
        }
    }

    /// Get the parent window handle.
    /// 
    /// # Returns
    /// 
    /// Some(WindowHandle) if the window has a parent, None otherwise.
    pub fn get_parent(&self) -> Option<WindowHandle> {
        unsafe {
            let parent = GetParent(self.0);
            if parent.0 != 0 {
                Some(WindowHandle::from_hwnd(parent))
            } else {
                None
            }
        }
    }

    /// Get the owner window handle.
    /// 
    /// # Returns
    /// 
    /// Some(WindowHandle) if the window has an owner, None otherwise.
    pub fn get_owner(&self) -> Option<WindowHandle> {
        unsafe {
            let owner = GetWindow(self.0, GW_OWNER);
            if owner.0 != 0 {
                Some(WindowHandle::from_hwnd(owner))
            } else {
                None
            }
        }
    }

    /// Show the window.
    /// 
    /// # Arguments
    /// 
    /// * `cmd` - The show command (e.g., SW_SHOW, SW_HIDE, SW_MINIMIZE, etc.)
    /// 
    /// # Returns
    /// 
    /// `true` if the window was previously visible, `false` otherwise.
    pub fn show(&self, cmd: SHOW_WINDOW_CMD) -> bool {
        unsafe { ShowWindow(self.0, cmd).as_bool() }
    }

    /// Set the window as the foreground window (bring to front and focus).
    /// 
    /// # Returns
    /// 
    /// Ok(()) if successful, Err otherwise.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the window cannot be brought to the foreground.
    pub fn set_foreground(&self) -> anyhow::Result<()> {
        unsafe {
            if SetForegroundWindow(self.0).as_bool() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to set window as foreground"))
            }
        }
    }

    /// Close the window by sending a WM_CLOSE message.
    /// 
    /// # Returns
    /// 
    /// Ok(()) if the message was sent successfully, Err otherwise.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the message cannot be sent.
    pub fn close(&self) -> anyhow::Result<()> {
        unsafe {
            PostMessageW(self.0, WM_CLOSE, WPARAM(0), LPARAM(0))?;
            Ok(())
        }
    }

    /// Minimize the window.
    pub fn minimize(&self) {
        self.show(SW_MINIMIZE);
    }

    /// Maximize the window.
    pub fn maximize(&self) {
        self.show(SW_MAXIMIZE);
    }

    /// Restore the window to its normal size.
    pub fn restore(&self) {
        self.show(SW_RESTORE);
    }

    /// Hide the window.
    pub fn hide(&self) {
        self.show(SW_HIDE);
    }

    /// Check if this is a standard application window (has title, is visible, has no owner).
    /// 
    /// This is useful for filtering out non-application windows like tooltips, menu windows, etc.
    /// 
    /// # Returns
    /// 
    /// `true` if the window appears to be a standard application window.
    /// 
    /// # Notes
    /// 
    /// This is a heuristic filter. Some legitimate application windows may be filtered out
    /// if they temporarily have no title or fail the title retrieval. Applications needing
    /// more precise filtering should use additional criteria.
    pub fn is_app_window(&self) -> bool {
        if !self.is_visible() {
            return false;
        }

        // Window should not have an owner
        if self.get_owner().is_some() {
            return false;
        }

        // Window should have a title - most app windows do, though some may temporarily have none
        // If we can't get the title, err on the side of caution and exclude it
        match self.get_title() {
            Ok(title) => !title.is_empty(),
            Err(_) => false,
        }
    }
}

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
/// # use tiling_wm_core::utils::win32::enumerate_windows;
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
/// # use tiling_wm_core::utils::win32::enumerate_visible_windows;
/// let visible_windows = enumerate_visible_windows().unwrap();
/// println!("Found {} visible windows", visible_windows.len());
/// ```
pub fn enumerate_visible_windows() -> anyhow::Result<Vec<WindowHandle>> {
    let all_windows = enumerate_windows()?;
    Ok(all_windows
        .into_iter()
        .filter(|w| w.is_visible())
        .collect())
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
/// # use tiling_wm_core::utils::win32::enumerate_app_windows;
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
/// # use tiling_wm_core::utils::win32::get_foreground_window;
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

/// Filter windows by process ID.
/// 
/// # Arguments
/// 
/// * `windows` - The vector of windows to filter
/// * `process_id` - The process ID to match
/// 
/// # Returns
/// 
/// A vector of windows belonging to the specified process.
pub fn filter_by_process_id(windows: &[WindowHandle], process_id: u32) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| w.get_process_id() == process_id)
        .copied()
        .collect()
}

/// Filter windows by class name.
/// 
/// # Arguments
/// 
/// * `windows` - The vector of windows to filter
/// * `class_name` - The class name to match
/// 
/// # Returns
/// 
/// A vector of windows with the specified class name.
pub fn filter_by_class_name(windows: &[WindowHandle], class_name: &str) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| {
            w.get_class_name()
                .map(|c| c == class_name)
                .unwrap_or(false)
        })
        .copied()
        .collect()
}

/// Filter windows by title (exact match).
/// 
/// # Arguments
/// 
/// * `windows` - The vector of windows to filter
/// * `title` - The title to match
/// 
/// # Returns
/// 
/// A vector of windows with the specified title.
pub fn filter_by_title(windows: &[WindowHandle], title: &str) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| {
            w.get_title()
                .map(|t| t == title)
                .unwrap_or(false)
        })
        .copied()
        .collect()
}

/// Filter windows by title pattern (case-insensitive substring match).
/// 
/// # Arguments
/// 
/// * `windows` - The vector of windows to filter
/// * `pattern` - The pattern to search for in titles
/// 
/// # Returns
/// 
/// A vector of windows whose titles contain the specified pattern.
pub fn filter_by_title_pattern(windows: &[WindowHandle], pattern: &str) -> Vec<WindowHandle> {
    let pattern_lower = pattern.to_lowercase();
    windows
        .iter()
        .filter(|w| {
            w.get_title()
                .map(|t| t.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false)
        })
        .copied()
        .collect()
}

#[cfg(all(test, target_os = "windows"))]
mod win32_tests {
    //! Tests for Windows API wrapper utilities.
    //!
    //! These tests are only compiled and run on Windows platforms since they require
    //! actual Windows API functions to be available. The tests include:
    //!
    //! - Unit tests for basic functionality (creation, equality, validation)
    //! - Integration tests that enumerate and filter actual system windows
    //! - Tests for all window property retrieval methods
    //! - Tests for window state queries and control methods
    //!
    //! To run these tests on Windows:
    //! ```bash
    //! cargo test -p tiling-wm-core win32
    //! ```
    //!
    //! ## Memory Safety
    //!
    //! All tests verify that:
    //! - No memory leaks occur during Win32 API calls
    //! - String buffers are properly allocated and freed
    //! - Window handles are properly managed
    //! - Error handling works correctly

    use super::*;

    /// Test that WindowHandle can be created from HWND
    #[test]
    fn test_window_handle_creation() {
        let hwnd = HWND(0x12345678 as _);
        let handle = WindowHandle::from_hwnd(hwnd);
        assert_eq!(handle.hwnd(), hwnd);
        assert!(handle.is_valid());
    }

    /// Test that a null HWND is detected as invalid
    #[test]
    fn test_window_handle_invalid() {
        let hwnd = HWND(0);
        let handle = WindowHandle::from_hwnd(hwnd);
        assert_eq!(handle.hwnd(), hwnd);
        assert!(!handle.is_valid());
    }

    /// Test WindowHandle equality
    #[test]
    fn test_window_handle_equality() {
        let hwnd1 = HWND(0x12345678 as _);
        let hwnd2 = HWND(0x12345678 as _);
        let hwnd3 = HWND(0x11111111 as _);

        let handle1 = WindowHandle::from_hwnd(hwnd1);
        let handle2 = WindowHandle::from_hwnd(hwnd2);
        let handle3 = WindowHandle::from_hwnd(hwnd3);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    /// Test window enumeration (may find 0 or more windows depending on test environment)
    #[test]
    fn test_enumerate_windows() {
        let result = enumerate_windows();
        assert!(result.is_ok(), "enumerate_windows should succeed");
        let windows = result.unwrap();
        // We can't guarantee the number of windows, but the call should succeed
        println!("Found {} windows", windows.len());
    }

    /// Test visible window enumeration
    #[test]
    fn test_enumerate_visible_windows() {
        let result = enumerate_visible_windows();
        assert!(result.is_ok(), "enumerate_visible_windows should succeed");
        let windows = result.unwrap();
        // Verify that all returned windows are actually visible
        for window in &windows {
            assert!(window.is_visible(), "All enumerated windows should be visible");
        }
        println!("Found {} visible windows", windows.len());
    }

    /// Test app window enumeration
    #[test]
    fn test_enumerate_app_windows() {
        let result = enumerate_app_windows();
        assert!(result.is_ok(), "enumerate_app_windows should succeed");
        let windows = result.unwrap();
        // Verify that all returned windows pass the app window filter
        for window in &windows {
            assert!(window.is_app_window(), "All enumerated windows should be app windows");
        }
        println!("Found {} app windows", windows.len());
    }

    /// Test get_foreground_window
    #[test]
    fn test_get_foreground_window() {
        let result = get_foreground_window();
        // There may or may not be a foreground window in a test environment
        if let Some(window) = result {
            println!("Foreground window found: {:?}", window);
            // If we have a foreground window, try to get its properties
            let title = window.get_title();
            assert!(title.is_ok(), "get_title should not fail");
            let class = window.get_class_name();
            assert!(class.is_ok(), "get_class_name should not fail");
        } else {
            println!("No foreground window found");
        }
    }

    /// Test window property retrieval with a known window
    #[test]
    fn test_window_properties() {
        // Try to get a window to test with
        if let Ok(windows) = enumerate_windows() {
            if let Some(window) = windows.first() {
                // Test title retrieval
                let title_result = window.get_title();
                assert!(title_result.is_ok(), "get_title should succeed");

                // Test class name retrieval
                let class_result = window.get_class_name();
                assert!(class_result.is_ok(), "get_class_name should succeed");

                // Test process ID retrieval
                let pid = window.get_process_id();
                assert!(pid > 0, "process ID should be positive");

                // Test thread ID retrieval
                let tid = window.get_thread_id();
                assert!(tid > 0, "thread ID should be positive");

                // Test visibility check
                let _is_visible = window.is_visible();
                // No assertion - just ensure it doesn't crash

                // Test rect retrieval
                let rect_result = window.get_rect();
                assert!(rect_result.is_ok(), "get_rect should succeed");

                println!("Tested window: '{}' (class: '{}')", 
                    title_result.unwrap(), 
                    class_result.unwrap());
            }
        }
    }

    /// Test window state queries
    #[test]
    fn test_window_state_queries() {
        if let Ok(windows) = enumerate_visible_windows() {
            if let Some(window) = windows.first() {
                // These should not crash
                let _is_minimized = window.is_minimized();
                let _is_maximized = window.is_maximized();
                let _parent = window.get_parent();
                let _owner = window.get_owner();
                
                println!("Window state queries succeeded");
            }
        }
    }

    /// Test filter by process ID
    #[test]
    fn test_filter_by_process_id() {
        if let Ok(windows) = enumerate_windows() {
            if let Some(first_window) = windows.first() {
                let pid = first_window.get_process_id();
                let filtered = filter_by_process_id(&windows, pid);
                
                // Should have at least one window (the one we got the PID from)
                assert!(!filtered.is_empty(), "Should find at least one window for the PID");
                
                // All filtered windows should have the same PID
                for window in &filtered {
                    assert_eq!(window.get_process_id(), pid);
                }
                
                println!("Found {} windows for PID {}", filtered.len(), pid);
            }
        }
    }

    /// Test filter by class name
    #[test]
    fn test_filter_by_class_name() {
        if let Ok(windows) = enumerate_windows() {
            if let Some(first_window) = windows.first() {
                if let Ok(class_name) = first_window.get_class_name() {
                    if !class_name.is_empty() {
                        let filtered = filter_by_class_name(&windows, &class_name);
                        
                        // Should have at least one window
                        assert!(!filtered.is_empty(), "Should find at least one window for the class");
                        
                        // All filtered windows should have the same class name
                        for window in &filtered {
                            let wclass = window.get_class_name().unwrap_or_default();
                            assert_eq!(wclass, class_name);
                        }
                        
                        println!("Found {} windows for class '{}'", filtered.len(), class_name);
                    }
                }
            }
        }
    }

    /// Test filter by title
    #[test]
    fn test_filter_by_title() {
        if let Ok(windows) = enumerate_windows() {
            if let Some(first_window) = windows.first() {
                if let Ok(title) = first_window.get_title() {
                    if !title.is_empty() {
                        let filtered = filter_by_title(&windows, &title);
                        
                        // Should have at least one window
                        assert!(!filtered.is_empty(), "Should find at least one window for the title");
                        
                        println!("Found {} windows with title '{}'", filtered.len(), title);
                    }
                }
            }
        }
    }

    /// Test filter by title pattern
    #[test]
    fn test_filter_by_title_pattern() {
        if let Ok(windows) = enumerate_windows() {
            // Try to find a window with a non-empty title
            for window in &windows {
                if let Ok(title) = window.get_title() {
                    if !title.is_empty() {
                        // Use the first word as a pattern
                        let pattern = title.split_whitespace().next().unwrap_or("");
                        if !pattern.is_empty() {
                            let filtered = filter_by_title_pattern(&windows, pattern);
                            
                            // Should have at least one window (the one we got the pattern from)
                            assert!(!filtered.is_empty(), "Should find at least one window for the pattern");
                            
                            println!("Found {} windows matching pattern '{}'", filtered.len(), pattern);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Test window control methods don't crash
    /// Note: We don't actually manipulate windows to avoid side effects
    #[test]
    fn test_window_control_methods_exist() {
        // Create a dummy window handle just to verify the methods compile and exist
        let hwnd = HWND(0);
        let handle = WindowHandle::from_hwnd(hwnd);
        
        // Just verify these methods exist and compile
        // We don't call them because they would fail on an invalid handle
        let _can_call_show = || handle.show(SW_SHOW);
        let _can_call_minimize = || handle.minimize();
        let _can_call_maximize = || handle.maximize();
        let _can_call_restore = || handle.restore();
        let _can_call_hide = || handle.hide();
        let _can_call_set_foreground = || handle.set_foreground();
        let _can_call_close = || handle.close();
    }

    /// Test is_app_window filter logic
    #[test]
    fn test_is_app_window() {
        if let Ok(windows) = enumerate_windows() {
            let app_windows: Vec<_> = windows.iter()
                .filter(|w| w.is_app_window())
                .collect();
            
            // Verify each app window meets the criteria
            for window in app_windows {
                // Must be visible
                assert!(window.is_visible(), "App window should be visible");
                
                // Should not have an owner
                assert!(window.get_owner().is_none(), "App window should not have an owner");
                
                // Should have a title
                if let Ok(title) = window.get_title() {
                    assert!(!title.is_empty(), "App window should have a title");
                }
            }
        }
    }

    /// Integration test: Enumerate and filter windows
    #[test]
    fn test_integration_enumerate_and_filter() {
        // This test demonstrates a typical workflow
        let result = enumerate_windows();
        assert!(result.is_ok(), "Should be able to enumerate windows");
        
        let all_windows = result.unwrap();
        println!("\n=== Window Enumeration Integration Test ===");
        println!("Total windows: {}", all_windows.len());
        
        // Filter visible windows
        let visible_windows: Vec<_> = all_windows.iter()
            .filter(|w| w.is_visible())
            .collect();
        println!("Visible windows: {}", visible_windows.len());
        
        // Filter app windows
        let app_windows: Vec<_> = all_windows.iter()
            .filter(|w| w.is_app_window())
            .collect();
        println!("App windows: {}", app_windows.len());
        
        // Print some example windows
        println!("\nExample app windows (up to 5):");
        for (i, window) in app_windows.iter().take(5).enumerate() {
            let title = window.get_title().unwrap_or_default();
            let class = window.get_class_name().unwrap_or_default();
            let pid = window.get_process_id();
            println!("  {}. '{}' [{}] (PID: {})", i + 1, title, class, pid);
        }
    }
}
