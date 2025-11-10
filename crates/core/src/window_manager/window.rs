//! Window state management module.
//!
//! This module provides comprehensive window state tracking and management,
//! including tiled, floating, fullscreen, and minimized states.

use crate::utils::win32::WindowHandle;
use crate::window_manager::tree::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use windows::Win32::Foundation::RECT;

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
        self.state == WindowState::Tiled 
            && self.managed 
            && !self.handle.is_minimized()
            && self.handle.is_visible()
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

    /// Get all registered windows.
    ///
    /// # Returns
    ///
    /// A vector of references to all managed windows.
    pub fn get_all(&self) -> Vec<&ManagedWindow> {
        self.windows.values().collect()
    }

    /// Check if a window is registered.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value (HWND.0)
    ///
    /// # Returns
    ///
    /// `true` if the window is registered, `false` otherwise.
    pub fn contains(&self, hwnd: isize) -> bool {
        self.windows.contains_key(&hwnd)
    }
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self::new()
    }
}