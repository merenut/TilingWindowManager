//! Focus management module for window focus tracking and navigation.
//!
//! This module provides focus management with history tracking and directional navigation:
//! - FocusManager tracks recently focused windows with history size limit
//! - Alt-Tab style navigation (focus_previous, focus_next)
//! - Directional focus navigation using geometry-based calculations
//! - History maintenance when windows are removed
//!
//! # Example
//!
//! ```no_run
//! use tiling_wm_core::window_manager::focus::{FocusManager, Direction};
//! use tiling_wm_core::utils::win32::WindowHandle;
//! use windows::Win32::Foundation::HWND;
//!
//! let mut focus_manager = FocusManager::new();
//! let window = WindowHandle::from_hwnd(HWND(12345 as _));
//!
//! // Focus a window
//! focus_manager.focus_window(&window).ok();
//!
//! // Navigate focus history (Alt-Tab style)
//! if let Some(prev_hwnd) = focus_manager.focus_previous() {
//!     println!("Focused previous window: {:?}", prev_hwnd);
//! }
//! ```

use crate::utils::win32::WindowHandle;
use crate::window_manager::tree::Rect;
use std::collections::VecDeque;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Direction for directional focus navigation.
///
/// Used to navigate focus between windows based on their spatial relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Focus the window to the left
    Left,
    /// Focus the window to the right
    Right,
    /// Focus the window above
    Up,
    /// Focus the window below
    Down,
}

/// Manages window focus with history tracking and navigation.
///
/// The FocusManager maintains a history of recently focused windows and provides
/// methods for navigating focus, both through history (Alt-Tab style) and
/// directionally (based on window positions).
///
/// # Example
///
/// ```no_run
/// use tiling_wm_core::window_manager::focus::FocusManager;
///
/// let mut fm = FocusManager::new();
/// assert_eq!(fm.current(), None);
/// ```
pub struct FocusManager {
    /// Recently focused windows (most recent first)
    focus_history: VecDeque<isize>,
    /// Currently focused window
    current_focus: Option<isize>,
    /// Maximum history size
    history_size: usize,
}

