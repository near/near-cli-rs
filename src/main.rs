use clap::Clap;

mod commands;
mod common;
mod consts;

type CliResult = color_eyre::eyre::Result<()>;

/// near-cli is a toolbox for interacting with NEAR protocol
#[derive(Debug, Clap)]
#[clap(
    version,
    author,
    about,
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands),
    // setting(clap::AppSettings::NextLineHelp)
)]
struct CliArgs {
    #[clap(subcommand)]
    top_level_command: Option<self::commands::CliTopLevelCommand>,
}

#[derive(Debug)]
struct Args {
    top_level_command: self::commands::TopLevelCommand,
}

impl CliArgs {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .top_level_command
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        args.push_front("./near-cli".to_owned());
        args
    }
}

impl From<Args> for CliArgs {
    fn from(cli_args: Args) -> Self {
        Self {
            top_level_command: Some(cli_args.top_level_command.into()),
        }
    }
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

    //let completed_cli = CliArgs::from(args.clone());
    let completed_cli = CliArgs::from(args);
    println!("{:?}", completed_cli.to_cli_args());

    color_eyre::install()?;

    //actix::System::new().block_on(args.process())
    Ok(())
}
