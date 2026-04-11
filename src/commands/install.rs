use std::fs;

use anyhow::{Context, Result, anyhow};
use colored::*;
use git2::Repository;

pub fn install_hook() -> Result<()> {
    println!("Installing AI-assisted Git hooks...");

    let repo = Repository::open_from_env().unwrap_or_else(|e| {
        eprintln!("Failed to open git repository. Make sure you're in a git repository: {e}");
        std::process::exit(1);
    });
    let hooks_dir = repo.path().join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");
    let hook_content = include_str!("../../templates/prepare-commit-msg");

    if hook_path.exists() {
        // Check if it's an ai-commit hook by looking for "ai-commit" in the content
        let is_ai_commit_hook = match fs::read_to_string(&hook_path) {
            Ok(content) => content.contains("ai-commit"),
            Err(_) => false, // If we can't read it, assume it's not ours
        };

        if is_ai_commit_hook {
            // It's our hook, we can safely overwrite it
            println!("{}", "ℹ️  prepare-commit-msg hook already exists (ai-commit version). Overwriting.".yellow());
        } else {
            // It's not our hook, skip it to avoid overwriting user's custom hook
            println!("{}",
                    "⚠️  prepare-commit-msg hook already exists and doesn't appear to be an ai-commit hook. Skipping to avoid overwriting.".yellow()
                );
            return Ok(());
        }
    }

    // Write template to hook location
    fs::write(&hook_path, hook_content).with_context(|| anyhow!("Failed to write prepare-commit-msg hook template"))?;

    // Make executable on Unix systems
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
