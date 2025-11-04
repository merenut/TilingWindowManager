# Phase 2: Core Window Management - Detailed Task List

**Timeline:** Weeks 4-8 (5 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 2 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 2 Goals:**
- Implement complete dwindle layout algorithm with smart split direction
- Implement master-stack layout algorithm
- Create comprehensive window state management system
- Build focus management with history tracking
- Implement floating window support
- Add fullscreen window capability
- Create command system for window operations
- Support directional window navigation
- Enable window moving and swapping in tree

**Prerequisites:**
- Phase 1 completed successfully
- All Phase 1 tests passing
- Basic window enumeration and tiling working
- Event loop detecting window events

---

## Success Criteria for Phase 2 Completion

Phase 2 is considered complete when:

1. **Dwindle layout fully functional:**
   - Smart split direction based on window aspect ratio
   - Windows tile correctly in binary tree structure
   - Tree rebalances properly when windows added/removed

2. **Master layout working:**
   - Master window takes configurable portion of screen
   - Stack windows split remaining space
   - Can adjust master factor dynamically
   - Supports multiple master windows

3. **Window states implemented:**
   - Tiled state (managed by layout)
   - Floating state (user-positioned)
   - Fullscreen state (covers entire screen)
   - Minimized state (hidden but tracked)

4. **Focus management complete:**
   - Can focus windows in all four directions
   - Focus history tracks recently focused windows
   - Alt-Tab style focus switching works
   - Focus follows window operations

5. **Window operations work:**
   - Close active window
   - Toggle floating/tiled
   - Toggle fullscreen
   - Move window in tree (swap positions)
   - Resize windows

6. **All tests passing:**
   - Unit tests for layouts
   - Unit tests for window states
   - Integration tests for window management
   - Manual validation successful

---

## Task Breakdown

### Week 4: Dwindle Layout Algorithm

#### Task 2.1: Implement Smart Split Direction Logic

**Objective:** Create logic to automatically determine whether to split horizontally or vertically based on window dimensions.

**File:** `crates/core/src/window_manager/layout/dwindle.rs`

**Required Implementations:**

1. **Create layout module structure:**
   ```bash
   mkdir -p crates/core/src/window_manager/layout
   touch crates/core/src/window_manager/layout/mod.rs
   touch crates/core/src/window_manager/layout/dwindle.rs
   touch crates/core/src/window_manager/layout/master.rs
   ```

2. **DwindleLayout struct:**

   ```rust
   use crate::window_manager::tree::{TreeNode, Rect, Split};
   use crate::utils::win32::WindowHandle;

   #[derive(Debug, Clone)]
   pub struct DwindleLayout {
       /// Ratio for splits (default 0.5 = 50/50)
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
       pub fn new() -> Self {
           Self::default()
       }

       pub fn with_ratio(mut self, ratio: f32) -> Self {
           self.ratio = ratio.clamp(0.1, 0.9);
           self
       }

       pub fn with_smart_split(mut self, enabled: bool) -> Self {
           self.smart_split = enabled;
           self
       }

       /// Determine split direction based on rectangle dimensions
       pub fn calculate_split_direction(&self, rect: &Rect) -> Split {
           if self.smart_split {
               if rect.width > rect.height {
                   // Wider than tall: split vertically (left/right)
                   Split::Horizontal
               } else {
                   // Taller than wide: split horizontally (top/bottom)
                   Split::Vertical
               }
           } else {
               // Default to vertical split
               Split::Vertical
           }
       }

       /// Calculate optimal split ratio based on window count
       pub fn calculate_split_ratio(&self, depth: usize) -> f32 {
           // Use golden ratio for pleasant proportions at deeper levels
           if depth > 2 {
               0.618  // Golden ratio
           } else {
               self.ratio
           }
       }
   }
   ```

**Acceptance Criteria:**
- [ ] DwindleLayout struct compiles without errors
- [ ] Smart split correctly chooses horizontal for wide rectangles
- [ ] Smart split correctly chooses vertical for tall rectangles
- [ ] Ratio clamping prevents invalid values
- [ ] Default configuration is sensible

**Testing Requirements:**

Create `crates/core/src/window_manager/layout/dwindle_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_manager::tree::Rect;

    #[test]
    fn test_smart_split_wide_rectangle() {
        let layout = DwindleLayout::new();
        let wide_rect = Rect::new(0, 0, 1920, 1080);
        
        assert_eq!(
            layout.calculate_split_direction(&wide_rect),
            Split::Horizontal,
            "Wide rectangle should split horizontally"
        );
    }

    #[test]
    fn test_smart_split_tall_rectangle() {
        let layout = DwindleLayout::new();
        let tall_rect = Rect::new(0, 0, 800, 1200);
        
        assert_eq!(
            layout.calculate_split_direction(&tall_rect),
            Split::Vertical,
            "Tall rectangle should split vertically"
        );
    }

    #[test]
    fn test_smart_split_disabled() {
        let layout = DwindleLayout::new().with_smart_split(false);
        let wide_rect = Rect::new(0, 0, 1920, 1080);
        
        assert_eq!(
            layout.calculate_split_direction(&wide_rect),
            Split::Vertical,
            "With smart_split disabled, should default to vertical"
        );
    }

    #[test]
    fn test_ratio_clamping() {
        let layout = DwindleLayout::new().with_ratio(1.5);
        assert!(layout.ratio <= 0.9, "Ratio should be clamped to max 0.9");
        
        let layout = DwindleLayout::new().with_ratio(-0.5);
        assert!(layout.ratio >= 0.1, "Ratio should be clamped to min 0.1");
    }

    #[test]
    fn test_split_ratio_calculation() {
        let layout = DwindleLayout::new();
        
        assert_eq!(layout.calculate_split_ratio(0), 0.5);
        assert_eq!(layout.calculate_split_ratio(1), 0.5);
        assert_eq!(layout.calculate_split_ratio(3), 0.618);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core dwindle
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 2.2: Implement Dwindle Window Insertion

**Objective:** Implement logic to insert windows into the tree using dwindle algorithm.

**File:** `crates/core/src/window_manager/layout/dwindle.rs` (continue)

**Required Implementations:**

```rust
impl DwindleLayout {
    /// Insert a window into the tree using dwindle layout
    pub fn insert_window(
        &self,
        tree: &mut TreeNode,
        window: WindowHandle,
    ) -> anyhow::Result<()> {
        // If tree is empty (container with no children), make this a leaf
        if tree.is_container() && tree.left.is_none() && tree.right.is_none() {
            tree.window = Some(window);
            return Ok(());
        }

        // Find the best insertion point
        let insert_node = self.find_insertion_point(tree);
        let split_dir = self.calculate_split_direction(&insert_node.rect);
        
        // Insert the window
        self.insert_at_node(insert_node, window, split_dir)?;
        
        Ok(())
    }

    /// Find the optimal node to insert a new window
    fn find_insertion_point<'a>(&self, tree: &'a mut TreeNode) -> &'a mut TreeNode {
        // Simple strategy: find the rightmost leaf (most recently added)
        if tree.is_leaf() {
            return tree;
        }

        // Prefer inserting on the right (most recent position)
        if let Some(ref mut right) = tree.right {
            return self.find_insertion_point(right);
        }

        // Fallback to left if right doesn't exist
        if let Some(ref mut left) = tree.left {
            return self.find_insertion_point(left);
        }

        // If both are None, this node is the insertion point
        tree
    }

    /// Insert window at a specific node
    fn insert_at_node(
        &self,
        node: &mut TreeNode,
        window: WindowHandle,
        split: Split,
    ) -> anyhow::Result<()> {
        if !node.is_leaf() {
            anyhow::bail!("Can only insert at leaf nodes");
        }

        // Get the current window from this leaf
        let current_window = node.window.take()
            .ok_or_else(|| anyhow::anyhow!("Leaf node has no window"))?;

        // Convert this leaf to a container
        node.split = split;

        // Split the rectangle
        let (left_rect, right_rect) = match split {
            Split::Horizontal => node.rect.split_horizontal(self.ratio),
            Split::Vertical => node.rect.split_vertical(self.ratio),
        };

        // Apply gaps
        let left_rect = if self.no_gaps_when_only && node.count_windows() == 0 {
            left_rect
        } else {
            left_rect.apply_gaps(self.gaps_in, self.gaps_out)
        };

        let right_rect = if self.no_gaps_when_only && node.count_windows() == 0 {
            right_rect
        } else {
            right_rect.apply_gaps(self.gaps_in, self.gaps_out)
        };

        // Create child nodes
        node.left = Some(Box::new(TreeNode::new_leaf(current_window, left_rect)));
        node.right = Some(Box::new(TreeNode::new_leaf(window, right_rect)));

        Ok(())
    }

    /// Remove a window from the tree
    pub fn remove_window(
        &self,
        tree: &mut TreeNode,
        window: &WindowHandle,
    ) -> anyhow::Result<bool> {
        // Find and remove the window, then collapse the tree
        self.remove_window_recursive(tree, window)
    }

    fn remove_window_recursive(
        &self,
        node: &mut TreeNode,
        window: &WindowHandle,
    ) -> anyhow::Result<bool> {
        // If this is a leaf, check if it's the window we're looking for
        if node.is_leaf() {
            if let Some(ref w) = node.window {
                return Ok(w.0 == window.0);
            }
            return Ok(false);
        }

        // Check left child
        if let Some(ref mut left) = node.left {
            if self.remove_window_recursive(left, window)? {
                // Window was in left subtree
                // Replace this node with the right child
                if let Some(right) = node.right.take() {
                    *node = *right;
                }
                return Ok(true);
            }
        }

        // Check right child
        if let Some(ref mut right) = node.right {
            if self.remove_window_recursive(right, window)? {
                // Window was in right subtree
                // Replace this node with the left child
                if let Some(left) = node.left.take() {
                    *node = *left;
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Apply the layout to the tree (recalculate and position all windows)
    pub fn apply(&self, tree: &mut TreeNode) -> anyhow::Result<()> {
        // Recalculate split ratios and apply geometry
        self.recalculate_tree(tree)?;
        tree.apply_geometry()?;
        Ok(())
    }

    fn recalculate_tree(&self, node: &mut TreeNode) -> anyhow::Result<()> {
        if node.is_leaf() {
            return Ok(());
        }

        // Recalculate split for this container
        node.split = self.calculate_split_direction(&node.rect);

        // Split the rectangle
        let (left_rect, right_rect) = match node.split {
            Split::Horizontal => node.rect.split_horizontal(self.ratio),
            Split::Vertical => node.rect.split_vertical(self.ratio),
        };

        // Update child rectangles and recurse
        if let Some(ref mut left) = node.left {
            left.rect = left_rect;
            self.recalculate_tree(left)?;
        }

        if let Some(ref mut right) = node.right {
            right.rect = right_rect;
            self.recalculate_tree(right)?;
        }

        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Can insert windows into tree successfully
- [ ] Windows are positioned according to dwindle algorithm
- [ ] Smart split direction is applied correctly
- [ ] Window removal works and collapses tree properly
- [ ] Tree remains balanced after operations
- [ ] Gaps are applied correctly
- [ ] No memory leaks with Box pointers

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_first_window() {
        let layout = DwindleLayout::new();
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut tree = TreeNode::new_container(Split::Vertical, rect);
        
        let window = WindowHandle::from_hwnd(HWND(12345));
        layout.insert_window(&mut tree, window).unwrap();
        
        assert_eq!(tree.count_windows(), 1);
        assert!(tree.is_leaf());
    }

    #[test]
    fn test_insert_second_window() {
        let layout = DwindleLayout::new();
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut tree = TreeNode::new_container(Split::Vertical, rect);
        
        let window1 = WindowHandle::from_hwnd(HWND(1));
        let window2 = WindowHandle::from_hwnd(HWND(2));
        
        layout.insert_window(&mut tree, window1).unwrap();
        layout.insert_window(&mut tree, window2).unwrap();
        
        assert_eq!(tree.count_windows(), 2);
        assert!(tree.is_container());
        assert!(tree.left.is_some());
        assert!(tree.right.is_some());
    }

    #[test]
    fn test_remove_window() {
        let layout = DwindleLayout::new();
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut tree = TreeNode::new_container(Split::Vertical, rect);
        
        let window1 = WindowHandle::from_hwnd(HWND(1));
        let window2 = WindowHandle::from_hwnd(HWND(2));
        
        layout.insert_window(&mut tree, window1).unwrap();
        layout.insert_window(&mut tree, window2).unwrap();
        
        let removed = layout.remove_window(&mut tree, &window1).unwrap();
        assert!(removed, "Window should be found and removed");
        assert_eq!(tree.count_windows(), 1);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core layout::dwindle
cargo run -p tiling-wm-core  # Manual testing with multiple windows
```

---

### Week 5: Master Layout Algorithm

#### Task 2.3: Implement Master-Stack Layout

**Objective:** Create master-stack layout where one or more "master" windows take a portion of the screen, and remaining windows stack in the other portion.

**File:** `crates/core/src/window_manager/layout/master.rs`

**Required Implementations:**

```rust
use crate::window_manager::tree::Rect;
use crate::utils::win32::WindowHandle;

#[derive(Debug, Clone)]
pub struct MasterLayout {
    /// Portion of screen for master area (0.0 - 1.0)
    pub master_factor: f32,
    /// Number of windows in master area
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_master_factor(mut self, factor: f32) -> Self {
        self.master_factor = factor.clamp(0.1, 0.9);
        self
    }

    pub fn with_master_count(mut self, count: usize) -> Self {
        self.master_count = count.max(1);
        self
    }

    /// Apply master layout to a list of windows
    pub fn apply(&self, windows: &[WindowHandle], area: Rect) -> anyhow::Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        // Single window: take full area
        if windows.len() == 1 {
            let rect = area.apply_gaps(0, self.gaps_out);
            windows[0].set_pos(rect.x, rect.y, rect.width, rect.height)?;
            return Ok(());
        }

        // Determine master and stack window counts
        let master_count = self.master_count.min(windows.len());
        let stack_count = windows.len() - master_count;

        if stack_count == 0 {
            // All windows are masters: split them vertically
            self.tile_masters_only(&windows[..master_count], area)?;
        } else {
            // Split between master and stack
            self.tile_master_stack(
                &windows[..master_count],
                &windows[master_count..],
                area,
            )?;
        }

        Ok(())
    }

    fn tile_masters_only(&self, masters: &[WindowHandle], area: Rect) -> anyhow::Result<()> {
        let height_per_window = area.height / masters.len() as i32;

        for (i, window) in masters.iter().enumerate() {
            let y = area.y + (i as i32 * height_per_window);
            let rect = Rect::new(area.x, y, area.width, height_per_window)
                .apply_gaps(self.gaps_in, self.gaps_out);
            
            window.set_pos(rect.x, rect.y, rect.width, rect.height)?;
        }

        Ok(())
    }

    fn tile_master_stack(
        &self,
        masters: &[WindowHandle],
        stack: &[WindowHandle],
        area: Rect,
    ) -> anyhow::Result<()> {
        // Calculate master and stack areas
        let master_width = (area.width as f32 * self.master_factor) as i32;
        let stack_width = area.width - master_width;

        let master_area = Rect::new(area.x, area.y, master_width, area.height);
        let stack_area = Rect::new(
            area.x + master_width,
            area.y,
            stack_width,
            area.height,
        );

        // Tile master windows
        self.tile_vertical(masters, master_area)?;

        // Tile stack windows
        self.tile_vertical(stack, stack_area)?;

        Ok(())
    }

    fn tile_vertical(&self, windows: &[WindowHandle], area: Rect) -> anyhow::Result<()> {
        if windows.is_empty() {
            return Ok(());
        }

        let height_per_window = area.height / windows.len() as i32;

        for (i, window) in windows.iter().enumerate() {
            let y = area.y + (i as i32 * height_per_window);
            let rect = Rect::new(area.x, y, area.width, height_per_window)
                .apply_gaps(self.gaps_in, self.gaps_out);
            
            window.set_pos(rect.x, rect.y, rect.width, rect.height)?;
        }

        Ok(())
    }

    /// Increase master count
    pub fn increase_master_count(&mut self) {
        self.master_count += 1;
    }

    /// Decrease master count (minimum 1)
    pub fn decrease_master_count(&mut self) {
        if self.master_count > 1 {
            self.master_count -= 1;
        }
    }

    /// Adjust master factor
    pub fn adjust_master_factor(&mut self, delta: f32) {
        self.master_factor = (self.master_factor + delta).clamp(0.1, 0.9);
    }
}
```

**Acceptance Criteria:**
- [ ] Single window takes full screen
- [ ] Two windows split according to master_factor
- [ ] Master area contains master_count windows
- [ ] Stack area contains remaining windows
- [ ] Can adjust master factor dynamically
- [ ] Can adjust master count dynamically
- [ ] Gaps are applied correctly
- [ ] Windows are positioned accurately

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_window() {
        let layout = MasterLayout::new();
        let area = Rect::new(0, 0, 1920, 1080);
        let windows = vec![WindowHandle::from_hwnd(HWND(1))];
        
        // Should not panic
        let result = layout.apply(&windows, area);
        // We can't fully test without actual windows, but verify no panic
    }

    #[test]
    fn test_master_factor_clamping() {
        let layout = MasterLayout::new().with_master_factor(1.5);
        assert!(layout.master_factor <= 0.9);
        
        let layout = MasterLayout::new().with_master_factor(-0.5);
        assert!(layout.master_factor >= 0.1);
    }

    #[test]
    fn test_adjust_master_factor() {
        let mut layout = MasterLayout::new();
        let original = layout.master_factor;
        
        layout.adjust_master_factor(0.1);
        assert!(layout.master_factor > original);
        
        layout.adjust_master_factor(-0.2);
        assert!(layout.master_factor < original);
    }

    #[test]
    fn test_master_count_adjustment() {
        let mut layout = MasterLayout::new();
        assert_eq!(layout.master_count, 1);
        
        layout.increase_master_count();
        assert_eq!(layout.master_count, 2);
        
        layout.decrease_master_count();
        assert_eq!(layout.master_count, 1);
        
        layout.decrease_master_count();
        assert_eq!(layout.master_count, 1); // Should not go below 1
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core layout::master
cargo run -p tiling-wm-core  # Manual test with 3+ windows
```

---

### Week 6: Window State Management

#### Task 2.4: Implement Window States and ManagedWindow

**Objective:** Create comprehensive window state management system supporting tiled, floating, fullscreen, and minimized states.

**File:** `crates/core/src/window_manager/window.rs`

**Required Implementations:**

```rust
use crate::utils::win32::WindowHandle;
use windows::Win32::Foundation::RECT;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Tiled,
    Floating,
    Fullscreen,
    Minimized,
}

#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub handle: WindowHandle,
    pub state: WindowState,
    pub workspace: usize,
    pub monitor: usize,
    pub title: String,
    pub class: String,
    pub process_name: String,
    /// Saved position before entering fullscreen/floating
    pub original_rect: Option<RECT>,
    /// Whether this window should be managed
    pub managed: bool,
    /// User-specified floating state
    pub user_floating: bool,
}

impl ManagedWindow {
    pub fn new(
        handle: WindowHandle,
        workspace: usize,
        monitor: usize,
    ) -> anyhow::Result<Self> {
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

    /// Set window to floating state
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

    /// Set window to tiled state
    pub fn set_tiled(&mut self) -> anyhow::Result<()> {
        self.state = WindowState::Tiled;
        self.user_floating = false;
        self.original_rect = None;
        Ok(())
    }

    /// Set window to fullscreen
    pub fn set_fullscreen(&mut self, monitor_rect: &crate::window_manager::tree::Rect) -> anyhow::Result<()> {
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

    /// Exit fullscreen, returning to previous state
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

    /// Toggle between tiled and floating
    pub fn toggle_floating(&mut self) -> anyhow::Result<()> {
        match self.state {
            WindowState::Tiled => self.set_floating()?,
            WindowState::Floating => self.set_tiled()?,
            _ => {}
        }
        Ok(())
    }

    /// Minimize the window
    pub fn minimize(&mut self) -> anyhow::Result<()> {
        if self.state != WindowState::Minimized {
            self.original_rect = Some(self.handle.get_rect()?);
            self.state = WindowState::Minimized;
            
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(self.handle.0, SW_MINIMIZE);
            }
        }
        Ok(())
    }

    /// Restore from minimized state
    pub fn restore(&mut self) -> anyhow::Result<()> {
        if self.state == WindowState::Minimized {
            self.state = if self.user_floating {
                WindowState::Floating
            } else {
                WindowState::Tiled
            };
            
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(self.handle.0, SW_RESTORE);
            }
        }
        Ok(())
    }

    /// Check if window should be tiled in layout
    pub fn should_tile(&self) -> bool {
        self.state == WindowState::Tiled && self.managed
    }

    /// Update metadata (title, class, etc.)
    pub fn update_metadata(&mut self) -> anyhow::Result<()> {
        self.title = self.handle.get_title().unwrap_or_default();
        self.class = self.handle.get_class_name().unwrap_or_default();
        Ok(())
    }
}

/// Registry for managing all windows
pub struct WindowRegistry {
    windows: HashMap<isize, ManagedWindow>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Register a new window
    pub fn register(&mut self, window: ManagedWindow) {
        self.windows.insert(window.handle.0 .0, window);
    }

    /// Unregister a window
    pub fn unregister(&mut self, hwnd: isize) -> Option<ManagedWindow> {
        self.windows.remove(&hwnd)
    }

    /// Get window by handle
    pub fn get(&self, hwnd: isize) -> Option<&ManagedWindow> {
        self.windows.get(&hwnd)
    }

    /// Get mutable window by handle
    pub fn get_mut(&mut self, hwnd: isize) -> Option<&mut ManagedWindow> {
        self.windows.get_mut(&hwnd)
    }

    /// Get all windows in a workspace
    pub fn get_by_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace)
            .collect()
    }

    /// Get all tiled windows in a workspace
    pub fn get_tiled_in_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace && w.should_tile())
            .collect()
    }

    /// Get all floating windows in a workspace
    pub fn get_floating_in_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace && w.state == WindowState::Floating)
            .collect()
    }

    /// Get window count
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Get window count in workspace
    pub fn count_in_workspace(&self, workspace: usize) -> usize {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace)
            .count()
    }
}
```

**Acceptance Criteria:**
- [ ] ManagedWindow tracks all required state
- [ ] Can transition between tiled/floating/fullscreen states
- [ ] Original position is saved and restored correctly
- [ ] WindowRegistry efficiently tracks all windows
- [ ] Can query windows by workspace
- [ ] Can filter by state (tiled, floating, etc.)
- [ ] Metadata updates work correctly

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_managed_window_creation() {
        let handle = WindowHandle::from_hwnd(HWND(12345));
        let window = ManagedWindow::new(handle, 1, 0);
        
        // Note: This will fail without real window, but test structure
        // In real testing, we'd use actual windows
    }

    #[test]
    fn test_window_state_transitions() {
        // Test state machine transitions
        let mut window = create_test_window();
        
        assert_eq!(window.state, WindowState::Tiled);
        
        window.set_floating().ok();
        assert_eq!(window.state, WindowState::Floating);
        
        window.set_tiled().ok();
        assert_eq!(window.state, WindowState::Tiled);
    }

    #[test]
    fn test_window_registry() {
        let mut registry = WindowRegistry::new();
        assert_eq!(registry.count(), 0);
        
        let window = create_test_window();
        let hwnd = window.handle.0 .0;
        
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
        
        registry.register(w1);
        registry.register(w2);
        registry.register(w3);
        
        let ws1_windows = registry.get_by_workspace(1);
        assert_eq!(ws1_windows.len(), 2);
        
        let ws2_windows = registry.get_by_workspace(2);
        assert_eq!(ws2_windows.len(), 1);
    }

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

    fn create_test_window_with_workspace(workspace: usize) -> ManagedWindow {
        let mut w = create_test_window();
        w.workspace = workspace;
        w
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core window
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 2.5: Integrate Window States with WindowManager

**Objective:** Update WindowManager to use ManagedWindow and WindowRegistry for comprehensive state tracking.

**File:** `crates/core/src/window_manager/mod.rs`

**Required Implementations:**

Update WindowManager to include window registry:

```rust
use window::{ManagedWindow, WindowRegistry, WindowState};
use layout::dwindle::DwindleLayout;
use layout::master::MasterLayout;

pub struct WindowManager {
    trees: HashMap<usize, TreeNode>,
    active_workspace: usize,
    monitors: Vec<MonitorInfo>,
    registry: WindowRegistry,
    dwindle_layout: DwindleLayout,
    master_layout: MasterLayout,
    current_layout: LayoutType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    Dwindle,
    Master,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            active_workspace: 1,
            monitors: Vec::new(),
            registry: WindowRegistry::new(),
            dwindle_layout: DwindleLayout::new(),
            master_layout: MasterLayout::new(),
            current_layout: LayoutType::Dwindle,
        }
    }

    pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()> {
        if !self.should_manage_window(&window)? {
            return Ok(());
        }

        let hwnd = window.0 .0;
        
        // Check if already managed
        if self.registry.get(hwnd).is_some() {
            return Ok(());
        }

        // Create managed window
        let managed = ManagedWindow::new(
            window,
            self.active_workspace,
            0, // TODO: Determine correct monitor
        )?;

        // Register the window
        self.registry.register(managed);

        // Add to current workspace tree
        self.retile_workspace(self.active_workspace)?;

        Ok(())
    }

    pub fn unmanage_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.0 .0;
        
        if let Some(managed) = self.registry.unregister(hwnd) {
            // Remove from tree and retile
            self.retile_workspace(managed.workspace)?;
        }

        Ok(())
    }

    pub fn retile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        // Get all tiled windows for this workspace
        let tiled_windows: Vec<WindowHandle> = self.registry
            .get_tiled_in_workspace(workspace_id)
            .iter()
            .map(|w| w.handle)
            .collect();

        if tiled_windows.is_empty() {
            return Ok(());
        }

        // Get monitor area for this workspace
        let monitor = self.monitors.get(0)
            .ok_or_else(|| anyhow::anyhow!("No monitors found"))?;

        // Apply layout based on current layout type
        match self.current_layout {
            LayoutType::Dwindle => {
                // Rebuild tree with dwindle layout
                let mut tree = TreeNode::new_container(Split::Vertical, monitor.work_area);
                
                for window in tiled_windows {
                    self.dwindle_layout.insert_window(&mut tree, window)?;
                }
                
                self.dwindle_layout.apply(&mut tree)?;
                self.trees.insert(workspace_id, tree);
            }
            LayoutType::Master => {
                // Apply master layout directly to window list
                self.master_layout.apply(&tiled_windows, monitor.work_area)?;
            }
        }

        Ok(())
    }

    pub fn toggle_floating(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.0 .0;
        
        if let Some(managed) = self.registry.get_mut(hwnd) {
            let workspace = managed.workspace;
            managed.toggle_floating()?;
            
            // Retile workspace to adjust for window state change
            self.retile_workspace(workspace)?;
        }

        Ok(())
    }

    pub fn toggle_fullscreen(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        let hwnd = window.0 .0;
        
        if let Some(managed) = self.registry.get_mut(hwnd) {
            let monitor = self.monitors.get(managed.monitor)
                .ok_or_else(|| anyhow::anyhow!("Monitor not found"))?;

            match managed.state {
                WindowState::Fullscreen => {
                    managed.exit_fullscreen()?;
                    // Retile workspace
                    self.retile_workspace(managed.workspace)?;
                }
                _ => {
                    managed.set_fullscreen(&monitor.work_area)?;
                }
            }
        }

        Ok(())
    }

    pub fn set_layout(&mut self, layout: LayoutType) -> anyhow::Result<()> {
        self.current_layout = layout;
        self.retile_workspace(self.active_workspace)?;
        Ok(())
    }

    pub fn get_active_window(&self) -> Option<&ManagedWindow> {
        let fg_window = crate::utils::win32::get_foreground_window()?;
        self.registry.get(fg_window.0 .0)
    }

    pub fn get_active_window_mut(&mut self) -> Option<&mut ManagedWindow> {
        let fg_window = crate::utils::win32::get_foreground_window()?;
        self.registry.get_mut(fg_window.0 .0)
    }
}
```

**Acceptance Criteria:**
- [ ] WindowManager tracks all windows in registry
- [ ] Can toggle floating state for active window
- [ ] Can toggle fullscreen for active window
- [ ] Can switch between dwindle and master layouts
- [ ] Retiling works correctly when windows change state
- [ ] Floating windows are excluded from tiling
- [ ] Fullscreen windows cover entire monitor

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_layout_switching() {
        let mut wm = WindowManager::new();
        wm.initialize().ok();
        
        assert_eq!(wm.current_layout, LayoutType::Dwindle);
        
        wm.set_layout(LayoutType::Master).ok();
        assert_eq!(wm.current_layout, LayoutType::Master);
    }

    #[test]
    #[ignore] // Requires real windows
    fn test_toggle_floating() {
        let mut wm = WindowManager::new();
        wm.initialize().unwrap();
        
        // Would test with real window
    }
}
```

---

### Week 7: Focus Management

#### Task 2.6: Implement Focus Manager with History

**Objective:** Create focus management system that tracks focus history and supports directional focus navigation.

**File:** `crates/core/src/window_manager/focus.rs`

**Required Implementations:**

```rust
use crate::utils::win32::WindowHandle;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct FocusManager {
    /// Recently focused windows (most recent first)
    focus_history: VecDeque<isize>,
    /// Currently focused window
    current_focus: Option<isize>,
    /// Maximum history size
    history_size: usize,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focus_history: VecDeque::with_capacity(10),
            current_focus: None,
            history_size: 10,
        }
    }

    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self.focus_history = VecDeque::with_capacity(size);
        self
    }

    /// Focus a specific window
    pub fn focus_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        unsafe {
            // Bring window to foreground
            SetForegroundWindow(window.0)?;
            
            // Ensure window is visible
            ShowWindow(window.0, SW_SHOW);
            
            // Restore if minimized
            if IsIconic(window.0).as_bool() {
                ShowWindow(window.0, SW_RESTORE);
            }
        }
        
        let hwnd_val = window.0 .0;
        
        // Update focus history
        self.add_to_history(hwnd_val);
        self.current_focus = Some(hwnd_val);
        
        Ok(())
    }

    /// Add window to focus history
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

    /// Get currently focused window
    pub fn current(&self) -> Option<isize> {
        self.current_focus
    }

    /// Focus previous window in history (Alt-Tab behavior)
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

    /// Focus next window in history
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

    /// Remove window from focus history
    pub fn remove_from_history(&mut self, hwnd: isize) {
        self.focus_history.retain(|&h| h != hwnd);
        
        if self.current_focus == Some(hwnd) {
            self.current_focus = self.focus_history.front().copied();
        }
    }

    /// Get focus history
    pub fn get_history(&self) -> &VecDeque<isize> {
        &self.focus_history
    }

    /// Clear focus history
    pub fn clear_history(&mut self) {
        self.focus_history.clear();
        self.current_focus = None;
    }
}

/// Helper for directional focus
pub struct DirectionalFocus;

impl DirectionalFocus {
    /// Find window in a specific direction from current window
    pub fn find_window_in_direction(
        current_rect: &crate::window_manager::tree::Rect,
        direction: Direction,
        candidates: &[(isize, crate::window_manager::tree::Rect)],
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

    fn is_in_direction(
        from: &crate::window_manager::tree::Rect,
        to: &crate::window_manager::tree::Rect,
        direction: Direction,
    ) -> bool {
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

    fn calculate_distance(
        from: &crate::window_manager::tree::Rect,
        to: &crate::window_manager::tree::Rect,
        direction: Direction,
    ) -> f32 {
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
```

**Acceptance Criteria:**
- [ ] Can focus windows programmatically
- [ ] Focus history tracks recently focused windows
- [ ] Can navigate focus history (Alt-Tab style)
- [ ] Directional focus finds correct windows
- [ ] History maintains size limit
- [ ] Removing windows updates focus correctly

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_history() {
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
    fn test_focus_previous() {
        let mut fm = FocusManager::new();
        
        fm.add_to_history(1);
        fm.add_to_history(2);
        fm.current_focus = Some(2);
        
        let prev = fm.focus_previous();
        assert_eq!(prev, Some(1));
    }

    #[test]
    fn test_history_size_limit() {
        let mut fm = FocusManager::new().with_history_size(3);
        
        for i in 1..=5 {
            fm.add_to_history(i);
        }
        
        assert_eq!(fm.focus_history.len(), 3);
        assert_eq!(fm.focus_history[0], 5);
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
    }

    #[test]
    fn test_directional_focus() {
        use crate::window_manager::tree::Rect;
        
        let current = Rect::new(100, 100, 200, 200);
        let left = Rect::new(0, 100, 90, 200);
        let right = Rect::new(310, 100, 200, 200);
        
        let candidates = vec![
            (1, left),
            (2, right),
        ];
        
        let result = DirectionalFocus::find_window_in_direction(
            &current,
            Direction::Left,
            &candidates,
        );
        
        assert_eq!(result, Some(1));
        
        let result = DirectionalFocus::find_window_in_direction(
            &current,
            Direction::Right,
            &candidates,
        );
        
        assert_eq!(result, Some(2));
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core focus
```

---

### Week 8: Window Operations & Commands

#### Task 2.7: Implement Command System

**Objective:** Create comprehensive command system for all window operations.

**File:** `crates/core/src/commands.rs`

**Required Implementations:**

```rust
use crate::window_manager::{WindowManager, LayoutType};
use crate::window_manager::focus::Direction;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum Command {
    // Window commands
    CloseActiveWindow,
    ToggleFloating,
    ToggleFullscreen,
    MinimizeActive,
    
    // Focus commands
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    FocusPrevious,
    FocusNext,
    
    // Move commands
    MoveWindowLeft,
    MoveWindowRight,
    MoveWindowUp,
    MoveWindowDown,
    SwapWithMaster,
    
    // Layout commands
    SetLayoutDwindle,
    SetLayoutMaster,
    IncreaseMasterCount,
    DecreaseMasterCount,
    IncreaseMasterFactor,
    DecreaseMasterFactor,
    
    // Workspace commands
    SwitchWorkspace(usize),
    MoveToWorkspace(usize),
    MoveToWorkspaceAndFollow(usize),
    
    // System commands
    Reload,
    Quit,
}

pub struct CommandExecutor {
    // Commands will be executed on the window manager
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn execute(&self, command: Command, wm: &mut WindowManager) -> Result<()> {
        match command {
            Command::CloseActiveWindow => self.close_active_window(wm),
            Command::ToggleFloating => self.toggle_floating(wm),
            Command::ToggleFullscreen => self.toggle_fullscreen(wm),
            Command::MinimizeActive => self.minimize_active(wm),
            
            Command::FocusLeft => self.focus_direction(wm, Direction::Left),
            Command::FocusRight => self.focus_direction(wm, Direction::Right),
            Command::FocusUp => self.focus_direction(wm, Direction::Up),
            Command::FocusDown => self.focus_direction(wm, Direction::Down),
            Command::FocusPrevious => self.focus_previous(wm),
            Command::FocusNext => self.focus_next(wm),
            
            Command::SetLayoutDwindle => wm.set_layout(LayoutType::Dwindle),
            Command::SetLayoutMaster => wm.set_layout(LayoutType::Master),
            
            Command::IncreaseMasterCount => self.adjust_master_count(wm, 1),
            Command::DecreaseMasterCount => self.adjust_master_count(wm, -1),
            Command::IncreaseMasterFactor => self.adjust_master_factor(wm, 0.05),
            Command::DecreaseMasterFactor => self.adjust_master_factor(wm, -0.05),
            
            Command::SwitchWorkspace(id) => wm.switch_workspace(id),
            Command::MoveToWorkspace(id) => self.move_to_workspace(wm, id),
            
            _ => Ok(()),
        }
    }
    
    fn close_active_window(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            let handle = window.handle;
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::Win32::Foundation::*;
                SendMessageW(handle.0, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        Ok(())
    }
    
    fn toggle_floating(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            let handle = window.handle;
            wm.toggle_floating(&handle)?;
        }
        Ok(())
    }
    
    fn toggle_fullscreen(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window() {
            let handle = window.handle;
            wm.toggle_fullscreen(&handle)?;
        }
        Ok(())
    }
    
    fn minimize_active(&self, wm: &mut WindowManager) -> Result<()> {
        if let Some(window) = wm.get_active_window_mut() {
            window.minimize()?;
        }
        Ok(())
    }
    
    fn focus_direction(&self, wm: &mut WindowManager, direction: Direction) -> Result<()> {
        // Implementation would find window in direction and focus it
        // This requires integration with tree structure and focus manager
        Ok(())
    }
    
    fn focus_previous(&self, wm: &mut WindowManager) -> Result<()> {
        // Use focus manager to focus previous window
        Ok(())
    }
    
    fn focus_next(&self, wm: &mut WindowManager) -> Result<()> {
        // Use focus manager to focus next window
        Ok(())
    }
    
    fn adjust_master_count(&self, wm: &mut WindowManager, delta: i32) -> Result<()> {
        if delta > 0 {
            wm.master_layout.increase_master_count();
        } else {
            wm.master_layout.decrease_master_count();
        }
        wm.retile_workspace(wm.get_active_workspace())?;
        Ok(())
    }
    
    fn adjust_master_factor(&self, wm: &mut WindowManager, delta: f32) -> Result<()> {
        wm.master_layout.adjust_master_factor(delta);
        wm.retile_workspace(wm.get_active_workspace())?;
        Ok(())
    }
    
    fn move_to_workspace(&self, wm: &mut WindowManager, workspace_id: usize) -> Result<()> {
        // Move active window to specified workspace
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] All window commands work correctly
- [ ] Focus commands navigate windows properly
- [ ] Layout commands switch and adjust layouts
- [ ] Master layout adjustments work
- [ ] Commands integrate with WindowManager
- [ ] Error handling is comprehensive

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::CloseActiveWindow;
        // Verify command can be created
    }

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        // Verify executor can be created
    }

    #[test]
    #[ignore] // Requires window manager instance
    fn test_command_execution() {
        let executor = CommandExecutor::new();
        let mut wm = WindowManager::new();
        wm.initialize().ok();
        
        // Would test actual command execution
    }
}
```

---

#### Task 2.8: Update Main Application with Commands

**Objective:** Integrate command system into main event loop.

**File:** `crates/core/src/main.rs`

**Required Implementations:**

Update main.rs to use command system:

```rust
mod window_manager;
mod event_loop;
mod utils;
mod commands;

use window_manager::WindowManager;
use event_loop::{EventLoop, WindowEvent};
use commands::{Command, CommandExecutor};
use tracing::{info, error, debug};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    initialize_logging();
    
    info!("Starting Tiling Window Manager v0.2.0");
    
    // Initialize window manager
    let mut wm = WindowManager::new();
    wm.initialize()?;
    info!("Window manager initialized with layout: {:?}", wm.get_current_layout());
    
    // Set up command executor
    let executor = CommandExecutor::new();
    
    // Set up event loop
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    info!("Event loop started");
    
    // Main event loop
    loop {
        // Process Windows events
        for event in event_loop.poll_events() {
            match event {
                WindowEvent::WindowCreated(hwnd) => {
                    debug!("Window created: {:?}", hwnd);
                    let window = utils::win32::WindowHandle::from_hwnd(hwnd);
                    if let Err(e) = wm.manage_window(window) {
                        error!("Failed to manage window: {}", e);
                    }
                }
                WindowEvent::WindowDestroyed(hwnd) => {
                    debug!("Window destroyed: {:?}", hwnd);
                    let window = utils::win32::WindowHandle::from_hwnd(hwnd);
                    if let Err(e) = wm.unmanage_window(&window) {
                        error!("Failed to unmanage window: {}", e);
                    }
                }
                WindowEvent::WindowFocused(hwnd) => {
                    debug!("Window focused: {:?}", hwnd);
                    // Update focus tracking
                }
                _ => {}
            }
        }
        
        // Small sleep to avoid busy loop
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_core=debug,info")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
}
```

**Acceptance Criteria:**
- [ ] Application compiles with new command system
- [ ] Commands can be executed on window manager
- [ ] Event loop continues to work correctly
- [ ] Logging shows command execution
- [ ] Application runs stably

---

## Phase 2 Completion Checklist

### Build & Compilation
- [ ] `cargo build --workspace` succeeds without errors
- [ ] `cargo build --workspace --release` succeeds
- [ ] No warnings from `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted with `cargo fmt --workspace --check`

### Core Functionality
- [ ] Dwindle layout tiles windows correctly
- [ ] Smart split direction works based on window aspect ratio
- [ ] Master layout positions master and stack windows correctly
- [ ] Can switch between dwindle and master layouts
- [ ] Can adjust master factor and master count
- [ ] Window states (tiled/floating/fullscreen) work correctly
- [ ] Can toggle floating for any window
- [ ] Can toggle fullscreen for any window
- [ ] Focus management tracks window focus
- [ ] Directional focus navigation works
- [ ] Focus history enables Alt-Tab style switching
- [ ] Command system executes all commands correctly

### Testing
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Layout algorithm tests pass
- [ ] Window state tests pass
- [ ] Focus management tests pass
- [ ] No test failures or panics

### Integration
- [ ] Windows automatically tile when created
- [ ] Layout updates when windows are added/removed
- [ ] Floating windows stay floating after retile
- [ ] Fullscreen windows cover entire screen
- [ ] Window closing is handled correctly
- [ ] Focus follows window operations

### Documentation
- [ ] All new public APIs have doc comments
- [ ] `cargo doc --no-deps` builds successfully
- [ ] README updated with Phase 2 features
- [ ] Examples in documentation work

### Manual Validation
- [ ] Open 3+ windows and verify tiling
- [ ] Toggle floating on a window - verify it floats
- [ ] Toggle fullscreen - verify it covers screen
- [ ] Switch between dwindle and master layouts
- [ ] Adjust master factor - verify visual change
- [ ] Close windows - verify retiling happens
- [ ] Application runs stable for 15+ minutes
- [ ] CPU usage remains reasonable
- [ ] Memory usage is stable

---

## Troubleshooting

### Common Issues

**Issue: Windows not tiling correctly**
- Solution: Check that windows are being marked as "should_tile"
- Verify layout algorithm is being called
- Check rectangle calculations for accuracy
- Ensure gaps aren't making windows too small

**Issue: Floating toggle not working**
- Solution: Verify window state transitions
- Check that retile is called after state change
- Ensure floating windows are filtered from tile list

**Issue: Fullscreen doesn't cover entire screen**
- Solution: Check monitor work_area vs full_area
- Verify no gaps are being applied to fullscreen
- Ensure window borders aren't affecting size

**Issue: Focus commands don't work**
- Solution: Check SetForegroundWindow permissions
- Verify window is visible and not minimized
- Test with different window types

**Issue: Master layout not adjusting**
- Solution: Verify master_factor changes are applied
- Check that retile is called after adjustment
- Ensure calculations use updated factor

**Issue: Memory leaks with window operations**
- Solution: Verify WindowRegistry cleanup
- Check that Box pointers in tree are dropped
- Use valgrind or similar to detect leaks

---

## Success Criteria Summary

Phase 2 is complete when:

1. ✅ **Both layout algorithms work:**
   - Dwindle with smart split
   - Master-stack with adjustable parameters

2. ✅ **Window state management operational:**
   - Tiled, floating, fullscreen, minimized states
   - State transitions work correctly
   - States persist and restore properly

3. ✅ **Focus management functional:**
   - Focus history tracking
   - Directional navigation
   - Alt-Tab style switching

4. ✅ **Command system complete:**
   - All window operations
   - All layout operations
   - All focus operations

5. ✅ **Integration solid:**
   - Commands work with WindowManager
   - Layout updates happen automatically
   - Focus follows operations

6. ✅ **Quality standards met:**
   - All tests passing
   - No clippy warnings
   - Stable operation
   - Good performance

---

## Deliverables

At the end of Phase 2, you should have:

1. **Complete tiling system:**
   - Two working layout algorithms
   - Dynamic layout switching
   - Configurable layout parameters

2. **Robust window management:**
   - Comprehensive state tracking
   - Multiple window states
   - Proper state transitions

3. **Effective focus system:**
   - Focus history with size limit
   - Directional navigation
   - Focus switching commands

4. **Comprehensive commands:**
   - Window operations
   - Layout adjustments
   - Focus control
   - Workspace operations (basic)

5. **Quality assurance:**
   - Full test coverage
   - Manual validation successful
   - Documentation complete
   - No critical bugs

---

## Next Steps

After completing Phase 2, proceed to **Phase 3: Workspace System** (Weeks 9-12), which will implement:

- Virtual Desktop API integration
- Workspace manager with create/switch/delete
- Per-monitor workspace support
- Workspace persistence across sessions
- Enhanced multi-monitor support

See DETAILED_ROADMAP.md for Phase 3 specifications.

---

**End of Phase 2 Task Document**
