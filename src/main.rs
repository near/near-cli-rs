use cargo_util::{ProcessBuilder, ProcessError};
use clap::Clap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
extern crate shell_words;

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

#[derive(Debug, Clone)]
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
    let cli = match CliArgs::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind,
                clap::ErrorKind::UnknownArgument | clap::ErrorKind::InvalidSubcommand
            ) {
                return try_external_subcommand_execution();
            }
            return Err(color_eyre::eyre::eyre!(error));
        }
    };

    if let Some(self::commands::CliTopLevelCommand::GenerateShellCompletions(subcommand)) =
        cli.top_level_command
    {
        subcommand.process();
        return Ok(());
    }

    let args = Args::from(cli);

    let completed_cli = CliArgs::from(args.clone());

    color_eyre::install()?;

    let process_result = actix::System::new().block_on(args.process());

    println!(
        "Your console command:\n{}",
        shell_words::join(&completed_cli.to_cli_args())
    );

    process_result
}

fn try_external_subcommand_execution() -> CliResult {
    let args: Vec<String> = env::args().collect();
    let subcommand = match args.get(1) {
        Some(subcommand) => subcommand,
        None => {
            return Err(color_eyre::eyre::eyre!("subcommand is not provided"));
        }
    };

    let subcommand_exe = format!("near-{}{}", subcommand, env::consts::EXE_SUFFIX);

    let path = get_path_directories()
        .iter()
        .map(|dir| dir.join(&subcommand_exe))
        .find(|file| is_executable(file));

    let command = match path {
        Some(command) => command,
        None => {
            return Err(color_eyre::eyre::eyre!(
                "{} command or {} extension does not exist",
                subcommand,
                subcommand_exe
            ));
        }
    };

    // let cargo_exe = config.cargo_exe()?;
    let err = match ProcessBuilder::new(&command)
        // .env(cargo::CARGO_ENV, cargo_exe)
        .args(&args)
        .exec_replace()
    {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if let Some(perr) = err.downcast_ref::<ProcessError>() {
        if let Some(code) = perr.code {
            return Err(color_eyre::eyre::eyre!("perror occured, code: {}", code));
        }
    }
    return Err(color_eyre::eyre::eyre!(err));
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

fn get_path_directories() -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Some(val) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&val));
    }
    dirs
}
