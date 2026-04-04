use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    commit_msgs: HashMap<String, String>,
}

impl Cache {
    pub fn cache_file_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("Could not find HOME directory"))?;
        Ok(PathBuf::from(home).join(".config").join("ai-commit").join(".cache"))
    }

    pub fn load() -> Result<Self> {
        let cache_file_path = Self::cache_file_path()?;
        debug!("Loading cache commits message from: {}", cache_file_path.display());
        if cache_file_path.exists() {
            let cache_content = fs::read_to_string(&cache_file_path)?;
            let cache: Cache = toml::from_str(&cache_content)?;
            Ok(cache)
        } else {
            let default_cache = toml::to_string_pretty(&Self::default())?;
            let default_cache: Cache = toml::from_str(&default_cache)?;
            default_cache.save()?;
            Ok(default_cache)
        }
    }

    pub fn save(&self) -> Result<()> {
        let cache_file_path = Self::cache_file_path()?;

        if let Some(parent) = cache_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let cache_content = toml::to_string_pretty(self)?;
        fs::write(&cache_file_path, cache_content)?;
        Ok(())
    }

    pub fn get_commit_message(&self, diff_content_hash: u64) -> Option<String> {
        self.commit_msgs.get(&diff_content_hash.to_string()).cloned()
    }

    pub fn store_commit_message(&mut self, diff_content_hash: u64, message: String) {
        self.commit_msgs.insert(diff_content_hash.to_string(), message);
    }

    pub fn delete_commit_message(&mut self, diff_content_hash: u64) {
        self.commit_msgs.remove(&diff_content_hash.to_string());
    }
}
