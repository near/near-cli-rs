use clap::Clap;
use shell_words;

mod commands;

//TODO: move CLI result to a common crate/workspace
pub type CliResult = color_eyre::eyre::Result<()>;
/// near-cli-validator is a set of commands intended to be used by NEAR validators
// TODO: check this settings
// TODO: can we avoid duplication of this settings in each extension?
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

struct ValidatorArgs {
    #[clap(subcommand)]
    top_level_command: Option<self::commands::CliValidatorCommand>,
}

#[derive(Debug, Clone)]
struct Args {
    top_level_command: self::commands::ValidatorCommand,
}

// TODO: de we need this fucntion? Should it be reimplemented?
// impl ValidatorArgs {
//     pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
//         let mut args = self
//             .top_level_command
//             .as_ref()
//             .map(|subcommand| subcommand.to_cli_args())
//             .unwrap_or_default();
//         args.push_front("./near-cli".to_owned());
//         args
//     }
// }

// impl From<Args> for ValidatorArgs {
//     fn from(cli_args: Args) -> Self {
//         Self {
//             top_level_command: Some(cli_args.top_level_command.into()),
//         }
//     }
// }

// impl From<ValidatorArgs> for Args {
//     fn from(cli_args: ValidatorArgs) -> Self {
//         let top_level_command = match cli_args.top_level_command {
//             Some(cli_subcommand) => self::commands::ValidatorCommand::from(cli_subcommand),
//             None => self::commands::ValidatorCommand::choose_command(),
//         };
//         Self { top_level_command }
//     }
// }

impl Args {
    async fn process(self) -> CliResult {
        self.top_level_command.process().await
    }
}

fn main() -> CliResult {
    //TODO: shoud it be moved to macro?
    color_eyre::install()?;

    let cli = ValidatorArgs::parse();

    if let Some(self::commands::CliValidatorCommand::GenerateShellCompletions(subcommand)) =
        cli.top_level_command
    {
        subcommand.process();
        return Ok(());
    }

    let args = Args::from(cli);

    let completed_cli = ValidatorArgs::from(args.clone());
    

    let process_result = actix::System::new().block_on(args.process());

    //TODO: Looks like we will be forsed to duplicate it in each extension. Should we move it to a macro?
    println!(
        "Your console command:\n{}",
        shell_words::join(&completed_cli.to_cli_args())
    );

    process_result
}
