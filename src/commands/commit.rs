use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use anyhow::Result;
use colored::*;
use dialoguer::Editor;
use log::debug;

use crate::{
    ai::AiClient,
    commands::show_confirm,
    config::{AppConfig, Cache, CommitMsg, get_now_timestamp},
    git::{execute_commit_with_cli, get_staged_diff, get_unstaged_diff},
};

pub async fn handle_commit(
    generate_only: bool,
    custom_config_file: Option<&PathBuf>,
    output_file: Option<&PathBuf>,
) -> Result<()> {
    let ai_client = match custom_config_file {
        Some(config_path) => AiClient::with_config(AppConfig::load_from_path(config_path)?),
        None => AiClient::new(),
    };

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

    let (mut message, should_print_generate_message) =
        if let Some(cached_msg) = cache.get_commit_message(diff_content_hash) {
            if generate_only {
                (cached_msg.get_msg(), false)
            } else {
                println!("{}", "♻️ Reusing cached commit message:".bright_cyan().bold());
                println!("{}", "─────────────────────".bright_blue());
                println!("{}", cached_msg.get_msg().bright_green().bold());
                println!("{}", "─────────────────────".bright_blue());
                if show_confirm("Do you want to regenerate this commit message?", false)? {
                    let Some(message) =
                        generate_and_cache_message(&ai_client, &diff_content, diff_content_hash, &mut cache).await?
                    else {
                        return Ok(());
                    };
                    (message, true)
                } else {
                    (cached_msg.get_msg(), false)
                }
            }
        } else {
            let Some(message) =
                generate_and_cache_message(&ai_client, &diff_content, diff_content_hash, &mut cache).await?
            else {
                return Ok(());
            };
            (message, true)
        };

    if let Some(output_path) = output_file {
        std::fs::write(output_path, &message)?;
        if !generate_only {
            println!("{}", format!("Commit message written to: {}", output_path.display()).green());
        }
        return Ok(());
    }

    if generate_only {
        println!("{}", message.bright_green().bold());
        return Ok(());
    }

    if should_print_generate_message {
        println!("{}", "✨ Generated commit message:".bright_cyan().bold());
        println!("{}", "─────────────────────".bright_blue());
        println!("{}", message.bright_green().bold());
        println!("{}", "─────────────────────".bright_blue());
    }

    if is_dry_run {
        println!("{}", "(Dry run mode - no actual commit made)".yellow());
        println!("{}", "To commit these changes:".yellow());
        println!("{}", "1. Stage your changes: git add <files>".yellow());
        println!("{}", "2. Run ai-commit again".yellow());
        return Ok(());
    }

    if !ai_client.config.commit.auto_confirm {
        if confirm_edit_message()?
            && let Some(edited) = Editor::new().edit(&message)?
        {
            message = edited.clone();
            println!("{}", "✍️ Edited commit message:".bright_cyan().bold());
            println!("{}", "─────────────────────".bright_blue());
            println!("{}", message.bright_green().bold());
            println!("{}", "─────────────────────".bright_blue());

            let commit_msg = CommitMsg::new(diff_content_hash, edited, get_now_timestamp()?);
            cache.store_commit_message(commit_msg)?;
        }

        if !confirm_commit()? {
            println!("{}", "❌ Commit cancelled.".red());
            return Ok(());
        }
    }

    execute_commit_with_cli(&message)?;

    Ok(())
}

async fn generate_and_cache_message(
    ai_client: &AiClient,
    diff_content: &str,
    diff_content_hash: u64,
    cache: &mut Cache,
) -> Result<Option<String>> {
    let msg = match ai_client.generate_commit_message(diff_content).await {
        Ok(msg) => msg,
        Err(e) => {
            println!("{}", format!("❌ Failed to generate commit message: {e}").red());
            return Ok(None);
        }
    };

    if msg.is_empty() {
        println!("{}", "❌ AI did not generate a commit message.".red());
        return Ok(None);
    }

    debug!("save commit message: {} -> {}", diff_content_hash, msg);
    let commit_msg = CommitMsg::new(diff_content_hash, msg.clone(), get_now_timestamp()?);
    cache.store_commit_message(commit_msg)?;

    Ok(Some(msg))
}

fn confirm_edit_message() -> Result<bool> {
    show_confirm("Do you want to edit the commit message?", false)
}

fn confirm_commit() -> Result<bool> {
    show_confirm("Do you want to commit with this message?", true)
}
