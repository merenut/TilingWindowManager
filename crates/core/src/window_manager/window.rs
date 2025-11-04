//! Window state management module.
//!
//! This module provides comprehensive window state tracking and management,
//! including tiled, floating, fullscreen, and minimized states.

use crate::utils::win32::WindowHandle;
use crate::window_manager::tree::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use windows::Win32::Foundation::{HWND, RECT};

/// The state of a managed window.
///
/// Windows can be in one of four states:
/// - Tiled: Managed by the tiling layout algorithm
/// - Floating: User-positioned, not managed by layout
/// - Fullscreen: Covers the entire monitor
/// - Minimized: Hidden from view but tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    /// Window is managed by the tiling layout
    Tiled,
    /// Window is floating and positioned by user
    Floating,
    /// Window covers the entire monitor
    Fullscreen,
    /// Window is minimized
    Minimized,
}

/// A managed window with state tracking.
///
/// ManagedWindow wraps a WindowHandle and adds state management,
/// workspace tracking, and position saving/restoring capabilities.
#[derive(Debug, Clone)]
pub struct ManagedWindow {
    /// The underlying window handle
    pub handle: WindowHandle,
    /// Current state of the window
    pub state: WindowState,
    /// The workspace this window belongs to
    pub workspace: usize,
    /// The monitor this window is on
    pub monitor: usize,
    /// Window title (cached)
    pub title: String,
    /// Window class name (cached)
    pub class: String,
    /// Process name (cached)
    pub process_name: String,
    /// Saved position before entering fullscreen/floating
    pub original_rect: Option<RECT>,
    /// Whether this window should be managed
    pub managed: bool,
    /// User-specified floating state preference
    pub user_floating: bool,
}

impl ManagedWindow {
    /// Create a new ManagedWindow.
    ///
    /// # Arguments
    ///
    /// * `handle` - The window handle to manage
    /// * `workspace` - The workspace this window belongs to
    /// * `monitor` - The monitor this window is on
    ///
    /// # Returns
    ///
    /// A new ManagedWindow instance with default tiled state.
    pub fn new(handle: WindowHandle, workspace: usize, monitor: usize) -> anyhow::Result<Self> {
        let title = handle.get_title().unwrap_or_default();
        let class = handle.get_class_name().unwrap_or_default();
        let process_name = handle.get_process_name().unwrap_or_default();

        Ok(Self {
            handle,
            state: WindowState::Tiled,
            workspace,
            monitor,
            title,
            class,
            process_name,
            original_rect: None,
            managed: true,
            user_floating: false,
        })
    }

    /// Set window to floating state.
    ///
    /// Saves the current position and marks the window as floating.
    pub fn set_floating(&mut self) -> anyhow::Result<()> {
        if self.state != WindowState::Floating {
            // Save current position if tiled
            if self.state == WindowState::Tiled {
                self.original_rect = Some(self.handle.get_rect()?);
            }
            self.state = WindowState::Floating;
            self.user_floating = true;
        }
        Ok(())
    }

    /// Set window to tiled state.
    ///
    /// Clears floating preferences and saved position.
    pub fn set_tiled(&mut self) -> anyhow::Result<()> {
        self.state = WindowState::Tiled;
        self.user_floating = false;
        self.original_rect = None;
        Ok(())
    }

    /// Set window to fullscreen.
    ///
    /// Saves the current position and applies fullscreen geometry.
    ///
    /// # Arguments
    ///
    /// * `monitor_rect` - The full monitor area to fill
    pub fn set_fullscreen(&mut self, monitor_rect: &Rect) -> anyhow::Result<()> {
        if self.state != WindowState::Fullscreen {
            // Save current position
            self.original_rect = Some(self.handle.get_rect()?);
            self.state = WindowState::Fullscreen;

            // Apply fullscreen geometry
            self.handle.set_pos(
                monitor_rect.x,
                monitor_rect.y,
                monitor_rect.width,
                monitor_rect.height,
            )?;
        }
        Ok(())
    }

    /// Exit fullscreen, returning to previous state.
    ///
    /// Restores the window to either tiled or floating state based on
    /// user preference, and restores the original position if available.
    pub fn exit_fullscreen(&mut self) -> anyhow::Result<()> {
        if self.state == WindowState::Fullscreen {
            // Restore to tiled or floating based on user preference
            self.state = if self.user_floating {
                WindowState::Floating
            } else {
                WindowState::Tiled
            };

            // Restore original position if available
            if let Some(rect) = self.original_rect {
                self.handle.set_pos(
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                )?;
            }

            self.original_rect = None;
        }
        Ok(())
    }

