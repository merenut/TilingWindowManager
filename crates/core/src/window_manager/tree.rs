//! Binary tree data structure for managing window layouts in a tiling window manager.
//!
//! This module provides a binary tree implementation where:
//! - Leaf nodes contain windows (identified by HWND)
//! - Container nodes split space horizontally or vertically between two children
//! - The tree structure determines window positions and sizes
//!
//! # Example
//!
//! ```no_run
//! use tiling_wm_core::window_manager::{TreeNode, Rect, Split};
//! use windows::Win32::Foundation::HWND;
//!
//! // Create a root node with a window
//! let rect = Rect::new(0, 0, 1920, 1080);
//! let mut root = TreeNode::new_leaf(HWND(1 as _), rect);
//!
//! // Insert another window, creating a horizontal split
//! root = root.insert(HWND(2 as _), Split::Horizontal);
//!
//! // Collect all window positions
//! let windows = root.collect();
//! ```

use serde::{Serialize, Deserialize};
use windows::Win32::Foundation::HWND;

/// Represents a rectangle with position and dimensions.
///
/// Used to define the screen area occupied by windows and containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    /// X coordinate of the top-left corner
    pub x: i32,
    /// Y coordinate of the top-left corner
    pub y: i32,
    /// Width of the rectangle
    pub width: i32,
    /// Height of the rectangle
    pub height: i32,
}

impl Rect {
    /// Create a new rectangle with the specified position and dimensions.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `width` - Width of the rectangle
    /// * `height` - Height of the rectangle
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// assert_eq!(rect.width, 1920);
    /// assert_eq!(rect.height, 1080);
    /// ```
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    /// Calculate the area of the rectangle.
    ///
    /// # Returns
    ///
    /// The area in square pixels (width * height).
    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    /// Check if a point is contained within the rectangle.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the point
    /// * `y` - Y coordinate of the point
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the rectangle, `false` otherwise.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Check if this rectangle intersects with another rectangle.
    ///
    /// # Arguments
    ///
    /// * `other` - The other rectangle to check for intersection
    ///
    /// # Returns
    ///
    /// `true` if the rectangles overlap, `false` otherwise.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Split the rectangle horizontally into left and right parts.
    ///
    /// # Arguments
    ///
    /// * `ratio` - The ratio of the split (0.0 to 1.0), where 0.5 means equal split
    ///
    /// # Returns
    ///
    /// A tuple containing the left and right rectangles.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 100, 100);
    /// let (left, right) = rect.split_horizontal(0.5);
    /// assert_eq!(left.width, 50);
    /// assert_eq!(right.width, 50);
    /// ```
    pub fn split_horizontal(&self, ratio: f32) -> (Rect, Rect) {
        let split_width = (self.width as f32 * ratio) as i32;
        let left = Rect::new(self.x, self.y, split_width, self.height);
        let right = Rect::new(
            self.x + split_width,
            self.y,
            self.width - split_width,
            self.height,
        );
        (left, right)
    }

    /// Split the rectangle vertically into top and bottom parts.
    ///
    /// # Arguments
    ///
    /// * `ratio` - The ratio of the split (0.0 to 1.0), where 0.5 means equal split
    ///
    /// # Returns
    ///
    /// A tuple containing the top and bottom rectangles.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::Rect;
    ///
    /// let rect = Rect::new(0, 0, 100, 100);
    /// let (top, bottom) = rect.split_vertical(0.5);
    /// assert_eq!(top.height, 50);
    /// assert_eq!(bottom.height, 50);
    /// ```
    pub fn split_vertical(&self, ratio: f32) -> (Rect, Rect) {
        let split_height = (self.height as f32 * ratio) as i32;
        let top = Rect::new(self.x, self.y, self.width, split_height);
        let bottom = Rect::new(
            self.x,
            self.y + split_height,
            self.width,
            self.height - split_height,
        );
        (top, bottom)
    }

