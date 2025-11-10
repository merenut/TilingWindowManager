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
use std::sync::mpsc::{channel, Receiver};
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
    /// # use tenraku_core::config::ConfigWatcher;
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
    /// # use tenraku_core::config::ConfigWatcher;
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
    /// # use tenraku_core::config::ConfigWatcher;
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