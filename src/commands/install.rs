use anyhow::{Context, Result};
use colored::*;
use log::debug;
use std::fs;
use std::path::Path;

pub fn install_hook() -> Result<()> {
    println!("Installing AI-assisted Git hooks...");

    let hook_files = ["prepare-commit-msg"];
    let hooks_dir = Path::new(".git/hooks");

    let mut installed_count = 0;
    let mut skipped_count = 0;

    for hook_type in &hook_files {
        let template_path = Path::new("templates").join(hook_type);
        let hook_path = hooks_dir.join(hook_type);

        // Check if template exists
        if !template_path.exists() {
            println!(
                "{}",
                format!("⚠️  Template for {} hook not found at {}. Skipping.", hook_type, template_path.display())
                    .yellow()
            );
            skipped_count += 1;
            continue;
        }

        // Check if hook already exists
        if hook_path.exists() {
            // Check if it's an ai-commit hook by looking for "ai-commit" in the content
            let is_ai_commit_hook = match fs::read_to_string(&hook_path) {
                Ok(content) => content.contains("ai-commit"),
                Err(_) => false, // If we can't read it, assume it's not ours
            };

            if is_ai_commit_hook {
                // It's our hook, we can safely overwrite it
                println!(
                    "{}",
                    format!("ℹ️  {hook_type} hook already exists (ai-commit version). Overwriting.").yellow()
                );
            } else {
                // It's not our hook, skip it to avoid overwriting user's custom hook
                println!("{}",
                    format!("⚠️  {hook_type} hook already exists and doesn't appear to be an ai-commit hook. Skipping to avoid overwriting.").yellow()
                );
                skipped_count += 1;
                continue;
            }
        }

        // Copy template to hook location
        fs::copy(&template_path, &hook_path).with_context(|| format!("Failed to copy {hook_type} hook template"))?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        println!("{}", format!("✅ Installed {hook_type} hook").green());
        installed_count += 1;
    }

    if installed_count > 0 {
        debug!("Installation complete. Installed {installed_count} hook file(s).");
        if skipped_count > 0 {
            debug!("Skipped {skipped_count} hook file(s) (templates not found or existing non-ai-commit hooks).");
        }
    } else {
        println!("{}", "No hooks were installed.".yellow());
        if skipped_count > 0 {
            debug!(
                "All {skipped_count} hook file(s) were skipped (templates not found or existing non-ai-commit hooks)."
            );
        }
    }

    Ok(())
}
