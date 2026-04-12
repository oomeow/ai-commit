#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]

use std::path::PathBuf;

use ai_commit::{commands::execute_command, dirs::get_work_dir, utils::get_optional_value};
use anyhow::Result;
use clap::{Arg, Command};
use clap_complete::{Shell, generate};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // check and init work dir
    let work_dir = get_work_dir()?;
    if !work_dir.exists() {
        std::fs::create_dir_all(&work_dir)?;
    }

    let matches = build_cli().get_matches();

    let mut custom_config: Option<&PathBuf> = None;
    if matches.args_present() {
        // 只解析 --config 参数
        custom_config = get_optional_value::<PathBuf>(Some(&matches), "config");
    }

    match matches.subcommand() {
        Some(("completion", sub_matches)) => {
            let shell = *sub_matches.get_one::<Shell>("shell").expect("shell is required");
            let mut command = build_cli();
            generate(shell, &mut command, "ai-commit", &mut std::io::stdout());
            Ok(())
        }
        Some(("install", _)) => execute_command("install", None, custom_config).await,
        Some(("uninstall", _)) => execute_command("uninstall", None, custom_config).await,
        Some(("amend", sub_matches)) => execute_command("amend", Some(sub_matches), custom_config).await,
        Some(("commit", sub_matches)) => execute_command("commit", Some(sub_matches), custom_config).await,
        Some(("config", sub_matches)) => {
            let command = match sub_matches.subcommand() {
                Some(("show", _)) => "config-show",
                Some(("init", _)) => "config-init",
                Some(("edit", _)) => "config-edit",
                _ => "config-show",
            };
            execute_command(command, Some(sub_matches), custom_config).await
        }
        // When no subcommand is provided, pass the top-level matches so flags like --add are honored
        _ => execute_command("commit", Some(&matches), custom_config).await,
    }
}

fn build_cli() -> Command {
    let version = env!("CARGO_PKG_VERSION");
    Command::new("ai-commit")
        .version(version)
        .about("AI-assisted Git commit message generator (defaults to 'commit' if no subcommand)")
        .author("John & oomeow")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .arg(
            Arg::new("add")
                .short('a')
                .help("Stage all changes before generating the commit message")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('f')
                .value_name("FILE")
                .help("Use a custom config file")
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .subcommand(Command::new("install").about("Install git hooks for AI commit assistance"))
        .subcommand(Command::new("uninstall").about("Remove AI commit hooks"))
        .subcommand(
            Command::new("completion").about("Generate shell completion script").arg(
                Arg::new("shell")
                    .value_name("SHELL")
                    .help("Shell to generate completions for")
                    .required(true)
                    .value_parser(clap::value_parser!(Shell)),
            ),
        )
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
                .subcommand(Command::new("edit").about("Edit configuration in your terminal editor")),
        )
}