    /// Apply gaps to the rectangle, reducing its size for visual separation.
    ///
    /// # Arguments
    ///
    /// * `gaps_in` - Inner gap size (between windows)
    /// * `gaps_out` - Outer gap size (from screen edges)
    ///
    /// # Returns
    ///
    /// A new rectangle with gaps applied.
    pub fn apply_gaps(&self, gaps_in: i32, gaps_out: i32) -> Rect {
        Rect::new(
            self.x + gaps_out,
            self.y + gaps_out,
            self.width - 2 * gaps_out - gaps_in,
            self.height - 2 * gaps_out - gaps_in,
        )
    }

    /// Shrink the rectangle by a specified amount on all sides.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to shrink by
    ///
    /// # Returns
    ///
    /// A new rectangle shrunk by the specified amount.
    pub fn shrink(&self, amount: i32) -> Rect {
        Rect::new(
            self.x + amount,
            self.y + amount,
            self.width - 2 * amount,
            self.height - 2 * amount,
        )
    }

    /// Expand the rectangle by a specified amount on all sides.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to expand by
    ///
    /// # Returns
    ///
    /// A new rectangle expanded by the specified amount.
    pub fn expand(&self, amount: i32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + 2 * amount,
            self.height + 2 * amount,
        )
    }
}

/// Represents the split direction of a container node.
///
/// A container node divides its space between two children either
/// horizontally (left/right) or vertically (top/bottom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Split {
    /// Horizontal split (left/right)
    Horizontal,
    /// Vertical split (top/bottom)
    Vertical,
}

impl Split {
    /// Get the opposite split direction.
    ///
    /// # Returns
    ///
    /// `Split::Vertical` if this is `Horizontal`, and vice versa.
    ///
    /// # Example
    ///
    /// ```
    /// use tiling_wm_core::window_manager::Split;
    ///
    /// let split = Split::Horizontal;
    /// assert_eq!(split.opposite(), Split::Vertical);
    /// ```
    pub fn opposite(&self) -> Split {
        match self {
            Split::Horizontal => Split::Vertical,
            Split::Vertical => Split::Horizontal,
        }
    }
}

/// A node in the binary tree representing window layout.
///
/// Nodes can be either:
/// - **Leaf nodes**: Contain a single window (HWND)
/// - **Container nodes**: Split space between two child nodes
///
/// The tree structure automatically calculates window positions based on
/// the split directions and available space.
pub struct TreeNode {
    rect: Rect,
    node_type: NodeType,
}

/// Internal enum to distinguish between leaf and container nodes.
enum NodeType {
    /// A leaf node containing a window handle
    Leaf { hwnd: HWND },
    /// A container node with two children and a split direction
    Container {
        split: Split,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
        ratio: f32,
    },
}

impl TreeNode {
    /// Create a new leaf node containing a window.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle
    /// * `rect` - The rectangle defining the window's position and size
    ///
    /// # Returns
    ///
    /// A new leaf node.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::{TreeNode, Rect};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let node = TreeNode::new_leaf(HWND(1 as _), rect);
    /// ```
    pub fn new_leaf(hwnd: HWND, rect: Rect) -> Self {
        TreeNode {
            rect,
            node_type: NodeType::Leaf { hwnd },
        }
    }

    /// Create a new container node with a split direction.
    ///
    /// # Arguments
    ///
    /// * `split` - The split direction (horizontal or vertical)
    /// * `left` - The left/top child node
    /// * `right` - The right/bottom child node
    /// * `rect` - The rectangle defining the container's space
    /// * `ratio` - The split ratio between left and right children (default 0.5)
    ///
    /// # Returns
    ///
    /// A new container node.
    pub fn new_container(split: Split, left: TreeNode, right: TreeNode, rect: Rect, ratio: f32) -> Self {
        TreeNode {
            rect,
            node_type: NodeType::Container {
                split,
                left: Box::new(left),
                right: Box::new(right),
                ratio,
            },
        }
    }

    /// Check if this node is a leaf (contains a window).
    ///
    /// # Returns
    ///
    /// `true` if this is a leaf node, `false` if it's a container.
    pub fn is_leaf(&self) -> bool {
        matches!(self.node_type, NodeType::Leaf { .. })
    }

