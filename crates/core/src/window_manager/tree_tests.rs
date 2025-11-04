//! Tests for the binary tree window layout data structure.

#[cfg(test)]
mod tests {
    use super::super::{Rect, Split, TreeNode};
    use windows::Win32::Foundation::HWND;

    // Helper function to create test HWND values
    fn test_hwnd(id: isize) -> HWND {
        HWND(id as _)
    }

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 200);
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0, 0, 100, 200);
        assert_eq!(rect.area(), 20000);
    }

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(10, 10, 100, 100);
        
        // Inside the rectangle
        assert!(rect.contains_point(50, 50));
        assert!(rect.contains_point(10, 10)); // Top-left corner
        
        // Outside the rectangle
        assert!(!rect.contains_point(5, 50)); // Left
        assert!(!rect.contains_point(150, 50)); // Right
        assert!(!rect.contains_point(50, 5)); // Above
        assert!(!rect.contains_point(50, 150)); // Below
        assert!(!rect.contains_point(110, 110)); // Bottom-right corner (exclusive)
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0, 0, 100, 100);
        let rect2 = Rect::new(50, 50, 100, 100);
        let rect3 = Rect::new(200, 200, 100, 100);
        
        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
        assert!(!rect1.intersects(&rect3));
        assert!(!rect3.intersects(&rect1));
    }

    #[test]
    fn test_rect_split_horizontal() {
        let rect = Rect::new(0, 0, 100, 100);
        let (left, right) = rect.split_horizontal(0.5);
        
        assert_eq!(left, Rect::new(0, 0, 50, 100));
        assert_eq!(right, Rect::new(50, 0, 50, 100));
    }

    #[test]
    fn test_rect_split_horizontal_unequal() {
        let rect = Rect::new(0, 0, 100, 100);
        let (left, right) = rect.split_horizontal(0.3);
        
        assert_eq!(left, Rect::new(0, 0, 30, 100));
        assert_eq!(right, Rect::new(30, 0, 70, 100));
    }

    #[test]
    fn test_rect_split_vertical() {
        let rect = Rect::new(0, 0, 100, 100);
        let (top, bottom) = rect.split_vertical(0.5);
        
        assert_eq!(top, Rect::new(0, 0, 100, 50));
        assert_eq!(bottom, Rect::new(0, 50, 100, 50));
    }

    #[test]
    fn test_rect_split_vertical_unequal() {
        let rect = Rect::new(0, 0, 100, 100);
        let (top, bottom) = rect.split_vertical(0.7);
        
        assert_eq!(top, Rect::new(0, 0, 100, 70));
        assert_eq!(bottom, Rect::new(0, 70, 100, 30));
    }

    #[test]
    fn test_rect_apply_gaps() {
        let rect = Rect::new(0, 0, 100, 100);
        let gapped = rect.apply_gaps(5, 10);
        
        // gaps_out = 10, gaps_in = 5
        // x = 0 + 10 = 10
        // y = 0 + 10 = 10
        // width = 100 - 2*10 - 5 = 75
        // height = 100 - 2*10 - 5 = 75
        assert_eq!(gapped, Rect::new(10, 10, 75, 75));
    }

    #[test]
    fn test_rect_shrink() {
        let rect = Rect::new(10, 10, 100, 100);
        let shrunk = rect.shrink(5);
        
        assert_eq!(shrunk, Rect::new(15, 15, 90, 90));
    }

    #[test]
    fn test_rect_expand() {
        let rect = Rect::new(10, 10, 100, 100);
        let expanded = rect.expand(5);
        
        assert_eq!(expanded, Rect::new(5, 5, 110, 110));
    }

    #[test]
    fn test_split_opposite() {
        assert_eq!(Split::Horizontal.opposite(), Split::Vertical);
        assert_eq!(Split::Vertical.opposite(), Split::Horizontal);
    }

    #[test]
    fn test_tree_node_new_leaf() {
        let hwnd = test_hwnd(1);
        let rect = Rect::new(0, 0, 100, 100);
        let node = TreeNode::new_leaf(hwnd, rect);
        
        assert!(node.is_leaf());
        assert!(!node.is_container());
        assert_eq!(node.rect(), rect);
        assert_eq!(node.hwnd(), Some(hwnd));
    }

    #[test]
    fn test_tree_node_new_container() {
        let hwnd1 = test_hwnd(1);
        let hwnd2 = test_hwnd(2);
        let rect = Rect::new(0, 0, 100, 100);
        
        let left = TreeNode::new_leaf(hwnd1, Rect::new(0, 0, 50, 100));
        let right = TreeNode::new_leaf(hwnd2, Rect::new(50, 0, 50, 100));
        let container = TreeNode::new_container(Split::Horizontal, left, right, rect, 0.5);
        
        assert!(!container.is_leaf());
        assert!(container.is_container());
        assert_eq!(container.rect(), rect);
        assert_eq!(container.hwnd(), None);
    }

    #[test]
    fn test_tree_insert_into_leaf() {
        let hwnd1 = test_hwnd(1);
        let hwnd2 = test_hwnd(2);
        let rect = Rect::new(0, 0, 100, 100);
        
        let node = TreeNode::new_leaf(hwnd1, rect);
        let node = node.insert(hwnd2, Split::Horizontal);
        
        assert!(node.is_container());
        
        let windows = node.collect();
        assert_eq!(windows.len(), 2);
        assert!(windows.iter().any(|(h, _)| *h == hwnd1));
        assert!(windows.iter().any(|(h, _)| *h == hwnd2));
    }

    #[test]
    fn test_tree_insert_multiple() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        
        node = node.insert(test_hwnd(2), Split::Horizontal);
        node = node.insert(test_hwnd(3), Split::Vertical);
        node = node.insert(test_hwnd(4), Split::Horizontal);
        
        let windows = node.collect();
        assert_eq!(windows.len(), 4);
        
        for i in 1..=4 {
            assert!(windows.iter().any(|(h, _)| *h == test_hwnd(i)));
        }
    }

    #[test]
    fn test_tree_remove_from_two_node_tree() {
        let hwnd1 = test_hwnd(1);
        let hwnd2 = test_hwnd(2);
        let rect = Rect::new(0, 0, 100, 100);
        
        let node = TreeNode::new_leaf(hwnd1, rect);
        let node = node.insert(hwnd2, Split::Horizontal);
        
        // Remove hwnd2
        let node = node.remove(hwnd2).expect("Tree should not be empty");
        
        assert!(node.is_leaf());
        assert_eq!(node.hwnd(), Some(hwnd1));
        assert_eq!(node.rect(), rect);
    }

    #[test]
    fn test_tree_remove_from_three_node_tree() {
        let rect = Rect::new(0, 0, 100, 100);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        node = node.insert(test_hwnd(2), Split::Horizontal);
        node = node.insert(test_hwnd(3), Split::Vertical);
        
        // Remove middle window
        let node = node.remove(test_hwnd(2)).expect("Tree should not be empty");
        
        let windows = node.collect();
        assert_eq!(windows.len(), 2);
        assert!(windows.iter().any(|(h, _)| *h == test_hwnd(1)));
        assert!(windows.iter().any(|(h, _)| *h == test_hwnd(3)));
        assert!(!windows.iter().any(|(h, _)| *h == test_hwnd(2)));
    }

    #[test]
    fn test_tree_remove_last_window() {
        let hwnd = test_hwnd(1);
        let rect = Rect::new(0, 0, 100, 100);
        
        let node = TreeNode::new_leaf(hwnd, rect);
        let result = node.remove(hwnd);
        
        assert!(result.is_none(), "Removing the last window should return None");
    }

    #[test]
    fn test_tree_remove_nonexistent_window() {
        let rect = Rect::new(0, 0, 100, 100);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        node = node.insert(test_hwnd(2), Split::Horizontal);
        
        let node = node.remove(test_hwnd(999)).expect("Tree should still exist");
        
        let windows = node.collect();
        assert_eq!(windows.len(), 2);
    }

    #[test]
    fn test_tree_rebalance() {
        let rect = Rect::new(0, 0, 1000, 1000);
        
        // Create a tree with custom ratios
        let left = TreeNode::new_leaf(test_hwnd(1), Rect::new(0, 0, 300, 1000));
        let right = TreeNode::new_leaf(test_hwnd(2), Rect::new(300, 0, 700, 1000));
        let node = TreeNode::new_container(Split::Horizontal, left, right, rect, 0.3);
        
        // Rebalance should reset ratio to 0.5
        let balanced = node.rebalance();
        
        let windows = balanced.collect();
        assert_eq!(windows.len(), 2);
        
        // After rebalancing, windows should have equal width (500 each)
        for (_, window_rect) in windows {
            assert_eq!(window_rect.width, 500);
        }
    }

    #[test]
    fn test_tree_collect_empty() {
        // Can't have an empty tree, smallest is a single leaf
        let rect = Rect::new(0, 0, 100, 100);
        let node = TreeNode::new_leaf(test_hwnd(1), rect);
        
        let windows = node.collect();
        assert_eq!(windows.len(), 1);
    }

    #[test]
    fn test_tree_collect_preserves_order() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        
        // Insert in order
        for i in 2..=5 {
            node = node.insert(test_hwnd(i), Split::Horizontal);
        }
        
        let windows = node.collect();
        assert_eq!(windows.len(), 5);
        
        // First window should be hwnd(1)
        assert_eq!(windows[0].0, test_hwnd(1));
    }

    #[test]
    fn test_tree_horizontal_split_geometry() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let node = TreeNode::new_leaf(test_hwnd(1), rect);
        let node = node.insert(test_hwnd(2), Split::Horizontal);
        
        let windows = node.collect();
        assert_eq!(windows.len(), 2);
        
        // Find the two windows
        let (_, rect1) = windows.iter().find(|(h, _)| *h == test_hwnd(1)).unwrap();
        let (_, rect2) = windows.iter().find(|(h, _)| *h == test_hwnd(2)).unwrap();
        
        // Should be split horizontally with equal width
        assert_eq!(rect1.width, 500);
        assert_eq!(rect2.width, 500);
        assert_eq!(rect1.height, 1000);
        assert_eq!(rect2.height, 1000);
        
        // Left should start at x=0, right should start at x=500
        assert_eq!(rect1.x, 0);
        assert_eq!(rect2.x, 500);
    }

    #[test]
    fn test_tree_vertical_split_geometry() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let node = TreeNode::new_leaf(test_hwnd(1), rect);
        let node = node.insert(test_hwnd(2), Split::Vertical);
        
        let windows = node.collect();
        assert_eq!(windows.len(), 2);
        
        // Find the two windows
        let (_, rect1) = windows.iter().find(|(h, _)| *h == test_hwnd(1)).unwrap();
        let (_, rect2) = windows.iter().find(|(h, _)| *h == test_hwnd(2)).unwrap();
        
        // Should be split vertically with equal height
        assert_eq!(rect1.height, 500);
        assert_eq!(rect2.height, 500);
        assert_eq!(rect1.width, 1000);
        assert_eq!(rect2.width, 1000);
        
        // Top should start at y=0, bottom should start at y=500
        assert_eq!(rect1.y, 0);
        assert_eq!(rect2.y, 500);
    }

    #[test]
    fn test_tree_complex_layout() {
        // Create a complex layout:
        //   +-------+-------+
        //   |   1   |   2   |
        //   +-------+---+---+
        //   |     3     | 4 |
        //   +-----------+---+
        
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        
        // Add window 2 with horizontal split
        node = node.insert(test_hwnd(2), Split::Horizontal);
        // Add window 3 with vertical split
        node = node.insert(test_hwnd(3), Split::Vertical);
        // Add window 4 with horizontal split
        node = node.insert(test_hwnd(4), Split::Horizontal);
        
        let windows = node.collect();
        assert_eq!(windows.len(), 4);
        
        // Verify all windows have valid rectangles
        for (_, window_rect) in windows {
            assert!(window_rect.width > 0);
            assert!(window_rect.height > 0);
            assert!(window_rect.x >= 0);
            assert!(window_rect.y >= 0);
        }
    }

    #[test]
    fn test_rect_split_with_gaps() {
        let rect = Rect::new(0, 0, 100, 100);
        let gapped = rect.apply_gaps(5, 10);
        
        // After applying gaps, the rectangle should be smaller
        assert!(gapped.width < rect.width);
        assert!(gapped.height < rect.height);
        assert!(gapped.x > rect.x);
        assert!(gapped.y > rect.y);
    }

    #[test]
    fn test_tree_with_gaps_maintains_all_windows() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        node = node.insert(test_hwnd(2), Split::Horizontal);
        node = node.insert(test_hwnd(3), Split::Vertical);
        
        // Collect windows (gaps will be applied later in apply_layout)
        let windows = node.collect();
        assert_eq!(windows.len(), 3);
        
        // All windows should still be present
        assert!(windows.iter().any(|(h, _)| *h == test_hwnd(1)));
        assert!(windows.iter().any(|(h, _)| *h == test_hwnd(2)));
        assert!(windows.iter().any(|(h, _)| *h == test_hwnd(3)));
    }

    #[test]
    fn test_tree_remove_and_rebalance() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        node = node.insert(test_hwnd(2), Split::Horizontal);
        node = node.insert(test_hwnd(3), Split::Vertical);
        node = node.insert(test_hwnd(4), Split::Horizontal);
        
        // Remove a window
        node = node.remove(test_hwnd(3)).expect("Tree should not be empty");
        
        // Rebalance
        node = node.rebalance();
        
        let windows = node.collect();
        assert_eq!(windows.len(), 3);
        assert!(!windows.iter().any(|(h, _)| *h == test_hwnd(3)));
    }

    #[test]
    fn test_multiple_inserts_and_removes() {
        let rect = Rect::new(0, 0, 1000, 1000);
        let mut node = TreeNode::new_leaf(test_hwnd(1), rect);
        
        // Add several windows
        for i in 2..=10 {
            node = node.insert(test_hwnd(i), if i % 2 == 0 { Split::Horizontal } else { Split::Vertical });
        }
        
        assert_eq!(node.collect().len(), 10);
        
        // Remove some windows
        node = node.remove(test_hwnd(3)).expect("Tree should not be empty");
        node = node.remove(test_hwnd(7)).expect("Tree should not be empty");
        node = node.remove(test_hwnd(9)).expect("Tree should not be empty");
        
        let windows = node.collect();
        assert_eq!(windows.len(), 7);
        assert!(!windows.iter().any(|(h, _)| *h == test_hwnd(3)));
        assert!(!windows.iter().any(|(h, _)| *h == test_hwnd(7)));
        assert!(!windows.iter().any(|(h, _)| *h == test_hwnd(9)));
    }
}
