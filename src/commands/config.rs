use crate::{
    ai::{AiClient, provider_names},
    commands::show_confirm,
    config::AppConfig,
    dirs::get_config_file_path,
};
use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, FuzzySelect, Input, Password, Select, theme::ColorfulTheme};

pub async fn init_config() -> Result<()> {
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

    let theme = ColorfulTheme::default();
    let mut config = AppConfig::default_config()?;

    let provider_names = provider_names();
    let provider_index =
        Select::with_theme(&theme).with_prompt("Select an AI provider").items(provider_names).default(0).interact()?;
    let provider_name = provider_names[provider_index];

    let api_key = Password::with_theme(&theme)
        .with_prompt(format!("Enter API key for {provider_name}"))
        .allow_empty_password(true)
        .interact()?;

    config.api.provider = provider_name.to_string();
    config.api.api_key = if api_key.is_empty() { None } else { Some(api_key) };

    config.api.model = select_model(&theme, &config).await?;

    println!();
    println!("{}", "Configuration preview:".bright_cyan().bold());
    println!("Provider: {}", config.api.provider.bright_green());
    println!("Model: {}", config.api.model.bright_green());
    println!("Config file: {}", config_path.display().to_string().bright_blue());

    if !Confirm::with_theme(&theme).with_prompt("Save this configuration?").default(true).interact()? {
        println!("{}", "❌ Configuration initialization cancelled.".red());
        return Ok(());
    }

    config.save()?;

    println!("{}", "✅ Configuration file created successfully!".green());
    println!("Location: {}", config_path.display().to_string().bright_blue());
    println!();
    println!("{}", "📝 Next steps:".bright_cyan().bold());
    println!("1. Review config: {}", "ai-commit config show".yellow());
    println!("2. Edit prompts if needed: {}", "ai-commit config edit-prompts".yellow());
    println!("3. Start using: {}", "ai-commit".yellow());

    Ok(())
}

async fn select_model(theme: &ColorfulTheme, config: &AppConfig) -> Result<String> {
    match AiClient::with_config(config.clone()).fetch_available_models().await {
        Ok(models) if !models.is_empty() => {
            let mut models = models;
            models.sort();

            let model_refs: Vec<&str> = models.iter().map(String::as_str).collect();
            let model_index = FuzzySelect::with_theme(theme)
                .with_prompt("Select a model")
                .items(&model_refs)
                .default(0)
                .max_length(12)
                .interact()?;
            Ok(models[model_index].clone())
        }
        Ok(_) => prompt_model_fallback(theme, "No models were returned by the provider."),
        Err(error) => prompt_model_fallback(theme, &format!("Failed to fetch models: {error}")),
    }
}

fn prompt_model_fallback(theme: &ColorfulTheme, reason: &str) -> Result<String> {
    println!("{}", format!("⚠️  {reason}").yellow());

    let should_continue =
        Confirm::with_theme(theme).with_prompt("Do you want to enter a model manually?").default(true).interact()?;
    if !should_continue {
        return Err(anyhow::anyhow!("Configuration initialization cancelled."));
    }

    Ok(Input::with_theme(theme).with_prompt("Enter model name").interact_text()?)
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
