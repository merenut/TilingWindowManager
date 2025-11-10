//! Window management operations.
//!
//! This module contains operations for adding, removing, and manipulating windows.

use crate::utils::win32::WindowHandle;
use crate::window_manager::{ManagedWindow, WindowManager, WindowState};

impl WindowManager {
    /// Check if a window should be managed by the window manager.
    ///
    /// This filters out windows that should not be tiled, such as:
    /// - Invisible windows
    /// - Popup windows
    /// - Tool windows
    /// - Windows without owners or parents in special cases
    ///
    /// # Arguments
    ///
    /// * `window` - The window to check
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the window should be managed, `Ok(false)` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    /// use tenraku_core::utils::win32;
    ///
    /// let wm = WindowManager::new();
    /// let windows = win32::enumerate_windows().unwrap();
    ///
    /// for window in windows {
    ///     if wm.should_manage_window(&window).unwrap_or(false) {
    ///         println!("Should manage: {}", window.get_title().unwrap_or_default());
    ///     }
    /// }
    /// ```
    pub fn should_manage_window(&self, window: &WindowHandle) -> anyhow::Result<bool> {
        // Use the is_app_window heuristic from WindowHandle
        // This already filters for visible windows with titles and no owners
        if !window.is_app_window() {
            return Ok(false);
        }

        // Don't manage AppBars (taskbar, status bars, etc.)
        if window.is_app_bar() {
            return Ok(false);
        }

        // Don't manage minimized windows at creation time
        if window.is_minimized() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Add a window to be managed by the window manager.
    ///
    /// The window is added to the current workspace's tree and tiled accordingly.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to manage
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    /// use tenraku_core::utils::win32;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    ///
    /// if let Some(window) = win32::get_foreground_window() {
    ///     if wm.should_manage_window(&window).unwrap_or(false) {
    ///         wm.manage_window(window).ok();
    ///     }
    /// }
    /// ```
    pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();

        // Check if already managed
        if self.registry.get(hwnd.0).is_some() {
            return Ok(());
        }

        // Determine monitor for this window
        let monitor_index = self.get_monitor_for_window(hwnd);

        // Create managed window
        let mut managed = ManagedWindow::new(window, self.active_workspace, monitor_index)?;

        // Apply rules if rule matcher is available
        if let Some(ref matcher) = self.rule_matcher {
            // Check if window should be managed at all
            if !matcher.should_manage(&managed) {
                tracing::info!(
                    "Window '{}' (process: {}) excluded by NoManage rule",
                    managed.title,
                    managed.process_name
                );
                return Ok(());
            }

            // Get initial workspace from rules
            if let Some(workspace_id) = matcher.get_initial_workspace(&managed) {
                // Validate workspace exists (workspace IDs start at 1)
                if workspace_id > 0 && workspace_id <= 10 {
                    tracing::info!(
                        "Assigning window '{}' to workspace {} per rule",
                        managed.title,
                        workspace_id
                    );
                    managed.workspace = workspace_id;
                } else {
                    tracing::warn!(
                        "Invalid workspace ID {} in rule for window '{}', using current workspace",
                        workspace_id,
                        managed.title
                    );
                }
            }

            // Get initial monitor from rules
            if let Some(monitor_id) = matcher.get_initial_monitor(&managed) {
                // Validate monitor exists
                if monitor_id < self.monitors.len() {
                    tracing::info!(
                        "Assigning window '{}' to monitor {} per rule",
                        managed.title,
                        monitor_id
                    );
                    managed.monitor = monitor_id;
                } else {
                    tracing::warn!(
                        "Invalid monitor ID {} in rule for window '{}', using monitor 0",
                        monitor_id,
                        managed.title
                    );
                    managed.monitor = 0;
                }
            }

            // Check if should be floating
            if matcher.should_float(&managed) {
                tracing::info!("Setting window '{}' to floating per rule", managed.title);
                managed.set_floating()?;
            }

            // Check if should be fullscreen
            if matcher.should_fullscreen(&managed) {
                tracing::info!("Setting window '{}' to fullscreen per rule", managed.title);
                // Get the monitor's work area for fullscreen
                if let Some(monitor) = self.monitors.get(managed.monitor) {
                    managed.set_fullscreen(&monitor.work_area)?;
                }
            }

            // Check if should not be focused
            if matcher.should_not_focus(&managed) {
                tracing::debug!("Window '{}' marked as no-focus per rule", managed.title);
                // This flag can be checked by the focus manager
            }
        }

        // Register the window
        self.registry.register(managed);

        // Retile the workspace
        self.retile_workspace(self.active_workspace)?;

        Ok(())
    }

