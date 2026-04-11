use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use anyhow::Result;
use colored::*;
use log::debug;

use crate::{
    ai::AiClient,
    commands::show_confirm,
    config::{AppConfig, Cache, CommitMsg, get_now_timestamp},
    git::{add_all_files_to_git, execute_commit_with_cli, get_staged_diff, get_unstaged_diff},
};

pub async fn handle_commit(
    add: bool,
    generate_only: bool,
    custom_config_file: Option<&PathBuf>,
    output_file: Option<&PathBuf>,
) -> Result<()> {
    let ai_client = if let Some(config_path) = custom_config_file {
        let config = AppConfig::load_from_path(config_path)?;
        AiClient::with_config(config)
    } else {
        AiClient::new()
    };

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
            println!("{}", "Running in dry-run mode (no actual commit will be made).".yellow());
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
    let generate_msg = async |cache: &mut Cache| -> Result<String> {
        match ai_client.generate_commit_message(&diff_content).await {
            Ok(msg) => {
                if msg.is_empty() {
                    println!("{}", "❌ AI did not generate a commit message.".red());
                    std::process::exit(0);
                }
                debug!("save commit message: {} -> {}", diff_content_hash, msg);
                let now = get_now_timestamp()?;
                let commit_msg = CommitMsg::new(diff_content_hash, msg.clone(), now);
                cache.store_commit_message(commit_msg)?;
                Ok(msg)
            }
            Err(e) => {
                println!("{}", format!("❌ Failed to generate commit message: {e}").red());
                std::process::exit(0);
            }
        }
    };
    let message = if let Some(cached_msg) = cache.get_commit_message(diff_content_hash) {
        if generate_only {
            cached_msg.get_msg()
        } else {
            println!("Cache hit: {}", cached_msg.get_msg().bright_green().bold());
            if show_confirm("Do you want to regenerate this commit message?", false)? {
                generate_msg(&mut cache).await?
            } else {
                cached_msg.get_msg()
            }
        }
    } else {
        generate_msg(&mut cache).await?
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
    show_confirm("Do you want to commit with this message?", true)
}
