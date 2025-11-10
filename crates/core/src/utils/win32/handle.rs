//! WindowHandle struct and implementation.
//!
//! This module provides the WindowHandle wrapper around Windows HWND with safe
//! operations for window properties and control.

use windows::{
    Win32::Foundation::{HWND, LPARAM, RECT, WPARAM},
    Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetParent, GetWindow, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed, PostMessageW,
        SetForegroundWindow, SetWindowPos, ShowWindow, GW_OWNER, HWND_TOP, SHOW_WINDOW_CMD,
        SWP_NOZORDER, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE,
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
/// use tenraku_core::utils::win32::WindowHandle;
///
/// // Get the foreground window
/// if let Some(window) = tenraku_core::utils::win32::get_foreground_window() {
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
    /// # use tenraku_core::utils::win32::WindowHandle;
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
        self.0 .0 != 0
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
    /// # use tenraku_core::utils::win32::get_foreground_window;
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
    /// # use tenraku_core::utils::win32::get_foreground_window;
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
    /// # use tenraku_core::utils::win32::get_foreground_window;
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tenraku_core::utils::win32::get_foreground_window;
    /// if let Some(window) = get_foreground_window() {
    ///     let tid = window.get_thread_id();
    ///     println!("Window thread ID: {}", tid);
    /// }
    /// ```
    pub fn get_thread_id(&self) -> u32 {
        unsafe { GetWindowThreadProcessId(self.0, None) }
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
    ///
    /// This method requests the window to be minimized using the Windows API.
    /// The result of the underlying API call (a boolean indicating success) is silently discarded.
    /// This method does not report success or failure.
    pub fn minimize(&self) {
        self.show(SW_MINIMIZE);
    }

    /// Maximize the window.
    ///
    /// This method requests the window to be maximized using the Windows API.
    /// The result of the underlying API call (a boolean indicating success) is silently discarded.
    /// This method does not report success or failure.
    pub fn maximize(&self) {
        self.show(SW_MAXIMIZE);
    }

    /// Restore the window to its normal size.
    ///
    /// This method requests the window to be restored to its normal size using the Windows API.
    /// The result of the underlying API call (a boolean indicating success) is silently discarded.
    /// This method does not report success or failure.
    pub fn restore(&self) {
        self.show(SW_RESTORE);
    }

    /// Hide the window.
    ///
    /// This method requests the window to be hidden using the Windows API.
    /// The result of the underlying API call (a boolean indicating success) is silently discarded.
    /// This method does not report success or failure.
    pub fn hide(&self) {
        self.show(SW_HIDE);
    }

    /// Get the process name for this window.
    ///
    /// # Returns
    ///
    /// The process name as a String, or an error if retrieval fails.
    pub fn get_process_name(&self) -> anyhow::Result<String> {
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
            PROCESS_QUERY_LIMITED_INFORMATION,
        };

        let process_id = self.get_process_id();
        if process_id == 0 {
            return Ok(String::new());
        }

        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)?;

            // Query the full process image name
            let mut buffer = vec![0u16; 260]; // MAX_PATH
            let mut size = buffer.len() as u32;

            if QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_FORMAT(0),
                windows::core::PWSTR(buffer.as_mut_ptr()),
                &mut size,
            )
            .is_ok()
            {
                let path = String::from_utf16_lossy(&buffer[..size as usize]);
                // Extract just the filename from the path
                if let Some(filename) = path.split('\\').next_back() {
                    return Ok(filename.to_string());
                }
                Ok(path)
            } else {
                Ok(String::new())
            }
        }
    }

    /// Set the window position and size.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the window's top-left corner
    /// * `y` - The y-coordinate of the window's top-left corner
    /// * `width` - The width of the window
    /// * `height` - The height of the window
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    pub fn set_pos(&self, x: i32, y: i32, width: i32, height: i32) -> anyhow::Result<()> {
        unsafe {
            SetWindowPos(self.0, HWND_TOP, x, y, width, height, SWP_NOZORDER)?;
        }

        Ok(())
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
    /// This is a heuristic filter that uses common characteristics of application windows.
    /// While most application windows have non-empty titles, some may temporarily lack a title
    /// during initialization or in edge cases. Such windows will be excluded by this filter.
    /// If you need more precise filtering, consider using additional criteria or combining
    /// this with other filter functions.
    pub fn is_app_window(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, GWL_EXSTYLE, GWL_STYLE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW, WS_POPUP,
        };

        if !self.is_visible() {
            return false;
        }

        // Window should not have an owner (unless it has WS_EX_APPWINDOW)
        let has_owner = self.get_owner().is_some();

        unsafe {
            let ex_style = GetWindowLongPtrW(self.0, GWL_EXSTYLE) as u32;
            let style = GetWindowLongPtrW(self.0, GWL_STYLE) as u32;

            // Exclude tool windows (these are typically floating palettes, tooltips, etc.)
            if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
                return false;
            }

            // Exclude windows with WS_EX_NOACTIVATE (these can't receive keyboard focus)
            if (ex_style & WS_EX_NOACTIVATE.0) != 0 {
                return false;
            }

            // If window has an owner, only accept if it explicitly has WS_EX_APPWINDOW
            if has_owner && (ex_style & WS_EX_APPWINDOW.0) == 0 {
                return false;
            }

            // Exclude popup windows without WS_EX_APPWINDOW
            // (these are typically menus, tooltips, notification popups)
            if (style & WS_POPUP.0) != 0 && (ex_style & WS_EX_APPWINDOW.0) == 0 {
                return false;
            }
        }

        // Check for common system window class names
        if let Ok(class) = self.get_class_name() {
            let class_lower = class.to_lowercase();

            // Filter out Windows system UI elements
            let system_classes = [
                "windows.ui.core.corewindow", // UWP background windows
                "applicationframewindow",     // UWP app frames (often invisible containers)
                "shell_traywnd",              // System tray
                "progman",                    // Desktop window
                "workerw",                    // Desktop worker windows
                "imewnd",                     // IME windows
                "tooltips_class32",           // Tooltips
                "msctls_statusbar32",         // Status bars
                "button",                     // Standalone buttons
                "windows.internal.shell",     // Shell internal windows
                "xamlislandrootwindow",       // XAML Islands
            ];

            for system_class in &system_classes {
                if class_lower.contains(system_class) {
                    return false;
                }
            }
        }

        // Check for common system window titles
        if let Ok(title) = self.get_title() {
            if title.is_empty() {
                return false;
            }

            let title_lower = title.to_lowercase();

            // Filter out notification/system windows by title patterns
            let system_titles = [
                "notification",
                "gdi+ window",
                "default ime",
                "msctfime ui",
                "fmodex window",        // Audio library window
                "chrome legacy window", // Chrome background processes
            ];

            for system_title in &system_titles {
                if title_lower.contains(system_title) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Check if this window is registered as a Windows AppBar.
    ///
    /// AppBars are system UI elements (like taskbar, status bars) that reserve
    /// space at screen edges and should not be managed by tiling window managers.
    ///
    /// # Returns
    ///
    /// `true` if the window is registered as an AppBar.
    pub fn is_app_bar(&self) -> bool {
        use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETSTATE, APPBARDATA};

        unsafe {
            // Create an APPBARDATA structure for this window
            let mut abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                hWnd: self.0,
                uCallbackMessage: 0,
                uEdge: 0,
                rc: windows::Win32::Foundation::RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                lParam: windows::Win32::Foundation::LPARAM(0),
            };

            // Try to query AppBar state
            // For registered AppBars, this returns the state flags
            // For non-AppBar windows, behavior is undefined but typically returns 0 or fails
            let state = SHAppBarMessage(ABM_GETSTATE, &mut abd);

            // Additionally check if the window class suggests it's an AppBar
            if let Ok(class) = self.get_class_name() {
                let class_lower = class.to_lowercase();
                if class_lower.contains("shell_traywnd")
                    || class_lower.contains("shell_secondarytray")
                {
                    return true;
                }
            }

            // Check by title as a fallback for custom AppBars
            if let Ok(title) = self.get_title() {
                // Status bars often have "bar" in the title and are thin windows
                if title.to_lowercase().contains("status bar")
                    || title.to_lowercase().contains("tenraku status bar")
                {
                    if let Ok(rect) = self.get_rect() {
                        let height = rect.bottom - rect.top;
                        // If it's a thin window at the edge (< 100px height), likely an AppBar
                        if height < 100 && (rect.top == 0 || rect.bottom >= 1000) {
                            return true;
                        }
                    }
                }
            }

            false
        }
    }
}
