//! Automatic workspace state persistence.
//!
//! This module provides the AutoSaver component that periodically saves workspace state
//! to disk in the background without blocking the main application.

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

/// Manages automatic periodic saving of workspace state.
///
/// The AutoSaver runs a background task that periodically saves the workspace state
/// to disk using the PersistenceManager. It can be started and stopped, and saves
/// happen at a configurable interval.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
/// use tiling_wm_core::workspace::{WorkspaceManager, WorkspaceConfig};
/// use tiling_wm_core::workspace::persistence::PersistenceManager;
/// use tiling_wm_core::workspace::auto_save::AutoSaver;
///
/// # #[tokio::main]
/// # async fn main() {
/// let config = WorkspaceConfig::default();
/// let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
/// let persistence = Arc::new(PersistenceManager::new());
///
/// // Create auto-saver with 60-second interval
/// let auto_saver = AutoSaver::new(manager, persistence, 60);
///
/// // Start auto-saving
/// auto_saver.start().await;
///
/// // ... application runs ...
///
/// // Stop auto-saving
/// auto_saver.stop().await;
/// # }
/// ```
pub struct AutoSaver {
    manager: Arc<Mutex<crate::workspace::manager::WorkspaceManager>>,
    persistence: Arc<crate::workspace::persistence::PersistenceManager>,
    interval: Duration,
    running: Arc<Mutex<bool>>,
}

impl AutoSaver {
    /// Create a new AutoSaver.
    ///
    /// # Arguments
    ///
    /// * `manager` - Shared reference to the WorkspaceManager
    /// * `persistence` - Shared reference to the PersistenceManager
    /// * `interval_secs` - Auto-save interval in seconds
    ///
    /// # Returns
    ///
    /// A new AutoSaver instance that is not yet started.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use tiling_wm_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// use tiling_wm_core::workspace::persistence::PersistenceManager;
    /// use tiling_wm_core::workspace::auto_save::AutoSaver;
    ///
    /// let config = WorkspaceConfig::default();
    /// let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    /// let persistence = Arc::new(PersistenceManager::new());
    ///
    /// // Create auto-saver with 5-minute interval
    /// let auto_saver = AutoSaver::new(manager, persistence, 300);
    /// ```
    pub fn new(
        manager: Arc<Mutex<crate::workspace::manager::WorkspaceManager>>,
        persistence: Arc<crate::workspace::persistence::PersistenceManager>,
        interval_secs: u64,
    ) -> Self {
        Self {
            manager,
            persistence,
            interval: Duration::from_secs(interval_secs),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the auto-save task.
    ///
    /// This method spawns a background tokio task that periodically saves the workspace
    /// state. If the auto-saver is already running, this method does nothing.
    ///
    /// The background task will continue running until `stop()` is called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio::sync::Mutex;
    /// # use tiling_wm_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// # use tiling_wm_core::workspace::persistence::PersistenceManager;
    /// # use tiling_wm_core::workspace::auto_save::AutoSaver;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let config = WorkspaceConfig::default();
    /// # let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    /// # let persistence = Arc::new(PersistenceManager::new());
    /// let auto_saver = AutoSaver::new(manager, persistence, 60);
    ///
    /// // Start the background auto-save task
    /// auto_saver.start().await;
    /// # }
    /// ```
    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);
        
        let manager = self.manager.clone();
        let persistence = self.persistence.clone();
        let interval = self.interval;
        let running = self.running.clone();
        
        // Note: The task handle is intentionally not stored because:
        // 1. The task is designed to check the running flag and exit gracefully
        // 2. Storing the handle would complicate the struct lifetime
        // 3. The stop() method sets the flag, allowing the task to complete naturally
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                let should_continue = *running.lock().await;
                if !should_continue {
                    break;
                }
                
                let manager_guard = manager.lock().await;
                manager_guard.auto_save(&persistence);
                drop(manager_guard);
                
                tracing::debug!("Auto-saved workspace state");
            }
        });
    }
    
    /// Stop the auto-save task.
    ///
    /// This method signals the background auto-save task to stop. The task will
    /// complete its current sleep cycle before stopping.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio::sync::Mutex;
    /// # use tiling_wm_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// # use tiling_wm_core::workspace::persistence::PersistenceManager;
    /// # use tiling_wm_core::workspace::auto_save::AutoSaver;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let config = WorkspaceConfig::default();
    /// # let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    /// # let persistence = Arc::new(PersistenceManager::new());
    /// # let auto_saver = AutoSaver::new(manager, persistence, 60);
    /// # auto_saver.start().await;
    /// // Stop the auto-save task
    /// auto_saver.stop().await;
    /// # }
    /// ```
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{WorkspaceManager, WorkspaceConfig};
    use crate::workspace::persistence::PersistenceManager;
    use tempfile::tempdir;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_auto_saver_creation() {
        let config = WorkspaceConfig::default();
        let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
        let persistence = Arc::new(PersistenceManager::new());
        
        let auto_saver = AutoSaver::new(manager, persistence, 60);
        
        // Verify the auto-saver is not running initially
        let running = auto_saver.running.lock().await;
        assert!(!*running);
    }

    #[tokio::test]
    async fn test_auto_saver_start_stop() {
        let config = WorkspaceConfig::default();
        let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
        let persistence = Arc::new(PersistenceManager::new());
        
        let auto_saver = AutoSaver::new(manager, persistence, 60);
        
        // Start the auto-saver
        auto_saver.start().await;
        
        // Verify it's running
        let running = auto_saver.running.lock().await;
        assert!(*running);
        drop(running);
        
        // Stop the auto-saver
        auto_saver.stop().await;
        
        // Verify it's stopped
        let running = auto_saver.running.lock().await;
        assert!(!*running);
    }

    #[tokio::test]
    async fn test_auto_saver_double_start() {
        let config = WorkspaceConfig::default();
        let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
        let persistence = Arc::new(PersistenceManager::new());
        
        let auto_saver = AutoSaver::new(manager, persistence, 60);
        
        // Start twice
        auto_saver.start().await;
        auto_saver.start().await;
        
        // Should still be running (no panic or error)
        let running = auto_saver.running.lock().await;
        assert!(*running);
    }

    #[tokio::test]
    async fn test_auto_saver_saves_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        // Create config with persistence enabled
        let mut config = WorkspaceConfig::default();
        config.persist_state = true;
        
        let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
        let persistence = Arc::new(PersistenceManager::with_custom_path(state_path.clone()));
        
        // Create auto-saver with very short interval for testing
        let auto_saver = AutoSaver::new(manager, persistence, 1);
        
        // Start auto-saver
        auto_saver.start().await;
        
        // Wait for at least one save cycle
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Stop auto-saver
        auto_saver.stop().await;
        
        // Verify state file was created
        assert!(state_path.exists());
    }

    #[tokio::test]
    async fn test_auto_saver_respects_persist_state_flag() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        // Create config with persistence DISABLED
        let mut config = WorkspaceConfig::default();
        config.persist_state = false;
        
        let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
        let persistence = Arc::new(PersistenceManager::with_custom_path(state_path.clone()));
        
        // Create auto-saver with very short interval for testing
        let auto_saver = AutoSaver::new(manager, persistence, 1);
        
        // Start auto-saver
        auto_saver.start().await;
        
        // Wait for at least one save cycle
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Stop auto-saver
        auto_saver.stop().await;
        
        // Verify state file was NOT created (persist_state was false)
        assert!(!state_path.exists());
    }
}
