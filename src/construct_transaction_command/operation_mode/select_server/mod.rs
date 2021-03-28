use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use strum_macros::{Display, EnumVariantNames};

use crate::consts;
use consts::{BETANET_API_SERVER_URL, MAINNET_API_SERVER_URL, TESTNET_API_SERVER_URL};
pub mod server;
use server::{CliCustomServer, CliServer, Server};

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SelectServer {
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(Server),
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(Server),
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(Server),
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(Server),
}

#[derive(Debug, Display, EnumVariantNames, StructOpt)]
pub enum CliSelectServer {
    Testnet(CliServer),
    Mainnet(CliServer),
    Betanet(CliServer),
    Custom(CliCustomServer),
}

impl From<CliSelectServer> for SelectServer {
    fn from(item: CliSelectServer) -> Self {
        match item {
            CliSelectServer::Testnet(cli_server) => {
                Self::Testnet(cli_server.into_server(TESTNET_API_SERVER_URL.to_string()))
            }
            CliSelectServer::Mainnet(cli_server) => {
                Self::Mainnet(cli_server.into_server(MAINNET_API_SERVER_URL.to_string()))
            }
            CliSelectServer::Betanet(cli_server) => {
                Self::Betanet(cli_server.into_server(BETANET_API_SERVER_URL.to_string()))
            }
            CliSelectServer::Custom(cli_custom_server) => {
                Self::Custom(cli_custom_server.into_server())
            }
        }
    }
}

impl SelectServer {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            SelectServer::Testnet(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
            SelectServer::Mainnet(_server) => {}
            SelectServer::Betanet(_server) => {}
            SelectServer::Custom(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
        }
        Ok(())
    }
    pub fn select_server() -> CliSelectServer {
        println!();
        let variants = SelectServerDiscriminants::iter().collect::<Vec<_>>();
        let servers = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_server = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select NEAR protocol RPC server:")
            .items(&servers)
            .default(0)
            .interact()
            .unwrap();
        match variants[selected_server] {
            SelectServerDiscriminants::Testnet => CliSelectServer::Testnet(Default::default()),
            SelectServerDiscriminants::Mainnet => CliSelectServer::Mainnet(Default::default()),
            SelectServerDiscriminants::Betanet => CliSelectServer::Betanet(Default::default()),
            SelectServerDiscriminants::Custom => CliSelectServer::Custom(Default::default()),
        }
    }
}
