mod post_commit;
mod prepare_commit_msg;

use std::{
    path::{Path, PathBuf},
    process::Command,
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

/// Build a command that runs the given hook the way Git would on this platform.
///
/// On Windows a hook is often a POSIX shell script (e.g. one managed by husky),
/// which cannot be launched directly: `CreateProcess` rejects it with
/// `os error 193` ("not a valid Win32 application"). Git for Windows runs such
/// hooks through its bundled `sh.exe`, so we do the same when the hook declares
/// a shell interpreter in its shebang line.
pub fn create_hook_command(hook_path: &Path) -> Command {
    #[cfg(windows)]
    if let Some(shell) = windows_shell_for(hook_path) {
        let mut cmd = Command::new(shell);
        cmd.arg(hook_path);
        return cmd;
    }

    Command::new(hook_path)
}

/// Resolve the shell that should run a shebang hook, preferring the interpreter
/// shipped with Git for Windows and falling back to `PATH` lookup by name.
#[cfg(windows)]
fn windows_shell_for(hook_path: &Path) -> Option<PathBuf> {
    let interpreter = shebang_interpreter(hook_path)?;
    let executable = format!("{interpreter}.exe");

    if let Some(root) = git_install_root() {
        for dir in ["bin", "usr/bin"] {
            let candidate = root.join(dir).join(&executable);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    Some(PathBuf::from(interpreter))
}

/// Parse the interpreter name from a hook's shebang line, e.g.
/// `#!/usr/bin/env sh` and `#!/bin/bash` yield `sh` and `bash`.
#[cfg(windows)]
fn shebang_interpreter(hook_path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(hook_path).ok()?;
    let mut first_line = String::new();
    BufReader::new(file).read_line(&mut first_line).ok()?;

    let rest = first_line.trim().strip_prefix("#!")?;
    let mut tokens = rest.split_whitespace();
    let program = tokens.next()?;

    let name = if Path::new(program).file_name()?.to_str()? == "env" {
        tokens.find(|token| !token.starts_with('-'))?
    } else {
        program
    };

    Some(Path::new(name).file_name()?.to_str()?.to_string())
}

/// Resolve the Git installation root from `git --exec-path`
/// (e.g. `<root>/mingw64/libexec/git-core` -> `<root>`).
#[cfg(windows)]
fn git_install_root() -> Option<PathBuf> {
    let output = Command::new("git").arg("--exec-path").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let exec_path = String::from_utf8_lossy(&output.stdout);
    let exec_path = Path::new(exec_path.trim());
    exec_path.ancestors().nth(3).map(Path::to_path_buf)
}

pub fn is_ai_commit_hook(hook: &Path) -> bool {
    std::fs::read_to_string(hook).is_ok_and(|content| content.contains("ai-commit"))
}
