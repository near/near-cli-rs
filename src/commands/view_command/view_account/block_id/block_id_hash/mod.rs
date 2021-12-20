use dialoguer::Input;

use crate::common::{display_access_key_list, display_account_info, ConnectionConfig};

use near_primitives::types::{AccountId, BlockId, BlockReference};

// /// Specify the block_id hash for this account to view
// #[derive(Debug, Default, Clone, clap::Clap)]
// pub struct CliBlockIdHash {
//     block_id_hash: Option<near_primitives::hash::CryptoHash>,
// }

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext)]
pub struct BlockIdHash {
    block_id_hash: crate::types::crypto_hash::CryptoHash,
}

impl BlockIdHash {
    pub fn input_block_id_hash(
        _context: &super::super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
        Ok(Input::new()
            .with_prompt("Type the block ID hash for this account")
            .interact_text()
            .unwrap())
    }

    pub async fn process(self, account_id: AccountId, conf: ConnectionConfig) -> crate::CliResult {
        let block_ref = BlockReference::BlockId(BlockId::Hash(self.block_id_hash.clone().into()));
        display_account_info(account_id.clone(), &conf, block_ref.clone()).await?;
        display_access_key_list(account_id, &conf, block_ref).await?;
        Ok(())
    }
}
