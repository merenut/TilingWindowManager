//! Windows AppBar registration utilities.
//!
//! This module provides functionality to register the status bar as a Windows AppBar,
//! which signals to the OS and tiling window managers that this window should not be managed.

use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
    },
};

/// Register a window as an AppBar at the top of the screen.
///
/// This tells Windows that this window is a taskbar-like application that should:
/// - Not be managed by tiling window managers
/// - Reserve space at the edge of the screen
/// - Remain always-on-top
///
/// # Arguments
///
/// * `hwnd` - The window handle to register
/// * `x` - The X position of the appbar
/// * `y` - The Y position of the appbar  
/// * `width` - The width of the appbar
/// * `height` - The height of the appbar
///
/// # Returns
///
/// `Ok(())` if registration succeeds, error otherwise.
pub fn register_appbar(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> anyhow::Result<()> {
    unsafe {
        // Create APPBARDATA structure
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: 0,
            uEdge: ABE_TOP,
            rc: RECT {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            },
            lParam: LPARAM(0),
        };

        // Register the appbar
        let result = SHAppBarMessage(ABM_NEW, &mut abd);
        if result == 0 {
            anyhow::bail!("Failed to register AppBar");
        }

        tracing::info!("Registered window as AppBar");

        // Set the position
        let result = SHAppBarMessage(ABM_SETPOS, &mut abd);
        if result == 0 {
            tracing::warn!("Failed to set AppBar position");
        }

        Ok(())
    }
}

/// Unregister a window as an AppBar.
///
/// Should be called when the application closes to clean up the AppBar registration.
///
/// # Arguments
///
/// * `hwnd` - The window handle to unregister
pub fn unregister_appbar(hwnd: HWND) -> anyhow::Result<()> {
    unsafe {
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: 0,
            uEdge: ABE_TOP,
            rc: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            lParam: LPARAM(0),
        };

        let result = SHAppBarMessage(ABM_REMOVE, &mut abd);
        if result == 0 {
            tracing::warn!("Failed to unregister AppBar");
        } else {
            tracing::info!("Unregistered AppBar");
        }

        Ok(())
    }
}