    /// Toggle between tiled and floating states.
    ///
    /// If the window is currently tiled, it becomes floating.
    /// If the window is floating, it becomes tiled.
    /// Other states (fullscreen, minimized) are not affected.
    pub fn toggle_floating(&mut self) -> anyhow::Result<()> {
        match self.state {
            WindowState::Tiled => self.set_floating()?,
            WindowState::Floating => self.set_tiled()?,
            _ => {}
        }
        Ok(())
    }

    /// Minimize the window.
    ///
    /// Saves the current position and hides the window using the Windows API.
    pub fn minimize(&mut self) -> anyhow::Result<()> {
        if self.state != WindowState::Minimized {
            self.original_rect = Some(self.handle.get_rect()?);
            self.state = WindowState::Minimized;

            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(self.handle.hwnd(), SW_MINIMIZE);
            }
        }
        Ok(())
    }

    /// Restore from minimized state.
    ///
    /// Returns the window to either tiled or floating state based on
    /// user preference, and shows the window using the Windows API.
    pub fn restore(&mut self) -> anyhow::Result<()> {
        if self.state == WindowState::Minimized {
            self.state = if self.user_floating {
                WindowState::Floating
            } else {
                WindowState::Tiled
            };

            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(self.handle.hwnd(), SW_RESTORE);
            }
        }
        Ok(())
    }

    /// Check if window should be included in tiling layout.
    ///
    /// # Returns
    ///
    /// `true` if the window is in tiled state and managed, `false` otherwise.
    pub fn should_tile(&self) -> bool {
        self.state == WindowState::Tiled && self.managed
    }

    /// Update window metadata (title, class, etc.).
    ///
    /// Refreshes cached metadata from the actual window.
    pub fn update_metadata(&mut self) -> anyhow::Result<()> {
        self.title = self.handle.get_title().unwrap_or_default();
        self.class = self.handle.get_class_name().unwrap_or_default();
        Ok(())
    }
}

/// Registry for managing all windows.
///
/// The WindowRegistry maintains a collection of all managed windows
/// and provides efficient querying by workspace, state, and other criteria.
pub struct WindowRegistry {
    /// Map of window handles (hwnd.0) to managed windows
    windows: HashMap<isize, ManagedWindow>,
}

impl WindowRegistry {
    /// Create a new empty WindowRegistry.
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Register a new window.
    ///
    /// # Arguments
    ///
    /// * `window` - The managed window to register
    pub fn register(&mut self, window: ManagedWindow) {
        self.windows.insert(window.handle.hwnd().0, window);
    }

    /// Unregister a window.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value (HWND.0)
    ///
    /// # Returns
    ///
    /// The unregistered ManagedWindow, or None if not found.
    pub fn unregister(&mut self, hwnd: isize) -> Option<ManagedWindow> {
        self.windows.remove(&hwnd)
    }

    /// Get window by handle.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value (HWND.0)
    ///
    /// # Returns
    ///
    /// A reference to the ManagedWindow, or None if not found.
    pub fn get(&self, hwnd: isize) -> Option<&ManagedWindow> {
        self.windows.get(&hwnd)
    }

    /// Get mutable window by handle.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value (HWND.0)
    ///
    /// # Returns
    ///
    /// A mutable reference to the ManagedWindow, or None if not found.
    pub fn get_mut(&mut self, hwnd: isize) -> Option<&mut ManagedWindow> {
        self.windows.get_mut(&hwnd)
    }

