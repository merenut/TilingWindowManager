//! Workspace management operations.
//!
//! This module contains operations for managing workspaces, including
//! switching, tiling, and retiling operations.

use crate::utils::win32::WindowHandle;
use crate::window_manager::{LayoutType, Rect, Split, TreeNode, WindowManager};
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

impl WindowManager {
    /// Switch to a different workspace.
    ///
    /// This hides windows in the current workspace and shows windows in the target workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace to switch to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let mut wm = WindowManager::new();
    /// wm.initialize().unwrap();
    /// wm.switch_workspace(2).ok();
    /// ```
    pub fn switch_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if workspace_id == self.active_workspace {
            return Ok(());
        }

        // Hide windows in current workspace across all monitors
        #[cfg(target_os = "windows")]
        {
            let current_trees = self.get_workspace_trees(self.active_workspace);
            for (_, tree) in current_trees {
                for (hwnd, _) in tree.collect() {
                    if hwnd.0 != 0 {
                        WindowHandle::from_hwnd(hwnd).hide();
                    }
                }
            }
        }

        // Show windows in target workspace across all monitors
        let target_trees = self.get_workspace_trees(workspace_id);
        for (_, tree) in target_trees {
            for (hwnd, _) in tree.collect() {
                if hwnd.0 != 0 {
                    #[cfg(target_os = "windows")]
                    {
                        use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
                        WindowHandle::from_hwnd(hwnd).show(SW_SHOW);
                    }
                }
            }
        }

        self.active_workspace = workspace_id;

        // Re-tile the new workspace to ensure proper layout
        self.tile_workspace(workspace_id)?;

