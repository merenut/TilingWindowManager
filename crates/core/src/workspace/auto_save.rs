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
/// use tenraku_core::workspace::{WorkspaceManager, WorkspaceConfig};
/// use tenraku_core::workspace::persistence::PersistenceManager;
/// use tenraku_core::workspace::auto_save::AutoSaver;
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
    manager: Arc<Mutex<crate::workspace::core::WorkspaceManager>>,
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
    /// use tenraku_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    /// use tenraku_core::workspace::auto_save::AutoSaver;
    ///
    /// let config = WorkspaceConfig::default();
    /// let manager = Arc::new(Mutex::new(WorkspaceManager::new(config)));
    /// let persistence = Arc::new(PersistenceManager::new());
    ///
    /// // Create auto-saver with 5-minute interval
    /// let auto_saver = AutoSaver::new(manager, persistence, 300);
    /// ```
    pub fn new(
        manager: Arc<Mutex<crate::workspace::core::WorkspaceManager>>,
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
    /// This method must be invoked from within a [`tokio::task::LocalSet`]
    /// because the workspace state contains non-`Send` data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio::sync::Mutex;
    /// # use tenraku_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// # use tenraku_core::workspace::persistence::PersistenceManager;
    /// # use tenraku_core::workspace::auto_save::AutoSaver;
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
        tokio::task::spawn_local(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

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
    /// # use tenraku_core::workspace::{WorkspaceManager, WorkspaceConfig};
    /// # use tenraku_core::workspace::persistence::PersistenceManager;
    /// # use tenraku_core::workspace::auto_save::AutoSaver;
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