    /// Get all windows in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID
    ///
    /// # Returns
    ///
    /// A vector of references to windows in the workspace.
    pub fn get_by_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace)
            .collect()
    }

    /// Get all tiled windows in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID
    ///
    /// # Returns
    ///
    /// A vector of references to tiled windows in the workspace.
    pub fn get_tiled_in_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace && w.should_tile())
            .collect()
    }

    /// Get all floating windows in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID
    ///
    /// # Returns
    ///
    /// A vector of references to floating windows in the workspace.
    pub fn get_floating_in_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace && w.state == WindowState::Floating)
            .collect()
    }

    /// Get total window count.
    ///
    /// # Returns
    ///
    /// The total number of registered windows.
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Get window count in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace ID
    ///
    /// # Returns
    ///
    /// The number of windows in the workspace.
    pub fn count_in_workspace(&self, workspace: usize) -> usize {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace)
            .count()
    }
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn create_test_window() -> ManagedWindow {
        ManagedWindow {
            handle: WindowHandle::from_hwnd(HWND(12345)),
            state: WindowState::Tiled,
            workspace: 1,
            monitor: 0,
            title: "Test".to_string(),
            class: "TestClass".to_string(),
            process_name: "test.exe".to_string(),
            original_rect: None,
            managed: true,
            user_floating: false,
        }
    }

    #[allow(dead_code)]
    fn create_test_window_with_workspace(workspace: usize) -> ManagedWindow {
        let mut w = create_test_window();
        w.workspace = workspace;
        w
    }

    #[test]
    fn test_window_state_default() {
        let window = create_test_window();
        assert_eq!(window.state, WindowState::Tiled);
        assert!(!window.user_floating);
    }

    #[test]
    fn test_set_floating() {
        let mut window = create_test_window();
        assert_eq!(window.state, WindowState::Tiled);

        window.set_floating().ok();
        assert_eq!(window.state, WindowState::Floating);
        assert!(window.user_floating);
    }

    #[test]
    fn test_set_tiled() {
        let mut window = create_test_window();
        window.set_floating().ok();
        assert_eq!(window.state, WindowState::Floating);

        window.set_tiled().ok();
        assert_eq!(window.state, WindowState::Tiled);
        assert!(!window.user_floating);
    }

    #[test]
    fn test_toggle_floating() {
        let mut window = create_test_window();
        assert_eq!(window.state, WindowState::Tiled);

        window.toggle_floating().ok();
        assert_eq!(window.state, WindowState::Floating);

        window.toggle_floating().ok();
        assert_eq!(window.state, WindowState::Tiled);
    }

    #[test]
    fn test_should_tile() {
        let mut window = create_test_window();
        assert!(window.should_tile());

        window.set_floating().ok();
        assert!(!window.should_tile());

        window.managed = false;
        assert!(!window.should_tile());
    }

    #[test]
    fn test_window_registry() {
        let mut registry = WindowRegistry::new();
        assert_eq!(registry.count(), 0);

        let window = create_test_window();
        let hwnd = window.handle.hwnd().0;

        registry.register(window);
        assert_eq!(registry.count(), 1);

        let retrieved = registry.get(hwnd);
        assert!(retrieved.is_some());

        registry.unregister(hwnd);
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_workspace_filtering() {
        let mut registry = WindowRegistry::new();

        // Add windows to different workspaces
        let w1 = create_test_window_with_workspace(1);
        let w2 = create_test_window_with_workspace(1);
        let w3 = create_test_window_with_workspace(2);

        // Give them different HWNDs
        let mut w1 = w1;
        w1.handle = WindowHandle::from_hwnd(HWND(1));
        let mut w2 = w2;
        w2.handle = WindowHandle::from_hwnd(HWND(2));
        let mut w3 = w3;
        w3.handle = WindowHandle::from_hwnd(HWND(3));

        registry.register(w1);
        registry.register(w2);
        registry.register(w3);

        let ws1_windows = registry.get_by_workspace(1);
        assert_eq!(ws1_windows.len(), 2);

        let ws2_windows = registry.get_by_workspace(2);
        assert_eq!(ws2_windows.len(), 1);
    }

    #[test]
    fn test_get_tiled_in_workspace() {
        let mut registry = WindowRegistry::new();

        let mut w1 = create_test_window_with_workspace(1);
        w1.handle = WindowHandle::from_hwnd(HWND(1));

        let mut w2 = create_test_window_with_workspace(1);
        w2.handle = WindowHandle::from_hwnd(HWND(2));
        w2.set_floating().ok();

        registry.register(w1);
        registry.register(w2);

        let tiled = registry.get_tiled_in_workspace(1);
        assert_eq!(tiled.len(), 1);
    }

    #[test]
    fn test_get_floating_in_workspace() {
        let mut registry = WindowRegistry::new();

        let mut w1 = create_test_window_with_workspace(1);
        w1.handle = WindowHandle::from_hwnd(HWND(1));

        let mut w2 = create_test_window_with_workspace(1);
        w2.handle = WindowHandle::from_hwnd(HWND(2));
        w2.set_floating().ok();

        registry.register(w1);
        registry.register(w2);

        let floating = registry.get_floating_in_workspace(1);
        assert_eq!(floating.len(), 1);
    }
}
