//! Dwindle layout algorithm for tiling window managers.
//!
//! The dwindle layout uses a binary tree structure where each window insertion
//! creates a split in the available space. The split direction is automatically
//! determined based on the rectangle's dimensions (smart split).
//!
//! # Features
//!
//! - Smart split direction based on window aspect ratio
//! - Configurable split ratios
//! - Gap support (inner and outer)
//! - Automatic tree balancing on window removal
//!
//! # Example
//!
//! ```no_run
//! use tenraku_core::window_manager::layout::DwindleLayout;
//! use tenraku_core::window_manager::{TreeNode, Rect, Split};
//! use windows::Win32::Foundation::HWND;
//!
//! let layout = DwindleLayout::new();
//! let rect = Rect::new(0, 0, 1920, 1080);
//! let mut tree = TreeNode::new_leaf(HWND(0), rect);
//!
//! // Insert windows using dwindle layout
//! layout.insert_window(&mut tree, HWND(1 as _)).ok();
//! layout.insert_window(&mut tree, HWND(2 as _)).ok();
//!
//! // Apply the layout to position windows
//! layout.apply(&tree).ok();
//! ```

use crate::window_manager::tree::{Rect, Split, TreeNode};
use windows::Win32::Foundation::HWND;

/// Configuration and logic for the dwindle layout algorithm.
///
/// The dwindle layout automatically determines split directions based on
/// the available space's dimensions and applies configurable gaps between windows.
#[derive(Debug, Clone)]
pub struct DwindleLayout {
    /// Ratio for splits (0.1 to 0.9, default 0.5 = 50/50)
    pub ratio: f32,
    /// Auto-choose split direction based on dimensions
    pub smart_split: bool,
    /// Remove gaps when only one window
    pub no_gaps_when_only: bool,
    /// Gap size between windows
    pub gaps_in: i32,
    /// Gap size from screen edges
    pub gaps_out: i32,
}

impl Default for DwindleLayout {
    fn default() -> Self {
        Self {
            ratio: 0.5,
            smart_split: true,
            no_gaps_when_only: false,
            gaps_in: 5,
            gaps_out: 10,
        }
    }
}

