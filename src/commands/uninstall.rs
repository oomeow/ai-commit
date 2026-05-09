use std::fs;

use anyhow::Result;
use colored::*;

use crate::git::open_repo;

pub fn uninstall_hook() -> Result<()> {
    println!("Uninstalling AI-assisted Git hooks...");

    let repo = open_repo();
    let hooks_dir = repo.path().join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");

    if hook_path.exists() {
        let is_ai_commit_hook = fs::read_to_string(&hook_path).is_ok_and(|content| content.contains("ai-commit"));

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
