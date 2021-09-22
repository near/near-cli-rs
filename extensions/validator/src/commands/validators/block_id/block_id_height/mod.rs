use dialoguer::Input;

use crate::common::display_validators_info;

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

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        display_validators_info(
            near_primitives::types::EpochReference::BlockId(
                near_primitives::types::BlockId::Height(self.block_id_height),
            ),
            &network_connection_config,
        )
        .await?;
        Ok(())
    }
}
