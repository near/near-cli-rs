#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use clap::Parser;
#[cfg(feature = "self-update")]
use color_eyre::eyre::WrapErr;
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

pub use common::CliResult;
pub type GlobalContext = (crate::config::Config,);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    top_level: crate::commands::TopLevelCommand,
}

fn main() -> crate::common::CliResult {
    let config = crate::common::get_config_toml()?;

    color_eyre::install()?;

    #[cfg(feature = "self-update")]
    let handle = std::thread::spawn(|| -> color_eyre::eyre::Result<String> {
        crate::commands::extensions::self_update::get_latest_version()
    });

    let near_cli_exec_path = std::env::args()
        .next()
        .unwrap_or_else(|| "./near".to_owned());

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {}
                _ => match crate::js_command_match::JsCmd::try_parse() {
                    Ok(js_cmd) => {
                        match js_cmd.rust_command_generation() {
                            Ok(vec_cmd) => {
                                println!("The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:");
                                println!();
                                println!(
                                    "{}",
                                    shell_words::join(
                                        std::iter::once(near_cli_exec_path).chain(vec_cmd)
                                    )
                                );
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

            if let clap::error::ErrorKind::UnknownArgument
            | clap::error::ErrorKind::InvalidSubcommand = error.kind()
            {
                return crate::common::try_external_subcommand_execution(error);
            }
            error.exit();
        }
    };

    let cli_cmd = match <Cmd as interactive_clap::FromCli>::from_cli(Some(cli), (config,)) {
        interactive_clap::ResultFromCli::Ok(cli_cmd)
        | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
            eprintln!(
                "Your console command:\n{}",
                shell_words::join(
                    std::iter::once(&near_cli_exec_path).chain(&cli_cmd.to_cli_args())
                )
            );
            Ok(Some(cli_cmd))
        }
        interactive_clap::ResultFromCli::Cancel(None) => {
            println!("Goodbye!");
            Ok(None)
        }
        interactive_clap::ResultFromCli::Back => {
            unreachable!("TopLevelCommand does not have back option");
        }
        interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
            if let Some(cli_cmd) = optional_cli_cmd {
                println!(
                    "Your console command:\n{}",
                    shell_words::join(
                        std::iter::once(&near_cli_exec_path).chain(&cli_cmd.to_cli_args())
                    )
                );
            }
            Err(err)
        }
    };

    #[cfg(feature = "self-update")]
    // We don't need to check the version if user has just called self-update
    if !matches!(
        cli_cmd,
        Ok(Some(CliCmd {
            top_level: Some(crate::commands::CliTopLevelCommand::Extensions(
                crate::commands::extensions::CliExtensionsCommands {
                    extensions_actions: Some(
                        crate::commands::extensions::CliExtensionsActions::SelfUpdate(
                            crate::commands::extensions::self_update::CliSelfUpdateCommand {},
                        )
                    ),
                },
            )),
        }))
    ) {
        if let Ok(Ok(latest_version)) = handle.join() {
            let current_version = semver::Version::parse(self_update::cargo_crate_version!())
                .wrap_err("Failed to parse current version of `near` CLI")?;

            let latest_version = semver::Version::parse(&latest_version)
                .wrap_err("Failed to parse latest version of `near` CLI")?;

            if current_version < latest_version {
                println!();
                println!(
                    "`near` CLI has a new update available \x1b[2m{current_version}\x1b[0m →  \x1b[32m{latest_version}\x1b[0m"
                );
                let self_update_cli_cmd = CliCmd {
                    top_level:
                        Some(crate::commands::CliTopLevelCommand::Extensions(
                            crate::commands::extensions::CliExtensionsCommands {
                                extensions_actions:
                                    Some(crate::commands::extensions::CliExtensionsActions::SelfUpdate(
                                        crate::commands::extensions::self_update::CliSelfUpdateCommand {},
                                    )),
                            },
                        )),
                };
                println!(
                    "To update `near` CLI use: {}",
                    shell_words::join(
                        std::iter::once(near_cli_exec_path)
                            .chain(self_update_cli_cmd.to_cli_args())
                    )
                );
            }
        }
    };

    cli_cmd.map(|_| ())
}