impl DwindleLayout {
    /// Create a new DwindleLayout with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    ///
    /// let layout = DwindleLayout::new();
    /// assert_eq!(layout.ratio, 0.5);
    /// assert!(layout.smart_split);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the split ratio.
    ///
    /// The ratio determines how space is divided between children.
    /// Values are clamped to the range [0.1, 0.9].
    ///
    /// # Arguments
    ///
    /// * `ratio` - The split ratio (0.1 to 0.9)
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    ///
    /// let layout = DwindleLayout::new().with_ratio(0.6);
    /// assert_eq!(layout.ratio, 0.6);
    /// ```
    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.1, 0.9);
        self
    }

    /// Enable or disable smart split direction.
    ///
    /// When enabled, the split direction is chosen based on the rectangle's
    /// aspect ratio (wider = horizontal split, taller = vertical split).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable smart split
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    ///
    /// let layout = DwindleLayout::new().with_smart_split(false);
    /// assert!(!layout.smart_split);
    /// ```
    pub fn with_smart_split(mut self, enabled: bool) -> Self {
        self.smart_split = enabled;
        self
    }

    /// Determine split direction based on rectangle dimensions.
    ///
    /// If smart split is enabled:
    /// - Wide rectangles (width > height) split horizontally (left/right)
    /// - Tall rectangles (height >= width) split vertically (top/bottom)
    ///
    /// If smart split is disabled, always returns vertical split.
    ///
    /// # Arguments
    ///
    /// * `rect` - The rectangle to analyze
    ///
    /// # Returns
    ///
    /// The recommended split direction.
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    /// use tenraku_core::window_manager::{Rect, Split};
    ///
    /// let layout = DwindleLayout::new();
    /// let wide_rect = Rect::new(0, 0, 1920, 1080);
    /// assert_eq!(layout.calculate_split_direction(&wide_rect), Split::Horizontal);
    ///
    /// let tall_rect = Rect::new(0, 0, 800, 1200);
    /// assert_eq!(layout.calculate_split_direction(&tall_rect), Split::Vertical);
    /// ```
    pub fn calculate_split_direction(&self, rect: &Rect) -> Split {
        if self.smart_split {
            if rect.width > rect.height {
                // Wider than tall: split horizontally (left/right)
                Split::Horizontal
            } else {
                // Taller than wide: split vertically (top/bottom)
                Split::Vertical
            }
        } else {
            // Default to vertical split
            Split::Vertical
        }
    }

    /// Calculate optimal split ratio based on tree depth.
    ///
    /// Uses golden ratio (0.618) for deeper levels to create more pleasant
    /// proportions. Uses configured ratio for shallow levels.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth in the tree (0 = root)
    ///
    /// # Returns
    ///
    /// The split ratio to use.
    ///
    /// # Note
    ///
    /// This method is currently not used in the implementation as TreeNode
    /// uses a fixed 0.5 ratio. It's provided for future enhancements where
    /// dynamic ratio calculation based on tree depth might be desired.
    pub fn calculate_split_ratio(&self, depth: usize) -> f32 {
        // Use golden ratio for pleasant proportions at deeper levels
        if depth > 2 {
            0.618 // Golden ratio
        } else {
            self.ratio
        }
    }

    /// Insert a window into the tree using dwindle layout.
    ///
    /// This method modifies the tree in place by:
    /// 1. Finding the best insertion point (rightmost leaf)
    /// 2. Determining the optimal split direction
    /// 3. Converting the leaf into a container with the new window
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree to insert into
    /// * `hwnd` - The window handle to insert
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    /// use tenraku_core::window_manager::{TreeNode, Rect};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let layout = DwindleLayout::new();
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut tree = TreeNode::new_leaf(HWND(0), rect);
    ///
    /// layout.insert_window(&mut tree, HWND(1 as _)).ok();
    /// ```
    pub fn insert_window(&self, tree: &mut TreeNode, hwnd: HWND) -> anyhow::Result<()> {
        // If tree is a placeholder (HWND 0), replace it
        if tree.hwnd() == Some(HWND(0)) {
            let rect = tree.rect();
            *tree = TreeNode::new_leaf(hwnd, rect);
            return Ok(());
        }

        // Use insert_with_fn to calculate split direction at each level
        let rect = tree.rect();
        let new_tree = std::mem::replace(tree, TreeNode::new_leaf(HWND(0), rect));

        let split_fn = |r: &Rect| self.calculate_split_direction(r);
        *tree = new_tree.insert_with_fn(hwnd, &split_fn);

        Ok(())
    }

    /// Remove a window from the tree.
    ///
    /// When a window is removed, the tree is collapsed by promoting the
    /// sibling node to replace the parent container.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree to remove from
    /// * `hwnd` - The window handle to remove
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the window was found and removed, `Ok(false)` if not found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    /// use tenraku_core::window_manager::{TreeNode, Rect};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let layout = DwindleLayout::new();
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut tree = TreeNode::new_leaf(HWND(1 as _), rect);
    ///
    /// layout.remove_window(&mut tree, HWND(1 as _)).ok();
    /// ```
    pub fn remove_window(&self, tree: &mut TreeNode, hwnd: HWND) -> anyhow::Result<bool> {
        // Check if the window exists in the tree first
        // Note: This traverses the entire tree. For large trees, a dedicated
        // contains_window() method that can short-circuit would be more efficient.
        // However, for typical window counts (< 20), this is acceptable.
        let window_exists = tree.collect().iter().any(|(h, _)| *h == hwnd);

        if !window_exists {
            return Ok(false);
        }

        let rect = tree.rect();
        let temp_tree = std::mem::replace(tree, TreeNode::new_leaf(HWND(0), rect));

        match temp_tree.remove(hwnd) {
            Some(new_tree) => {
                *tree = new_tree;
                Ok(true)
            }
            None => {
                // Tree is now empty, replace with placeholder
                *tree = TreeNode::new_leaf(HWND(0), rect);
                Ok(true)
            }
        }
    }

    /// Apply the layout to the tree (recalculate and position all windows).
    ///
    /// This method:
    /// 1. Ensures the tree structure is valid
    /// 2. Applies the configured gaps
    /// 3. Positions all windows using the Win32 API
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree to apply layout to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if window positioning fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::layout::DwindleLayout;
    /// use tenraku_core::window_manager::{TreeNode, Rect};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let layout = DwindleLayout::new();
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let tree = TreeNode::new_leaf(HWND(1 as _), rect);
    ///
    /// layout.apply(&tree).ok();
    /// ```
    pub fn apply(&self, tree: &TreeNode) -> anyhow::Result<()> {
        // Don't apply layout to empty placeholder trees
        if tree.hwnd() == Some(HWND(0)) {
            return Ok(());
        }

        // Use TreeNode's built-in apply_layout with our gap settings
        tree.apply_layout(self.gaps_in, self.gaps_out)?;

        Ok(())
    }
}