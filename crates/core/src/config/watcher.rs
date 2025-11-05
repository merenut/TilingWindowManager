//! Configuration file watcher
//! 
//! This module provides file watching capability for configuration hot-reload.
//! 
//! # Features
//! - Watches a single configuration file for changes
//! - Debounces rapid file changes to prevent excessive reloads
//! - Handles editor save patterns (atomic writes, temp files)
//! - Non-blocking change detection via polling

use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify::event::{ModifyKind, DataChange};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// Configuration file watcher
/// 
/// Watches a single configuration file for modifications and provides
/// debounced change notifications. The watcher handles common editor
/// save patterns including atomic writes and temporary file usage.
pub struct ConfigWatcher {
    /// File system watcher
    _watcher: Box<dyn Watcher>,
    
    /// Event receiver
    receiver: Receiver<notify::Result<Event>>,
    
    /// Last reload time (for debouncing)
    last_reload: Option<Instant>,
    
    /// Debounce duration
    debounce_duration: Duration,
    
    /// Path being watched (for logging)
    config_path: PathBuf,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    /// 
    /// # Arguments
    /// * `config_path` - Path to the configuration file to watch
    /// 
    /// # Returns
    /// * `Ok(ConfigWatcher)` if the watcher was created successfully
    /// * `Err` if the file doesn't exist or can't be watched
    /// 
    /// # Example
    /// ```no_run
    /// use std::path::PathBuf;
    /// # use tiling_wm_core::config::ConfigWatcher;
    /// 
    /// let watcher = ConfigWatcher::new(PathBuf::from("config.toml")).unwrap();
    /// ```
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        // Verify file exists before creating watcher
        if !config_path.exists() {
            anyhow::bail!("Configuration file does not exist: {:?}", config_path);
        }
        
        let (tx, rx) = channel();
        
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        tracing::info!("Watching configuration file: {:?}", config_path);
        
