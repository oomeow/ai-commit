use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};

use crate::utils::{get_optional_flag, get_optional_value};
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
) -> Result<()> {
    match command {
        "install" => install::install_hook(),
        "uninstall" => uninstall::uninstall_hook(),
        "commit" => {
            let add = get_optional_flag(matches, "add");
            let generate_only = get_optional_flag(matches, "generate-only");
            let output_file = get_optional_value::<PathBuf>(matches, "output-file");
            commit::handle_commit(add, generate_only, custom_config_file, output_file).await
        }
        "amend" => amend::handle_amend(custom_config_file).await,
        "config-init" => config::init_config(custom_config_file).await,
        "config-show" => config::show_config(custom_config_file),
        "config-edit" => config::edit_config(custom_config_file),
        _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
    }
}

pub fn show_confirm(title: &str, default_yes: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&*THEME).with_prompt(title).default(default_yes).interact()?)
}
