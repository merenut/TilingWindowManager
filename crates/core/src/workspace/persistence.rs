//! Workspace state persistence module.
//!
//! This module provides functionality for serializing and deserializing workspace state
//! to and from JSON files, with support for backup and recovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Represents the serializable state of a single workspace.
///
/// This structure contains all the information needed to restore a workspace,
/// including its ID, name, monitor assignment, windows, and virtual desktop association.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceState {
    /// Unique workspace ID
    pub id: usize,
    
    /// Human-readable workspace name
    pub name: String,
    
    /// Monitor this workspace is assigned to
    pub monitor: usize,
    
    /// Windows in this workspace
    pub windows: Vec<WindowState>,
    
    /// Virtual Desktop ID (if using Virtual Desktop integration)
    pub virtual_desktop_id: Option<String>,
}

/// Represents the serializable state of a window.
///
/// This structure captures the essential information about a window
/// that allows it to be tracked and restored to the correct workspace.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    /// Window handle as a string (for cross-platform compatibility)
    pub hwnd: String,
    
    /// Process name of the window owner
    pub process_name: String,
    
    /// Window title
    pub title: String,
    
    /// Window class name
    pub class_name: String,
    
    /// Workspace ID this window belongs to
    pub workspace: usize,
}

/// Represents the complete session state including all workspaces.
///
/// This is the top-level structure that gets serialized to disk,
/// containing versioning information and all workspace states.
#[derive(Serialize, Deserialize, Debug)]
pub struct SessionState {
    /// Version string for compatibility checking
    pub version: String,
    
    /// Unix timestamp of when this state was saved
    pub timestamp: u64,
    
    /// All workspace states
    pub workspaces: Vec<WorkspaceState>,
    
    /// Currently active workspace ID
    pub active_workspace: usize,
    
    /// Mapping of window handles (as strings) to workspace IDs
    pub window_to_workspace: HashMap<String, usize>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            workspaces: Vec::new(),
            active_workspace: 1,
            window_to_workspace: HashMap::new(),
        }
    }
}

/// Manages persistence of workspace state to disk.
///
/// The PersistenceManager handles saving and loading workspace state to JSON files,
/// with automatic backup creation and recovery from corrupted files.
pub struct PersistenceManager {
    /// Path to the main state file
    state_file: PathBuf,
    
    /// Path to the backup state file
    backup_file: PathBuf,
}

impl PersistenceManager {
    /// Create a new PersistenceManager with default paths.
    ///
    /// The state files are stored in the user's config directory under "tiling-wm".
    /// If the config directory cannot be determined, falls back to the current directory.
    ///
    /// # Returns
    ///
    /// A new PersistenceManager instance.
    ///
    /// # Example
    ///
    /// ```
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    ///
    /// let manager = PersistenceManager::new();
    /// ```
    pub fn new() -> Self {
        let state_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiling-wm");
        
        if let Err(e) = fs::create_dir_all(&state_dir) {
            tracing::warn!("Failed to create state directory {:?}: {}", state_dir, e);
        }
        
        Self {
            state_file: state_dir.join("session.json"),
            backup_file: state_dir.join("session.backup.json"),
        }
    }
    
    /// Create a new PersistenceManager with custom paths.
    ///
    /// This is primarily useful for testing with temporary directories.
    ///
    /// # Arguments
    ///
    /// * `state_path` - Custom path for the state file
    ///
    /// # Returns
    ///
    /// A new PersistenceManager instance with custom paths.
    #[cfg(test)]
    pub fn with_custom_path(state_path: PathBuf) -> Self {
        let backup_path = if let Some(parent) = state_path.parent() {
            parent.join("session.backup.json")
        } else {
            PathBuf::from("session.backup.json")
        };
        
        Self {
            state_file: state_path,
            backup_file: backup_path,
        }
    }
    