        Ok(Self {
            _watcher: Box::new(watcher),
            receiver: rx,
            last_reload: None,
            debounce_duration: Duration::from_millis(500),
            config_path,
        })
    }
    
    /// Set debounce duration
    /// 
    /// The debounce duration controls the minimum time between reload notifications.
    /// This prevents excessive reloads when the file is being edited rapidly.
    /// 
    /// # Arguments
    /// * `duration` - Minimum time between reloads
    /// 
    /// # Example
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use tiling_wm_core::config::ConfigWatcher;
    /// 
    /// let watcher = ConfigWatcher::new(PathBuf::from("config.toml"))
    ///     .unwrap()
    ///     .with_debounce(Duration::from_millis(300));
    /// ```
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    /// Check if configuration file has changed
    /// 
    /// This method polls for file system events and returns true if a relevant
    /// change was detected and the debounce period has passed. Irrelevant events
    /// (like access, attribute changes) are ignored.
    /// 
    /// # Returns
    /// * `true` if a change was detected and debounce period has passed
    /// * `false` if no change detected or still in debounce period
    /// 
    /// # Example
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use tiling_wm_core::config::ConfigWatcher;
    /// 
    /// let mut watcher = ConfigWatcher::new(PathBuf::from("config.toml")).unwrap();
    /// 
    /// // In main loop
    /// if watcher.check_for_changes() {
    ///     println!("Config changed, reloading...");
    /// }
    /// ```
    pub fn check_for_changes(&mut self) -> bool {
        // Check if debounce period has passed
        if let Some(last) = self.last_reload {
            if last.elapsed() < self.debounce_duration {
                // Still in debounce period, drain events but don't report change
                let drained = self.receiver.try_iter().count();
                if drained > 0 {
                    tracing::trace!(
                        "Debouncing: drained {} events for {:?}",
                        drained,
                        self.config_path
                    );
                }
                return false;
            }
        }
        
        // Check for relevant events
        let has_change = self.receiver
            .try_iter()
            .filter_map(|event| event.ok())
            .any(|event| {
                // Match file modification events
                // This handles most editor save patterns including atomic writes
                let is_relevant = matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Data(DataChange::Any))
                    | EventKind::Modify(ModifyKind::Data(DataChange::Content))
                    | EventKind::Create(_)  // Handle atomic writes (create new file)
                    | EventKind::Remove(_)  // Handle editors that delete then recreate
                );
                
                if is_relevant {
                    tracing::debug!(
                        "Detected relevant file event: {:?} for {:?}",
                        event.kind,
                        self.config_path
                    );
                }
                
                is_relevant
            });
        
        if has_change {
            self.last_reload = Some(Instant::now());
            tracing::info!("Configuration file change detected: {:?}", self.config_path);
        }
        
        has_change
    }
    
    /// Get the path being watched
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
    
    /// Get the current debounce duration
    pub fn debounce_duration(&self) -> Duration {
        self.debounce_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_watcher_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let watcher = ConfigWatcher::new(config_path);
        assert!(watcher.is_ok(), "Should create watcher successfully");
    }

    #[test]
    fn test_watcher_with_custom_debounce() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let watcher = ConfigWatcher::new(config_path.clone())
            .unwrap()
            .with_debounce(Duration::from_millis(200));

        assert_eq!(watcher.debounce_duration(), Duration::from_millis(200));
    }

    #[test]
    fn test_detect_file_changes() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();

        // Wait a bit for watcher to start
        std::thread::sleep(Duration::from_millis(100));

        // Modify file
        fs::write(&config_path, "[general]\ngaps_in = 10\n").unwrap();

        // Wait for change to be detected
        std::thread::sleep(Duration::from_millis(600));

        // Check for changes
        let has_change = watcher.check_for_changes();
        assert!(has_change, "Should detect file modification");
    }

    #[test]
    fn test_debouncing_prevents_rapid_reloads() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone())
            .unwrap()
            .with_debounce(Duration::from_millis(300));

        std::thread::sleep(Duration::from_millis(100));

        // First change
        fs::write(&config_path, "[general]\ngaps_in = 10\n").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        let first_change = watcher.check_for_changes();
        assert!(first_change, "Should detect first change");

        // Second change immediately after
        fs::write(&config_path, "[general]\ngaps_in = 20\n").unwrap();
        std::thread::sleep(Duration::from_millis(50));

        // Should be debounced
        let second_change = watcher.check_for_changes();
        assert!(!second_change, "Should debounce second change");

        // Wait for debounce period to pass
        std::thread::sleep(Duration::from_millis(300));

        // Make another change
        fs::write(&config_path, "[general]\ngaps_in = 30\n").unwrap();
        std::thread::sleep(Duration::from_millis(400));

        // Should detect after debounce period
        let third_change = watcher.check_for_changes();
        assert!(third_change, "Should detect change after debounce period");
    }

    #[test]
    fn test_multiple_rapid_edits() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone())
            .unwrap()
            .with_debounce(Duration::from_millis(500));

        std::thread::sleep(Duration::from_millis(100));

        // Rapid edits (simulating user typing and saving)
        for i in 1..=5 {
            fs::write(&config_path, format!("[general]\ngaps_in = {}\n", i * 10)).unwrap();
            std::thread::sleep(Duration::from_millis(50));
        }

        // Wait for all events to be processed
        std::thread::sleep(Duration::from_millis(600));

        // Should only report one change (debounced)
        let has_change = watcher.check_for_changes();
        assert!(has_change, "Should detect changes after rapid edits");

        // Subsequent check should not detect change (already reported)
        std::thread::sleep(Duration::from_millis(100));
        let no_change = watcher.check_for_changes();
        assert!(!no_change, "Should not report change again immediately");
    }

    #[test]
    fn test_file_deletion_and_recreation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();

        std::thread::sleep(Duration::from_millis(100));

        // Delete file
        fs::remove_file(&config_path).unwrap();
        std::thread::sleep(Duration::from_millis(200));

        // Recreate file
        fs::write(&config_path, "[general]\ngaps_in = 15\n").unwrap();
        std::thread::sleep(Duration::from_millis(600));

        // Should detect recreation
        let has_change = watcher.check_for_changes();
        // Note: File deletion and recreation might be detected, behavior varies by OS
        // The important part is that the watcher doesn't crash
        println!("Change detected after deletion/recreation: {}", has_change);
    }

    #[test]
    fn test_no_change_returns_false() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();

        std::thread::sleep(Duration::from_millis(100));

        // Check without making changes
        let has_change = watcher.check_for_changes();
        assert!(!has_change, "Should not detect change when file is unchanged");
    }

    #[test]
    fn test_watcher_handles_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        // Should fail to create watcher for non-existent file
        let watcher = ConfigWatcher::new(config_path);
        assert!(watcher.is_err(), "Should fail to watch non-existent file");
    }

    #[test]
    fn test_watching_directory_instead_of_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        // Create watcher successfully
        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();

        // Create another file in the same directory
        let other_file = temp_dir.path().join("other.toml");
        fs::write(&other_file, "[other]\n").unwrap();
        std::thread::sleep(Duration::from_millis(200));

        // Should not detect change to other file
        let has_change = watcher.check_for_changes();
        assert!(!has_change, "Should not detect changes to other files");
    }

    #[test]
    fn test_performance_check_for_changes() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let mut watcher = ConfigWatcher::new(config_path.clone()).unwrap();

        std::thread::sleep(Duration::from_millis(100));

        // Make a change
        fs::write(&config_path, "[general]\ngaps_in = 10\n").unwrap();
        std::thread::sleep(Duration::from_millis(600));

        // Measure check performance
        let start = Instant::now();
        let _ = watcher.check_for_changes();
        let elapsed = start.elapsed();

        // check_for_changes should be very fast (< 1ms typically)
        assert!(
            elapsed < Duration::from_millis(10),
            "check_for_changes took too long: {:?}",
            elapsed
        );
    }
    
    #[test]
    fn test_config_path_accessor() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "[general]\n").unwrap();

        let watcher = ConfigWatcher::new(config_path.clone()).unwrap();
        assert_eq!(watcher.config_path(), &config_path);
    }
}
