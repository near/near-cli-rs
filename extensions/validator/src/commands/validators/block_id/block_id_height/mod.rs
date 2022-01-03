use dialoguer::Input;

use crate::common::display_validators_info;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::NetworkContext)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
}

impl BlockIdHeight {
    pub fn input_block_id_height(
        _context: &super::super::operation_mode::online_mode::select_server::NetworkContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::BlockHeight> {
        Ok(Input::new()
            .with_prompt("Type the block ID height to view validators")
            .interact_text()?)
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