    /// Remove a window from management.
    ///
    /// The window is removed from its workspace tree and the layout is re-applied.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to unmanage
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    /// use windows::Win32::Foundation::HWND;
    /// use tenraku_core::utils::win32::WindowHandle;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    ///
    /// let window = WindowHandle::from_hwnd(HWND(12345 as _));
    /// wm.unmanage_window(&window).ok();
    /// ```
    pub fn unmanage_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();

        // Remove from registry
        if let Some(managed) = self.registry.unregister(hwnd.0) {
            // Retile the workspace this window belonged to
            self.retile_workspace(managed.workspace)?;
        }

        Ok(())
    }

    /// Toggle floating state for a window.
    ///
    /// If the window is currently tiled, it becomes floating.
    /// If the window is floating, it becomes tiled.
    /// After toggling, the workspace is retiled to adjust layout.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to toggle
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    pub fn toggle_floating(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();

        if let Some(managed) = self.registry.get_mut(hwnd.0) {
            let workspace = managed.workspace;
            managed.toggle_floating()?;

            // Retile workspace to adjust for window state change
            self.retile_workspace(workspace)?;
        }

        Ok(())
    }

    /// Toggle fullscreen state for a window.
    ///
    /// If the window is not fullscreen, it becomes fullscreen covering the entire monitor.
    /// If the window is fullscreen, it returns to its previous state (tiled or floating).
    ///
    /// # Arguments
    ///
    /// * `window` - The window to toggle
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    pub fn toggle_fullscreen(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.hwnd();

        if let Some(managed) = self.registry.get_mut(hwnd.0) {
            let monitor = self
                .monitors
                .get(managed.monitor)
                .ok_or_else(|| anyhow::anyhow!("Monitor not found"))?;

            match managed.state {
                WindowState::Fullscreen => {
                    let workspace = managed.workspace;
                    managed.exit_fullscreen()?;
                    // Retile workspace
                    self.retile_workspace(workspace)?;
                }
                _ => {
                    managed.set_fullscreen(&monitor.work_area)?;
                }
            }
        }

        Ok(())
    }

    /// Get the active window.
    ///
    /// # Returns
    ///
    /// A reference to the active ManagedWindow, or None if no window is active.
    pub fn get_active_window(&self) -> Option<&ManagedWindow> {
        let fg_window = crate::utils::win32::get_foreground_window()?;
        self.registry.get(fg_window.hwnd().0)
    }

    /// Get a mutable reference to the active window.
    ///
    /// # Returns
    ///
    /// A mutable reference to the active ManagedWindow, or None if no window is active.
    pub fn get_active_window_mut(&mut self) -> Option<&mut ManagedWindow> {
        let fg_window = crate::utils::win32::get_foreground_window()?;
        self.registry.get_mut(fg_window.hwnd().0)
    }

    /// Get all managed windows, optionally filtered by workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - Optional workspace ID to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to managed windows.
    pub fn get_windows(&self, workspace: Option<usize>) -> Vec<&ManagedWindow> {
        if let Some(ws_id) = workspace {
            self.registry.get_by_workspace(ws_id)
        } else {
            self.registry.get_all()
        }
    }

    /// Focus a window by its HWND.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the window is not found or focus fails.
    pub fn focus_window_by_hwnd(&mut self, hwnd: isize) -> anyhow::Result<()> {
        use crate::utils::win32::WindowHandle;
        use windows::Win32::Foundation::HWND;

        let window = WindowHandle::from_hwnd(HWND(hwnd));

        // Check if window is managed
        if !self.registry.contains(hwnd) {
            anyhow::bail!("Window {:?} is not managed", hwnd);
        }

        // Use focus manager to focus the window
        self.focus_manager.focus_window(&window)?;

        Ok(())
    }

    /// Get a reference to the window registry.
    ///
    /// # Returns
    ///
    /// A reference to the WindowRegistry.
    pub fn registry(&self) -> &crate::window_manager::WindowRegistry {
        &self.registry
    }

    /// Check if a window is managed by the window manager.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to check
    ///
    /// # Returns
    ///
    /// `true` if the window is managed, `false` otherwise.
    pub fn is_window_managed(&self, window: &WindowHandle) -> bool {
        self.registry.contains(window.hwnd().0)
    }

    /// Get a reference to a managed window.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle value
    ///
    /// # Returns
    ///
    /// An option containing a reference to the managed window, or None if not found.
    pub fn get_window(&self, hwnd: isize) -> Option<&ManagedWindow> {
        self.registry.get(hwnd)
    }
}
