use anyhow::Result;
use colored::*;
use log::debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use crate::ai::AiClient;
use crate::commands::show_confirm;
use crate::config::{Cache, CommitMsg, get_now_timestamp};
use crate::git::{add_all_files_to_git, execute_commit_with_cli, get_staged_diff, get_unstaged_diff};

pub async fn handle_commit(add: bool, generate_only: bool, output_file: Option<&Path>) -> Result<()> {
    let ai_client = AiClient::new();

    if add {
        add_all_files_to_git()?;
    }
    let staged_diff = get_staged_diff(&ai_client.config.commit)?;
    let unstaged_diff = get_unstaged_diff(&ai_client.config.commit)?;

    let (diff_content, is_dry_run) = if !staged_diff.is_empty() {
        if !generate_only {
            println!("{}", "✅ Staged changes found. Generating commit message...".green());
        }
        (staged_diff, false)
    } else if !unstaged_diff.is_empty() {
        if !generate_only {
            println!("{}", "⚠️ No staged changes found, but found unstaged changes.".yellow());
        }
        if !generate_only {
            println!("{}", "Running in dry-run mode (no actual commit will be made).".yellow());
        }
        if !generate_only {
            println!("{}", "To commit these changes, please stage them first with 'git add'.".yellow());
        }
        (unstaged_diff, true)
    } else {
        println!("{}", "❌ No changes found to commit.".red());
        return Ok(());
    };

    if !generate_only {
        println!("{}", "🤖 Generating commit message using AI service...".cyan());
    }

    let mut hasher = DefaultHasher::new();
    diff_content.hash(&mut hasher);
    let diff_content_hash = hasher.finish();
    let mut cache = Cache::load()?;
    let mut message = if let Some(msg) = cache.get_commit_message(diff_content_hash) {
        println!("Cache hit: {}", msg.get_msg().bright_green().bold());
        if show_confirm("Do you want to regenerate this commit message?")? { "".to_string() } else { msg.get_msg() }
    } else {
        "".to_string()
    };

    if message.is_empty() {
        match ai_client.generate_commit_message(&diff_content).await {
            Ok(msg) => {
                if msg.is_empty() {
                    println!("{}", "❌ AI did not generate a commit message.".red());
                    return Ok(());
                }

                debug!("save commit message: {} -> {}", diff_content_hash, msg);
                let now = get_now_timestamp()?;
                let commit_msg = CommitMsg::new(diff_content_hash, msg.clone(), now);
                cache.store_commit_message(commit_msg)?;

                message = msg;
            }
            Err(e) => {
                println!("{}", format!("❌ Failed to generate commit message: {e}").red());
                return Ok(());
            }
        };
    };

    // Handle output based on parameters
    if let Some(output_path) = output_file {
        // Write message to file
        std::fs::write(output_path, &message)?;
        if !generate_only {
            println!("{}", format!("Commit message written to: {}", output_path.display()).green());
        }
    } else if generate_only {
        // Generate-only mode: output only the message
        println!("{}", message.bright_green().bold());
    } else {
        // Normal verbose output
        println!("{}", "✨ Generated commit message:".bright_cyan().bold());
        println!("{}", "─────────────────────".bright_blue());
        println!("{}", message.bright_green().bold());
        println!("{}", "─────────────────────".bright_blue());

        if is_dry_run {
            println!("{}", "(Dry run mode - no actual commit made)".yellow());
            println!("{}", "To commit these changes:".yellow());
            println!("{}", "1. Stage your changes: git add <files>".yellow());
            println!("{}", "2. Run ai-commit again".yellow());
        } else {
            // Only ask for confirmation if not in generate-only mode
            // (generate-only is handled above, but keep this for safety)
            if !ai_client.config.commit.auto_confirm && !confirm_commit()? {
                println!("{}", "❌ Commit cancelled.".red());
                return Ok(());
            }
            execute_commit_with_cli(&message)?;
        }
    }

    Ok(())
}

fn confirm_commit() -> Result<bool> {
    show_confirm("Do you want to commit with this message?")
}
