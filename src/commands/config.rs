use std::{fs, path::PathBuf};

use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Editor, FuzzySelect, Input, Password, Select, theme::ColorfulTheme};

use crate::{
    ai::{AiClient, provider_specs},
    commands::{THEME, show_confirm},
    config::{ApiConfig, AppConfig},
    dirs::get_config_file_path,
};

fn resolve_config_path(custom_config_file: Option<&PathBuf>) -> Result<PathBuf> {
    match custom_config_file {
        Some(path) => Ok(path.clone()),
        None => get_config_file_path(),
    }
}

/// Load an existing config from the resolved path, or fall back to defaults when
/// the file does not exist yet.
fn load_or_default(config_path: &PathBuf) -> Result<AppConfig> {
    if config_path.exists() { AppConfig::load_from_path(config_path) } else { Ok(AppConfig::default()) }
}

pub async fn init_config(custom_config_file: Option<&PathBuf>) -> Result<()> {
    println!("{}", "🔧 Initializing AI Commit configuration...".cyan());

    let config_path = resolve_config_path(custom_config_file)?;
    let mut config = load_or_default(&config_path)?;

    if config_path.exists() {
        println!("{}", "⚠️  Configuration file already exists.".yellow());
        println!("Location: {}", config_path.display().to_string().bright_blue());
        if !show_confirm("Do you want to overwrite it?", true)? {
            println!("{}", "❌ Configuration initialization cancelled.".red());
            return Ok(());
        }
    }

    // Start from a clean provider list on init.
    config.providers.clear();
    config.default_provider = None;

    let theme = &*THEME;
    let Some(entry) = prompt_provider_entry(theme, &config).await? else {
        println!("{}", "❌ Configuration initialization cancelled.".red());
        return Ok(());
    };
    let entry_name = entry.entry_name();
    config.default_provider = Some(entry_name.clone());
    config.providers.push(entry);

    println!();
    println!("{}", "Configuration preview:".bright_cyan().bold());
    println!("Provider entry: {}", entry_name.bright_green());
    println!("Config file: {}", config_path.display().to_string().bright_blue());

    if !Confirm::with_theme(theme).with_prompt("Save this configuration?").default(true).interact()? {
        println!("{}", "❌ Configuration initialization cancelled.".red());
        return Ok(());
    }

    config.save(&config_path)?;

    println!("{}", "✅ Configuration file created successfully!".green());
    println!("Location: {}", config_path.display().to_string().bright_blue());
    println!();
    println!("{}", "📝 Next steps:".bright_cyan().bold());
    println!("1. Review config: {}", "ai-commit config show".yellow());
    println!("2. Add another provider: {}", "ai-commit config provider add".yellow());
    println!("3. Start using: {}", "ai-commit".yellow());

    Ok(())
}

/// Interactively build a single provider entry: name, built-in provider, API
/// key, and model. Returns `None` when the user cancels model selection.
async fn prompt_provider_entry(theme: &ColorfulTheme, config: &AppConfig) -> Result<Option<ApiConfig>> {
    let specs = provider_specs();
    let display_items: Vec<&str> = specs.iter().map(|spec| spec.display_name).collect();
    let provider_index =
        Select::with_theme(theme).with_prompt("Select an AI provider").items(&display_items).default(0).interact()?;
    let provider_name = specs[provider_index].name;

    let existing: Vec<String> = config.provider_names();
    let default_name = if existing.iter().any(|name| name == provider_name) {
        format!("{provider_name}-2")
    } else {
        provider_name.to_string()
    };
    let entry_name: String = Input::with_theme(theme)
        .with_prompt("Name this provider entry")
        .default(default_name)
        .validate_with(|input: &String| -> Result<(), String> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("Name cannot be empty".to_string());
            }
            if existing.iter().any(|name| name == trimmed) {
                return Err(format!("Provider '{trimmed}' already exists"));
            }
            Ok(())
        })
        .interact_text()?;

    let api_key = Password::with_theme(theme)
        .with_prompt(format!("Enter API key for {provider_name}"))
        .allow_empty_password(true)
        .interact()?;

    let mut entry = ApiConfig {
        name: Some(entry_name.trim().to_string()),
        provider: Some(provider_name.to_string()),
        endpoint: None,
        model: String::new(),
        api_key: if api_key.is_empty() { None } else { Some(api_key) },
        max_tokens: config.providers.first().and_then(|p| p.max_tokens).or(Some(1000)),
        temperature: config.providers.first().and_then(|p| p.temperature).or(Some(0.7)),
        protocol: None,
    };

    let Some(model) = select_model(theme, config, &entry).await? else {
        return Ok(None);
    };
    entry.model = model;

    Ok(Some(entry))
}

async fn select_model(theme: &ColorfulTheme, base: &AppConfig, entry: &ApiConfig) -> Result<Option<String>> {
    // Build a throwaway config containing just this entry so the client resolves it.
    let mut probe = base.clone();
    let mut probe_entry = entry.clone();
    // `fetch_provider_models` doesn't use the model, but `resolve_api_config`
    // rejects an empty one — set a placeholder so resolution succeeds.
    if probe_entry.model.trim().is_empty() {
        probe_entry.model = "_".to_string();
    }
    probe.providers = vec![probe_entry];
    probe.default_provider = entry.name.clone();

    match AiClient::with_config(probe).fetch_provider_models().await {
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
            Ok(Some(models[model_index].clone()))
        }
        Ok(_) => prompt_model_fallback(theme, "No models were returned by the provider."),
        Err(error) => prompt_model_fallback(theme, &format!("Failed to fetch models: {error}")),
    }
}

