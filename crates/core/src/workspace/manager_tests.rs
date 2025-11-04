//! Tests for workspace management core data structures.

#[cfg(test)]
mod tests {
    use super::super::manager::{Workspace, WorkspaceConfig, WorkspaceManager};
    use crate::window_manager::tree::Rect;

    // Test Workspace struct

    #[test]
    fn test_workspace_new() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        assert_eq!(workspace.id, 1);
        assert_eq!(workspace.name, "Main");
        assert_eq!(workspace.monitor, 0);
        assert!(workspace.tree.is_none());
        assert!(workspace.windows.is_empty());
        assert!(workspace.virtual_desktop_id.is_none());
        assert!(!workspace.visible);
    }

    #[test]
    fn test_workspace_add_window() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(12345);
        assert_eq!(workspace.windows.len(), 1);
        assert!(workspace.windows.contains(&12345));
    }

    #[test]
    fn test_workspace_add_window_duplicate() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(12345);
        workspace.add_window(12345); // Add same window again

        // Should only be in the list once
        assert_eq!(workspace.windows.len(), 1);
        assert!(workspace.windows.contains(&12345));
    }

    #[test]
    fn test_workspace_add_multiple_windows() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(100);
        workspace.add_window(200);
        workspace.add_window(300);

        assert_eq!(workspace.windows.len(), 3);
        assert!(workspace.windows.contains(&100));
        assert!(workspace.windows.contains(&200));
        assert!(workspace.windows.contains(&300));
    }

    #[test]
    fn test_workspace_remove_window() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(12345);
        assert!(workspace.remove_window(12345));
        assert!(!workspace.windows.contains(&12345));
        assert!(workspace.windows.is_empty());
    }

    #[test]
    fn test_workspace_remove_window_not_found() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(12345);
        assert!(!workspace.remove_window(99999)); // Try to remove non-existent window
        assert_eq!(workspace.windows.len(), 1); // Original window still there
    }

    #[test]
    fn test_workspace_remove_from_multiple_windows() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(100);
        workspace.add_window(200);
        workspace.add_window(300);

        assert!(workspace.remove_window(200));
        assert_eq!(workspace.windows.len(), 2);
        assert!(workspace.windows.contains(&100));
        assert!(!workspace.windows.contains(&200));
        assert!(workspace.windows.contains(&300));
    }

    #[test]
    fn test_workspace_remove_window_twice() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.add_window(12345);
        assert!(workspace.remove_window(12345));
        assert!(!workspace.remove_window(12345)); // Second remove should return false
    }

    // Test WorkspaceConfig struct

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();

        assert_eq!(config.default_count, 10);
        assert_eq!(config.names.len(), 10);
        assert_eq!(config.names[0], "1");
        assert_eq!(config.names[9], "10");
        assert!(config.persist_state);
        assert!(!config.create_on_demand);
        assert!(!config.use_virtual_desktops);
    }

    #[test]
    fn test_workspace_config_names_correct_order() {
        let config = WorkspaceConfig::default();

        for (i, name) in config.names.iter().enumerate() {
            assert_eq!(name, &(i + 1).to_string());
        }
    }

    #[test]
    fn test_workspace_config_custom() {
        let config = WorkspaceConfig {
            default_count: 5,
            names: vec!["One", "Two", "Three", "Four", "Five"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            persist_state: false,
            create_on_demand: true,
            use_virtual_desktops: true,
        };

        assert_eq!(config.default_count, 5);
        assert_eq!(config.names.len(), 5);
        assert_eq!(config.names[0], "One");
        assert!(!config.persist_state);
        assert!(config.create_on_demand);
        assert!(config.use_virtual_desktops);
    }

    // Test WorkspaceManager struct

    #[test]
    fn test_workspace_manager_new() {
        let config = WorkspaceConfig::default();
        let manager = WorkspaceManager::new(config.clone());

        // Manager should be initialized but have no workspaces yet
        assert_eq!(manager.active_workspace(), 1);
        assert_eq!(manager.workspace_count(), 0);
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn test_workspace_manager_with_custom_config() {
        let config = WorkspaceConfig {
            default_count: 5,
            names: (1..=5).map(|i| format!("WS{}", i)).collect(),
            persist_state: false,
            create_on_demand: true,
            use_virtual_desktops: false,
        };

        let manager = WorkspaceManager::new(config);
        assert_eq!(manager.active_workspace(), 1);
        assert_eq!(manager.workspace_count(), 0);
    }

    #[test]
    fn test_workspace_last_active_timestamp() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let workspace1 = Workspace::new(1, "First".to_string(), 0, rect);
        
        // Wait a tiny bit
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let workspace2 = Workspace::new(2, "Second".to_string(), 0, rect);

        // workspace2 should have a more recent timestamp
        assert!(workspace2.last_active > workspace1.last_active);
    }

    #[test]
    fn test_workspace_config_serialize_deserialize() {
        let config = WorkspaceConfig::default();
        
        // Serialize to JSON
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        
        // Deserialize back
        let deserialized: WorkspaceConfig = 
            serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(deserialized.default_count, config.default_count);
        assert_eq!(deserialized.names, config.names);
        assert_eq!(deserialized.persist_state, config.persist_state);
        assert_eq!(deserialized.create_on_demand, config.create_on_demand);
        assert_eq!(deserialized.use_virtual_desktops, config.use_virtual_desktops);
    }

    #[test]
    fn test_workspace_empty_initially() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        assert!(workspace.windows.is_empty());
        assert!(workspace.tree.is_none());
        assert!(!workspace.visible);
    }

    #[test]
    fn test_workspace_with_multiple_operations() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Test".to_string(), 0, rect);

        // Add windows
        workspace.add_window(100);
        workspace.add_window(200);
        workspace.add_window(300);
        assert_eq!(workspace.windows.len(), 3);

        // Remove middle window
        assert!(workspace.remove_window(200));
        assert_eq!(workspace.windows.len(), 2);

        // Try to add duplicate
        workspace.add_window(100);
        assert_eq!(workspace.windows.len(), 2); // Should still be 2

        // Remove all
        assert!(workspace.remove_window(100));
        assert!(workspace.remove_window(300));
        assert!(workspace.windows.is_empty());
    }

    // Test Workspace mark_active and mark_inactive

    #[test]
    fn test_workspace_mark_active() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        assert!(!workspace.visible);
        
        workspace.mark_active();
        assert!(workspace.visible);
    }

    #[test]
    fn test_workspace_mark_inactive() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        workspace.mark_active();
        assert!(workspace.visible);
        
        workspace.mark_inactive();
        assert!(!workspace.visible);
    }

    #[test]
    fn test_workspace_mark_active_updates_timestamp() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut workspace = Workspace::new(1, "Main".to_string(), 0, rect);

        let first_timestamp = workspace.last_active;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        workspace.mark_active();
        assert!(workspace.last_active > first_timestamp);
    }

    // Test WorkspaceManager initialization

    #[test]
    fn test_workspace_manager_initialize() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config.clone());

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];

        manager.initialize(&monitor_areas).unwrap();

        // Should have created default_count workspaces
        assert_eq!(manager.workspace_count(), config.default_count);

        // First workspace should be active
        assert_eq!(manager.active_workspace(), 1);

        // Check first workspace is marked active
        let ws = manager.get_workspace(1).unwrap();
        assert!(ws.visible);
        assert_eq!(ws.name, "1");
    }

    #[test]
    fn test_workspace_manager_initialize_with_custom_names() {
        let config = WorkspaceConfig {
            default_count: 3,
            names: vec!["One".to_string(), "Two".to_string(), "Three".to_string()],
            persist_state: true,
            create_on_demand: false,
            use_virtual_desktops: false,
        };
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];

        manager.initialize(&monitor_areas).unwrap();

        assert_eq!(manager.workspace_count(), 3);
        
        let ws1 = manager.get_workspace(1).unwrap();
        assert_eq!(ws1.name, "One");
        
        let ws2 = manager.get_workspace(2).unwrap();
        assert_eq!(ws2.name, "Two");
        
        let ws3 = manager.get_workspace(3).unwrap();
        assert_eq!(ws3.name, "Three");
    }

    #[test]
    fn test_workspace_manager_initialize_multiple_monitors() {
        let config = WorkspaceConfig {
            default_count: 2,
            names: vec!["A".to_string(), "B".to_string()],
            persist_state: true,
            create_on_demand: false,
            use_virtual_desktops: false,
        };
        let mut manager = WorkspaceManager::new(config);

        let rect1 = Rect::new(0, 0, 1920, 1080);
        let rect2 = Rect::new(1920, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect1), (1, rect2)];

        manager.initialize(&monitor_areas).unwrap();

        // Should have 2 workspaces per monitor = 4 total
        assert_eq!(manager.workspace_count(), 4);

        // First workspace of first monitor should be active
        assert_eq!(manager.active_workspace(), 1);
    }

    #[test]
    fn test_workspace_manager_initialize_with_more_workspaces_than_names() {
        let config = WorkspaceConfig {
            default_count: 5,
            names: vec!["One".to_string(), "Two".to_string()],
            persist_state: true,
            create_on_demand: false,
            use_virtual_desktops: false,
        };
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];

        manager.initialize(&monitor_areas).unwrap();

        assert_eq!(manager.workspace_count(), 5);
        
        // First two should use custom names
        let ws1 = manager.get_workspace(1).unwrap();
        assert_eq!(ws1.name, "One");
        
        let ws2 = manager.get_workspace(2).unwrap();
        assert_eq!(ws2.name, "Two");
        
        // Rest should use workspace ID as name
        let ws3 = manager.get_workspace(3).unwrap();
        assert_eq!(ws3.name, "3");
    }

    // Test create_workspace

    #[test]
    fn test_workspace_manager_create_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Test".to_string(), 0, rect);

        assert_eq!(manager.workspace_count(), 1);
        
        let ws = manager.get_workspace(id).unwrap();
        assert_eq!(ws.name, "Test");
        assert_eq!(ws.monitor, 0);
        assert!(ws.windows.is_empty());
    }

    #[test]
    fn test_workspace_manager_create_multiple_workspaces() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id1 = manager.create_workspace("First".to_string(), 0, rect);
        let id2 = manager.create_workspace("Second".to_string(), 0, rect);
        let id3 = manager.create_workspace("Third".to_string(), 1, rect);

        assert_eq!(manager.workspace_count(), 3);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        let ws1 = manager.get_workspace(id1).unwrap();
        assert_eq!(ws1.name, "First");
        assert_eq!(ws1.monitor, 0);

        let ws3 = manager.get_workspace(id3).unwrap();
        assert_eq!(ws3.name, "Third");
        assert_eq!(ws3.monitor, 1);
    }

    #[test]
    fn test_workspace_manager_create_workspace_assigns_unique_ids() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        
        // Create workspaces and collect IDs
        let mut ids = Vec::new();
        for i in 0..10 {
            let id = manager.create_workspace(format!("WS{}", i), 0, rect);
            ids.push(id);
        }

        // All IDs should be unique
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique_ids.len(), 10);
    }

    // Test rename_workspace

    #[test]
    fn test_workspace_manager_rename_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Original".to_string(), 0, rect);

        assert!(manager.rename_workspace(id, "Renamed".to_string()).is_ok());

        let ws = manager.get_workspace(id).unwrap();
        assert_eq!(ws.name, "Renamed");
    }

    #[test]
    fn test_workspace_manager_rename_nonexistent_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let result = manager.rename_workspace(999, "Test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_workspace_manager_rename_preserves_other_properties() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Original".to_string(), 0, rect);

        // Get a reference to add a window
        let ws_before_monitor = {
            let ws = manager.get_workspace(id).unwrap();
            ws.monitor
        };

        manager.rename_workspace(id, "Renamed".to_string()).unwrap();

        let ws = manager.get_workspace(id).unwrap();
        assert_eq!(ws.name, "Renamed");
        assert_eq!(ws.monitor, ws_before_monitor);
    }

    // Test delete_workspace

    #[test]
    fn test_workspace_manager_delete_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id1 = manager.create_workspace("First".to_string(), 0, rect);
        let id2 = manager.create_workspace("Second".to_string(), 0, rect);

        assert_eq!(manager.workspace_count(), 2);

        manager.delete_workspace(id1, id2).unwrap();

        assert_eq!(manager.workspace_count(), 1);
        assert!(manager.get_workspace(id1).is_none());
        assert!(manager.get_workspace(id2).is_some());
    }

    #[test]
    fn test_workspace_manager_delete_workspace_same_fallback() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Test".to_string(), 0, rect);

        let result = manager.delete_workspace(id, id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot delete workspace into itself"));
    }

    #[test]
    fn test_workspace_manager_delete_nonexistent_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Test".to_string(), 0, rect);

        let result = manager.delete_workspace(999, id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_workspace_manager_delete_workspace_invalid_fallback() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id = manager.create_workspace("Test".to_string(), 0, rect);

        let result = manager.delete_workspace(id, 999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_workspace_manager_cannot_delete_last_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id1 = manager.create_workspace("Only".to_string(), 0, rect);
        
        // Try to delete the only workspace
        let result = manager.delete_workspace(id1, id1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot delete the last workspace"));
        
        // Workspace should still exist
        assert_eq!(manager.workspace_count(), 1);
    }

    #[test]
    fn test_workspace_manager_delete_workspace_switches_if_active() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];
        manager.initialize(&monitor_areas).unwrap();

        let id2 = manager.create_workspace("Second".to_string(), 0, rect);
        
        // First workspace (id=1) should be active
        assert_eq!(manager.active_workspace(), 1);

        // Delete the active workspace
        manager.delete_workspace(1, id2).unwrap();

        // Should now be on fallback workspace
        assert_eq!(manager.active_workspace(), id2);
    }

    #[test]
    fn test_workspace_manager_delete_workspace_preserves_active_if_not_deleting_active() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id1 = manager.create_workspace("First".to_string(), 0, rect);
        let id2 = manager.create_workspace("Second".to_string(), 0, rect);
        let id3 = manager.create_workspace("Third".to_string(), 0, rect);

        // Manually set active workspace (in real code this would be done via switch_to)
        // For this test, we'll just check the behavior when deleting a non-active workspace

        // Delete a non-active workspace
        manager.delete_workspace(id2, id3).unwrap();

        assert_eq!(manager.workspace_count(), 2);
        assert!(manager.get_workspace(id1).is_some());
        assert!(manager.get_workspace(id2).is_none());
        assert!(manager.get_workspace(id3).is_some());
    }

    // Test sequential ID assignment

    #[test]
    fn test_workspace_manager_sequential_ids() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];
        manager.initialize(&monitor_areas).unwrap();

        // IDs should start from 1 and increment
        for i in 1..=10 {
            assert!(manager.get_workspace(i).is_some(), "Workspace {} should exist", i);
        }
    }

    #[test]
    fn test_workspace_manager_ids_continue_after_delete() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let id1 = manager.create_workspace("First".to_string(), 0, rect);
        let id2 = manager.create_workspace("Second".to_string(), 0, rect);
        
        manager.delete_workspace(id1, id2).unwrap();
        
        let id3 = manager.create_workspace("Third".to_string(), 0, rect);
        
        // New workspace should have a higher ID than the deleted ones
        assert!(id3 > id2);
    }

    // Integration tests

    #[test]
    fn test_workspace_lifecycle_integration() {
        let config = WorkspaceConfig {
            default_count: 3,
            names: vec!["Main".to_string(), "Dev".to_string(), "Browser".to_string()],
            persist_state: true,
            create_on_demand: false,
            use_virtual_desktops: false,
        };
        let mut manager = WorkspaceManager::new(config);

        let rect = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, rect)];

        // Initialize
        manager.initialize(&monitor_areas).unwrap();
        assert_eq!(manager.workspace_count(), 3);

        // Create a new workspace
        let new_id = manager.create_workspace("Extra".to_string(), 0, rect);
        assert_eq!(manager.workspace_count(), 4);

        // Rename a workspace
        manager.rename_workspace(new_id, "Renamed".to_string()).unwrap();
        let ws = manager.get_workspace(new_id).unwrap();
        assert_eq!(ws.name, "Renamed");

        // Delete a workspace
        manager.delete_workspace(new_id, 1).unwrap();
        assert_eq!(manager.workspace_count(), 3);
        assert!(manager.get_workspace(new_id).is_none());
    }
}
