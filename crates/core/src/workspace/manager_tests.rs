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
}