    /// Check if this node is a container (has children).
    ///
    /// # Returns
    ///
    /// `true` if this is a container node, `false` if it's a leaf.
    pub fn is_container(&self) -> bool {
        matches!(self.node_type, NodeType::Container { .. })
    }

    /// Get the rectangle defining this node's space.
    ///
    /// # Returns
    ///
    /// The node's rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the window handle if this is a leaf node.
    ///
    /// # Returns
    ///
    /// `Some(HWND)` if this is a leaf node, `None` if it's a container.
    pub fn hwnd(&self) -> Option<HWND> {
        match &self.node_type {
            NodeType::Leaf { hwnd } => Some(*hwnd),
            NodeType::Container { .. } => None,
        }
    }

    /// Insert a new window into the tree, creating a split at this node.
    ///
    /// If this is a leaf node, it becomes a container with the old window in one child
    /// and the new window in the other. If this is already a container, the insertion
    /// happens in the right child.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle to insert
    /// * `split` - The split direction for the new container
    ///
    /// # Returns
    ///
    /// The modified tree with the new window inserted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::{TreeNode, Rect, Split};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut root = TreeNode::new_leaf(HWND(1 as _), rect);
    /// root = root.insert(HWND(2 as _), Split::Horizontal);
    /// ```
    pub fn insert(self, hwnd: HWND, split: Split) -> Self {
        match self.node_type {
            NodeType::Leaf { hwnd: old_hwnd } => {
                // Split this leaf into a container with two leaves
                let (left_rect, right_rect) = match split {
                    Split::Horizontal => self.rect.split_horizontal(0.5),
                    Split::Vertical => self.rect.split_vertical(0.5),
                };
                
                let left = TreeNode::new_leaf(old_hwnd, left_rect);
                let right = TreeNode::new_leaf(hwnd, right_rect);
                
                TreeNode::new_container(split, left, right, self.rect, 0.5)
            }
            NodeType::Container {
                split: current_split,
                left,
                right,
                ratio,
            } => {
                // Insert into the right child
                let new_right = right.insert(hwnd, split);
                TreeNode::new_container(current_split, *left, new_right, self.rect, ratio)
            }
        }
    }

    /// Remove a window from the tree.
    ///
    /// If the window is found in a leaf node, that node is removed and its sibling
    /// takes its place. Returns `None` if the tree becomes empty.
    ///
    /// # Arguments
    ///
    /// * `hwnd` - The window handle to remove
    ///
    /// # Returns
    ///
    /// `Some(TreeNode)` if the tree is not empty after removal, `None` if it becomes empty.
    pub fn remove(self, hwnd: HWND) -> Option<Self> {
        match self.node_type {
            NodeType::Leaf { hwnd: leaf_hwnd } => {
                if leaf_hwnd == hwnd {
                    None // Remove this leaf
                } else {
                    Some(self) // Not the target window
                }
            }
            NodeType::Container {
                split,
                left,
                right,
                ratio,
            } => {
                // Try removing from left child
                match left.remove(hwnd) {
                    Some(new_left) => {
                        // Window not found in left or left still has nodes
                        match right.remove(hwnd) {
                            Some(new_right) => {
                                // Window not found in right or right still has nodes
                                Some(TreeNode::new_container(split, new_left, new_right, self.rect, ratio))
                            }
                            None => {
                                // Right child was removed, promote left child
                                Some(new_left.with_rect(self.rect))
                            }
                        }
                    }
                    None => {
                        // Left child was removed, promote right child
                        Some(right.with_rect(self.rect))
                    }
                }
            }
        }
    }

    /// Update the node's rectangle, recursively updating all children.
    ///
    /// # Arguments
    ///
    /// * `rect` - The new rectangle for this node
    ///
    /// # Returns
    ///
    /// The node with updated rectangles.
    fn with_rect(self, rect: Rect) -> Self {
        match self.node_type {
            NodeType::Leaf { hwnd } => TreeNode::new_leaf(hwnd, rect),
            NodeType::Container {
                split,
                left,
                right,
                ratio,
            } => {
                let (left_rect, right_rect) = match split {
                    Split::Horizontal => rect.split_horizontal(ratio),
                    Split::Vertical => rect.split_vertical(ratio),
                };
                
                let new_left = left.with_rect(left_rect);
                let new_right = right.with_rect(right_rect);
                
                TreeNode::new_container(split, new_left, new_right, rect, ratio)
            }
        }
    }

