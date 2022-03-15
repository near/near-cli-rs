use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod server;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSelectServer {
    /// providing data to server https://rpc.testnet.near.org
    Testnet(self::server::CliServer),
    /// providing data to server https://rpc.mainnet.near.org
    Mainnet(self::server::CliServer),
    /// providing data to server https://rpc.betanet.near.org
    Betanet(self::server::CliServer),
    /// providing data to the manually specified server
    Custom(self::server::CliCustomServer),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SelectServer {
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(self::server::Server),
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(self::server::Server),
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(self::server::Server),
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(self::server::Server),
}

impl CliSelectServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Testnet(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("testnet".to_owned());
                args
            }
            Self::Mainnet(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("mainnet".to_owned());
                args
            }
            Self::Betanet(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("betanet".to_owned());
                args
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
            SelectServer::Testnet(server) => Self::Testnet(server.into()),
            SelectServer::Mainnet(server) => Self::Mainnet(server.into()),
            SelectServer::Betanet(server) => Self::Betanet(server.into()),
            SelectServer::Custom(server) => Self::Custom(server.into()),
        }
    }
}

impl From<CliSelectServer> for SelectServer {
    fn from(item: CliSelectServer) -> Self {
        match item {
            CliSelectServer::Testnet(cli_server) => {
                Self::Testnet(cli_server.into_server(crate::common::ConnectionConfig::Testnet))
            }
            CliSelectServer::Mainnet(cli_server) => {
                Self::Mainnet(cli_server.into_server(crate::common::ConnectionConfig::Mainnet))
            }
            CliSelectServer::Betanet(cli_server) => {
                Self::Betanet(cli_server.into_server(crate::common::ConnectionConfig::Betanet))
            }
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
            SelectServerDiscriminants::Testnet => CliSelectServer::Testnet(Default::default()),
            SelectServerDiscriminants::Mainnet => CliSelectServer::Mainnet(Default::default()),
            SelectServerDiscriminants::Betanet => CliSelectServer::Betanet(Default::default()),
            SelectServerDiscriminants::Custom => CliSelectServer::Custom(Default::default()),
        };
        Self::from(cli_select_server)
    }

    pub async fn process(self) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet(server) => {
                server.process().await?;
            }
            SelectServer::Mainnet(server) => {
                server.process().await?;
            }
            SelectServer::Betanet(server) => {
                server.process().await?;
            }
            SelectServer::Custom(server) => {
                server.process().await?;
            }
        })
    }
}