        Ok(())
    }

    /// Get the currently active workspace ID.
    ///
    /// # Returns
    ///
    /// The active workspace ID (typically 1-10).
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::window_manager::WindowManager;
    ///
    /// let wm = WindowManager::new();
    /// assert_eq!(wm.get_active_workspace(), 1);
    /// ```
    pub fn get_active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// Get a reference to a workspace's tree on the primary monitor (monitor 0).
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// `Some(&TreeNode)` if the workspace exists on monitor 0, `None` otherwise.
    pub fn get_workspace_tree(&self, workspace_id: usize) -> Option<&TreeNode> {
        self.trees.get(&(workspace_id, 0))
    }

    /// Get a mutable reference to a workspace's tree on the primary monitor (monitor 0).
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// `Some(&mut TreeNode)` if the workspace exists on monitor 0, `None` otherwise.
    pub fn get_workspace_tree_mut(&mut self, workspace_id: usize) -> Option<&mut TreeNode> {
        self.trees.get_mut(&(workspace_id, 0))
    }

    /// Get all trees for a workspace across all monitors.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (monitor_idx, tree reference).
    pub fn get_workspace_trees(&self, workspace_id: usize) -> Vec<(usize, &TreeNode)> {
        self.trees
            .iter()
            .filter_map(|((ws_id, mon_idx), tree)| {
                if *ws_id == workspace_id {
                    Some((*mon_idx, tree))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Apply the tiling layout to a workspace.
    ///
    /// This recalculates and applies window positions for all windows in the workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace to tile
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if layout application fails.
    pub fn tile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        // Apply layout for all monitors in this workspace
        let monitor_trees: Vec<_> = self
            .trees
            .iter()
            .filter(|((ws_id, _), _)| *ws_id == workspace_id)
            .map(|((_, mon_idx), tree)| (*mon_idx, tree))
            .collect();

        for (_, tree) in monitor_trees {
            // Don't tile empty placeholder trees
            if tree.hwnd() != Some(HWND(0)) {
                tree.apply_layout(5, 10)?;
            }
        }
        Ok(())
    }

    /// Retile a workspace using the current layout algorithm.
    ///
    /// This method rebuilds the window tree for the workspace based on
    /// the currently active layout type, considering only tiled windows.
    /// Floating and fullscreen windows are excluded from the tiling layout.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace to retile
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if layout application fails.
    pub fn retile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        // Prevent recursive retiling (windows being moved trigger events that call retile again)
        if self.is_tiling {
            tracing::debug!("Skipping retile - already tiling");
            return Ok(());
        }

        self.is_tiling = true;
        let result = self.retile_workspace_impl(workspace_id);
        self.is_tiling = false;
        result
    }

    fn retile_workspace_impl(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        let windows_by_monitor = self.group_windows_by_monitor(workspace_id)?;

        if windows_by_monitor.is_empty() {
            self.create_empty_workspace_placeholder(workspace_id)?;
            return Ok(());
        }

        for (monitor_idx, windows) in windows_by_monitor {
            self.tile_monitor(workspace_id, monitor_idx, &windows)?;
        }

        Ok(())
    }

    pub(super) fn group_windows_by_monitor(
        &self,
        workspace_id: usize,
    ) -> anyhow::Result<HashMap<usize, Vec<HWND>>> {
        let tiled_windows = self.registry.get_tiled_in_workspace(workspace_id);

        tracing::debug!(
            "Retiling workspace {} with {} windows",
            workspace_id,
            tiled_windows.len()
        );

        let mut windows_by_monitor: HashMap<usize, Vec<HWND>> = HashMap::new();

        for window in tiled_windows {
            let hwnd = window.handle.hwnd();
            let monitor_idx = self.get_monitor_for_window(hwnd);

            tracing::debug!(
                "Window {} on monitor {} (stored: {})",
                hwnd.0,
                monitor_idx,
                window.monitor
            );

            windows_by_monitor
                .entry(monitor_idx)
                .or_insert_with(Vec::new)
                .push(hwnd);
        }

        tracing::debug!(
            "Windows grouped by monitor: {:?}",
            windows_by_monitor
                .iter()
                .map(|(m, ws)| (m, ws.len()))
                .collect::<Vec<_>>()
        );

        Ok(windows_by_monitor)
    }

    pub(super) fn create_empty_workspace_placeholder(
        &mut self,
        workspace_id: usize,
    ) -> anyhow::Result<()> {
        let monitor = self
            .monitors
            .first()
            .ok_or_else(|| anyhow::anyhow!("No monitors found"))?;

        let work_area_with_gaps = self.apply_outer_gaps(&monitor.work_area);

        self.trees.insert(
            (workspace_id, 0),
            TreeNode::new_leaf(HWND(0), work_area_with_gaps),
        );

        Ok(())
    }

    pub(super) fn tile_monitor(
        &mut self,
        workspace_id: usize,
        monitor_idx: usize,
        windows: &[HWND],
    ) -> anyhow::Result<()> {
        let monitor = self
            .monitors
            .get(monitor_idx)
            .ok_or_else(|| anyhow::anyhow!("Monitor {} not found", monitor_idx))?;

        let work_area_with_gaps = self.apply_outer_gaps(&monitor.work_area);

        tracing::debug!(
            "Tiling {} windows on monitor {} (work area: {}x{} at {}, {})",
            windows.len(),
            monitor_idx,
            work_area_with_gaps.width,
            work_area_with_gaps.height,
            work_area_with_gaps.x,
            work_area_with_gaps.y
        );

        let tree = match self.current_layout {
            LayoutType::Dwindle => self.build_dwindle_tree(windows, work_area_with_gaps)?,
            LayoutType::Master => self.build_master_tree(windows, work_area_with_gaps)?,
        };

        self.trees.insert((workspace_id, monitor_idx), tree);
        Ok(())
    }

    pub(super) fn apply_outer_gaps(&self, work_area: &Rect) -> Rect {
        let gaps_out = 10;
        Rect::new(
            work_area.x + gaps_out,
            work_area.y + gaps_out,
            work_area.width - 2 * gaps_out,
            work_area.height - 2 * gaps_out,
        )
    }

    pub(super) fn build_dwindle_tree(
        &mut self,
        windows: &[HWND],
        work_area: Rect,
    ) -> anyhow::Result<TreeNode> {
        let mut tree = TreeNode::new_leaf(HWND(0), work_area);

        for &hwnd in windows {
            self.dwindle_layout.insert_window(&mut tree, hwnd)?;
        }

        self.dwindle_layout.apply(&tree)?;
        Ok(tree)
    }

    pub(super) fn build_master_tree(
        &mut self,
        windows: &[HWND],
        work_area: Rect,
    ) -> anyhow::Result<TreeNode> {
        self.master_layout.apply(windows, work_area)?;

        // Create a simple tree for tracking (master layout doesn't use tree structure)
        let mut tree = TreeNode::new_leaf(HWND(0), work_area);

        for &hwnd in windows {
            tree = if tree.hwnd() == Some(HWND(0)) {
                TreeNode::new_leaf(hwnd, work_area)
            } else {
                tree.insert(hwnd, Split::Horizontal)
            };
        }

        Ok(tree)
    }
}
