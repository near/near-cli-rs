#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use clap::Parser;
use common::{try_external_subcommand_execution, CliResult};
use interactive_clap::ToCliArgs;

mod commands;
mod common;
mod config;
mod js_command_match;
mod network;
mod network_for_transaction;
mod network_view_at_block;
mod transaction_signature_options;
mod types;
mod utils_command;

pub type GlobalContext = (crate::config::Config,);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    top_level: self::commands::TopLevelCommand,
}

fn main() -> CliResult {
    let config = crate::common::get_config_toml()?;

    color_eyre::install()?;

    #[cfg(feature = "self-update")]
    let handle = std::thread::spawn(|| -> color_eyre::eyre::Result<String> {
        crate::commands::extensions::self_update::SelfUpdateCommand::get_latest_version()
    });

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {}
                _ => match self::js_command_match::JsCmd::try_parse() {
                    Ok(js_cmd) => {
                        match js_cmd.rust_command_generation() {
                            Ok(vec_cmd) => {
                                let near_cli_exec_path = std::env::args()
                                    .next()
                                    .unwrap_or_else(|| "./near_cli".to_owned());
                                let mut suggested_cmd = Vec::with_capacity(vec_cmd.len() + 1);
                                suggested_cmd.push(near_cli_exec_path);
                                suggested_cmd.extend(vec_cmd);
                                println!("The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:");
                                println!();
                                println!("{}", shell_words::join(suggested_cmd));
                            }
                            Err(err) => {
                                println!("The command you tried to run is deprecated in the new NEAR CLI and there is no equivalent command in the new NEAR CLI.");
                                println!();
                                println!("{}", err);
                            }
                        }
                        std::process::exit(1);
                    }
                    Err(error) => {
                        if let clap::error::ErrorKind::DisplayHelp = error.kind() {
                            error.exit()
                        }
                    }
                },
            }

            if matches!(
                error.kind(),
                clap::error::ErrorKind::UnknownArgument | clap::error::ErrorKind::InvalidSubcommand
            ) {
                return try_external_subcommand_execution(error);
            }
            error.exit();
        }
    };

    #[cfg(feature = "self-update")]
    {
        if !matches!(
            cli,
            CliCmd {
                top_level: crate::commands::TopLevelCommand::Extensions(
                    crate::commands::extensions::ExtensionsCommands {
                        extensions_actions:
                            crate::commands::extensions::ExtensionsActions::SelfUpdate(
                                crate::commands::extensions::self_update::SelfUpdateCommand,
                            ),
                    },
                ),
            }
        ) {
            if let Ok(Ok(result)) = handle.join() {
                let current_version = semver::Version::parse(self_update::cargo_crate_version!())
                    .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to parse current version of near-cli-rs as Version: {:?}",
                        err
                    ))
                })?;

                let latest_version = semver::Version::parse(result.as_str()).map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to parse latest version of near-cli-rs as Version: {:?}",
                        err
                    ))
                })?;

                if current_version < latest_version {
                    println!(
                        "\nNEAR-CLI-RS has a new update available \x1b[2m{}\x1b[0m →  \x1b[32m{}\x1b[0m",
                        current_version.to_string(),
                        latest_version.to_string()
                    );

                    println!(
                        "To update NEAR-CLI-RS use: {} {}",
                        std::env::args().next().as_deref().unwrap_or("./near_cli"),
                        shell_words::join(cli.to_cli_args())
                    );
                }
            }
        }
    };

    loop {
        match <Cmd as interactive_clap::FromCli>::from_cli(Some(cli.clone()), (config.clone(),)) {
            interactive_clap::ResultFromCli::Ok(cli_cmd)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
                println!(
                    "Your console command:\n{} {}",
                    std::env::args().next().as_deref().unwrap_or("./near_cli"),
                    shell_words::join(&cli_cmd.to_cli_args())
                );
                return Ok(());
            }
            interactive_clap::ResultFromCli::Cancel(None) => {
                println!("Goodbye!");
                return Ok(());
            }
            interactive_clap::ResultFromCli::Back => {}
            interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
                if let Some(cli_cmd) = optional_cli_cmd {
                    println!(
                        "Your console command:\n{} {}",
                        std::env::args().next().as_deref().unwrap_or("./near_cli"),
                        shell_words::join(&cli_cmd.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