    /// Save workspace state to disk.
    ///
    /// Before saving, if a state file already exists, it is copied to the backup location.
    /// This ensures that the previous state can be recovered if the save operation fails
    /// or if the new state file becomes corrupted.
    ///
    /// # Arguments
    ///
    /// * `state` - The session state to save
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the save operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::workspace::persistence::{PersistenceManager, SessionState};
    ///
    /// let manager = PersistenceManager::new();
    /// let state = SessionState::default();
    /// manager.save_state(&state).unwrap();
    /// ```
    pub fn save_state(&self, state: &SessionState) -> anyhow::Result<()> {
        // Create backup of existing state file
        if self.state_file.exists() {
            fs::copy(&self.state_file, &self.backup_file)?;
        }
        
        // Serialize state to pretty JSON
        let json = serde_json::to_string_pretty(state)?;
        
        // Write to file
        fs::write(&self.state_file, json)?;
        
        tracing::info!("Saved workspace state to {:?}", self.state_file);
        Ok(())
    }
    
    /// Load workspace state from disk.
    ///
    /// Reads the state file and deserializes it to a SessionState structure.
    ///
    /// # Returns
    ///
    /// `Ok(SessionState)` on success, or an error if the state file doesn't exist
    /// or cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    ///
    /// let manager = PersistenceManager::new();
    /// match manager.load_state() {
    ///     Ok(state) => println!("Loaded state with {} workspaces", state.workspaces.len()),
    ///     Err(e) => eprintln!("Failed to load state: {}", e),
    /// }
    /// ```
    pub fn load_state(&self) -> anyhow::Result<SessionState> {
        if !self.state_file.exists() {
            anyhow::bail!("State file does not exist");
        }
        
        let json = fs::read_to_string(&self.state_file)?;
        let state: SessionState = serde_json::from_str(&json)?;
        
        tracing::info!("Loaded workspace state from {:?}", self.state_file);
        Ok(state)
    }
    
    /// Try to load state, fallback to backup if corrupted.
    ///
    /// This method first attempts to load the main state file. If that fails
    /// (due to corruption or parsing errors), it automatically tries to load
    /// from the backup file.
    ///
    /// # Returns
    ///
    /// `Ok(SessionState)` on success, or an error if both the main and backup
    /// files are unavailable or corrupted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    ///
    /// let manager = PersistenceManager::new();
    /// match manager.load_state_with_fallback() {
    ///     Ok(state) => println!("Loaded state successfully"),
    ///     Err(e) => eprintln!("Failed to load state even from backup: {}", e),
    /// }
    /// ```
    pub fn load_state_with_fallback(&self) -> anyhow::Result<SessionState> {
        match self.load_state() {
            Ok(state) => Ok(state),
            Err(e) => {
                tracing::warn!("Failed to load state: {}. Trying backup...", e);
                
                if self.backup_file.exists() {
                    let json = fs::read_to_string(&self.backup_file)?;
                    let state: SessionState = serde_json::from_str(&json)?;
                    
                    tracing::info!("Loaded workspace state from backup");
                    Ok(state)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Clear saved state files.
    ///
    /// Removes both the main state file and the backup file from disk.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if file removal fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    ///
    /// let manager = PersistenceManager::new();
    /// manager.clear_state().unwrap();
    /// ```
    pub fn clear_state(&self) -> anyhow::Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }
        if self.backup_file.exists() {
            fs::remove_file(&self.backup_file)?;
        }
        
        tracing::info!("Cleared workspace state");
        Ok(())
    }
    
    /// Check if state file exists.
    ///
    /// Returns true if either the main state file or the backup file exists.
    ///
    /// # Returns
    ///
    /// `true` if saved state is available, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tenraku_core::workspace::persistence::PersistenceManager;
    ///
    /// let manager = PersistenceManager::new();
    /// if manager.has_saved_state() {
    ///     println!("Saved state is available");
    /// }
    /// ```
    pub fn has_saved_state(&self) -> bool {
        self.state_file.exists() || self.backup_file.exists()
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        Self::new()
    }
}