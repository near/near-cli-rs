mod impls;
use impls::*;

use clap::Clap;
use near_cli_visual::Interactive;

#[derive(Debug, Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::DisableVersionForSubcommands)
)]
struct TopLevel {
    #[clap(subcommand)]
    cli: Option<CliQueryRequest>
}

fn main() {
    let x = TopLevel::parse().interactive();
    println!("{:?}", x);
}
