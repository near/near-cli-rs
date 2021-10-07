use impls::*;
use common:: {
    display_validators_info,
    display_proposals_info
};

use common::{CliResult};

use clap::Clap;
use near_cli_visual::Interactive;

mod impls;
mod common;
mod consts;

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
            // display_proposals_info();
        }
        CliQueryRequest::Validators(_) => {
            println!("Entered data: {:?}", x);
            // display_validators_info();
        }
    }
}
