use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::{display_access_key_list, display_account_info, ConnectionConfig};
use near_primitives::types::{AccountId, Finality};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext)]
///Choose Block ID
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this account at final block"))]
    /// Specify a block ID final to view this account
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this account at block height"))]
    /// Specify a block ID height to view this account
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this account at block hash"))]
    /// Specify a block ID hash to view this account
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl BlockId {
    pub async fn process(self, account_id: AccountId, conf: ConnectionConfig) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => block_id_height.process(account_id, conf).await,
            Self::AtBlockHash(block_id_hash) => block_id_hash.process(account_id, conf).await,
            Self::AtFinalBlock => {
                display_account_info(account_id.clone(), &conf, Finality::Final.into()).await?;
                display_access_key_list(account_id, &conf, Finality::Final.into()).await?;
                Ok(())
            }
        }
        // Ok(())
    }
}
