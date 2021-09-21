use dialoguer::Input;

use crate::common::{display_access_key_list, display_account_info, ConnectionConfig};
use near_primitives::types::{AccountId, BlockId, BlockReference};

/// Specify the block_id height for this account to view
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliBlockIdHeight {
    block_id_height: Option<near_primitives::types::BlockHeight>,
}

#[derive(Debug, Clone)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl CliBlockIdHeight {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(block_id_height) = &self.block_id_height {
            args.push_front(block_id_height.to_string());
        }
        args
    }
}

impl From<BlockIdHeight> for CliBlockIdHeight {
    fn from(block_id_height: BlockIdHeight) -> Self {
        Self {
            block_id_height: Some(block_id_height.block_id_height),
        }
    }
}

impl From<CliBlockIdHeight> for BlockIdHeight {
    fn from(item: CliBlockIdHeight) -> Self {
        let block_id_height: near_primitives::types::BlockHeight = match item.block_id_height {
            Some(cli_block_id_hash) => cli_block_id_hash,
            None => BlockIdHeight::input_block_id_height(),
        };
        Self { block_id_height }
    }
}

impl BlockIdHeight {
    pub fn input_block_id_height() -> near_primitives::types::BlockHeight {
        Input::new()
            .with_prompt("Type the block ID height for this account")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self, account_id: AccountId, conf: ConnectionConfig) -> crate::CliResult {
        let block_ref = BlockReference::BlockId(BlockId::Height(self.block_id_height));
        display_account_info(account_id.clone(), &conf, block_ref.clone()).await?;
        display_access_key_list(account_id, &conf, block_ref).await?;
        Ok(())
    }
}
