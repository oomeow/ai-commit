use std::path::PathBuf;

use anyhow::Result;
use colored::*;

use crate::{
    ai::AiClient,
    commands::show_confirm,
    config::AppConfig,
    git::{execute_amend_with_cli, get_amend_diff, get_last_commit_message, get_staged_diff},
};

pub async fn handle_amend(custom_config: Option<&PathBuf>, dry_run: bool) -> Result<()> {
    let ai_client = match custom_config {
        Some(config_path) => AiClient::with_config(AppConfig::load_from_path(config_path)?),
        None => AiClient::new(),
    };

    let staged_diff = get_staged_diff(&ai_client.config.commit)?;
    let amend_diff = get_amend_diff(&ai_client.config.commit)?;
    if !staged_diff.is_empty() {
        println!("{}", "✅ Found staged changes to amend.".green());
    } else {
        println!("{}", "⚠️  No staged changes found.".yellow());
        println!("{}", "Will generate new message for existing commit content.".yellow());
    }

    if amend_diff.is_empty() {
        println!("{}", "❌ No changes found to amend.".red());
        return Ok(());
    }

    let last_commit_msg = get_last_commit_message()?;
    println!("{}", "📝 Current commit message:".bright_blue().bold());
    println!("{}", "─────────────────────".bright_blue());
    println!("{}", last_commit_msg.trim().bright_yellow());
    println!("{}", "─────────────────────".bright_blue());

    println!("{}", "🤖 Generating new commit message using AI service...".cyan());

    let message = match ai_client.generate_commit_message(&amend_diff).await {
        Ok(message) => message,
        Err(e) => {
            println!("{}", format!("❌ Failed to generate commit message: {e}").red());
            return Ok(());
        }
    };

    if message.is_empty() {
        println!("{}", "❌ AI did not generate a commit message.".red());
        return Ok(());
    }

    println!("{}", "✨ Generated new commit message:".bright_cyan().bold());
    println!("{}", "─────────────────────".bright_blue());
    println!("{}", message.bright_green().bold());
    println!("{}", "─────────────────────".bright_blue());

    if dry_run {
        println!("{}", "(Dry run mode - no actual amend made)".yellow());
        println!("{}", "To amend: git commit --amend -m \"<message>\"".yellow());
        return Ok(());
    }

    if !ai_client.config.commit.auto_confirm && !confirm_amend()? {
        println!("{}", "❌ Amend cancelled.".red());
        return Ok(());
    }

    execute_amend_with_cli(&message)?;

    Ok(())
}

fn confirm_amend() -> Result<bool> {
    show_confirm("Do you want to amend the commit with this message?", true)
}
