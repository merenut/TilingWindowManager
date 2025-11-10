//! Master-stack layout algorithm for tiling window managers.
//!
//! The master layout divides the screen into a master area and a stack area.
//! One or more "master" windows occupy the master area (typically on the left),
//! while remaining windows stack vertically in the remaining area.
//!
//! # Features
//!
//! - Configurable master area size (master_factor)
//! - Adjustable number of master windows (master_count)
//! - Vertical tiling when all windows are masters
//! - Gap support (inner and outer)
//! - Dynamic adjustment of master factor and count
//!
//! # Example
//!
//! ```no_run
//! use tenraku_core::window_manager::layout::MasterLayout;
//! use tenraku_core::window_manager::Rect;
//! use windows::Win32::Foundation::HWND;
//!
//! let layout = MasterLayout::new();
//! let area = Rect::new(0, 0, 1920, 1080);
//! let windows = vec![HWND(1 as _), HWND(2 as _), HWND(3 as _)];
//!
//! // Apply master layout to windows
//! layout.apply(&windows, area).ok();
//! ```

use crate::window_manager::tree::Rect;
use windows::Win32::Foundation::HWND;

/// Configuration and logic for the master-stack layout algorithm.
///
/// The master layout positions one or more master windows in a configurable
/// portion of the screen, with remaining windows stacked in the rest of the space.
#[derive(Debug, Clone)]
pub struct MasterLayout {
    /// Portion of screen for master area (0.1 to 0.9)
    pub master_factor: f32,
    /// Number of windows in master area (minimum 1)
    pub master_count: usize,
    /// Gap size between windows
    pub gaps_in: i32,
    /// Gap size from screen edges
    pub gaps_out: i32,
}

impl Default for MasterLayout {
    fn default() -> Self {
        Self {
            master_factor: 0.55,
            master_count: 1,
            gaps_in: 5,
            gaps_out: 10,
        }
    }
}

impl MasterLayout {
    /// Create a new MasterLayout with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let layout = MasterLayout::new();
    /// assert_eq!(layout.master_factor, 0.55);
    /// assert_eq!(layout.master_count, 1);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the master factor.
    ///
    /// The master factor determines what portion of the screen width is
    /// allocated to the master area. Values are clamped to [0.1, 0.9].
    ///
    /// # Arguments
    ///
    /// * `factor` - The master factor (0.1 to 0.9)
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let layout = MasterLayout::new().with_master_factor(0.6);
    /// assert_eq!(layout.master_factor, 0.6);
    /// ```
    pub fn with_master_factor(mut self, factor: f32) -> Self {
        self.master_factor = factor.clamp(0.1, 0.9);
        self
    }

    /// Set the master count.
    ///
    /// The master count determines how many windows are placed in the master
    /// area. The value is clamped to a minimum of 1.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of master windows (minimum 1)
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let layout = MasterLayout::new().with_master_count(2);
    /// assert_eq!(layout.master_count, 2);
    /// ```
    pub fn with_master_count(mut self, count: usize) -> Self {
        self.master_count = count.max(1);
        self
    }

    /// Apply master layout to a list of windows (Windows platform).
    ///
    /// This method positions windows according to the master-stack layout:
    /// - If there's only one window, it takes the full area
    /// - If all windows fit in master area, they split vertically
    /// - Otherwise, master windows are on the left, stack windows on the right
    ///
    /// # Arguments
    ///
    /// * `windows` - Slice of window handles to position
    /// * `area` - The screen area to tile within
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if window positioning fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::layout::MasterLayout;
    /// use tenraku_core::window_manager::Rect;
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let layout = MasterLayout::new();
    /// let area = Rect::new(0, 0, 1920, 1080);
    /// let windows = vec![HWND(1 as _), HWND(2 as _)];
    ///
    /// layout.apply(&windows, area).ok();
    /// ```
    #[cfg(target_os = "windows")]
    pub fn apply(&self, windows: &[HWND], area: Rect) -> anyhow::Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        // Note: outer gaps should already be applied by the caller
        // We only apply inner gaps between windows here
        let work_area = area;

        // Single window: take full work area with inner gaps
        if windows.len() == 1 {
            let half_gap = self.gaps_in / 2;
            let final_width = (work_area.width - self.gaps_in).max(1);
            let final_height = (work_area.height - self.gaps_in).max(1);
            let final_rect = Rect::new(
                work_area.x + half_gap,
                work_area.y + half_gap,
                final_width,
                final_height,
            );
            self.position_window(windows[0], final_rect)?;
            return Ok(());
        }

        // Determine master and stack window counts
        let master_count = self.master_count.min(windows.len());
        let stack_count = windows.len() - master_count;

        if stack_count == 0 {
            // All windows are masters: split them vertically
            self.tile_masters_only(&windows[..master_count], work_area)?;
        } else {
            // Split between master and stack
            self.tile_master_stack(
                &windows[..master_count],
                &windows[master_count..],
                work_area,
            )?;
        }

