use clap::Clap;

mod commands;
mod common;
mod consts;

type CliResult = color_eyre::eyre::Result<()>;

/// near-cli is a toolbox for interacting with NEAR protocol
#[derive(Debug, Default, clap::Clap)]
struct CliArgs {
    #[clap(subcommand)]
    top_level_command: Option<self::commands::CliTopLevelCommand>,
}

#[derive(Debug)]
struct Args {
    top_level_command: self::commands::TopLevelCommand,
}

impl From<CliArgs> for Args {
    fn from(cli_args: CliArgs) -> Self {
        let top_level_command = match cli_args.top_level_command {
            Some(cli_subcommand) => self::commands::TopLevelCommand::from(cli_subcommand),
            None => self::commands::TopLevelCommand::choose_command(),
        };
        Self { top_level_command }
    }
}

impl Args {
    async fn process(self) -> CliResult {
        self.top_level_command.process().await
    }
}

fn main() -> CliResult {
    let cli = CliArgs::parse();

    if let Some(self::commands::CliTopLevelCommand::GenerateShellCompletions(subcommand)) =
        cli.top_level_command
    {
        subcommand.process();
        return Ok(());
    }

    let args = Args::from(cli);

    color_eyre::install()?;

    actix::System::new().block_on(args.process())
}
