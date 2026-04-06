use std::fs;

use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::dirs::get_cache_file_path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitMsg {
    hash: u64,
    commit_msg: String,
    timestamp: u64,
}

impl CommitMsg {
    pub fn new(hash: u64, commit_msg: String, timestamp: u64) -> Self {
        Self { hash, commit_msg, timestamp }
    }

    pub fn is_expired(&self, now: u64, expiry_seconds: u64) -> bool {
        let elapsed = now - self.timestamp;
        debug!("hash: {}, elapsed seconds: {}", self.hash, elapsed);
        elapsed > expiry_seconds
    }

    pub fn get_msg(&self) -> String {
        self.commit_msg.clone()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    commit_msgs: Vec<CommitMsg>,
}

impl Cache {
    pub fn load() -> Result<Self> {
        let cache_file_path = get_cache_file_path()?;
        debug!("Loading cache commits message from: {}", cache_file_path.display());
        if cache_file_path.exists() {
            let cache_content = fs::read_to_string(&cache_file_path)?;
            let mut cache: Cache = toml::from_str(&cache_content)?;

            // Remove expired commit messages (Default 7 days)
            // TODO: make expiry seconds configurable
            let now = get_now_timestamp()?;
            let expiry_seconds = 60 * 60 * 24 * 7;
            cache.commit_msgs.retain(|m| !m.is_expired(now, expiry_seconds));
            cache.save()?;

            Ok(cache)
        } else {
            let default_cache = toml::to_string_pretty(&Self::default())?;
            let default_cache: Cache = toml::from_str(&default_cache)?;
            default_cache.save()?;
            Ok(default_cache)
        }
    }

    pub fn save(&self) -> Result<()> {
        let cache_file_path = get_cache_file_path()?;

        if let Some(parent) = cache_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let cache_content = toml::to_string_pretty(self)?;
        fs::write(&cache_file_path, cache_content)?;
        Ok(())
    }

    pub fn get_commit_message(&self, hash: u64) -> Option<&CommitMsg> {
        self.commit_msgs.iter().find(|&m| m.hash == hash)
    }

    pub fn store_commit_message(&mut self, commit_msg: CommitMsg) -> Result<()> {
        self.commit_msgs.retain(|m| m.hash != commit_msg.hash);
        self.commit_msgs.push(commit_msg);
        self.save()
    }
}

/// Returns the current timestamp in seconds since the Unix epoch.
pub fn get_now_timestamp() -> Result<u64> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs())?;
    Ok(now)
}
