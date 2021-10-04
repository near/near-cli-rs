mod impls;
use impls::*;

use clap::Clap;
use near_cli_visual::Interactive;

#[derive(Debug, Clap, Clone, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
struct TopLevel {
    #[clap(subcommand)]
    cli: Option<CliQueryRequest>,
}

fn main() {
    let x = TopLevel::parse().interactive();
    match x.clone().cli.unwrap() {
        CliQueryRequest::AccountSummary(_) => println!("Entered data: {:?}", x),
        CliQueryRequest::Proposals(_) => {
            println!("Entered data: {:?}", x);
            //TODO: how to get data here? (in a nice way)
        }
        CliQueryRequest::Validators(_) => {
            println!("Entered data: {:?}", x);
            //TODO: how to get data here? (in a nice way)
        }
    }
}
