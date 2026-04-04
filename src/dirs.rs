use std::path::PathBuf;

use anyhow::Result;

pub fn get_work_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("Could not find HOME directory"))?;
    Ok(PathBuf::from(home).join(".config").join("ai-commit"))
}

pub fn init_work_dir() -> Result<()> {
    let work_dir = get_work_dir()?;
    if !work_dir.exists() {
        std::fs::create_dir_all(&work_dir).map_err(|e| anyhow::anyhow!("Failed to create work directory: {}", e))?;
    }
    Ok(())
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let work_dir = get_work_dir()?;
    Ok(work_dir.join("config.toml"))
}

pub fn get_cache_file_path() -> Result<PathBuf> {
    let work_dir = get_work_dir()?;
    Ok(work_dir.join(".cache"))
}
