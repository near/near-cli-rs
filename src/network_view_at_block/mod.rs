use near_primitives::types::{BlockId, BlockReference, Finality};
use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct NetworkViewAtBlockArgs {
    ///What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    next: ViewAtBlock,
}

impl NetworkViewAtBlockArgs {
    fn input_network_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(context)
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        let network_config = config.networks;
        network_config
            .get(self.network_name.as_str())
            .expect("Impossible to get network name!")
            .clone()
    }

    pub fn get_block_ref(&self) -> BlockReference {
        match self.next.clone() {
            ViewAtBlock::Now => Finality::Final.into(),
            ViewAtBlock::AtBlockHash(at_block_hash) => BlockReference::BlockId(BlockId::Hash(
                near_primitives::hash::CryptoHash::from_str(at_block_hash.block_id_hash.as_str())
                    .unwrap(),
            )),
            ViewAtBlock::AtBlockHeight(at_block_height) => {
                BlockReference::BlockId(BlockId::Height(at_block_height.block_id_height))
            }
        }
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Сhoose block for view
pub enum ViewAtBlock {
    #[strum_discriminants(strum(
        message = "now               - View properties in the final block"
    ))]
    ///View properties in the final block
    Now,
    #[strum_discriminants(strum(
        message = "at-block-height   - View properties in a height-selected block"
    ))]
    ///View properties in a height-selected block
    AtBlockHeight(AtBlockHeight),
    #[strum_discriminants(strum(
        message = "at-block-hash     - View properties in a hash-selected block"
    ))]
    ///View properties in a hash-selected block
    AtBlockHash(BlockIdHash),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AtBlockHeight {
    ///Type the block ID height
    block_id_height: near_primitives::types::BlockHeight,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct BlockIdHash {
    ///Type the block ID hash
    block_id_hash: String,
}
