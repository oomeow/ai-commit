use crate::{commands::show_confirm, config::AppConfig, dirs::get_config_file_path};
use anyhow::Result;
use colored::*;

pub fn init_config() -> Result<()> {
    println!("{}", "🔧 Initializing AI Commit configuration...".cyan());

    let config_path = get_config_file_path()?;

    if config_path.exists() {
        println!("{}", "⚠️  Configuration file already exists.".yellow());
        println!("Location: {}", config_path.display().to_string().bright_blue());
        if !show_confirm("Do you want to overwrite it?", true)? {
            println!("{}", "❌ Configuration initialization cancelled.".red());
            return Ok(());
        }
    }

    AppConfig::init()?;

    println!("{}", "✅ Configuration file created successfully!".green());
    println!("Location: {}", config_path.display().to_string().bright_blue());
    println!();
    println!("{}", "📝 Next steps:".bright_cyan().bold());
    println!("1. Edit API key: Edit the [api] section in the config file");
    println!("2. Edit prompts: Edit the [prompts] section in the config file");
    println!("3. Start using: {}", "ai-commit".yellow());

    Ok(())
}

pub fn show_config() -> Result<()> {
    println!("{}", "📋 Current AI Commit Configuration".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════".bright_blue());

    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        println!("Run {} to create it.", "ai-commit config init".yellow());
        return Ok(());
    }

    let config = AppConfig::load()?;
    let config_content = toml::to_string_pretty(&config)?;

    println!("File location: {}", config_path.display().to_string().bright_blue());
    println!();
    println!("{config_content}");

    Ok(())
}

pub fn edit_prompts_help() -> Result<()> {
    let config_path = get_config_file_path()?;

    println!("{}", "✏️  How to Edit AI Prompts".bright_cyan().bold());
    println!("{}", "═══════════════════════════".bright_blue());
    println!();
    println!("Configuration file location:");
    println!("{}", config_path.display().to_string().bright_blue());
    println!();
    println!("{}", "📝 Editable prompt sections:".bright_green().bold());
    println!();
    println!("{}", "[prompts.system_prompt]".yellow());
    println!("  • Defines AI behavior and commit format preferences");
    println!("  • Sets the overall style and rules for commit messages");
    println!();
    println!("{}", "[prompts.user_prompt_template]".yellow());
    println!("  • Template for analyzing git diffs");
    println!("  • Use {{diff}} as placeholder for the git diff content");
    println!("  • Controls how AI analyzes changes");
    println!();
    println!("{}", "[prompts.simple_prompt_template]".yellow());
    println!("  • Template for generating simple single-line messages");
    println!("  • Use {{diff}} as placeholder");
    println!("  • Used for straightforward changes");
    println!();
    println!("{}", "💡 Tips:".bright_green().bold());
    println!("  • Test changes with: ai-commit --dry-run");
    println!("  • Keep {{diff}} placeholder in templates");
    println!("  • Reload happens automatically on next run");
    println!("  • Back up your custom prompts before updates");

    if !config_path.exists() {
        println!();
        println!("{}", "⚠️  Configuration file not found.".yellow());
        println!("Run {} to create it first.", "ai-commit config init".cyan());
    }

    Ok(())
}
