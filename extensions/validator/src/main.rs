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
        CliQueryRequest::Validators(data) => {
            match data.mode.unwrap() {
                CliMode::Network(data) => {
                    let (connection_config, epoch) = match data.selected_server.unwrap() {
                        CliSelectServer::Testnet(data) => {
                            (crate::common::ConnectionConfig::Testnet, epoch_from_validators_structure(data))
                        }
                        CliSelectServer::Mainnet(data) => {
                            (crate::common::ConnectionConfig::Mainnet, epoch_from_validators_structure(data))
                        }
                        CliSelectServer::Betanet(data) => {
                            (crate::common::ConnectionConfig::Betanet, epoch_from_validators_structure(data))
                        }
                        CliSelectServer::Custom(_) => {
                            println!("Custom network is currently unsuported");
                            (crate::common::ConnectionConfig::Testnet, near_primitives::types::EpochReference::Latest)
                        }
                    };
                    actix::System::new()
                        .block_on(display_validators_info(epoch, &connection_config));
                }
            };
        }
    }
}

fn epoch_from_validators_structure(
    data: CliServer<CliSendTo<CliValidators>>,
) -> near_primitives::types::EpochReference {
    return match data.send_to.unwrap() {
        CliSendTo::SendTo(data) => match data.epoch.unwrap() {
            CliEpochCommand::Latest => near_primitives::types::EpochReference::Latest,
            CliEpochCommand::BlockId(data) => match data.cli_block_id.unwrap() {
                CliBlockId::AtFinalBlock => near_primitives::types::EpochReference::Latest,
                CliBlockId::AtBlockHeight(data) => near_primitives::types::EpochReference::BlockId(
                    near_primitives::types::BlockId::Height(data.block_id_height.unwrap()),
                ),
                CliBlockId::AtBlockHash(data) => {
                    let hash: near_primitives::types::BlockId =
                        near_primitives::types::BlockId::Hash(data.block_id_hash.unwrap());
                    near_primitives::types::EpochReference::BlockId(hash)
                }
            },
        },
    };
}
