use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::display_validators_info;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::NetworkContext)]
///Choose Block ID
pub enum EpochCommand {
    #[strum_discriminants(strum(message = "View latest validators"))]
    /// Specify latest validators
    Latest,
    // #[strum_discriminants(strum(
    //     message = "View validators by EpochId"
    // ))]
    // EpochId(self::view_command::ViewQueryRequest),
    #[strum_discriminants(strum(message = "View validators by BlockId"))]
    /// Specify validators by BlockId
    BlockId(super::block_id::BlockId),
}

impl EpochCommand {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            Self::Latest => {
                display_validators_info(
                    near_primitives::types::EpochReference::Latest,
                    &network_connection_config,
                )
                .await?;
                Ok(())
            },
            // Self::EpochId(validators_request) => validators_request.process().await,
            Self::BlockId(validators_request) => {
                validators_request.process(network_connection_config).await
            },
        }
    }
}
