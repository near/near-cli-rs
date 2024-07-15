#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use clap::Parser;
#[cfg(feature = "self-update")]
use color_eyre::eyre::WrapErr;
use color_eyre::owo_colors::OwoColorize;
use interactive_clap::ToCliArgs;

use indicatif::ProgressStyle;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub use near_cli_rs::commands;
pub use near_cli_rs::common::{self, CliResult};
pub use near_cli_rs::config;
pub use near_cli_rs::js_command_match;
pub use near_cli_rs::network;
pub use near_cli_rs::network_for_transaction;
pub use near_cli_rs::network_view_at_block;
pub use near_cli_rs::transaction_signature_options;
pub use near_cli_rs::types;
pub use near_cli_rs::utils_command;

pub use near_cli_rs::GlobalContext;

type ConfigContext = (crate::config::Config,);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ConfigContext)]
#[interactive_clap(output_context = CmdContext)]
struct Cmd {
    /// Offline mode
    #[interactive_clap(long)]
    offline: bool,
    #[interactive_clap(subcommand)]
    top_level: crate::commands::TopLevelCommand,
}

#[derive(Debug, Clone)]
struct CmdContext(crate::GlobalContext);

impl CmdContext {
    fn from_previous_context(
        previous_context: ConfigContext,
        scope: &<Cmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(crate::GlobalContext {
            config: previous_context.0,
            offline: scope.offline,
        }))
    }
}

impl From<CmdContext> for crate::GlobalContext {
    fn from(item: CmdContext) -> Self {
        item.0
    }
}

fn main() -> crate::common::CliResult {
    let config = crate::config::Config::get_config_toml()?;

    if !crate::common::is_used_account_list_exist(&config.credentials_home_dir) {
        crate::common::create_used_account_list_from_keychain(&config.credentials_home_dir)?;
    }

    color_eyre::install()?;

    let indicatif_layer = IndicatifLayer::new()
        .with_progress_style(
            ProgressStyle::with_template(
                "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
            )
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
        )
        .with_span_child_prefix_symbol("↳ ");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into())
                .add_directive("near_cli_rs=info".parse()?),
        )
        .init();

    #[cfg(feature = "self-update")]
    let handle = std::thread::spawn(|| -> color_eyre::eyre::Result<String> {
        crate::commands::extensions::self_update::get_latest_version()
    });

    let near_cli_exec_path = crate::common::get_near_exec_path();

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {}
                _ => match crate::js_command_match::JsCmd::try_parse() {
                    Ok(js_cmd) => {
                        match js_cmd.rust_command_generation() {
                            Ok((vec_cmd, success_message)) => {
                                eprintln!("{success_message}");
                                eprintln!();
                                eprintln!(
                                    "    {}",
                                    shell_words::join(
                                        std::iter::once(near_cli_exec_path).chain(vec_cmd)
                                    )
                                    .yellow()
                                );
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        }
                        std::process::exit(1);
                    }
                    Err(error) => {
                        if let clap::error::ErrorKind::DisplayHelp = error.kind() {
                            error.exit();
                        }
                        if let Some(cmd) = std::env::args().nth(1) {
                            match cmd.as_str() {
                                "add-credentials" | "add-key" | "call" | "create"
                                | "create-account" | "delete" | "delete-account" | "delete-key"
                                | "deploy" | "generate-key" | "import-account" | "keys"
                                | "list-keys" | "login" | "send" | "send-near" | "stake"
                                | "state" | "storage" | "tx-status" | "validator-stake"
                                | "validators" | "view" | "view-storage" => error.exit(),
                                _ => {}
                            }
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
                "\n\nHere is your console command if you need to script it or re-run:\n    {}\n",
                shell_words::join(
                    std::iter::once(&near_cli_exec_path).chain(&cli_cmd.to_cli_args())
                )
                .yellow()
            );
            Ok(Some(cli_cmd))
        }
        interactive_clap::ResultFromCli::Cancel(None) => {
            eprintln!("\nGoodbye!");
            Ok(None)
        }
        interactive_clap::ResultFromCli::Back => {
            unreachable!("TopLevelCommand does not have back option");
        }
        interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
            if let Some(cli_cmd) = optional_cli_cmd {
                eprintln!(
                    "\nHere is your console command if you need to script it or re-run:\n    {}\n",
                    shell_words::join(
                        std::iter::once(&near_cli_exec_path).chain(&cli_cmd.to_cli_args())
                    )
                    .yellow()
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
            ..
        }))
    ) {
        if let Ok(Ok(latest_version)) = handle.join() {
            let current_version = semver::Version::parse(self_update::cargo_crate_version!())
                .wrap_err("Failed to parse current version of `near` CLI")?;

            let latest_version = semver::Version::parse(&latest_version)
                .wrap_err("Failed to parse latest version of `near` CLI")?;

            if current_version < latest_version {
                eprintln!(
                    "\n`near` CLI has a new update available \x1b[2m{current_version}\x1b[0m →  \x1b[32m{latest_version}\x1b[0m"
                );
                let self_update_cli_cmd = CliCmd {
                    offline: false,
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
                eprintln!(
                    "To update `near` CLI use: {}",
                    shell_words::join(
                        std::iter::once(near_cli_exec_path)
                            .chain(self_update_cli_cmd.to_cli_args())
                    )
                    .yellow()
                );
            }
        }
    };

    cli_cmd.map(|_| ())
}
