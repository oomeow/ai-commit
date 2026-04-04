#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]

use ai_commit::{commands::execute_command, dirs::get_work_dir};
use anyhow::Result;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // check and init work dir
    let work_dir = get_work_dir()?;
    if !work_dir.exists() {
        std::fs::create_dir_all(&work_dir)?;
    }

    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new("ai-commit")
        .version(version)
        .about("AI-assisted Git commit message generator (defaults to 'commit' if no subcommand)")
        .author("John & oomeow")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .arg(
            Arg::new("add")
                .long("add")
                .short('a')
                .help("Stage all changes before generating the commit message")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(Command::new("install").about("Install git hooks for AI commit assistance"))
        .subcommand(Command::new("uninstall").about("Remove AI commit hooks"))
        .subcommand(
            Command::new("commit")
                .about("Generate AI commit message for staged changes")
                .arg(
                    Arg::new("add")
                        .long("add")
                        .short('a')
                        .help("Stage all changes before generating the commit message")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("context-limit")
                        .long("context-limit")
                        .value_name("CHARS")
                        .help("Maximum characters to send to AI (default: 200000)")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Show generated message without committing")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("generate-only")
                        .long("generate-only")
                        .help("Generate commit message only (no commit, no confirmation)")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("output-file")
                        .long("output-file")
                        .value_name("FILE")
                        .help("Write generated message to file")
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                ),
        )
        .subcommand(
            Command::new("amend")
                .about("Amend the last commit with staged changes using AI-generated message")
                .arg(
                    Arg::new("context-limit")
                        .long("context-limit")
                        .value_name("CHARS")
                        .help("Maximum characters to send to AI (default: 200000)")
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Show generated message without amending")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration")
                .subcommand(Command::new("show").about("Show current configuration"))
                .subcommand(Command::new("init").about("Initialize default configuration"))
                .subcommand(Command::new("edit-prompts").about("Show how to edit prompt templates")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("install", _)) => execute_command("install", None).await,
        Some(("uninstall", _)) => execute_command("uninstall", None).await,
        Some(("amend", sub_matches)) => execute_command("amend", Some(sub_matches)).await,
        Some(("commit", sub_matches)) => execute_command("commit", Some(sub_matches)).await,
        Some(("config", sub_matches)) => {
            let command = match sub_matches.subcommand() {
                Some(("show", _)) => "config-show",
                Some(("init", _)) => "config-init",
                Some(("edit-prompts", _)) => "config-edit-prompts",
                _ => "config-show",
            };
            execute_command(command, None).await
        }
        // When no subcommand is provided, pass the top-level matches so flags like --add are honored
        _ => execute_command("commit", Some(&matches)).await,
    }
}
