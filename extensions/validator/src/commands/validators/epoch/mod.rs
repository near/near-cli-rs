use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

use crate::common::display_validators_info;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliEpochCommand {
    /// View latest validators
    Latest,
    /// View validators by EpochId
    // EpochId(self::view_command::CliViewQueryRequest), //TODO
    /// View validators by BlockId
    BlockId(super::block_id::CliBlockIdWrapper),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum EpochCommand {
    #[strum_discriminants(strum(message = "View latest validators"))]
    Latest,
    // #[strum_discriminants(strum(
    //     message = "View validators by EpochId"
    // ))]
    // EpochId(self::view_command::ViewQueryRequest),
    #[strum_discriminants(strum(message = "View validators by BlockId"))]
    BlockId(super::block_id::BlockId),
}

impl CliEpochCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Latest => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("latest".to_owned());
                args
            }
            // Self::EpochId(subcommand) => {
            //     let mut args = subcommand.to_cli_args();
            //     args.push_front("epoch-id".to_owned());
            //     args
            // }
            Self::BlockId(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("block-id".to_owned());
                args
            }
        }
    }
}

impl From<EpochCommand> for CliEpochCommand {
    fn from(top_level_command: EpochCommand) -> Self {
        match top_level_command {
            EpochCommand::Latest => Self::Latest,
            // EpochCommand::EpochId(validators_request) => Self::BlockId(validators_request.into()),
            EpochCommand::BlockId(validators_request) => Self::BlockId(validators_request.into()),
        }
    }
}

impl From<CliEpochCommand> for EpochCommand {
    fn from(cli_top_level_command: CliEpochCommand) -> Self {
        match cli_top_level_command {
            CliEpochCommand::Latest => EpochCommand::Latest,
            // CliEpochCommand::EpochId(cli_validators_request) => {
            //     EpochCommand::EpochId(cli_validators_request.into())
            // }
            CliEpochCommand::BlockId(cli_validators_request) => {
                EpochCommand::BlockId(cli_validators_request.into())
            }
        }
    }
}

impl EpochCommand {
    pub fn choose_command() -> Self {
        println!();
        let variants = EpochCommandDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&commands)
            .default(0)
            .interact()
            .unwrap();
        let cli_top_level_command = match variants[selection] {
            EpochCommandDiscriminants::Latest => CliEpochCommand::Latest,
            // EpochCommandDiscriminants::EpochId => CliEpochCommand::EpochId(Default::default()),
            EpochCommandDiscriminants::BlockId => CliEpochCommand::BlockId(Default::default()),
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            Self::Latest => {
                display_validators_info(
                    near_primitives::types::EpochReference::Latest,
                    &network_connection_config,
                )
                .await?;
                Ok(())
            },
            // Self::EpochId(validators_request) => validators_request.process().await,
            Self::BlockId(validators_request) => {
                validators_request.process(network_connection_config).await
            },
        }
    }
}
