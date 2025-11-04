//! Configuration file watcher
//! 
//! This module provides file watching capability for configuration hot-reload.

use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify::event::{ModifyKind, DataChange};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// Configuration file watcher
pub struct ConfigWatcher {
    /// File system watcher
    _watcher: Box<dyn Watcher>,
    
    /// Event receiver
    receiver: Receiver<notify::Result<Event>>,
    
    /// Last reload time (for debouncing)
    last_reload: Option<Instant>,
    
    /// Debounce duration
    debounce_duration: Duration,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
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
        })
    }
    
    /// Set debounce duration
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    /// Check if configuration file has changed
    /// Returns true if a change was detected and debounce period has passed
    pub fn check_for_changes(&mut self) -> bool {
        // Check if debounce period has passed
        if let Some(last) = self.last_reload {
            if last.elapsed() < self.debounce_duration {
                // Still in debounce period, drain events but don't report change
                self.receiver.try_iter().count();
                return false;
            }
        }
        
        // Check for relevant events
        let has_change = self.receiver
            .try_iter()
            .any(|event| {
                if let Ok(event) = event {
                    matches!(
                        event.kind,
                        EventKind::Modify(ModifyKind::Data(DataChange::Any))
                        | EventKind::Modify(ModifyKind::Data(DataChange::Content))
                    )
                } else {
                    false
                }
            });
        
        if has_change {
            self.last_reload = Some(Instant::now());
            tracing::info!("Configuration file change detected");
        }
        
        has_change
    }
}
