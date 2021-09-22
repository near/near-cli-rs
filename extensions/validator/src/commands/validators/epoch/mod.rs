use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, Clone, clap::Clap)]
pub enum CliEpochCommand {
    /// View validators by EpochId
    // EpochId(self::view_command::CliViewQueryRequest), //TODO
    /// View validators by BlockId
    BlockId(super::block_id::CliBlockId),
    // / View latest validators //TODO: it should be a doc comment (///)
    // Latest(self::proposals::operation_mode::CliOperationMode), //TODO
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum EpochCommand {
    // #[strum_discriminants(strum(
    //     message = "View validators by EpochId"
    // ))]
    // EpochId(self::view_command::ViewQueryRequest),
    #[strum_discriminants(strum(message = "View validators by BlockId"))]
    BlockId(super::block_id::BlockId),
    // #[strum_discriminants(strum(
    //     message = "View latest validators"
    // ))]
    // Latest(self::proposals::operation_mode::CliOperationMode),
}

impl CliEpochCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            // Self::EpochId(subcommand) => {
            //     let mut args = subcommand.to_cli_args();
            //     args.push_front("epoch-id".to_owned());
            //     args
            // }
            Self::BlockId(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("blick-id".to_owned());
                args
            }
            // Self::Latest(subcommand) => {
            //     let mut args = subcommand.to_cli_args();
            //     args.push_front("latest".to_owned());
            //     args
            // }
        }
    }
}

impl From<EpochCommand> for CliEpochCommand {
    fn from(top_level_command: EpochCommand) -> Self {
        match top_level_command {
            // EpochCommand::EpochId(validators_request) => Self::BlockId(validators_request.into()),
            EpochCommand::BlockId(validators_request) => {
                Self::BlockId(validators_request.into())
            }
            // EpochCommand::Latest(validators_request) => Self::Latest(validators_request.into()),
        }
    }
}

impl From<CliEpochCommand> for EpochCommand {
    fn from(cli_top_level_command: CliEpochCommand) -> Self {
        match cli_top_level_command {
            // CliEpochCommand::EpochId(cli_validators_request) => {
            //     EpochCommand::EpochId(cli_validators_request.into())
            // }
            CliEpochCommand::BlockId(cli_validators_request) => {
                EpochCommand::BlockId(cli_validators_request.into())
            }
            // CliEpochCommand::Latest(cli_validators_request) => {
            //     EpochCommand::Latest(cli_validators_request.into())
            // }
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
            // EpochCommandDiscriminants::EpochId => EpochCommand::EpochId(Default::default()),
            EpochCommandDiscriminants::BlockId => {
                EpochCommand::BlockId(Default::default())
            }
            // EpochCommandDiscriminants::Latest => {
            //     EpochCommand::Latest(Default::default())
            // }
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            // Self::EpochId(validators_request) => validators_request.process().await,
            Self::BlockId(validators_request) => validators_request.process(network_connection_config).await,
            // Self::Latest(validators_request) => validators_request.process().await,
        }
    }
}
