use dialoguer::Input;

use crate::common::display_validators_info;

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
            .with_prompt("Type the block ID hash")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        display_validators_info(
            near_primitives::types::EpochReference::BlockId(near_primitives::types::BlockId::Hash(
                self.block_id_hash,
            )),
            &network_connection_config,
        )
        .await?;
        Ok(())
    }
}
