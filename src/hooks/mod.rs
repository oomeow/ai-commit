mod post_commit;
mod prepare_commit_msg;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use git2::Repository;
pub use post_commit::*;
pub use prepare_commit_msg::*;

pub const HOOK_PRE_COMMIT: &str = "pre-commit";

const CONFIG_HOOKS_PATH: &str = "core.hooksPath";
const DEFAULT_HOOKS_PATH: &str = "hooks";

pub fn find_available_hook(repo: &Repository, hook: &str) -> Result<Option<PathBuf>> {
    Ok(get_hook_path(repo, hook)?.filter(|hook_path| is_executable(hook_path)))
}

fn get_hook_path(repo: &Repository, hook: &str) -> Result<Option<PathBuf>> {
    let pwd = repo.workdir().unwrap_or_else(|| repo.path()).to_path_buf();

    if let Ok(config_path) = repo.config()?.get_string(CONFIG_HOOKS_PATH) {
        let hooks_path = PathBuf::from(config_path);
        let config_hook_path = expand_hook_path(&pwd, &hooks_path.join(hook))?;
        if config_hook_path.exists() {
            return Ok(Some(config_hook_path));
        }
    }

    let default_path = repo.path().to_path_buf().join(DEFAULT_HOOKS_PATH).join(hook);
    if default_path.exists() {
        return Ok(Some(default_path));
    }

    Ok(None)
}

/// Expand path according to the rule of githooks and config `core.hooksPath`
fn expand_hook_path(pwd: &Path, path: &Path) -> Result<PathBuf> {
    let path = path.as_os_str().to_str().context("fail to translate hook path to str")?;
    let hook_expanded = shellexpand::full(path)?;
    let hook_expanded = PathBuf::from_str(hook_expanded.as_ref())?;
    let path = { if hook_expanded.is_absolute() { hook_expanded } else { pwd.join(hook_expanded) } };
    Ok(path)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            log::error!("metadata error: {e}");
            return false;
        }
    };

    let permissions = metadata.permissions();

    permissions.mode() & 0o111 != 0
}

#[cfg(windows)]
/// windows does not consider shell scripts to be executable so we consider everything
/// to be executable (which is not far from the truth for windows platform.)
const fn is_executable(_: &Path) -> bool {
    true
}

pub fn is_ai_commit_hook(hook: &Path) -> bool {
    std::fs::read_to_string(hook).is_ok_and(|content| content.contains("ai-commit"))
}
