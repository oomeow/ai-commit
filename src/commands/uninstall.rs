use anyhow::Result;
use colored::*;
use log::debug;
use std::fs;
use std::path::Path;

pub fn uninstall_hook() -> Result<()> {
    println!("Uninstalling AI-assisted Git hooks...");

    // List of hook files that ai-commit might have installed
    let hook_files = ["prepare-commit-msg"];
    let hooks_dir = Path::new(".git/hooks");

    let mut removed_count = 0;

    for hook_file in &hook_files {
        let hook_path = hooks_dir.join(hook_file);

        if hook_path.exists() {
            // Optional: check if the file is actually an ai-commit hook
            // by checking if it contains "ai-commit" in its content
            let is_ai_commit_hook = match fs::read_to_string(&hook_path) {
                Ok(content) => content.contains("ai-commit"),
                Err(_) => false, // If we can't read it, assume it's not ours
            };

            if is_ai_commit_hook {
                fs::remove_file(&hook_path)?;
                println!("{}", format!("✅ Removed {hook_file} hook").green());
                removed_count += 1;
            } else {
                println!(
                    "{}",
                    format!("⚠️  {hook_file} hook exists but doesn't appear to be an ai-commit hook. Skipping.")
                        .yellow()
                );
            }
        } else {
            println!("{}", format!("ℹ️  {hook_file} hook not found. Skipping.").blue());
        }
    }

    if removed_count > 0 {
        debug!("Uninstallation complete. Removed {} hook file(s).", removed_count);
    } else {
        debug!("No ai-commit hooks found to uninstall.");
    }

    Ok(())
}
