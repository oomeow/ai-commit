use std::fs;

use anyhow::Result;
use colored::*;

use crate::git::open_repo;

pub fn install_hook() -> Result<()> {
    println!("Installing AI-assisted Git hooks...");

    let repo = open_repo();
    let hooks_dir = repo.path().join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");
    let hook_content = include_str!("../../templates/prepare-commit-msg");

    if hook_path.exists() {
        let is_ai_commit_hook = fs::read_to_string(&hook_path).is_ok_and(|content| content.contains("ai-commit"));

        if is_ai_commit_hook {
            println!("{}", "ℹ️  prepare-commit-msg hook already exists (ai-commit version). Overwriting.".yellow());
        } else {
            println!(
                "{}",
                "⚠️  prepare-commit-msg hook already exists and doesn't appear to be an ai-commit hook. Skipping to avoid overwriting.".yellow()
            );
            return Ok(());
        }
    }

    fs::write(&hook_path, hook_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    println!("{}", "✅ Installed prepare-commit-msg hook".green());

    Ok(())
}