    /// Rebalance the tree by resetting all split ratios to 0.5.
    ///
    /// This ensures all windows have equal space in their respective containers.
    ///
    /// # Returns
    ///
    /// The rebalanced tree.
    pub fn rebalance(self) -> Self {
        match self.node_type {
            NodeType::Leaf { .. } => self,
            NodeType::Container {
                split,
                left,
                right,
                ratio: _,
            } => {
                let new_ratio = 0.5;
                let (left_rect, right_rect) = match split {
                    Split::Horizontal => self.rect.split_horizontal(new_ratio),
                    Split::Vertical => self.rect.split_vertical(new_ratio),
                };
                
                let new_left = left.with_rect(left_rect).rebalance();
                let new_right = right.with_rect(right_rect).rebalance();
                
                TreeNode::new_container(split, new_left, new_right, self.rect, new_ratio)
            }
        }
    }

    /// Collect all leaf nodes (windows) with their rectangles.
    ///
    /// This traverses the tree and returns a list of all windows with their
    /// calculated positions and sizes.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing window handles and their rectangles.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tiling_wm_core::window_manager::{TreeNode, Rect, Split};
    /// use windows::Win32::Foundation::HWND;
    ///
    /// let rect = Rect::new(0, 0, 1920, 1080);
    /// let mut root = TreeNode::new_leaf(HWND(1 as _), rect);
    /// root = root.insert(HWND(2 as _), Split::Horizontal);
    ///
    /// let windows = root.collect();
    /// assert_eq!(windows.len(), 2);
    /// ```
    pub fn collect(&self) -> Vec<(HWND, Rect)> {
        let mut result = Vec::new();
        self.collect_recursive(&mut result);
        result
    }

    /// Helper function for recursive collection of leaf nodes.
    fn collect_recursive(&self, result: &mut Vec<(HWND, Rect)>) {
        match &self.node_type {
            NodeType::Leaf { hwnd } => {
                result.push((*hwnd, self.rect));
            }
            NodeType::Container { left, right, .. } => {
                left.collect_recursive(result);
                right.collect_recursive(result);
            }
        }
    }

    /// Apply the tree's window layout using Win32 API.
    ///
    /// This method positions and sizes all windows according to the tree structure.
    /// It uses the Win32 SetWindowPos API to move and resize windows.
    ///
    /// # Arguments
    ///
    /// * `gaps_in` - Inner gap size between windows
    /// * `gaps_out` - Outer gap size from screen edges
    ///
    /// # Returns
    ///
    /// `Ok(())` if all windows were positioned successfully, or an error if any operation failed.
    ///
    /// # Platform
    ///
    /// This function is only available on Windows platforms.
    #[cfg(target_os = "windows")]
    pub fn apply_layout(&self, gaps_in: i32, gaps_out: i32) -> anyhow::Result<()> {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE};
        
        for (hwnd, rect) in self.collect() {
            // Apply gaps to the rectangle
            let final_rect = if gaps_in > 0 || gaps_out > 0 {
                rect.apply_gaps(gaps_in, gaps_out)
            } else {
                rect
            };
            
            // Position and size the window
            unsafe {
                SetWindowPos(
                    hwnd,
                    None,
                    final_rect.x,
                    final_rect.y,
                    final_rect.width,
                    final_rect.height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )?;
            }
        }
        
        Ok(())
    }

    /// Apply the tree's window layout using Win32 API (non-Windows stub).
    ///
    /// This is a placeholder implementation for non-Windows platforms.
    #[cfg(not(target_os = "windows"))]
    pub fn apply_layout(&self, _gaps_in: i32, _gaps_out: i32) -> anyhow::Result<()> {
        Ok(())
    }
}
