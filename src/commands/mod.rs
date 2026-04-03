use std::io::{self, Write};

use anyhow::Result;
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
            let generate_only = matches.map(|m| m.get_flag("generate-only")).unwrap_or(false);
            let output_file = matches.and_then(|m| m.get_one::<std::path::PathBuf>("output-file"));
            commit::handle_commit(generate_only, output_file.map(|p| p.as_path())).await
        }
        "amend" => amend::handle_amend().await,
        "config-init" => config::init_config(),
        "config-show" => config::show_config(),
        "config-edit-prompts" => config::edit_prompts_help(),
        _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
    }
}

pub fn show_confirm(title: &str) -> Result<bool> {
    print!("{title} (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().is_empty() || matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}