impl FocusManager {
    /// Create a new FocusManager with default history size of 10.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let fm = FocusManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            focus_history: VecDeque::with_capacity(10),
            current_focus: None,
            history_size: 10,
        }
    }

    /// Create a FocusManager with a custom history size.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum number of windows to track in history
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let fm = FocusManager::new().with_history_size(20);
    /// ```
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self.focus_history = VecDeque::with_capacity(size);
        self
    }

    /// Focus a specific window.
    ///
    /// Brings the window to the foreground, ensures it's visible, and updates
    /// the focus history.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to focus
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    /// use tiling_wm_core::utils::win32::WindowHandle;
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let mut fm = FocusManager::new();
    /// let window = WindowHandle::from_hwnd(HWND(12345 as _));
    /// fm.focus_window(&window).ok();
    /// ```
    pub fn focus_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        unsafe {
            // Restore if minimized first
            if IsIconic(window.hwnd()).as_bool() {
                ShowWindow(window.hwnd(), SW_RESTORE);
            }

            // Ensure window is visible
            ShowWindow(window.hwnd(), SW_SHOW);

            // Bring window to foreground
            if !SetForegroundWindow(window.hwnd()).as_bool() {
                return Err(anyhow::anyhow!("Failed to set window as foreground"));
            }
        }

        let hwnd_val = window.hwnd().0;

        // Update focus history
        self.add_to_history(hwnd_val);
        self.current_focus = Some(hwnd_val);

        Ok(())
    }

    /// Add window to focus history.
    ///
    /// Adds the window to the front of the history, removing it from any previous
    /// position. Trims the history to maintain the size limit.
    fn add_to_history(&mut self, hwnd: isize) {
        // Remove if already in history
        if let Some(idx) = self.focus_history.iter().position(|&h| h == hwnd) {
            self.focus_history.remove(idx);
        }

        // Add to front
        self.focus_history.push_front(hwnd);

        // Trim to size
        while self.focus_history.len() > self.history_size {
            self.focus_history.pop_back();
        }
    }

    /// Get currently focused window.
    ///
    /// # Returns
    ///
    /// The HWND value of the currently focused window, or None if no window is focused.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let fm = FocusManager::new();
    /// assert_eq!(fm.current(), None);
    /// ```
    pub fn current(&self) -> Option<isize> {
        self.current_focus
    }

    /// Focus previous window in history (Alt-Tab behavior).
    ///
    /// Switches focus to the second item in the history (the previously focused window).
    /// This implements Alt-Tab style behavior.
    ///
    /// # Returns
    ///
    /// The HWND value of the previous window, or None if there is no previous window.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let mut fm = FocusManager::new();
    /// // After focusing windows...
    /// if let Some(prev_hwnd) = fm.focus_previous() {
    ///     println!("Switched to previous window: {}", prev_hwnd);
    /// }
    /// ```
    pub fn focus_previous(&mut self) -> Option<isize> {
        // Get second item in history (first is current)
        if self.focus_history.len() > 1 {
            let prev = self.focus_history[1];
            self.current_focus = Some(prev);

            // Move to front of history
            self.focus_history.remove(1);
            self.focus_history.push_front(prev);

            Some(prev)
        } else {
            None
        }
    }

    /// Focus next window in history.
    ///
    /// Cycles through the focus history in the forward direction.
    ///
    /// # Returns
    ///
    /// The HWND value of the next window, or None if there is no next window.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let mut fm = FocusManager::new();
    /// // After focusing windows...
    /// if let Some(next_hwnd) = fm.focus_next() {
    ///     println!("Switched to next window: {}", next_hwnd);
    /// }
    /// ```
    pub fn focus_next(&mut self) -> Option<isize> {
        if self.focus_history.len() > 1 {
            // Move current to back, focus next
            if let Some(current) = self.focus_history.pop_front() {
                self.focus_history.push_back(current);
            }

            let next = self.focus_history[0];
            self.current_focus = Some(next);
            Some(next)
        } else {
            None
        }
    }

    /// Remove window from focus history.
    ///
    /// Called when a window is closed or unmanaged. If the removed window
    /// was the current focus, updates the current focus to the next window
    /// in the history.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The HWND value to remove
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let mut fm = FocusManager::new();
    /// // When a window is closed
    /// let closed_hwnd = 12345;
    /// fm.remove_from_history(closed_hwnd);
    /// ```
    pub fn remove_from_history(&mut self, hwnd: isize) {
        self.focus_history.retain(|&h| h != hwnd);

        if self.current_focus == Some(hwnd) {
            self.current_focus = self.focus_history.front().copied();
        }
    }

    /// Get focus history.
    ///
    /// Returns the entire focus history with the most recently focused window first.
    ///
    /// # Returns
    ///
    /// A reference to the focus history deque.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let fm = FocusManager::new();
    /// // Initially empty
    /// assert_eq!(fm.get_history().len(), 0);
    /// ```
    pub fn get_history(&self) -> &VecDeque<isize> {
        &self.focus_history
    }

    /// Clear focus history.
    ///
    /// Removes all windows from the focus history and clears the current focus.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::FocusManager;
    ///
    /// let mut fm = FocusManager::new();
    /// // After focusing windows...
    /// fm.clear_history();
    /// assert_eq!(fm.get_history().len(), 0);
    /// assert_eq!(fm.current(), None);
    /// ```
    pub fn clear_history(&mut self) {
        self.focus_history.clear();
        self.current_focus = None;
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for directional focus navigation.
///
/// Provides utility methods for finding windows in specific directions based
/// on their geometric positions.
pub struct DirectionalFocus;

impl DirectionalFocus {
    /// Find window in a specific direction from current window.
    ///
    /// Uses geometry-based calculations to find the closest window in the
    /// specified direction. The distance is calculated using Euclidean distance
    /// between window centers.
    ///
    /// # Arguments
    ///
    /// * `current_rect` - The rectangle of the currently focused window
    /// * `direction` - The direction to search
    /// * `candidates` - List of (hwnd, rect) pairs for candidate windows
    ///
    /// # Returns
    ///
    /// The HWND of the closest window in the specified direction, or None if
    /// no window is found in that direction.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::focus::{DirectionalFocus, Direction};
    /// use tiling_wm_core::window_manager::tree::Rect;
    ///
    /// let current = Rect::new(100, 100, 200, 200);
    /// let left = Rect::new(0, 100, 90, 200);
    /// let candidates = vec![(1, left)];
    ///
    /// let result = DirectionalFocus::find_window_in_direction(
    ///     &current,
    ///     Direction::Left,
    ///     &candidates,
    /// );
    /// assert_eq!(result, Some(1));
    /// ```
    pub fn find_window_in_direction(
        current_rect: &Rect,
        direction: Direction,
        candidates: &[(isize, Rect)],
    ) -> Option<isize> {
        let mut best_candidate: Option<(isize, f32)> = None;

        for &(hwnd, ref rect) in candidates {
            if Self::is_in_direction(current_rect, rect, direction) {
                let distance = Self::calculate_distance(current_rect, rect, direction);

                match best_candidate {
                    None => best_candidate = Some((hwnd, distance)),
                    Some((_, best_dist)) if distance < best_dist => {
                        best_candidate = Some((hwnd, distance));
                    }
                    _ => {}
                }
            }
        }

        best_candidate.map(|(hwnd, _)| hwnd)
    }

    /// Check if a window is in the specified direction from another window.
    ///
    /// Compares the centers of the two rectangles to determine if the target
    /// is in the specified direction from the source.
    fn is_in_direction(from: &Rect, to: &Rect, direction: Direction) -> bool {
        let from_center_x = from.x + from.width / 2;
        let from_center_y = from.y + from.height / 2;
        let to_center_x = to.x + to.width / 2;
        let to_center_y = to.y + to.height / 2;

        match direction {
            Direction::Left => to_center_x < from_center_x,
            Direction::Right => to_center_x > from_center_x,
            Direction::Up => to_center_y < from_center_y,
            Direction::Down => to_center_y > from_center_y,
        }
    }

    /// Calculate Euclidean distance between centers of two rectangles.
    ///
    /// This distance is used to find the closest window in a given direction.
    fn calculate_distance(from: &Rect, to: &Rect, _direction: Direction) -> f32 {
        let from_center_x = (from.x + from.width / 2) as f32;
        let from_center_y = (from.y + from.height / 2) as f32;
        let to_center_x = (to.x + to.width / 2) as f32;
        let to_center_y = (to.y + to.height / 2) as f32;

        // Euclidean distance
        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_creation() {
        let fm = FocusManager::new();
        assert_eq!(fm.current(), None);
        assert_eq!(fm.get_history().len(), 0);
    }

    #[test]
    fn test_focus_manager_with_custom_size() {
        let fm = FocusManager::new().with_history_size(5);
        assert_eq!(fm.history_size, 5);
        assert_eq!(fm.get_history().capacity(), 5);
    }

    #[test]
    fn test_add_to_history() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.add_to_history(3);

        assert_eq!(fm.focus_history.len(), 3);
        assert_eq!(fm.focus_history[0], 3);
        assert_eq!(fm.focus_history[1], 2);
        assert_eq!(fm.focus_history[2], 1);
    }

    #[test]
    fn test_add_to_history_removes_duplicates() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.add_to_history(1); // Add 1 again

        assert_eq!(fm.focus_history.len(), 2);
        assert_eq!(fm.focus_history[0], 1);
        assert_eq!(fm.focus_history[1], 2);
    }

    #[test]
    fn test_focus_previous() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.current_focus = Some(2);

        let prev = fm.focus_previous();
        assert_eq!(prev, Some(1));
        assert_eq!(fm.current_focus, Some(1));
        // Check that 1 is now at the front
        assert_eq!(fm.focus_history[0], 1);
    }

    #[test]
    fn test_focus_previous_with_empty_history() {
        let mut fm = FocusManager::new();
        let prev = fm.focus_previous();
        assert_eq!(prev, None);
    }

    #[test]
    fn test_focus_previous_with_single_window() {
        let mut fm = FocusManager::new();
        fm.add_to_history(1);

        let prev = fm.focus_previous();
        assert_eq!(prev, None);
    }

    #[test]
    fn test_focus_next() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.add_to_history(3);

        let next = fm.focus_next();
        assert_eq!(next, Some(2));
        assert_eq!(fm.current_focus, Some(2));
    }

    #[test]
    fn test_focus_next_cycles() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);

        // First call: focus 1
        let next1 = fm.focus_next();
        assert_eq!(next1, Some(1));

        // Second call: focus 2 (cycles back)
        let next2 = fm.focus_next();
        assert_eq!(next2, Some(2));
    }

    #[test]
    fn test_history_size_limit() {
        let mut fm = FocusManager::new().with_history_size(3);

        for i in 1..=5 {
            fm.add_to_history(i);
        }

        assert_eq!(fm.focus_history.len(), 3);
        assert_eq!(fm.focus_history[0], 5);
        assert_eq!(fm.focus_history[1], 4);
        assert_eq!(fm.focus_history[2], 3);
    }

    #[test]
    fn test_remove_from_history() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.add_to_history(3);

        fm.remove_from_history(2);

        assert_eq!(fm.focus_history.len(), 2);
        assert!(!fm.focus_history.contains(&2));
        assert_eq!(fm.focus_history[0], 3);
        assert_eq!(fm.focus_history[1], 1);
    }

    #[test]
    fn test_remove_current_focus_updates_focus() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.current_focus = Some(2);

        fm.remove_from_history(2);

        // Current focus should update to the next window in history
        assert_eq!(fm.current_focus, Some(1));
    }

    #[test]
    fn test_clear_history() {
        let mut fm = FocusManager::new();

        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.current_focus = Some(2);

        fm.clear_history();

        assert_eq!(fm.focus_history.len(), 0);
        assert_eq!(fm.current_focus, None);
    }

    #[test]
    fn test_directional_focus_left() {
        let current = Rect::new(100, 100, 200, 200);
        let left = Rect::new(0, 100, 90, 200);
        let right = Rect::new(310, 100, 200, 200);

        let candidates = vec![(1, left), (2, right)];

        let result = DirectionalFocus::find_window_in_direction(
            &current,
            Direction::Left,
            &candidates,
        );

        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_directional_focus_right() {
        let current = Rect::new(100, 100, 200, 200);
        let left = Rect::new(0, 100, 90, 200);
        let right = Rect::new(310, 100, 200, 200);

        let candidates = vec![(1, left), (2, right)];

        let result = DirectionalFocus::find_window_in_direction(
            &current,
            Direction::Right,
            &candidates,
        );

        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_directional_focus_up() {
        let current = Rect::new(100, 100, 200, 200);
        let above = Rect::new(100, 0, 200, 90);
        let below = Rect::new(100, 310, 200, 200);

        let candidates = vec![(1, above), (2, below)];

        let result =
            DirectionalFocus::find_window_in_direction(&current, Direction::Up, &candidates);

        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_directional_focus_down() {
        let current = Rect::new(100, 100, 200, 200);
        let above = Rect::new(100, 0, 200, 90);
        let below = Rect::new(100, 310, 200, 200);

        let candidates = vec![(1, above), (2, below)];

        let result =
            DirectionalFocus::find_window_in_direction(&current, Direction::Down, &candidates);

        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_directional_focus_no_window_in_direction() {
        let current = Rect::new(100, 100, 200, 200);
        let right = Rect::new(310, 100, 200, 200);

        let candidates = vec![(1, right)];

        let result =
            DirectionalFocus::find_window_in_direction(&current, Direction::Left, &candidates);

        assert_eq!(result, None);
    }

    #[test]
    fn test_directional_focus_closest_window() {
        let current = Rect::new(100, 100, 200, 200);
        let far_right = Rect::new(500, 100, 200, 200);
        let near_right = Rect::new(310, 100, 200, 200);

        let candidates = vec![(1, far_right), (2, near_right)];

        let result = DirectionalFocus::find_window_in_direction(
            &current,
            Direction::Right,
            &candidates,
        );

        // Should select the nearer window (2)
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_is_in_direction() {
        let current = Rect::new(100, 100, 200, 200);
        let left = Rect::new(0, 100, 90, 200);
        let right = Rect::new(310, 100, 200, 200);

        assert!(DirectionalFocus::is_in_direction(
            &current,
            &left,
            Direction::Left
        ));
        assert!(!DirectionalFocus::is_in_direction(
            &current,
            &left,
            Direction::Right
        ));
        assert!(DirectionalFocus::is_in_direction(
            &current,
            &right,
            Direction::Right
        ));
        assert!(!DirectionalFocus::is_in_direction(
            &current,
            &right,
            Direction::Left
        ));
    }

    #[test]
    fn test_calculate_distance() {
        let rect1 = Rect::new(0, 0, 100, 100);
        let rect2 = Rect::new(300, 400, 100, 100);

        let distance = DirectionalFocus::calculate_distance(&rect1, &rect2, Direction::Right);

        // Distance between centers (50, 50) and (350, 450)
        // sqrt((300)^2 + (400)^2) = 500
        assert!((distance - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_default_focus_manager() {
        let fm = FocusManager::default();
        assert_eq!(fm.current(), None);
        assert_eq!(fm.get_history().len(), 0);
    }
}
