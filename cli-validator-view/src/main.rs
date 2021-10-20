use common::{display_proposals_info, display_validators_info};
use impls::*;

use common::CliResult;

use clap::Clap;
use near_cli_visual::Interactive;

mod common;
mod consts;
mod impls;

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
        CliQueryRequest::Proposals(data) => {
            match data.mode.unwrap() {
                CliMode::Network(data) => {
                    let mut connection_config = crate::common::ConnectionConfig::Testnet;
                    match data.selected_server.unwrap() {
                        CliSelectServer::Testnet(_) => {
                            connection_config = crate::common::ConnectionConfig::Testnet;
                        }
                        CliSelectServer::Mainnet(_) => {
                            connection_config = crate::common::ConnectionConfig::Mainnet;
                        }
                        CliSelectServer::Betanet(_) => {
                            connection_config = crate::common::ConnectionConfig::Betanet;
                        }
                        CliSelectServer::Custom(_) => {
                            println!("Custom network is currentlu unsuported"); //TODO
                        }
                    }
                    actix::System::new().block_on(display_proposals_info(&connection_config));
                }
            };
        }
        CliQueryRequest::Validators(_) => {
            println!("Entered data: {:?}", x);
        }
    }
}
