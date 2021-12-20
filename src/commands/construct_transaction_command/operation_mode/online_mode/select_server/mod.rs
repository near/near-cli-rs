use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod server;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(input_context = ())]
#[interactive_clap(output_context = SelectServerContext)]
///Select NEAR protocol RPC server
pub enum SelectServer {
    #[strum_discriminants(strum(message = "Testnet"))]
    /// предоставление данных для сервера https://rpc.testnet.near.org
    Testnet(self::server::Server),
    #[strum_discriminants(strum(message = "Mainnet"))]
    /// предоставление данных для сервера https://rpc.mainnet.near.org
    Mainnet(self::server::Server),
    #[strum_discriminants(strum(message = "Betanet"))]
    /// предоставление данных для сервера https://rpc.betanet.near.org
    Betanet(self::server::Server),
    #[strum_discriminants(strum(message = "Custom"))]
    /// предоставление данных для сервера, указанного вручную
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

impl From<SelectServerContext> for super::super::ConstructTransactionNetworkContext {
    fn from(item: SelectServerContext) -> Self {
        let connection_config = match item.selected_server {
            SelectServerDiscriminants::Testnet => crate::common::ConnectionConfig::Testnet,
            SelectServerDiscriminants::Mainnet => crate::common::ConnectionConfig::Mainnet,
            SelectServerDiscriminants::Betanet => crate::common::ConnectionConfig::Betanet,
            SelectServerDiscriminants::Custom => {
                unreachable!("Network context should not be constructed from Custom variant")
            }
        };
        Self {
            connection_config: Some(connection_config),
        }
    }
}

impl SelectServer {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Testnet;
                server
                    .process(prepopulated_unsigned_transaction, connection_config)
                    .await?;
            }
            SelectServer::Mainnet(server) => {
                let connection_config = crate::common::ConnectionConfig::Mainnet;
                server
                    .process(prepopulated_unsigned_transaction, connection_config)
                    .await?;
            }
            SelectServer::Betanet(server) => {
                let connection_config = crate::common::ConnectionConfig::Betanet;
                server
                    .process(prepopulated_unsigned_transaction, connection_config)
                    .await?;
            }
            SelectServer::Custom(custom_server) => {
                custom_server
                    .process(prepopulated_unsigned_transaction)
                    .await?;
            }
        })
    }
}
