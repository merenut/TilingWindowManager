use serde::{Serialize, Deserialize};

/// Represents a rectangle with position and dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    /// Calculate the area of the rectangle
    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    /// Check if a point is contained within the rectangle
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Split the rectangle horizontally (left/right)
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

    /// Split the rectangle vertically (top/bottom)
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

    /// Apply gaps to the rectangle
    pub fn apply_gaps(&self, gaps_in: i32, gaps_out: i32) -> Rect {
        Rect::new(
            self.x + gaps_out,
            self.y + gaps_out,
            self.width - 2 * gaps_out - gaps_in,
            self.height - 2 * gaps_out - gaps_in,
        )
    }

    /// Shrink the rectangle by a specified amount on all sides
    pub fn shrink(&self, amount: i32) -> Rect {
        Rect::new(
            self.x + amount,
            self.y + amount,
            self.width - 2 * amount,
            self.height - 2 * amount,
        )
    }

    /// Expand the rectangle by a specified amount on all sides
    pub fn expand(&self, amount: i32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + 2 * amount,
            self.height + 2 * amount,
        )
    }
}

/// Represents the split direction of a container node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Split {
    Horizontal,
    Vertical,
}

impl Split {
    /// Get the opposite split direction
    pub fn opposite(&self) -> Split {
        match self {
            Split::Horizontal => Split::Vertical,
            Split::Vertical => Split::Horizontal,
        }
    }
}

/// A node in the binary tree representing window layout
pub struct TreeNode {
    pub rect: Rect,
    pub split: Split,
}

impl TreeNode {
    /// Create a new container node with a split direction
    pub fn new_container(split: Split, rect: Rect) -> Self {
        TreeNode { rect, split }
    }

    /// Check if this node is a leaf (contains a window)
    pub fn is_leaf(&self) -> bool {
        // Placeholder implementation
        false
    }

    /// Check if this node is a container (has children)
    pub fn is_container(&self) -> bool {
        !self.is_leaf()
    }
}
