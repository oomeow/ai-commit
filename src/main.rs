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
    let mut provider: Option<&String> = None;
    if matches.args_present() {
        // 只解析 --config 参数
        custom_config = get_optional_value::<PathBuf>(Some(&matches), "config");
        provider = get_optional_value::<String>(Some(&matches), "provider");
    }

    match matches.subcommand() {
        Some(("completion", sub_matches)) => {
            let shell = *sub_matches.get_one::<Shell>("shell").expect("shell is required");
            let mut command = build_cli();
            generate(shell, &mut command, "ai-commit", &mut std::io::stdout());
            Ok(())
        }
        Some(("install", _)) => execute_command("install", None, custom_config, None).await,
        Some(("uninstall", _)) => execute_command("uninstall", None, custom_config, None).await,
        Some(("amend", sub_matches)) => {
            let provider = get_optional_value::<String>(Some(sub_matches), "provider").or(provider);
            execute_command("amend", Some(sub_matches), custom_config, provider).await
        }
        Some(("commit", sub_matches)) => {
            let provider = get_optional_value::<String>(Some(sub_matches), "provider").or(provider);
            execute_command("commit", Some(sub_matches), custom_config, provider).await
        }
        Some(("config", sub_matches)) => {
            let (command, inner) = match sub_matches.subcommand() {
                Some(("show", m)) => ("config-show", Some(m)),
                Some(("init", m)) => ("config-init", Some(m)),
                Some(("edit", m)) => ("config-edit", Some(m)),
                Some(("provider", m)) => match m.subcommand() {
                    Some(("list", pm)) => ("config-provider-list", Some(pm)),
                    Some(("use", pm)) => ("config-provider-use", Some(pm)),
                    Some(("add", pm)) => ("config-provider-add", Some(pm)),
                    Some(("remove", pm)) => ("config-provider-remove", Some(pm)),
                    _ => ("config-provider-list", Some(m)),
                },
                _ => ("config-show", Some(sub_matches)),
            };
            execute_command(command, inner, custom_config, None).await
        }
        // When no subcommand is provided, pass the top-level matches so flags like --add are honored
        _ => execute_command("commit", Some(&matches), custom_config, provider).await,
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
        .arg(
            Arg::new("provider")
                .long("provider")
                .short('p')
                .value_name("NAME")
                .help("Use a specific configured provider for this run"),
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
                // .arg(
                //     Arg::new("context-limit")
                //         .long("context-limit")
                //         .value_name("CHARS")
                //         .help("Maximum characters to send to AI (default: 200000)")
                //         .value_parser(clap::value_parser!(usize)),
                // )
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
                )
                .arg(
                    Arg::new("provider")
                        .long("provider")
                        .short('p')
                        .value_name("NAME")
                        .help("Use a specific configured provider for this run"),
                ),
        )
        .subcommand(
            Command::new("amend")
                .about("Amend the last commit with staged changes using AI-generated message")
                // .arg(
                //     Arg::new("context-limit")
                //         .long("context-limit")
                //         .value_name("CHARS")
                //         .help("Maximum characters to send to AI (default: 200000)")
                //         .value_parser(clap::value_parser!(usize)),
                // )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Show generated message without amending")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("provider")
                        .long("provider")
                        .short('p')
                        .value_name("NAME")
                        .help("Use a specific configured provider for this run"),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration")
                .subcommand(Command::new("show").about("Show current configuration"))
                .subcommand(Command::new("init").about("Initialize default configuration"))
                .subcommand(Command::new("edit").about("Edit configuration in your terminal editor"))
                .subcommand(
                    Command::new("provider")
                        .about("Manage AI providers")
                        .subcommand(Command::new("list").about("List configured providers"))
                        .subcommand(
                            Command::new("use")
                                .about("Set the default provider")
                                .arg(Arg::new("name").value_name("NAME").required(true).help("Provider entry name")),
                        )
                        .subcommand(Command::new("add").about("Add a new provider interactively"))
                        .subcommand(
                            Command::new("remove")
                                .about("Remove a provider")
                                .arg(Arg::new("name").value_name("NAME").required(true).help("Provider entry name")),
                        ),
                ),
        )
}
