use dialoguer::Input;

use crate::common::display_validators_info;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::super::operation_mode::online_mode::select_server::NetworkContext)]
pub struct BlockIdHash {
    block_id_hash: crate::types::crypto_hash::CryptoHash,
}

impl BlockIdHash {
    pub fn input_block_id_hash(
        _context: &super::super::operation_mode::online_mode::select_server::NetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
        Ok(Input::new()
            .with_prompt("Type the block ID hash to view validators")
            .interact_text()?)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        display_validators_info(
            near_primitives::types::EpochReference::BlockId(near_primitives::types::BlockId::Hash(
                self.block_id_hash.into(),
            )),
            &network_connection_config,
        )
        .await?;
        Ok(())
    }
}
