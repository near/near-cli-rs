use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

use crate::common::display_proposals_info;

pub mod server;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSelectServer {
    /// предоставление данных для сервера https://rpc.testnet.near.org
    Testnet,
    /// предоставление данных для сервера https://rpc.mainnet.near.org
    Mainnet,
    /// предоставление данных для сервера https://rpc.betanet.near.org
    Betanet,
    /// предоставление данных для сервера, указанного вручную
    Custom(self::server::CliCustomServer),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SelectServer {
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet,
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet,
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet,
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(self::server::Server),
}

impl CliSelectServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Testnet => {
                return std::collections::VecDeque::new();
            }
            Self::Mainnet => {
                return std::collections::VecDeque::new();
            }
            Self::Betanet => {
                return std::collections::VecDeque::new();
            }
            Self::Custom(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("custom".to_owned());
                args
            }
        }
    }
}

impl From<SelectServer> for CliSelectServer {
    fn from(select_server: SelectServer) -> Self {
        match select_server {
            SelectServer::Testnet => Self::Testnet,
            SelectServer::Mainnet => Self::Mainnet,
            SelectServer::Betanet => Self::Betanet,
            SelectServer::Custom(server) => Self::Custom(server.into()),
        }
    }
}

impl From<CliSelectServer> for SelectServer {
    fn from(item: CliSelectServer) -> Self {
        match item {
            CliSelectServer::Testnet => Self::Testnet,
            CliSelectServer::Mainnet => Self::Mainnet,
            CliSelectServer::Betanet => Self::Betanet,
            CliSelectServer::Custom(cli_custom_server) => {
                Self::Custom(cli_custom_server.into_server())
            }
        }
    }
}

impl SelectServer {
    pub fn choose_server() -> Self {
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
        let cli_select_server = match variants[selected_server] {
            SelectServerDiscriminants::Testnet => CliSelectServer::Testnet,
            SelectServerDiscriminants::Mainnet => CliSelectServer::Mainnet,
            SelectServerDiscriminants::Betanet => CliSelectServer::Betanet,
            SelectServerDiscriminants::Custom => CliSelectServer::Custom(Default::default()),
        };
        Self::from(cli_select_server)
    }

    pub async fn process(self) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet => {
                display_proposals_info(&crate::common::ConnectionConfig::Testnet).await?
            }
            SelectServer::Mainnet => {
                display_proposals_info(&crate::common::ConnectionConfig::Mainnet).await?
            }
            SelectServer::Betanet => {
                display_proposals_info(&crate::common::ConnectionConfig::Betanet).await?
            }
            SelectServer::Custom(server) => {
                server.process().await?;
            }
        })
    }
}
