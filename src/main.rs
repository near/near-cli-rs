#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use clap::Parser;
use common::{try_external_subcommand_execution, CliResult};
use interactive_clap::FromCli;
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

impl Cmd {
    async fn process(&self, config: crate::config::Config) -> CliResult {
        self.top_level.process(config).await
    }
}

fn main() -> CliResult {
    let config = crate::common::get_config_toml()?;

    color_eyre::install()?;

    #[cfg(feature = "self-update")]
    let handle = std::thread::spawn(|| -> color_eyre::eyre::Result<String> {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("near")
            .repo_name("near-cli-rs")
            .build()
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to build self_update: {:?}", err))
            })?
            .fetch()
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to fetch releases: {:?}", err))
            })?;

        Ok(releases[0].version.clone())
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

    // if let Some(self::commands::CliTopLevelCommand::GenerateShellCompletions(subcommand)) =
    //     cli.top_level_command
    // {
    //     subcommand.process();
    //     return Ok(());
    // }

    let cmd = loop {
        match Cmd::from_cli(Some(cli.clone()), (config.clone(),)) {
            Ok(Some(cmd)) => {
                break cmd;
            }
            Ok(None) => {}
            Err(err) => match err.downcast_ref() {
                Some(
                    inquire::InquireError::OperationCanceled
                    | inquire::InquireError::OperationInterrupted,
                ) => {
                    println!("<Operation was interrupted. Goodbye>");
                    return Ok(());
                }
                Some(_) | None => return Err(err),
            },
        }
    };

    let completed_cli = CliCmd::from(cmd.clone());

    let process_result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(cmd.process(config));

    println!(
        "Your console command:\n{} {}",
        std::env::args().next().as_deref().unwrap_or("./near_cli"),
        shell_words::join(completed_cli.to_cli_args())
    );

    #[cfg(feature = "self-update")]
    {
        let current_version = self_update::cargo_crate_version!()
            .parse::<crate::types::version::Version>()
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to parse current version of near-cli-rs as Version: {:?}",
                    err
                ))
            })?;

        let result = handle.join().map_err(|err| {
            color_eyre::Report::msg(format!("Failed to join handle: {:?}", err))
        })??;
        let latest_version = result
            .parse::<crate::types::version::Version>()
            .map_err(|err| {
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
                shell_words::join(
                    CliCmd::from(Cmd {
                        top_level: crate::commands::TopLevelCommand::Extensions(
                            crate::commands::extensions::ExtensionsCommands {
                                extensions_actions:
                                    crate::commands::extensions::ExtensionsActions::SelfUpdate(
                                        crate::commands::extensions::self_update::SelfUpdateCommand
                                    )
                            }
                        )
                    })
                    .to_cli_args()
                )
            );
        }
    }

    match process_result {
        Ok(()) => Ok(()),
        Err(err) => match err.downcast_ref() {
            Some(
                inquire::InquireError::OperationCanceled
                | inquire::InquireError::OperationInterrupted,
            ) => {
                println!("<Operation was interrupted. Goodbye>");
                Ok(())
            }
            Some(_) | None => Err(err),
        },
    }
}
