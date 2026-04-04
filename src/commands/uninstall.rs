use anyhow::Result;
use colored::*;
use git2::Repository;
use std::fs;

pub fn uninstall_hook() -> Result<()> {
    println!("Uninstalling AI-assisted Git hooks...");

    let repo = Repository::open_from_env().unwrap_or_else(|e| {
        eprintln!("Failed to open git repository. Make sure you're in a git repository: {e}");
        std::process::exit(1);
    });
    let hooks_dir = repo.path().join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");

    if hook_path.exists() {
        // Check if it's an ai-commit hook by looking for "ai-commit" in the content
        let is_ai_commit_hook = match fs::read_to_string(&hook_path) {
            Ok(content) => content.contains("ai-commit"),
            Err(_) => false, // If we can't read it, assume it's not ours
        };

        if is_ai_commit_hook {
            fs::remove_file(&hook_path)?;
            println!("{}", "✅ Removed prepare-commit-msg hook".green());
        } else {
            println!(
                "{}",
                "⚠️  prepare-commit-msg hook exists but doesn't appear to be an ai-commit hook. Skipping.".yellow()
            );
        }
    } else {
        println!("{}", "ℹ️  prepare-commit-msg hook not found. Skipping.".blue());
    }

    Ok(())
}
