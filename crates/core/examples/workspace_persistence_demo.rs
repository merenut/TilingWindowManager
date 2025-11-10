//! Example demonstrating workspace persistence and auto-save functionality.
//!
//! This example shows how to:
//! 1. Create a WorkspaceManager with persistence enabled
//! 2. Save and restore workspace state
//! 3. Set up automatic periodic saving

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::LocalSet;
use tenraku_core::workspace::{WorkspaceManager, WorkspaceConfig};
use tenraku_core::workspace::persistence::PersistenceManager;
use tenraku_core::workspace::auto_save::AutoSaver;
use tenraku_core::window_manager::tree::Rect;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    let local = LocalSet::new();

    local
        .run_until(async {
            // Create configuration with persistence enabled
            let mut config = WorkspaceConfig::default();
            config.persist_state = true;

            // Create workspace manager and persistence manager
            let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
            let persistence = Arc::new(PersistenceManager::new());

            // Initialize workspaces with monitor areas
            let monitor_areas = vec![
                (0, Rect::new(0, 0, 1920, 1080)),
                (1, Rect::new(1920, 0, 1920, 1080)),
            ];

            {
                let mut manager_guard = manager.lock().await;
                manager_guard.initialize(&monitor_areas)?;
                
                // Restore previous state if available
                if persistence.has_saved_state() {
                    tracing::info!("Restoring previous workspace state...");
                    manager_guard.restore_state(&persistence, &monitor_areas)?;
                }
            }

            // Create and start auto-saver with 5-minute interval
            let auto_saver = AutoSaver::new(
                manager.clone(),
                persistence.clone(),
                300, // 5 minutes
            );

            tracing::info!("Starting auto-save with 5-minute interval...");
            auto_saver.start().await;

            // Simulate application runtime
            tracing::info!("Application running... (press Ctrl+C to stop)");
            
            // In a real application, this would be your main event loop
            tokio::signal::ctrl_c().await?;

            // Cleanup: stop auto-saver and save final state
            tracing::info!("Shutting down...");
            auto_saver.stop().await;

            {
                let manager_guard = manager.lock().await;
                tracing::info!("Saving final workspace state...");
                manager_guard.save_state(&persistence)?;
            }

            tracing::info!("Shutdown complete");
            Ok(())
        })
        .await
}