        Ok(())
    }

    /// Apply master layout to a list of windows (non-Windows stub).
    ///
    /// This is a placeholder implementation for non-Windows platforms.
    #[cfg(not(target_os = "windows"))]
    pub fn apply(&self, _windows: &[HWND], _area: Rect) -> anyhow::Result<()> {
        Ok(())
    }

    /// Tile windows when all are masters (split vertically).
    ///
    /// # Arguments
    ///
    /// * `masters` - Slice of master window handles
    /// * `area` - The screen area to tile within
    #[cfg(target_os = "windows")]
    fn tile_masters_only(&self, masters: &[HWND], area: Rect) -> anyhow::Result<()> {
        self.tile_vertical(masters, area)
    }

    /// Tile windows with separate master and stack areas.
    ///
    /// # Arguments
    ///
    /// * `masters` - Slice of master window handles
    /// * `stack` - Slice of stack window handles
    /// * `area` - The screen area to tile within
    #[cfg(target_os = "windows")]
    fn tile_master_stack(
        &self,
        masters: &[HWND],
        stack: &[HWND],
        area: Rect,
    ) -> anyhow::Result<()> {
        // Calculate master and stack areas
        let master_width = ((area.width as f32 * self.master_factor) as i32).max(1);
        // Ensure stack_width is positive
        let stack_width = (area.width - master_width).max(1);

        let master_area = Rect::new(area.x, area.y, master_width, area.height);
        let stack_area = Rect::new(area.x + master_width, area.y, stack_width, area.height);

        // Tile master windows
        self.tile_vertical(masters, master_area)?;

        // Tile stack windows
        self.tile_vertical(stack, stack_area)?;

        Ok(())
    }

    /// Tile windows vertically in the given area.
    ///
    /// # Arguments
    ///
    /// * `windows` - Slice of window handles to tile
    /// * `area` - The area to tile within (outer gaps already applied)
    #[cfg(target_os = "windows")]
    fn tile_vertical(&self, windows: &[HWND], area: Rect) -> anyhow::Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        let window_count = windows.len() as i32;
        let height_per_window = area.height / window_count;
        let remaining_height = area.height % window_count;

        for (i, &hwnd) in windows.iter().enumerate() {
            let y = area.y + (i as i32 * height_per_window);

            // For the last window, add any remaining height from integer division
            let height = if i == windows.len() - 1 {
                height_per_window + remaining_height
            } else {
                height_per_window
            };

            let rect = Rect::new(area.x, y, area.width, height);

            // Apply inner gaps (half gap on each side creates space between windows)
            let half_gap = self.gaps_in / 2;

            // Ensure dimensions remain positive after applying gaps
            let final_width = (rect.width - self.gaps_in).max(1);
            let final_height = (rect.height - self.gaps_in).max(1);

            let final_rect = Rect::new(
                rect.x + half_gap,
                rect.y + half_gap,
                final_width,
                final_height,
            );

            tracing::debug!(
                "Positioning window {} ({}/{}) at ({}, {}) with size {}x{}",
                hwnd.0,
                i + 1,
                windows.len(),
                final_rect.x,
                final_rect.y,
                final_rect.width,
                final_rect.height
            );

            self.position_window(hwnd, final_rect)?;
        }

        Ok(())
    }

    /// Position a window at the specified rectangle using Win32 API.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - Window handle to position
    /// * `rect` - Target rectangle for the window
    #[cfg(target_os = "windows")]
    fn position_window(&self, hwnd: HWND, rect: Rect) -> anyhow::Result<()> {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

        unsafe {
            SetWindowPos(
                hwnd,
                None,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }

        Ok(())
    }

    /// Increase master count by 1.
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let mut layout = MasterLayout::new();
    /// assert_eq!(layout.master_count, 1);
    /// layout.increase_master_count();
    /// assert_eq!(layout.master_count, 2);
    /// ```
    pub fn increase_master_count(&mut self) {
        self.master_count += 1;
    }

    /// Decrease master count by 1 (minimum 1).
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let mut layout = MasterLayout::new().with_master_count(2);
    /// layout.decrease_master_count();
    /// assert_eq!(layout.master_count, 1);
    /// layout.decrease_master_count();
    /// assert_eq!(layout.master_count, 1); // Does not go below 1
    /// ```
    pub fn decrease_master_count(&mut self) {
        if self.master_count > 1 {
            self.master_count -= 1;
        }
    }

    /// Adjust master factor by a delta amount.
    ///
    /// The resulting factor is clamped to [0.1, 0.9].
    ///
    /// # Arguments
    ///
    /// * `delta` - The amount to adjust (positive or negative)
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::MasterLayout;
    ///
    /// let mut layout = MasterLayout::new();
    /// let original = layout.master_factor;
    /// layout.adjust_master_factor(0.1);
    /// assert!(layout.master_factor > original);
    /// ```
    pub fn adjust_master_factor(&mut self, delta: f32) {
        self.master_factor = (self.master_factor + delta).clamp(0.1, 0.9);
    }
}