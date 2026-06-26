use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};

use crate::{
    git::add_all_files_to_git,
    utils::{get_optional_flag, get_optional_value},
};
pub mod amend;
pub mod commit;
pub mod config;
pub mod install;
pub mod uninstall;

pub static THEME: LazyLock<ColorfulTheme> = LazyLock::new(ColorfulTheme::default);

pub async fn execute_command(
    command: &str,
    matches: Option<&clap::ArgMatches>,
    custom_config_file: Option<&PathBuf>,
    provider: Option<&String>,
) -> Result<()> {
    let provider = provider.map(String::as_str);
    match command {
        "install" => install::install_hook(),
        "uninstall" => uninstall::uninstall_hook(),
        "commit" => {
            let add_all_files = get_optional_flag(matches, "add");
            let generate_only = get_optional_flag(matches, "generate-only");
            let output_file = get_optional_value::<PathBuf>(matches, "output-file");
            if add_all_files {
                add_all_files_to_git()?;
            }
            commit::handle_commit(generate_only, custom_config_file, output_file, provider).await
        }
        "amend" => {
            let dry_run = get_optional_flag(matches, "dry-run");
            amend::handle_amend(custom_config_file, dry_run, provider).await
        }
        "config-init" => config::init_config(custom_config_file).await,
        "config-show" => config::show_config(custom_config_file),
        "config-edit" => config::edit_config(custom_config_file),
        "config-provider-list" => config::list_providers(custom_config_file),
        "config-provider-use" => {
            let name = get_optional_value::<String>(matches, "name").map(String::as_str);
            config::use_provider(custom_config_file, name)
        }
        "config-provider-add" => config::add_provider(custom_config_file).await,
        "config-provider-remove" => {
            let name = get_optional_value::<String>(matches, "name").map(String::as_str);
            config::remove_provider(custom_config_file, name)
        }
        _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
    }
}

pub fn show_confirm(title: &str, default_yes: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&*THEME).with_prompt(title).default(default_yes).interact()?)
}
