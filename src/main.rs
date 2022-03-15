use clap::Clap;
use shell_words;

use common::{try_external_subcommand_execution, CliResult};

mod commands;
mod common;
mod consts;
mod types;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
struct Args {
    #[interactive_clap(subcommand)]
    top_level_command: self::commands::TopLevelCommand,
}

impl Args {
    async fn process(self) -> CliResult {
        self.top_level_command.process().await
    }
}

fn main() -> CliResult {
    color_eyre::install()?;

    let cli = match CliArgs::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind,
                clap::ErrorKind::UnknownArgument | clap::ErrorKind::InvalidSubcommand
            ) {
                return try_external_subcommand_execution();
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

    let args = Args::from_cli(Some(cli), ())?;

    let completed_cli = CliArgs::from(args.clone());

    let process_result = actix::System::new().block_on(args.process());

    println!(
        "Your console command:\n{} {}",
        std::env::args().next().as_deref().unwrap_or("./near_cli"),
        shell_words::join(&completed_cli.to_cli_args())
    );

    process_result
}
