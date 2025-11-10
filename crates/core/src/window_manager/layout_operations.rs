//! Layout management operations.
//!
//! This module contains operations for changing and adjusting layouts.

use crate::window_manager::{LayoutType, WindowManager};

impl WindowManager {
    /// Set the layout type for the window manager.
    ///
    /// Changes the active layout algorithm and retiles the current workspace.
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout type to set
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if retiling fails.
    pub fn set_layout(&mut self, layout: LayoutType) -> anyhow::Result<()> {
        self.current_layout = layout;
        self.retile_workspace(self.active_workspace)?;
        Ok(())
    }

    /// Increase the master window count (for master layout).
    ///
    /// This increases the number of windows in the master area.
    /// After calling this, retile the workspace to apply changes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().expect("Failed to initialize");
    ///
    /// wm.increase_master_count();
    /// wm.retile_workspace(wm.get_active_workspace()).ok();
    /// ```
    pub fn increase_master_count(&mut self) {
        self.master_layout.increase_master_count();
    }

    /// Decrease the master window count (for master layout).
    ///
    /// This decreases the number of windows in the master area.
    /// After calling this, retile the workspace to apply changes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().expect("Failed to initialize");
    ///
    /// wm.decrease_master_count();
    /// wm.retile_workspace(wm.get_active_workspace()).ok();
    /// ```
    pub fn decrease_master_count(&mut self) {
        self.master_layout.decrease_master_count();
    }

    /// Adjust the master factor (for master layout).
    ///
    /// This changes the portion of the screen allocated to the master area.
    /// After calling this, retile the workspace to apply changes.
    ///
    /// # Arguments
    ///
    /// * `delta` - The change in master factor (typically 0.05 for 5% increments)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().expect("Failed to initialize");
    ///
    /// // Increase master area by 5%
    /// wm.adjust_master_factor(0.05);
    /// wm.retile_workspace(wm.get_active_workspace()).ok();
    /// ```
    pub fn adjust_master_factor(&mut self, delta: f32) {
        self.master_layout.adjust_master_factor(delta);
    }
}
