use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Command,
    Executable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub name: String,
    pub item_type: ItemType,
    pub timestamp: i64,
}

pub struct RecentItems {
    entries: Vec<RecentEntry>,
    file_path: PathBuf,
    max_items: usize,
}

impl RecentItems {
    pub fn new() -> Result<Self> {
        let file_path = Self::get_recent_file_path()?;
        let mut recent = Self {
            entries: Vec::new(),
            file_path,
            max_items: 20,
        };
        recent.load_from_json()?;
        Ok(recent)
    }

    fn get_recent_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("tenraku");

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir.join("recent.json"))
    }

    fn load_from_json(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&self.file_path).context("Failed to read recent items file")?;

        self.entries =
            serde_json::from_str(&content).context("Failed to parse recent items JSON")?;

        Ok(())
    }

    pub fn save_to_json(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize recent items")?;

        fs::write(&self.file_path, json).context("Failed to write recent items file")?;

        Ok(())
    }

    pub fn add(&mut self, name: String, item_type: ItemType) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Remove existing entry with same name if present
        self.entries.retain(|e| e.name != name);

        // Add new entry at the beginning
        self.entries.insert(
            0,
            RecentEntry {
                name,
                item_type,
                timestamp,
            },
        );

        // Keep only max_items
        if self.entries.len() > self.max_items {
            self.entries.truncate(self.max_items);
        }
    }

    pub fn get_recent(&self, limit: usize) -> Vec<RecentEntry> {
        self.entries.iter().take(limit).cloned().collect()
    }
}