fn prompt_model_fallback(theme: &ColorfulTheme, reason: &str) -> Result<Option<String>> {
    println!("{}", format!("⚠️  {reason}").yellow());

    let should_continue =
        Confirm::with_theme(theme).with_prompt("Do you want to enter a model manually?").default(true).interact()?;
    if !should_continue {
        return Ok(None);
    }

    Ok(Some(Input::with_theme(theme).with_prompt("Enter model name").interact_text()?))
}

pub fn list_providers(custom_config_file: Option<&PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(custom_config_file)?;
    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        println!("Run {} to create it.", "ai-commit config init".yellow());
        return Ok(());
    }

    let config = AppConfig::load_from_path(&config_path)?;
    if config.providers.is_empty() {
        println!("{}", "No providers configured.".yellow());
        println!("Run {} to add one.", "ai-commit config provider add".yellow());
        return Ok(());
    }

    println!("{}", "📋 Configured providers".bright_cyan().bold());
    println!("{}", "═══════════════════════".bright_blue());
    let default = config.default_provider.as_deref();
    for entry in &config.providers {
        let name = entry.entry_name();
        let marker = if default == Some(name.as_str()) { "* ".green() } else { "  ".normal() };
        let provider_type = entry.provider.as_deref().unwrap_or("custom");
        println!("{}{}  {} ({})", marker, name.bright_green(), entry.model.bright_yellow(), provider_type);
    }
    println!();
    println!("{}", "* = default provider".dimmed());

    Ok(())
}

pub fn use_provider(custom_config_file: Option<&PathBuf>, name: Option<&str>) -> Result<()> {
    let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
        println!("{}", "❌ Provider name is required.".red());
        return Ok(());
    };

    let config_path = resolve_config_path(custom_config_file)?;
    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        println!("Run {} to create it.", "ai-commit config init".yellow());
        return Ok(());
    }

    let mut config = AppConfig::load_from_path(&config_path)?;
    if config.find_provider(name).is_none() {
        println!("{}", format!("❌ Provider '{name}' not found.").red());
        println!("Available: {}", config.provider_names().join(", ").yellow());
        return Ok(());
    }

    config.default_provider = Some(name.to_string());
    config.save(&config_path)?;
    println!("{}", format!("✅ Default provider set to '{name}'.").green());

    Ok(())
}

pub async fn add_provider(custom_config_file: Option<&PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(custom_config_file)?;
    let mut config = load_or_default(&config_path)?;

    let theme = &*THEME;
    let Some(entry) = prompt_provider_entry(theme, &config).await? else {
        println!("{}", "❌ Cancelled.".red());
        return Ok(());
    };
    let entry_name = entry.entry_name();

    let make_default = config.providers.is_empty()
        || Confirm::with_theme(theme).with_prompt("Set as default provider?").default(false).interact()?;

    config.providers.push(entry);
    if make_default {
        config.default_provider = Some(entry_name.clone());
    }

    config.save(&config_path)?;
    println!("{}", format!("✅ Provider '{entry_name}' added.").green());
    if make_default {
        println!("{}", "   Set as default provider.".green());
    }

    Ok(())
}

pub fn remove_provider(custom_config_file: Option<&PathBuf>, name: Option<&str>) -> Result<()> {
    let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
        println!("{}", "❌ Provider name is required.".red());
        return Ok(());
    };

    let config_path = resolve_config_path(custom_config_file)?;
    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        return Ok(());
    }

    let mut config = AppConfig::load_from_path(&config_path)?;
    let original_len = config.providers.len();
    config.providers.retain(|entry| entry.entry_name() != name);

    if config.providers.len() == original_len {
        println!("{}", format!("❌ Provider '{name}' not found.").red());
        println!("Available: {}", config.provider_names().join(", ").yellow());
        return Ok(());
    }

    // Reset the default if it pointed at the removed entry.
    if config.default_provider.as_deref() == Some(name) {
        config.default_provider = config.providers.first().map(ApiConfig::entry_name);
    }

    config.save(&config_path)?;
    println!("{}", format!("✅ Provider '{name}' removed.").green());
    if let Some(default) = &config.default_provider {
        println!("Default provider is now '{}'.", default.bright_green());
    } else {
        println!("{}", "⚠️  No providers remain. Run `ai-commit config provider add`.".yellow());
    }

    Ok(())
}

pub fn show_config(custom_config_file: Option<&PathBuf>) -> Result<()> {
    println!("{}", "📋 Current AI Commit Configuration".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════".bright_blue());

    let config_path = resolve_config_path(custom_config_file)?;

    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        println!("Run {} to create it.", "ai-commit config init".yellow());
        return Ok(());
    }

    let config = AppConfig::load_from_path(&config_path)?;
    let config_content = toml::to_string_pretty(&config)?;

    println!("File location: {}", config_path.display().to_string().bright_blue());
    println!();
    println!("{config_content}");

    Ok(())
}

pub fn edit_config(custom_config_file: Option<&PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(custom_config_file)?;

    if !config_path.exists() {
        println!("{}", "❌ Configuration file not found.".red());
        println!("Run {} to create it.", "ai-commit config init".yellow());
        return Ok(());
    }

    let config_content = fs::read_to_string(&config_path)?;

    let Some(edited_content) = Editor::new().extension("toml").edit(&config_content)? else {
        println!("{}", "⚠️  Configuration edit cancelled.".yellow());
        return Ok(());
    };

    if edited_content == config_content {
        println!("{}", "ℹ️  No changes were made to the configuration.".yellow());
        return Ok(());
    }

    let _: AppConfig = toml::from_str(&edited_content)?;
    fs::write(&config_path, edited_content)?;

    println!("{}", "✅ Configuration updated successfully!".green());
    println!("Location: {}", config_path.display().to_string().bright_blue());

    Ok(())
}
