use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

use crate::common::display_validators_info;

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliBlockId {
    /// Specify a block ID final to view this account
    AtFinalBlock,
    /// Specify a block ID height to view this account
    AtBlockHeight(self::block_id_height::CliBlockIdHeight),
    /// Specify a block ID hash to view this account
    AtBlockHash(self::block_id_hash::CliBlockIdHash),
}

#[derive(clap::Clap, Default, Debug, Clone)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliBlockIdWrapper {
    #[clap(subcommand)]
    cli_block_id: Option<CliBlockId>,
}


#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this account at final block"))]
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this account at block heigt"))]
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "+ View this account at block hash"))]
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl CliBlockId {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AtFinalBlock => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("at-final-block".to_owned());
                args
            }
            Self::AtBlockHeight(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-height".to_owned());
                args
            }
            Self::AtBlockHash(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-hash".to_owned());
                args
            }
        }
    }
}

impl From<BlockId> for CliBlockId {
    fn from(block_id: BlockId) -> Self {
        match block_id {
            BlockId::AtFinalBlock => Self::AtFinalBlock,
            BlockId::AtBlockHeight(block_id_height) => Self::AtBlockHeight(block_id_height.into()),
            BlockId::AtBlockHash(block_id_hash) => Self::AtBlockHash(block_id_hash.into()),
        }
    }
}

impl From<CliBlockId> for BlockId {
    fn from(item: CliBlockId) -> Self {
        match item {
            CliBlockId::AtFinalBlock => Self::AtFinalBlock,
            CliBlockId::AtBlockHeight(cli_block_id_height) => {
                Self::AtBlockHeight(cli_block_id_height.into())
            }
            CliBlockId::AtBlockHash(cli_block_id_hash) => {
                Self::AtBlockHash(cli_block_id_hash.into())
            }
        }
    }
}

impl BlockId {
    pub fn choose_block_id() -> Self {
        println!();
        let variants = BlockIdDiscriminants::iter().collect::<Vec<_>>();
        let blocks = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&blocks)
            .default(0)
            .interact()
            .unwrap();
        let cli_block_id = match variants[selection] {
            BlockIdDiscriminants::AtFinalBlock => CliBlockId::AtFinalBlock,
            BlockIdDiscriminants::AtBlockHeight => CliBlockId::AtBlockHeight(Default::default()),
            BlockIdDiscriminants::AtBlockHash => CliBlockId::AtBlockHash(Default::default()),
        };
        Self::from(cli_block_id)
    }

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
                display_validators_info(near_primitives::types::EpochReference::Latest, &network_connection_config).await?;
                Ok(())
            }
        }
    }
}
