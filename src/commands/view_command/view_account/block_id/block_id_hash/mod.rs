use dialoguer::Input;

use crate::common::{display_access_key_list, display_account_info, ConnectionConfig};

use near_primitives::types::{AccountId, BlockId, BlockReference};

/// Specify the block_id hash for this account to view
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliBlockIdHash {
    block_id_hash: Option<near_primitives::hash::CryptoHash>,
}

#[derive(Debug, Clone)]
pub struct BlockIdHash {
    block_id_hash: near_primitives::hash::CryptoHash,
}

impl CliBlockIdHash {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(block_id_hash) = &self.block_id_hash {
            args.push_front(block_id_hash.to_string());
        }
        args
    }
}

impl From<BlockIdHash> for CliBlockIdHash {
    fn from(block_id_hash: BlockIdHash) -> Self {
        Self {
            block_id_hash: Some(block_id_hash.block_id_hash),
        }
    }
}

impl From<CliBlockIdHash> for BlockIdHash {
    fn from(item: CliBlockIdHash) -> Self {
        let block_id_hash: near_primitives::hash::CryptoHash = match item.block_id_hash {
            Some(cli_block_id_hash) => cli_block_id_hash,
            None => BlockIdHash::input_block_id_hash(),
        };
        Self { block_id_hash }
    }
}

impl BlockIdHash {
    pub fn input_block_id_hash() -> near_primitives::hash::CryptoHash {
        Input::new()
            .with_prompt("Type the block ID hash for this account")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self, account_id: AccountId, conf: ConnectionConfig) -> crate::CliResult {
        let block_ref = BlockReference::BlockId(BlockId::Hash(self.block_id_hash.clone()));
        display_account_info(account_id.clone(), &conf, block_ref.clone()).await?;
        display_access_key_list(account_id, &conf, block_ref).await?;
        Ok(())
    }
}
