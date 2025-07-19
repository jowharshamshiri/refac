use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapMetadata {
    pub version: u32,
    pub entries: HashMap<String, ScrapEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapEntry {
    pub original_path: PathBuf,
    pub scrapped_at: DateTime<Utc>,
    pub scrapped_name: String,
}

impl ScrapMetadata {
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: HashMap::new(),
        }
    }

    pub fn load(scrap_dir: &Path) -> Result<Self> {
        let metadata_path = scrap_dir.join(".metadata.json");
        if !metadata_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&metadata_path)
            .context("Failed to read metadata file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse metadata file")
    }

    pub fn save(&self, scrap_dir: &Path) -> Result<()> {
        let metadata_path = scrap_dir.join(".metadata.json");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize metadata")?;
        
        fs::write(&metadata_path, content)
            .context("Failed to write metadata file")?;
        
        Ok(())
    }

    pub fn add_entry(&mut self, scrapped_name: &str, original_path: PathBuf) {
        self.entries.insert(
            scrapped_name.to_string(),
            ScrapEntry {
                original_path,
                scrapped_at: Utc::now(),
                scrapped_name: scrapped_name.to_string(),
            },
        );
    }

    pub fn remove_entry(&mut self, scrapped_name: &str) -> Option<ScrapEntry> {
        self.entries.remove(scrapped_name)
    }

    pub fn get_entry(&self, scrapped_name: &str) -> Option<&ScrapEntry> {
        self.entries.get(scrapped_name)
    }
}