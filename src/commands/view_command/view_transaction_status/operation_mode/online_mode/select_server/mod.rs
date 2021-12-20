use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod server;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(input_context = ())]
#[interactive_clap(output_context = SelectServerContext)]
///Select NEAR protocol RPC server
pub enum SelectServer {
    /// Provide data for the server https://rpc.testnet.near.org
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(self::server::Server),
    /// Provide data for the server https://rpc.mainnet.near.org
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(self::server::Server),
    /// Provide data for the server https://rpc.betanet.near.org
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(self::server::Server),
    /// Provide data for a manually specified server
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(self::server::CustomServer),
}

#[derive(Clone)]
pub struct SelectServerContext {
    selected_server: SelectServerDiscriminants,
}

impl SelectServerContext {
    fn from_previous_context(
        _previous_context: (),
        scope: &<SelectServer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            selected_server: scope.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ViewTransactionCommandNetworkContext {
    pub connection_config: crate::common::ConnectionConfig,
}

impl From<SelectServerContext> for ViewTransactionCommandNetworkContext {
    fn from(item: SelectServerContext) -> Self {
        let connection_config = match item.selected_server {
            SelectServerDiscriminants::Testnet => crate::common::ConnectionConfig::Testnet,
            SelectServerDiscriminants::Mainnet => crate::common::ConnectionConfig::Mainnet,
            SelectServerDiscriminants::Betanet => crate::common::ConnectionConfig::Betanet,
            SelectServerDiscriminants::Custom => {
                unreachable!("Network context should not be constructed from Custom variant")
            }
        };
        Self { connection_config }
    }
}

impl SelectServer {
    pub async fn process(self) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Testnet;
                server.process(connection_config).await?;
            }
            SelectServer::Mainnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Mainnet;
                server.process(connection_config).await?;
            }
            SelectServer::Betanet(server) => {
                let connection_config = crate::common::ConnectionConfig::Betanet;
                server.process(connection_config).await?;
            }
            SelectServer::Custom(custom_server) => {
                custom_server.process().await?;
            }
        })
    }
}
