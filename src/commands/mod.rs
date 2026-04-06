use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};
pub mod amend;
pub mod commit;
pub mod config;
pub mod install;
pub mod uninstall;

pub async fn execute_command(command: &str, matches: Option<&clap::ArgMatches>) -> Result<()> {
    match command {
        "install" => install::install_hook(),
        "uninstall" => uninstall::uninstall_hook(),
        "commit" => {
            let add = get_optional_flag(matches, "add");
            let generate_only = get_optional_flag(matches, "generate-only");
            let output_file = get_optional_value::<std::path::PathBuf>(matches, "output-file");
            commit::handle_commit(add, generate_only, output_file.map(|p| p.as_path())).await
        }
        "amend" => amend::handle_amend().await,
        "config-init" => config::init_config().await,
        "config-show" => config::show_config(),
        "config-edit" => config::edit_config(),
        _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
    }
}

fn get_optional_flag(matches: Option<&clap::ArgMatches>, id: &str) -> bool {
    matches.filter(|m| m.try_contains_id(id).unwrap_or(false)).map(|m| m.get_flag(id)).unwrap_or(false)
}

fn get_optional_value<'a, T: Clone + Send + Sync + 'static>(
    matches: Option<&'a clap::ArgMatches>,
    id: &str,
) -> Option<&'a T> {
    matches.filter(|m| m.try_contains_id(id).unwrap_or(false)).and_then(|m| m.try_get_one::<T>(id).ok().flatten())
}

pub fn show_confirm(title: &str, default_yes: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default()).with_prompt(title).default(default_yes).interact()?)
}
