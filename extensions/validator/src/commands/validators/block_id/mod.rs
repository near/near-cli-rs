use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::display_validators_info;

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::NetworkContext)]
pub struct BlockIdWrapper {
    #[interactive_clap(subcommand)]
    block_id: BlockId,
}

impl BlockIdWrapper {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.block_id.process(network_connection_config).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::NetworkContext)]
///Choose Block ID
pub enum BlockId {
    #[strum_discriminants(strum(message = "View validators at final block"))]
    /// Specify a block ID final to view validators
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View validators at block heigt"))]
    /// Specify a block ID height to view validators
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View validators at block hash"))]
    /// Specify a block ID hash to view validators
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl BlockId {
    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height.process(network_connection_config).await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash.process(network_connection_config).await
            }
            Self::AtFinalBlock => {
                display_validators_info(
                    near_primitives::types::EpochReference::Latest,
                    &network_connection_config,
                )
                .await?;
                Ok(())
            }
        }
    }
}
