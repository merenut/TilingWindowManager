use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// A wrapper around Windows HWND providing safe access to window operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle(pub HWND);

impl WindowHandle {
    /// Create a WindowHandle from a raw HWND
    pub fn from_hwnd(hwnd: HWND) -> Self {
        WindowHandle(hwnd)
    }

    /// Get the window title
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

    /// Get the window class name
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

    /// Check if the window is visible
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.0).as_bool() }
    }

    /// Get the window rectangle (position and size)
    pub fn get_rect(&self) -> anyhow::Result<RECT> {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect)?;
            Ok(rect)
        }
    }
}

/// Enumerate all windows in the system
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

/// Callback function for EnumWindows
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowHandle>);
    windows.push(WindowHandle::from_hwnd(hwnd));
    true.into()
}

/// Enumerate only visible windows
pub fn enumerate_visible_windows() -> anyhow::Result<Vec<WindowHandle>> {
    let all_windows = enumerate_windows()?;
    Ok(all_windows
        .into_iter()
        .filter(|w| w.is_visible())
        .collect())
}

/// Get the currently focused foreground window
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
